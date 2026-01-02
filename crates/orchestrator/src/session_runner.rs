//! Unified session runner for all OpenCode session types.
//!
//! This module provides a single abstraction for running AI sessions,
//! handling SSE streaming, activity tracking, and DB persistence.
//!
//! All sessions run asynchronously in the background - we never block waiting for completion.

use db::{SessionRepository, TaskRepository};
use events::{Event, EventBus, EventEnvelope};
use opencode_client::apis::configuration::Configuration;
use opencode_client::apis::default_api;
use opencode_client::models::{SessionPromptRequest, SessionPromptRequestPartsInner};
use opencode_core::{Session, SessionPhase, SessionStatus, TaskStatus, UpdateTaskRequest};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::activity_store::SessionActivityRegistry;
use crate::error::{OrchestratorError, Result};
use crate::executor::TaskExecutor;
use crate::files::FileManager;
use crate::opencode_events::{ExecutorEvent, OpenCodeEventSubscriber};

/// Configuration for running a session
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Task ID this session belongs to
    pub task_id: Uuid,
    /// Current task status (for determining next status after completion)
    pub task_status: TaskStatus,
    /// Session phase (Planning, Implementation, Review, Fix)
    pub phase: SessionPhase,
    /// The prompt to send to AI
    pub prompt: String,
    /// Working directory for the session
    pub working_dir: PathBuf,
    /// Provider ID (e.g., "anthropic")
    pub provider_id: String,
    /// Model ID (e.g., "claude-sonnet-4-20250514")
    pub model_id: String,
    /// Optional MCP server configuration
    pub mcp_config: Option<McpConfig>,
    /// Optional implementation phase info (phase_number, title)
    pub implementation_phase: Option<(u32, String)>,
    /// Skip task status update after completion (for phased implementation)
    pub skip_task_status_update: bool,
}

/// MCP server configuration
#[derive(Debug, Clone)]
pub struct McpConfig {
    /// Workspace path for MCP server
    pub workspace_path: PathBuf,
    /// Whether MCP setup was successful
    pub setup_success: bool,
}

/// Result of starting a session (returns immediately)
#[derive(Debug, Clone)]
pub struct SessionResult {
    /// Our internal session ID
    pub session_id: Uuid,
    /// OpenCode session ID
    pub opencode_session_id: String,
}

/// Dependencies needed for session execution
pub struct SessionDependencies {
    pub opencode_config: Arc<Configuration>,
    pub session_repo: Option<Arc<SessionRepository>>,
    pub task_repo: Option<Arc<TaskRepository>>,
    pub event_bus: Option<EventBus>,
    pub activity_registry: Option<SessionActivityRegistry>,
    pub file_manager: FileManager,
    pub base_url: String,
}

impl SessionDependencies {
    /// Create new dependencies, extracting base_url from opencode_config
    pub fn new(
        opencode_config: Arc<Configuration>,
        session_repo: Option<Arc<SessionRepository>>,
        task_repo: Option<Arc<TaskRepository>>,
        event_bus: Option<EventBus>,
        activity_registry: Option<SessionActivityRegistry>,
        file_manager: FileManager,
    ) -> Self {
        let base_url = opencode_config
            .base_path
            .trim_end_matches("/api")
            .to_string();

        Self {
            opencode_config,
            session_repo,
            task_repo,
            event_bus,
            activity_registry,
            file_manager,
            base_url,
        }
    }
}

/// Unified session runner - all sessions run in background
pub struct SessionRunner;

impl SessionRunner {
    /// Start a session and return immediately.
    ///
    /// This creates the OpenCode session, persists the session record,
    /// emits SessionStarted event, and spawns a background task to
    /// handle execution with SSE streaming.
    ///
    /// The session runs entirely in the background - we never block.
    /// Progress is streamed via SSE and stored in the activity registry.
    pub async fn start(
        config: SessionConfig,
        deps: SessionDependencies,
    ) -> Result<SessionResult> {
        // 1. Create OpenCode session
        let opencode_session = Self::create_opencode_session(
            &deps.opencode_config,
            config.working_dir.to_str(),
        )
        .await?;
        let opencode_session_id = opencode_session.id.to_string();

        // 2. Create our Session record
        let mut session = if let Some((phase_num, ref title)) = config.implementation_phase {
            Session::new_implementation_phase(config.task_id, phase_num, title)
        } else {
            Session::new(config.task_id, config.phase)
        };
        session.start(opencode_session_id.clone());

        // 3. Persist session to DB
        if let Some(ref repo) = deps.session_repo {
            repo.create(&session).await.map_err(|e| {
                OrchestratorError::ExecutionFailed(format!("Failed to persist session: {}", e))
            })?;
        }

        // 4. Emit SessionStarted event
        if let Some(ref bus) = deps.event_bus {
            bus.publish(EventEnvelope::new(Event::SessionStarted {
                session_id: session.id,
                task_id: config.task_id,
                phase: session.phase.as_str().to_string(),
                status: session.status.as_str().to_string(),
                opencode_session_id: session.opencode_session_id.clone(),
                created_at: session.created_at,
            }));
        }

        info!(
            task_id = %config.task_id,
            session_id = %session.id,
            opencode_session_id = %opencode_session_id,
            phase = %config.phase.as_str(),
            "Session started"
        );

        let session_id = session.id;
        let result_opencode_session_id = opencode_session_id.clone();

        // 5. Spawn background execution task
        tokio::spawn(async move {
            let _ = Self::execute_and_complete(config, deps, session_id, opencode_session_id).await;
        });

        // Return immediately with session info
        Ok(SessionResult {
            session_id,
            opencode_session_id: result_opencode_session_id,
        })
    }

    /// Execute session and wait for completion - returns (success, response_text)
    ///
    /// This is the core execution logic without spawning a background task.
    /// Used internally by `start` and can be used directly for serial execution
    /// (e.g., phased implementation loops).
    ///
    /// Note: This handles SSE streaming, completion events, and artifact saving,
    /// but the caller is responsible for session creation and task status updates
    /// if different behavior is needed (e.g., updating task status only after all phases).
    pub async fn execute_and_complete(
        config: SessionConfig,
        deps: SessionDependencies,
        session_id: Uuid,
        opencode_session_id: String,
    ) -> (bool, String) {
        info!(
            task_id = %config.task_id,
            opencode_session_id = %opencode_session_id,
            phase = %config.phase.as_str(),
            "Background execution started"
        );

        // Setup activity store for real-time streaming
        let activity_store = deps
            .activity_registry
            .as_ref()
            .map(|reg| reg.get_or_create(session_id));

        // Setup SSE subscriber
        let subscriber = OpenCodeEventSubscriber::new(
            &deps.base_url,
            &opencode_session_id,
            config.working_dir.to_string_lossy().to_string(),
        );
        let mut event_rx = subscriber.subscribe();

        // Build request
        let model = opencode_client::models::SessionPromptRequestModel {
            provider_id: config.provider_id.clone(),
            model_id: config.model_id.clone(),
        };

        let request = SessionPromptRequest {
            parts: vec![Self::create_text_part(&config.prompt)],
            model: Some(Box::new(model)),
            message_id: None,
            agent: None,
            no_reply: None,
            tools: None,
            system: None,
            variant: None,
        };

        info!(
            task_id = %config.task_id,
            opencode_session_id = %opencode_session_id,
            "Sending prompt to OpenCode..."
        );

        // Spawn SSE event processor for real-time streaming
        let activity_store_for_sse = activity_store.clone();
        let opencode_session_id_for_sse = opencode_session_id.clone();
        let task_id_for_sse = config.task_id;

        let sse_task = tokio::spawn(async move {
            debug!("SSE event processor started");
            while let Some(event) = event_rx.recv().await {
                match event {
                    ExecutorEvent::SessionIdle { .. } => {
                        info!(
                            task_id = %task_id_for_sse,
                            opencode_session_id = %opencode_session_id_for_sse,
                            "Session completed (idle via SSE)"
                        );
                        break;
                    }
                    ExecutorEvent::MessagePartUpdated { part, .. } => {
                        if let Some(ref store) = activity_store_for_sse {
                            if let Some(activity) = TaskExecutor::parse_sse_part(&part) {
                                store.push(activity);
                            }
                        }
                    }
                    ExecutorEvent::DirectActivity { activity } => {
                        if let Some(ref store) = activity_store_for_sse {
                            store.push(activity);
                        }
                    }
                    ExecutorEvent::Error { message } => {
                        error!(error = %message, "SSE error during execution");
                        break;
                    }
                    ExecutorEvent::Disconnected => {
                        debug!("SSE disconnected");
                        break;
                    }
                    _ => {}
                }
            }
            debug!("SSE event processor finished");
        });

        // Send prompt asynchronously (returns immediately)
        let directory = config.working_dir.to_str();
        let send_result = default_api::session_prompt_async(
            &deps.opencode_config,
            &opencode_session_id,
            directory,
            Some(request),
        )
        .await;

        let mut success = true;
        let mut error_msg: Option<String> = None;
        let mut response_text = String::new();

        match send_result {
            Ok(()) => {
                info!(
                    task_id = %config.task_id,
                    opencode_session_id = %opencode_session_id,
                    "Prompt sent, streaming via SSE"
                );

                // Wait for SSE to signal completion
                let _ = sse_task.await;

                // Extract response text for artifacts
                response_text = Self::extract_response_text(
                    &deps.opencode_config,
                    &opencode_session_id,
                    directory,
                )
                .await;
            }
            Err(e) => {
                error!(task_id = %config.task_id, error = %e, "Failed to send prompt");
                success = false;
                error_msg = Some(e.to_string());
            }
        }

        // Mark finished in activity store
        if let Some(ref store) = activity_store {
            store.push_finished(success, error_msg.clone());
        }

        // Handle completion (update DB, save artifacts, emit events)
        Self::handle_completion(&config, &deps, session_id, &opencode_session_id, success, &response_text).await;

        info!(
            task_id = %config.task_id,
            session_id = %session_id,
            success = success,
            "Execution completed"
        );

        (success, response_text)
    }

    /// Handle post-completion tasks
    async fn handle_completion(
        config: &SessionConfig,
        deps: &SessionDependencies,
        session_id: Uuid,
        opencode_session_id: &str,
        success: bool,
        response_text: &str,
    ) {
        // Update session status in DB
        if let Some(ref repo) = deps.session_repo {
            let mut session = if let Some((phase_num, ref title)) = config.implementation_phase {
                let mut s = Session::new_implementation_phase(config.task_id, phase_num, title);
                s.id = session_id;
                s
            } else {
                let mut s = Session::new(config.task_id, config.phase);
                s.id = session_id;
                s
            };
            session.opencode_session_id = Some(opencode_session_id.to_string());
            session.status = if success {
                SessionStatus::Completed
            } else {
                SessionStatus::Failed
            };
            if let Err(e) = repo.update(&session).await {
                error!(error = %e, "Failed to update session status");
            }
        }

        // Save artifacts based on phase
        if success {
            if config.phase == SessionPhase::Planning && !response_text.is_empty() {
                if let Err(e) = deps.file_manager.write_plan(config.task_id, response_text).await {
                    error!(error = %e, "Failed to save plan");
                } else {
                    info!(task_id = %config.task_id, "Plan saved successfully");
                }
            }

            // Update task status (skip for phased implementation - handled separately)
            if !config.skip_task_status_update {
                let next_status = Self::determine_next_status(config.phase, config.task_status);
                if let Some(ref repo) = deps.task_repo {
                    let update = UpdateTaskRequest {
                        status: Some(next_status),
                        ..Default::default()
                    };
                    if let Err(e) = repo.update(config.task_id, &update).await {
                        error!(error = %e, "Failed to update task status");
                    } else {
                        info!(
                            task_id = %config.task_id,
                            new_status = %next_status.as_str(),
                            "Task status updated"
                        );
                    }
                }

                // Emit task status changed event
                if let Some(ref bus) = deps.event_bus {
                    bus.publish(EventEnvelope::new(Event::TaskStatusChanged {
                        task_id: config.task_id,
                        from_status: config.task_status.as_str().to_string(),
                        to_status: next_status.as_str().to_string(),
                    }));
                }
            }
        }

        // Emit SessionEnded event
        if let Some(ref bus) = deps.event_bus {
            bus.publish(EventEnvelope::new(Event::SessionEnded {
                session_id,
                task_id: config.task_id,
                success,
            }));
        }

        // Cleanup MCP if configured
        if let Some(ref mcp_config) = config.mcp_config {
            if mcp_config.setup_success {
                debug!("Cleaning up MCP findings server");
                if let Err(e) = default_api::mcp_disconnect(
                    &deps.opencode_config,
                    "opencode-findings",
                    mcp_config.workspace_path.to_str(),
                )
                .await
                {
                    warn!(error = %e, "Failed to disconnect MCP server");
                }
            }
        }
    }

    /// Determine next task status based on current phase
    fn determine_next_status(phase: SessionPhase, current_status: TaskStatus) -> TaskStatus {
        match phase {
            SessionPhase::Planning => TaskStatus::PlanningReview,
            SessionPhase::Implementation => TaskStatus::AiReview,
            SessionPhase::Review => {
                if current_status == TaskStatus::AiReview {
                    TaskStatus::Review
                } else {
                    TaskStatus::Done
                }
            }
            SessionPhase::Fix => TaskStatus::AiReview,
        }
    }

    /// Create OpenCode session
    async fn create_opencode_session(
        config: &Configuration,
        directory: Option<&str>,
    ) -> Result<opencode_client::models::Session> {
        let request = opencode_client::models::SessionCreateRequest {
            title: None,
            parent_id: None,
        };

        default_api::session_create(config, directory, Some(request))
            .await
            .map_err(|e| OrchestratorError::OpenCodeError(format!("Failed to create session: {}", e)))
    }

    /// Extract response text from session messages
    async fn extract_response_text(
        config: &Configuration,
        session_id: &str,
        directory: Option<&str>,
    ) -> String {
        match default_api::session_messages(config, session_id, directory, None).await {
            Ok(messages) => {
                if let Some(last_msg) = messages.iter().rev().find(|m| {
                    matches!(
                        m.info.role,
                        opencode_client::models::message::Role::Assistant
                    )
                }) {
                    TaskExecutor::extract_text_from_parts(&last_msg.parts)
                } else {
                    warn!("No assistant message found");
                    String::new()
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to fetch messages");
                String::new()
            }
        }
    }

    /// Create text part for prompt
    fn create_text_part(text: &str) -> SessionPromptRequestPartsInner {
        SessionPromptRequestPartsInner {
            r#type: opencode_client::models::session_prompt_request_parts_inner::Type::Text,
            text: text.to_string(),
            id: None,
            synthetic: None,
            ignored: None,
            time: None,
            metadata: None,
            mime: String::new(),
            filename: None,
            url: String::new(),
            source: None,
            name: String::new(),
            prompt: String::new(),
            description: String::new(),
            agent: String::new(),
            command: None,
        }
    }
}
