use db::{SessionRepository, TaskRepository};
use events::{Event, EventBus, EventEnvelope};
use opencode_client::apis::configuration::Configuration;
use opencode_client::apis::default_api;
use opencode_client::models::{
    McpAddRequest, McpAddRequestConfig,
    Part, SessionCreateRequest, SessionPromptRequest, SessionPromptRequestPartsInner,
    Session as OpenCodeSession,
};
use opencode_core::{Session, SessionPhase, Task, TaskStatus, UpdateTaskRequest};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;
use vcs::{Workspace, WorkspaceManager};

use crate::activity_store::{SessionActivityMsg, SessionActivityRegistry, SessionActivityStore};
use crate::error::{OrchestratorError, Result};
use crate::files::{
    FileManager, FindingSeverity, FindingStatus, ReviewFinding, ReviewFindings,
};
use crate::opencode_events::{ExecutorEvent, OpenCodeEventSubscriber};
use crate::prompts::PhasePrompts;
use crate::state_machine::TaskStateMachine;

/// Raw JSON response from AI review
#[derive(Debug, serde::Deserialize)]
struct RawReviewResponse {
    approved: bool,
    summary: String,
    #[serde(default)]
    findings: Vec<RawFinding>,
}

#[derive(Debug, serde::Deserialize)]
struct RawFinding {
    #[serde(default)]
    file_path: Option<String>,
    #[serde(default)]
    line_start: Option<i32>,
    #[serde(default)]
    line_end: Option<i32>,
    title: String,
    description: String,
    #[serde(default = "default_severity")]
    severity: String,
}

fn default_severity() -> String {
    "warning".to_string()
}

const DEFAULT_PROVIDER_ID: &str = "anthropic";
const DEFAULT_MODEL_ID: &str = "claude-sonnet-4-20250514";

#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    pub require_plan_approval: bool,
    pub require_human_review: bool,
    pub max_review_iterations: u32,
    pub repo_path: PathBuf,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            require_plan_approval: true,
            require_human_review: true,
            max_review_iterations: 3,
            repo_path: PathBuf::from("."),
        }
    }
}

impl ExecutorConfig {
    pub fn new(repo_path: impl Into<PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
            ..Default::default()
        }
    }

    pub fn with_plan_approval(mut self, require: bool) -> Self {
        self.require_plan_approval = require;
        self
    }

    pub fn with_human_review(mut self, require: bool) -> Self {
        self.require_human_review = require;
        self
    }

    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_review_iterations = max;
        self
    }
}

pub struct TaskExecutor {
    opencode_config: Arc<Configuration>,
    config: ExecutorConfig,
    file_manager: FileManager,
    workspace_manager: Option<Arc<WorkspaceManager>>,
    session_repo: Option<Arc<SessionRepository>>,
    task_repo: Option<Arc<TaskRepository>>,
    event_bus: Option<EventBus>,
    activity_registry: Option<SessionActivityRegistry>,
    provider_id: String,
    model_id: String,
}

/// Result returned immediately when starting async execution
#[derive(Debug, Clone)]
pub struct StartedExecution {
    /// The Studio session ID (our internal ID)
    pub session_id: Uuid,
    /// The OpenCode session ID (external)
    pub opencode_session_id: String,
    /// The phase being executed
    pub phase: SessionPhase,
}

impl TaskExecutor {
    pub fn new(opencode_config: Arc<Configuration>, config: ExecutorConfig) -> Self {
        let file_manager = FileManager::new(&config.repo_path);
        Self {
            opencode_config,
            config,
            file_manager,
            workspace_manager: None,
            session_repo: None,
            task_repo: None,
            event_bus: None,
            activity_registry: None,
            provider_id: DEFAULT_PROVIDER_ID.to_string(),
            model_id: DEFAULT_MODEL_ID.to_string(),
        }
    }

    pub fn with_model(mut self, provider_id: &str, model_id: &str) -> Self {
        self.provider_id = provider_id.to_string();
        self.model_id = model_id.to_string();
        self
    }

    pub fn with_workspace_manager(mut self, manager: Arc<WorkspaceManager>) -> Self {
        self.workspace_manager = Some(manager);
        self
    }

    pub fn with_session_repo(mut self, repo: Arc<SessionRepository>) -> Self {
        self.session_repo = Some(repo);
        self
    }

    pub fn with_task_repo(mut self, repo: Arc<TaskRepository>) -> Self {
        self.task_repo = Some(repo);
        self
    }

    pub fn with_event_bus(mut self, bus: EventBus) -> Self {
        self.event_bus = Some(bus);
        self
    }

    pub fn with_activity_registry(mut self, registry: SessionActivityRegistry) -> Self {
        self.activity_registry = Some(registry);
        self
    }

    /// Get a reference to the file manager for reading plans/reviews
    pub fn file_manager(&self) -> &FileManager {
        &self.file_manager
    }

    #[instrument(skip(self, task), fields(task_id = %task.id, task_title = %task.title))]
    pub fn transition(&self, task: &mut Task, to: TaskStatus) -> Result<()> {
        let from = task.status;
        info!(
            from = %from.as_str(),
            to = %to.as_str(),
            "Task state transition"
        );

        TaskStateMachine::validate_transition(&task.status, &to)?;
        task.status = to;
        task.updated_at = chrono::Utc::now();

        self.emit_event(Event::TaskStatusChanged {
            task_id: task.id,
            from_status: from.as_str().to_string(),
            to_status: to.as_str().to_string(),
        });

        debug!(
            task_id = %task.id,
            new_status = %to.as_str(),
            "State transition completed"
        );

        Ok(())
    }

    fn emit_event(&self, event: Event) {
        if let Some(ref bus) = self.event_bus {
            bus.publish(EventEnvelope::new(event));
        }
    }

    async fn persist_session(&self, session: &Session) -> Result<()> {
        if let Some(ref repo) = self.session_repo {
            repo.create(session).await?;
        }
        Ok(())
    }

    async fn update_session(&self, session: &Session) -> Result<()> {
        if let Some(ref repo) = self.session_repo {
            repo.update(session).await?;
        }
        Ok(())
    }

    fn extract_text_from_parts(parts: &[Part]) -> String {
        parts
            .iter()
            .filter_map(|part| {
                if part.r#type == opencode_client::models::part::Type::Text {
                    part.text.as_deref()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn extract_text_from_messages_inner(
        msg: &opencode_client::models::SessionMessages200ResponseInner,
    ) -> String {
        Self::extract_text_from_parts(&msg.parts)
    }

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

    pub fn parse_message_parts(parts: &[Part]) -> Vec<SessionActivityMsg> {
        use opencode_client::models::part::Type;

        let mut activities = Vec::new();

        for part in parts {
            match part.r#type {
                Type::Text => {
                    let id = format!("text-{}", uuid::Uuid::new_v4());
                    let text = part.text.as_deref().unwrap_or("");
                    activities.push(SessionActivityMsg::agent_message(&id, text, false));
                }
                Type::Reasoning => {
                    let id = format!("reasoning-{}", uuid::Uuid::new_v4());
                    activities.push(SessionActivityMsg::Reasoning {
                        id,
                        content: part.text.clone().unwrap_or_default(),
                        timestamp: chrono::Utc::now(),
                    });
                }
                Type::Tool => {
                    let call_id = part.call_id.as_deref().unwrap_or("");
                    let tool_name = part.tool.as_deref().unwrap_or("unknown");

                    if let Some(ref state) = part.state {
                        let output = state.output.as_deref().unwrap_or("");
                        let error = state.error.as_deref().unwrap_or("");
                        let is_completed = !output.is_empty() || !error.is_empty();

                        if is_completed {
                            let success = error.is_empty();
                            let result = if success { output } else { error };
                            activities.push(SessionActivityMsg::tool_result(
                                call_id,
                                tool_name,
                                None,
                                result,
                                success,
                            ));
                        } else {
                            activities.push(SessionActivityMsg::tool_call(
                                call_id,
                                tool_name,
                                None,
                            ));
                        }
                    } else {
                        // No state yet, treat as pending tool call
                        activities.push(SessionActivityMsg::tool_call(
                            call_id,
                            tool_name,
                            None,
                        ));
                    }
                }
                Type::StepStart => {
                    let id = format!("step-{}", uuid::Uuid::new_v4());
                    activities.push(SessionActivityMsg::StepStart {
                        id,
                        step_name: None,
                        timestamp: chrono::Utc::now(),
                    });
                }
                _ => {
                    debug!("Skipping part type: {:?}", part.r#type);
                }
            }
        }

        activities
    }

    /// Parse SSE part format (different from HTTP response Part struct)
    /// SSE parts have: id, messageID, sessionID, text, time.start/end, type
    /// Tool parts have: callID, tool, state.status/input/output/error
    fn parse_sse_part(part: &serde_json::Value) -> Option<SessionActivityMsg> {
        let part_type = part.get("type")?.as_str()?;
        let id = part.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");

        match part_type {
            "text" => {
                let text = part.get("text").and_then(|v| v.as_str()).unwrap_or("");
                // Check if this is a partial or complete message
                let is_partial = part.get("time")
                    .and_then(|t| t.get("end"))
                    .is_none();
                Some(SessionActivityMsg::agent_message(id, text, is_partial))
            }
            "reasoning" => {
                let content = part.get("text").and_then(|v| v.as_str()).unwrap_or("");
                Some(SessionActivityMsg::Reasoning {
                    id: id.to_string(),
                    content: content.to_string(),
                    timestamp: chrono::Utc::now(),
                })
            }
            "tool" => {
                let call_id = part.get("callID").and_then(|v| v.as_str()).unwrap_or(id);
                let tool_name = part.get("tool").and_then(|v| v.as_str()).unwrap_or("unknown");
                let state = part.get("state");

                let status = state
                    .and_then(|s| s.get("status"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("pending");

                // If status is "completed" or "error", it's a finished tool call
                // OpenCode uses "completed" for success, not "success"
                if status == "completed" || status == "error" {
                    let success = status == "completed";
                    let output = state
                        .and_then(|s| s.get("output"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let error = state
                        .and_then(|s| s.get("error"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let result = if success { output } else { error };

                    Some(SessionActivityMsg::tool_result(
                        call_id,
                        tool_name,
                        None,
                        result,
                        success,
                    ))
                } else {
                    // Pending or running - emit tool call
                    Some(SessionActivityMsg::tool_call(call_id, tool_name, None))
                }
            }
            "step-start" => {
                Some(SessionActivityMsg::StepStart {
                    id: id.to_string(),
                    step_name: None,
                    timestamp: chrono::Utc::now(),
                })
            }
            _ => {
                debug!(part_type = %part_type, "Skipping unknown SSE part type");
                None
            }
        }
    }

    fn push_activities_to_store(&self, store: &SessionActivityStore, parts: &[Part]) {
        for activity in Self::parse_message_parts(parts) {
            store.push(activity);
        }
    }

    fn get_activity_store(&self, session_id: Uuid) -> Option<Arc<SessionActivityStore>> {
        self.activity_registry
            .as_ref()
            .map(|reg| reg.get_or_create(session_id))
    }

    async fn create_opencode_session(&self) -> Result<OpenCodeSession> {
        self.create_opencode_session_in_dir(&self.config.repo_path).await
    }

    async fn create_opencode_session_in_dir(&self, working_dir: &std::path::Path) -> Result<OpenCodeSession> {
        let request = SessionCreateRequest {
            title: None,
            parent_id: None,
        };

        // Pass the working directory so OpenCode works in the correct context
        let directory = working_dir.to_str();
        info!(
            directory = ?directory,
            "Creating OpenCode session in directory"
        );

        default_api::session_create(&self.opencode_config, directory, Some(request))
            .await
            .map_err(|e| {
                error!(error = %e, directory = ?directory, "Failed to create OpenCode session");
                OrchestratorError::OpenCodeError(format!("Failed to create session: {}", e))
            })
    }

    /// Add the MCP findings server to OpenCode for a specific review session
    async fn add_mcp_findings_server(
        &self,
        task_id: Uuid,
        session_id: Uuid,
        workspace_path: &std::path::Path,
    ) -> Result<()> {
        let mcp_binary = self.get_mcp_binary_path();

        let mut environment = std::collections::HashMap::new();
        environment.insert("OPENCODE_TASK_ID".to_string(), task_id.to_string());
        environment.insert("OPENCODE_SESSION_ID".to_string(), session_id.to_string());
        environment.insert(
            "OPENCODE_WORKSPACE_PATH".to_string(),
            workspace_path.to_string_lossy().to_string(),
        );

        let mut config = McpAddRequestConfig::local(vec![mcp_binary]);
        config.environment = Some(environment);
        config.enabled = Some(true);
        config.timeout = Some(10000); // 10 seconds

        let request = McpAddRequest::new("opencode-findings".to_string(), config);
        let directory = workspace_path.to_str();

        info!(
            task_id = %task_id,
            session_id = %session_id,
            "Adding MCP findings server to OpenCode"
        );

        default_api::mcp_add(&self.opencode_config, directory, Some(request))
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to add MCP findings server");
                OrchestratorError::OpenCodeError(format!("Failed to add MCP server: {}", e))
            })?;

        // Connect the MCP server
        default_api::mcp_connect(&self.opencode_config, "opencode-findings", directory)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to connect MCP findings server");
                OrchestratorError::OpenCodeError(format!("Failed to connect MCP server: {}", e))
            })?;

        info!("MCP findings server connected");
        Ok(())
    }

    /// Remove the MCP findings server from OpenCode
    async fn remove_mcp_findings_server(&self, workspace_path: &std::path::Path) -> Result<()> {
        let directory = workspace_path.to_str();

        info!("Disconnecting MCP findings server");

        // Disconnect the MCP server (ignore errors - server might already be disconnected)
        if let Err(e) = default_api::mcp_disconnect(&self.opencode_config, "opencode-findings", directory).await {
            warn!(error = %e, "Failed to disconnect MCP findings server (may already be disconnected)");
        }

        Ok(())
    }

    /// Get the path to the MCP findings binary
    fn get_mcp_binary_path(&self) -> String {
        // In development, use the debug build path
        // In production, this would be installed alongside the main binary
        if cfg!(debug_assertions) {
            // Try to find the binary relative to the current executable
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(parent) = exe_path.parent() {
                    let mcp_path = parent.join("opencode-mcp-findings");
                    if mcp_path.exists() {
                        return mcp_path.to_string_lossy().to_string();
                    }
                }
            }
        }
        // Fall back to assuming it's in PATH
        "opencode-mcp-findings".to_string()
    }

    async fn send_opencode_message_with_activity(
        &self,
        session_id: &str,
        prompt: &str,
        activity_store: Option<&SessionActivityStore>,
    ) -> Result<String> {
        self.send_opencode_message_in_dir(session_id, prompt, activity_store, &self.config.repo_path).await
    }

    async fn send_opencode_message_in_dir(
        &self,
        session_id: &str,
        prompt: &str,
        activity_store: Option<&SessionActivityStore>,
        working_dir: &std::path::Path,
    ) -> Result<String> {
        let model = opencode_client::models::SessionPromptRequestModel {
            provider_id: self.provider_id.clone(),
            model_id: self.model_id.clone(),
        };

        let request = SessionPromptRequest {
            parts: vec![Self::create_text_part(prompt)],
            model: Some(Box::new(model)),
            message_id: None,
            agent: None,
            no_reply: None,
            tools: None,
            system: None,
            variant: None,
        };

        let directory = working_dir.to_str();
        let response =
            default_api::session_prompt(&self.opencode_config, session_id, directory, Some(request))
                .await
                .map_err(|e| {
                    error!(error = %e, directory = ?directory, "Failed to send message to OpenCode");
                    OrchestratorError::OpenCodeError(format!("Failed to send message: {}", e))
                })?;

        if let Some(store) = activity_store {
            self.push_activities_to_store(store, &response.parts);
        }

        Ok(Self::extract_text_from_parts(&response.parts))
    }

    #[instrument(skip(self, task), fields(task_id = %task.id, status = %task.status.as_str()))]
    pub async fn execute_phase(&self, task: &mut Task) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            current_status = %task.status.as_str(),
            "Executing phase for task"
        );

        let result = match task.status {
            TaskStatus::Todo => {
                debug!("Task in TODO, transitioning to PLANNING");
                self.transition(task, TaskStatus::Planning)?;
                self.run_planning_session(task).await
            }
            TaskStatus::Planning => {
                debug!("Task in PLANNING, running planning session");
                self.run_planning_session(task).await
            }
            TaskStatus::PlanningReview => {
                if self.config.require_plan_approval {
                    info!("Plan requires approval, awaiting human review");
                    Ok(PhaseResult::AwaitingApproval {
                        phase: SessionPhase::Planning,
                    })
                } else {
                    debug!("Auto-approving plan, transitioning to IN_PROGRESS");
                    self.transition(task, TaskStatus::InProgress)?;
                    self.run_implementation_session(task).await
                }
            }
            TaskStatus::InProgress => {
                debug!("Task IN_PROGRESS, running implementation session");
                self.run_implementation_session(task).await
            }
            TaskStatus::AiReview => {
                debug!("Task in AI_REVIEW, running AI review");
                self.run_ai_review(task, 0).await
            }
            TaskStatus::Fix => {
                debug!("Task in FIX, running fix session");
                self.run_fix_session(task).await
            }
            TaskStatus::Review => {
                if self.config.require_human_review {
                    info!("Implementation requires human review, awaiting approval");
                    Ok(PhaseResult::AwaitingApproval {
                        phase: SessionPhase::Review,
                    })
                } else {
                    debug!("Auto-approving review, transitioning to DONE");
                    self.transition(task, TaskStatus::Done)?;
                    Ok(PhaseResult::Completed)
                }
            }
            TaskStatus::Done => {
                debug!("Task already DONE");
                Ok(PhaseResult::Completed)
            }
        };

        match &result {
            Ok(phase_result) => info!(
                task_id = %task.id,
                result = ?phase_result,
                "Phase execution completed"
            ),
            Err(e) => error!(
                task_id = %task.id,
                error = %e,
                "Phase execution failed"
            ),
        }

        result
    }

    pub async fn run_full_cycle(&self, task: &mut Task) -> Result<PhaseResult> {
        info!("Starting full cycle for task {}: {}", task.id, task.title);

        if task.status == TaskStatus::Done {
            return Ok(PhaseResult::Completed);
        }

        if task.status == TaskStatus::Todo {
            self.transition(task, TaskStatus::Planning)?;
        }

        if task.status == TaskStatus::Planning {
            let result = self.run_planning_session(task).await?;
            if self.config.require_plan_approval {
                return Ok(result);
            }
        }

        if task.status == TaskStatus::PlanningReview {
            self.transition(task, TaskStatus::InProgress)?;
        }

        if task.status == TaskStatus::InProgress {
            self.run_implementation_session(task).await?;
        }

        let mut iteration = 0;
        while task.status == TaskStatus::AiReview && iteration < self.config.max_review_iterations {
            let result = self.run_ai_review(task, iteration).await?;
            match result {
                PhaseResult::ReviewPassed { .. } => {
                    if self.config.require_human_review {
                        return Ok(PhaseResult::AwaitingApproval {
                            phase: SessionPhase::Review,
                        });
                    } else {
                        self.transition(task, TaskStatus::Done)?;
                        return Ok(PhaseResult::Completed);
                    }
                }
                PhaseResult::ReviewFailed { feedback, .. } => {
                    info!(
                        "AI review failed (iteration {}), running fix iteration",
                        iteration
                    );
                    self.run_fix_iteration(task, &feedback).await?;
                    iteration += 1;
                }
                _ => return Ok(result),
            }
        }

        if iteration >= self.config.max_review_iterations {
            warn!(
                "Task {} exceeded max review iterations ({})",
                task.id, self.config.max_review_iterations
            );
            return Ok(PhaseResult::MaxIterationsExceeded {
                iterations: iteration,
            });
        }

        if task.status == TaskStatus::Review {
            if self.config.require_human_review {
                return Ok(PhaseResult::AwaitingApproval {
                    phase: SessionPhase::Review,
                });
            }
            self.transition(task, TaskStatus::Done)?;
        }

        Ok(PhaseResult::Completed)
    }

    pub async fn start_phase_async(&self, task: &mut Task) -> Result<StartedExecution> {
        info!(
            task_id = %task.id,
            current_status = %task.status.as_str(),
            "Starting async phase execution"
        );

        let phase = match task.status {
            TaskStatus::Todo | TaskStatus::Planning => SessionPhase::Planning,
            TaskStatus::PlanningReview | TaskStatus::InProgress => SessionPhase::Implementation,
            TaskStatus::AiReview => SessionPhase::Review,
            TaskStatus::Fix => SessionPhase::Fix,
            TaskStatus::Review => SessionPhase::Review,
            TaskStatus::Done => {
                return Err(OrchestratorError::ExecutionFailed(
                    "Task is already done".to_string(),
                ));
            }
        };

        if task.status == TaskStatus::Todo {
            self.transition(task, TaskStatus::Planning)?;
        }

        // For implementation phase, create workspace if not already created
        if phase == SessionPhase::Implementation && task.workspace_path.is_none() {
            if let Some(ref wm) = self.workspace_manager {
                debug!("Setting up VCS workspace for async task execution");
                match wm.setup_workspace(&task.id.to_string()).await {
                    Ok(workspace) => {
                        task.workspace_path = Some(workspace.path.to_string_lossy().to_string());
                        info!(
                            workspace_path = %workspace.path.display(),
                            branch = %workspace.branch_name,
                            "VCS workspace created for async execution"
                        );
                        self.emit_event(Event::WorkspaceCreated {
                            task_id: task.id,
                            path: workspace.path.to_string_lossy().to_string(),
                        });
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to setup workspace for async execution");
                        return Err(OrchestratorError::ExecutionFailed(
                            format!("Failed to setup workspace: {}", e)
                        ));
                    }
                }
                // Persist workspace_path to database
                if let Some(ref repo) = self.task_repo {
                    let update = UpdateTaskRequest {
                        workspace_path: task.workspace_path.clone(),
                        ..Default::default()
                    };
                    if let Err(e) = repo.update(task.id, &update).await {
                        error!(error = %e, "Failed to persist workspace_path to database");
                    }
                }
            } else {
                warn!("No workspace manager configured, implementation will run in root directory");
            }
        }

        // Determine working directory - use workspace if available, otherwise root
        let working_dir = task.workspace_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.config.repo_path.clone());

        let opencode_session = self.create_opencode_session_in_dir(&working_dir).await?;
        let opencode_session_id = opencode_session.id.to_string();

        let mut session = Session::new(task.id, phase);
        session.start(opencode_session_id.clone());
        self.persist_session(&session).await?;

        self.emit_event(Event::SessionStarted {
            session_id: session.id,
            task_id: task.id,
            phase: session.phase.as_str().to_string(),
            status: session.status.as_str().to_string(),
            opencode_session_id: session.opencode_session_id.clone(),
            created_at: session.created_at,
        });

        // For review phase, set up MCP server before spawning background task
        let mcp_setup_success = if phase == SessionPhase::Review && task.status == TaskStatus::AiReview {
            match self
                .add_mcp_findings_server(task.id, session.id, &working_dir)
                .await
            {
                Ok(_) => {
                    info!(task_id = %task.id, "MCP findings server added for async review");
                    true
                }
                Err(e) => {
                    warn!(error = %e, "Failed to add MCP server, falling back to JSON parsing");
                    false
                }
            }
        } else {
            false
        };

        let prompt = match phase {
            SessionPhase::Planning => PhasePrompts::planning(task),
            SessionPhase::Implementation => {
                let plan = if self.file_manager.plan_exists(task.id).await {
                    self.file_manager.read_plan(task.id).await.ok()
                } else {
                    None
                };
                PhasePrompts::implementation_with_plan(task, plan.as_deref())
            }
            SessionPhase::Review => {
                let diff = self.get_workspace_diff(task).await.unwrap_or_default();
                if mcp_setup_success {
                    PhasePrompts::review_with_mcp(task, &diff)
                } else {
                    PhasePrompts::review(task, &diff)
                }
            }
            SessionPhase::Fix => {
                // Fix phase uses MCP to read findings and fix them
                PhasePrompts::fix_with_mcp(task)
            }
        };

        let task_id = task.id;
        let task_status = task.status;
        let studio_session_id = session.id;
        let opencode_session_id_clone = opencode_session_id.clone();
        let opencode_config = Arc::clone(&self.opencode_config);
        let session_repo = self.session_repo.clone();
        let task_repo = self.task_repo.clone();
        let event_bus = self.event_bus.clone();
        let activity_registry = self.activity_registry.clone();
        let file_manager = self.file_manager.clone();
        let provider_id = self.provider_id.clone();
        let model_id = self.model_id.clone();
        let base_url = opencode_config
            .base_path
            .trim_end_matches("/api")
            .to_string();
        // Use working_dir (workspace if available, otherwise root)
        let execution_dir = working_dir.clone();

        let mcp_workspace = if mcp_setup_success {
            Some(working_dir.clone())
        } else {
            None
        };

        tokio::spawn(async move {
            Self::run_background_execution(
                opencode_config,
                opencode_session_id_clone,
                studio_session_id,
                task_id,
                task_status,
                phase,
                prompt,
                session_repo,
                task_repo,
                event_bus,
                activity_registry,
                file_manager,
                provider_id,
                model_id,
                base_url,
                execution_dir,
                mcp_workspace,
            )
            .await
        });

        info!(
            task_id = %task.id,
            session_id = %session.id,
            opencode_session_id = %opencode_session_id,
            "Async execution started, returning immediately"
        );

        Ok(StartedExecution {
            session_id: session.id,
            opencode_session_id,
            phase,
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_background_execution(
        opencode_config: Arc<Configuration>,
        opencode_session_id: String,
        studio_session_id: Uuid,
        task_id: Uuid,
        initial_task_status: TaskStatus,
        phase: SessionPhase,
        prompt: String,
        session_repo: Option<Arc<SessionRepository>>,
        task_repo: Option<Arc<TaskRepository>>,
        event_bus: Option<EventBus>,
        activity_registry: Option<SessionActivityRegistry>,
        file_manager: FileManager,
        provider_id: String,
        model_id: String,
        base_url: String,
        repo_path: PathBuf,
        mcp_workspace: Option<PathBuf>,
    ) {
        info!(
            task_id = %task_id,
            opencode_session_id = %opencode_session_id,
            phase = %phase.as_str(),
            "Background execution started"
        );

        let activity_store = activity_registry
            .as_ref()
            .map(|reg| reg.get_or_create(studio_session_id));

        let subscriber = OpenCodeEventSubscriber::new(
            &base_url,
            &opencode_session_id,
            repo_path.to_string_lossy().to_string(),
        );
        let mut event_rx = subscriber.subscribe();

        let model = opencode_client::models::SessionPromptRequestModel {
            provider_id,
            model_id,
        };

        let request = SessionPromptRequest {
            parts: vec![Self::create_text_part(&prompt)],
            model: Some(Box::new(model)),
            message_id: None,
            agent: None,
            no_reply: None,
            tools: None,
            system: None,
            variant: None,
        };

        info!(
            task_id = %task_id,
            opencode_session_id = %opencode_session_id,
            "Sending prompt to OpenCode (this may take a while)..."
        );

        // Spawn a task to process SSE events
        // We use session_prompt_async which returns immediately, then wait for
        // SSE to signal completion via session.idle event
        let activity_store_for_sse = activity_store.clone();
        let opencode_session_id_for_sse = opencode_session_id.clone();
        let task_id_for_sse = task_id;
        let sse_task = tokio::spawn(async move {
            debug!("SSE event processor started");
            while let Some(event) = event_rx.recv().await {
                match event {
                    ExecutorEvent::SessionIdle { .. } => {
                        info!(
                            task_id = %task_id_for_sse,
                            opencode_session_id = %opencode_session_id_for_sse,
                            "OpenCode session completed (idle via SSE)"
                        );
                        break;
                    }
                    ExecutorEvent::MessagePartUpdated { session_id: event_session_id, part, .. } => {
                        debug!(
                            event_session_id = %event_session_id,
                            our_session_id = %opencode_session_id_for_sse,
                            "Received MessagePartUpdated event"
                        );
                        if let Some(ref store) = activity_store_for_sse {
                            // Parse SSE part format directly (different from HTTP response Part struct)
                            if let Some(activity) = TaskExecutor::parse_sse_part(&part) {
                                debug!("Parsed activity from SSE part");
                                store.push(activity);
                            }
                        } else {
                            warn!("activity_store is None, cannot store activity");
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
                    ExecutorEvent::DirectActivity { activity } => {
                        debug!(activity_type = ?std::mem::discriminant(&activity), "Received direct activity event");
                        if let Some(ref store) = activity_store_for_sse {
                            store.push(activity);
                        } else {
                            warn!("activity_store is None, cannot store direct activity");
                        }
                    }
                    _ => {}
                }
            }
            debug!("SSE event processor finished");
        });

        // Use session_prompt_async which returns immediately
        // We rely on SSE to monitor progress and detect completion
        // Pass the project directory so OpenCode works in the correct context
        let directory = repo_path.to_str();
        let send_result = default_api::session_prompt_async(
            &opencode_config,
            &opencode_session_id,
            directory,
            Some(request),
        )
        .await;

        let mut success = true;
        let mut error_msg = None;

        match send_result {
            Ok(()) => {
                info!(
                    task_id = %task_id,
                    opencode_session_id = %opencode_session_id,
                    "OpenCode prompt sent, waiting for completion via SSE"
                );

                // Wait for SSE task to finish (it will receive idle event when done)
                let _ = sse_task.await;
                info!(task_id = %task_id, "Background execution completed successfully");

                // After completion, fetch messages and save artifacts (plan, review) if needed
                if phase == SessionPhase::Planning {
                    match default_api::session_messages(
                        &opencode_config,
                        &opencode_session_id,
                        directory,
                        None,
                    )
                    .await
                    {
                        Ok(messages) => {
                            // Find the last assistant message and extract text
                            if let Some(last_msg) = messages.iter().rev().find(|m| {
                                matches!(
                                    m.info.role,
                                    opencode_client::models::message::Role::Assistant
                                )
                            }) {
                                let plan_content = Self::extract_text_from_messages_inner(last_msg);
                                if !plan_content.is_empty() {
                                    match file_manager.write_plan(task_id, &plan_content).await {
                                        Ok(path) => {
                                            info!(
                                                task_id = %task_id,
                                                plan_path = %path.display(),
                                                "Plan saved successfully"
                                            );
                                        }
                                        Err(e) => {
                                            error!(
                                                task_id = %task_id,
                                                error = %e,
                                                "Failed to save plan file"
                                            );
                                        }
                                    }
                                } else {
                                    warn!(task_id = %task_id, "No text content found in assistant message");
                                }
                            } else {
                                warn!(task_id = %task_id, "No assistant message found in session");
                            }
                        }
                        Err(e) => {
                            error!(
                                task_id = %task_id,
                                error = %e,
                                "Failed to fetch session messages for plan extraction"
                            );
                        }
                    }
                }
            }
            Err(e) => {
                error!(task_id = %task_id, error = %e, "Failed to send prompt to OpenCode");
                success = false;
                error_msg = Some(e.to_string());
            }
        }

        if let Some(ref store) = activity_store {
            store.push_finished(success, error_msg.clone());
        }

        if let Some(ref repo) = session_repo {
            let mut session = Session::new(task_id, phase);
            session.id = studio_session_id;
            session.opencode_session_id = Some(opencode_session_id.clone());
            session.status = if success {
                opencode_core::SessionStatus::Completed
            } else {
                opencode_core::SessionStatus::Failed
            };
            if let Err(e) = repo.update(&session).await {
                error!(error = %e, "Failed to update session status");
            }
        }

        if success {
            let next_status = match phase {
                SessionPhase::Planning => TaskStatus::PlanningReview,
                SessionPhase::Implementation => TaskStatus::AiReview,
                SessionPhase::Review => {
                    if initial_task_status == TaskStatus::AiReview {
                        TaskStatus::Review
                    } else {
                        TaskStatus::Done
                    }
                }
                // Fix phase transitions back to AiReview for re-evaluation
                SessionPhase::Fix => TaskStatus::AiReview,
            };

            if let Some(ref repo) = task_repo {
                let update = UpdateTaskRequest {
                    status: Some(next_status),
                    ..Default::default()
                };
                if let Err(e) = repo.update(task_id, &update).await {
                    error!(error = %e, "Failed to update task status");
                } else {
                    info!(
                        task_id = %task_id,
                        new_status = %next_status.as_str(),
                        "Task status updated after background execution"
                    );
                }
            }

            if let Some(ref bus) = event_bus {
                bus.publish(EventEnvelope::new(Event::TaskStatusChanged {
                    task_id,
                    from_status: initial_task_status.as_str().to_string(),
                    to_status: next_status.as_str().to_string(),
                }));
            }
        }

        if let Some(ref bus) = event_bus {
            bus.publish(EventEnvelope::new(Event::SessionEnded {
                session_id: studio_session_id,
                task_id,
                success,
            }));
        }

        // Cleanup MCP server if it was set up
        if let Some(workspace_path) = mcp_workspace {
            debug!("Cleaning up MCP findings server");
            if let Err(e) = default_api::mcp_disconnect(
                &opencode_config,
                "opencode-findings",
                workspace_path.to_str(),
            )
            .await
            {
                warn!(error = %e, "Failed to disconnect MCP findings server");
            }
        }
    }

    #[instrument(skip(self, task), fields(task_id = %task.id))]
    async fn run_planning_session(&self, task: &mut Task) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            task_title = %task.title,
            "Starting PLANNING session"
        );

        let mut session = Session::new(task.id, SessionPhase::Planning);

        debug!("Creating OpenCode session for planning");
        let opencode_session = self.create_opencode_session().await?;
        let session_id_str = opencode_session.id.to_string();

        info!(
            opencode_session_id = %session_id_str,
            "OpenCode session created"
        );

        session.start(session_id_str.clone());
        self.persist_session(&session).await?;

        let activity_store = self.get_activity_store(session.id);

        self.emit_event(Event::SessionStarted {
            session_id: session.id,
            task_id: task.id,
            phase: session.phase.as_str().to_string(),
            status: session.status.as_str().to_string(),
            opencode_session_id: session.opencode_session_id.clone(),
            created_at: session.created_at,
        });

        debug!("Generating planning prompt");
        let prompt = PhasePrompts::planning(task);
        debug!(prompt_length = prompt.len(), "Sending planning prompt to OpenCode");

        let response_content = self
            .send_opencode_message_with_activity(
                &session_id_str,
                &prompt,
                activity_store.as_deref(),
            )
            .await;

        let response_content = match response_content {
            Ok(content) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(true, None);
                }
                content
            }
            Err(e) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(false, Some(e.to_string()));
                }
                return Err(e);
            }
        };

        info!(
            response_length = response_content.len(),
            "Received planning response"
        );

        let plan_path = self
            .file_manager
            .write_plan(task.id, &response_content)
            .await?;

        info!(plan_path = %plan_path.display(), "Plan saved to file");

        session.complete();
        self.update_session(&session).await?;

        self.emit_event(Event::SessionEnded {
            session_id: session.id,
            task_id: task.id,
            success: true,
        });

        self.transition(task, TaskStatus::PlanningReview)?;

        info!(
            task_id = %task.id,
            "PLANNING session completed, awaiting review"
        );

        Ok(PhaseResult::PlanCreated {
            session_id: session_id_str,
            plan_path: plan_path.to_string_lossy().to_string(),
        })
    }

    #[instrument(skip(self, task), fields(task_id = %task.id))]
    async fn run_implementation_session(&self, task: &mut Task) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            task_title = %task.title,
            "Starting IMPLEMENTATION session"
        );

        let mut session = Session::new(task.id, SessionPhase::Implementation);

        if let Some(ref wm) = self.workspace_manager {
            debug!("Setting up VCS workspace for task");
            let workspace = wm.setup_workspace(&task.id.to_string()).await.map_err(|e| {
                error!(error = %e, "Failed to setup workspace");
                OrchestratorError::ExecutionFailed(format!("Failed to setup workspace: {}", e))
            })?;
            task.workspace_path = Some(workspace.path.to_string_lossy().to_string());

            info!(
                workspace_path = %workspace.path.display(),
                branch = %workspace.branch_name,
                "VCS workspace created"
            );

            self.emit_event(Event::WorkspaceCreated {
                task_id: task.id,
                path: workspace.path.to_string_lossy().to_string(),
            });
        } else {
            warn!("No workspace manager configured, skipping VCS workspace setup");
        }

        // Determine working directory - use workspace if available, otherwise root
        let working_dir = task.workspace_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.config.repo_path.clone());

        debug!(
            working_dir = %working_dir.display(),
            has_workspace = task.workspace_path.is_some(),
            "Creating OpenCode session for implementation"
        );
        let opencode_session = self.create_opencode_session_in_dir(&working_dir).await?;
        let session_id_str = opencode_session.id.to_string();

        info!(
            opencode_session_id = %session_id_str,
            working_dir = %working_dir.display(),
            "OpenCode session created for implementation"
        );

        session.start(session_id_str.clone());
        self.persist_session(&session).await?;

        let activity_store = self.get_activity_store(session.id);

        self.emit_event(Event::SessionStarted {
            session_id: session.id,
            task_id: task.id,
            phase: session.phase.as_str().to_string(),
            status: session.status.as_str().to_string(),
            opencode_session_id: session.opencode_session_id.clone(),
            created_at: session.created_at,
        });

        let plan = if self.file_manager.plan_exists(task.id).await {
            debug!("Loading existing plan for implementation");
            self.file_manager.read_plan(task.id).await.ok()
        } else {
            debug!("No existing plan found, proceeding without plan");
            None
        };

        debug!(
            has_plan = plan.is_some(),
            "Generating implementation prompt"
        );
        let prompt = PhasePrompts::implementation_with_plan(task, plan.as_deref());
        debug!(prompt_length = prompt.len(), "Sending implementation prompt to OpenCode");

        let response = self
            .send_opencode_message_in_dir(
                &session_id_str,
                &prompt,
                activity_store.as_deref(),
                &working_dir,
            )
            .await;

        match response {
            Ok(_) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(true, None);
                }
            }
            Err(e) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(false, Some(e.to_string()));
                }
                return Err(e);
            }
        }

        info!("Implementation response received from OpenCode");

        session.complete();
        self.update_session(&session).await?;

        self.emit_event(Event::SessionEnded {
            session_id: session.id,
            task_id: task.id,
            success: true,
        });

        self.transition(task, TaskStatus::AiReview)?;

        info!(
            task_id = %task.id,
            "IMPLEMENTATION session completed, proceeding to AI review"
        );

        Ok(PhaseResult::SessionCreated {
            session_id: session_id_str,
        })
    }

    #[instrument(skip(self, task), fields(task_id = %task.id, iteration = iteration))]
    async fn run_ai_review(&self, task: &mut Task, iteration: u32) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            iteration = iteration,
            max_iterations = self.config.max_review_iterations,
            "Starting AI_REVIEW session with MCP"
        );

        let mut session = Session::new(task.id, SessionPhase::Review);

        debug!("Creating OpenCode session for AI review");
        let opencode_session = self.create_opencode_session().await?;
        let session_id_str = opencode_session.id.to_string();

        info!(
            opencode_session_id = %session_id_str,
            "OpenCode session created for AI review"
        );

        session.start(session_id_str.clone());
        self.persist_session(&session).await?;

        let activity_store = self.get_activity_store(session.id);

        self.emit_event(Event::SessionStarted {
            session_id: session.id,
            task_id: task.id,
            phase: session.phase.as_str().to_string(),
            status: session.status.as_str().to_string(),
            opencode_session_id: session.opencode_session_id.clone(),
            created_at: session.created_at,
        });

        // Get workspace path for MCP server
        let workspace_path = task
            .workspace_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.config.repo_path.clone());

        // Add MCP findings server to OpenCode
        if let Err(e) = self
            .add_mcp_findings_server(task.id, session.id, &workspace_path)
            .await
        {
            warn!(error = %e, "Failed to add MCP server, falling back to JSON parsing");
            // Fall back to non-MCP review if MCP server fails to start
            return self.run_ai_review_json_fallback(task, session, session_id_str, activity_store, iteration).await;
        }

        debug!("Getting workspace diff for review");
        let diff = self.get_workspace_diff(task).await?;
        debug!(diff_length = diff.len(), "Workspace diff retrieved");

        // Use MCP-based prompt
        let prompt = PhasePrompts::review_with_mcp(task, &diff);
        debug!(prompt_length = prompt.len(), "Sending MCP review prompt to OpenCode");

        let response_content = self
            .send_opencode_message_with_activity(
                &session_id_str,
                &prompt,
                activity_store.as_deref(),
            )
            .await;

        // Disconnect MCP server after review (ignore errors)
        let _ = self.remove_mcp_findings_server(&workspace_path).await;

        let response_content = match response_content {
            Ok(content) => content,
            Err(e) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(false, Some(e.to_string()));
                }
                return Err(e);
            }
        };

        info!(
            response_length = response_content.len(),
            "Received AI review response"
        );

        // Save raw review for reference
        let _review_path = self
            .file_manager
            .write_review(task.id, &response_content)
            .await?;

        session.complete();
        self.update_session(&session).await?;

        // Read findings from file (written by MCP server)
        let review_result = match self.file_manager.read_findings(task.id).await {
            Ok(Some(findings)) => {
                info!(
                    approved = findings.approved,
                    finding_count = findings.findings.len(),
                    "AI review findings read from MCP server"
                );

                if findings.approved || findings.findings.is_empty() {
                    ReviewResult::Approved
                } else {
                    ReviewResult::FindingsDetected(findings.findings.len())
                }
            }
            Ok(None) => {
                // No findings file - try to parse from response as fallback
                warn!("No MCP findings file found, falling back to JSON parsing");
                match Self::parse_review_json(&response_content, task.id, session.id) {
                    Ok(findings) => {
                        self.file_manager.write_findings(task.id, &findings).await?;
                        if findings.approved || findings.findings.is_empty() {
                            ReviewResult::Approved
                        } else {
                            ReviewResult::FindingsDetected(findings.findings.len())
                        }
                    }
                    Err(_) => Self::parse_review_response(&response_content),
                }
            }
            Err(e) => {
                warn!(error = %e, "Failed to read MCP findings, falling back to JSON parsing");
                match Self::parse_review_json(&response_content, task.id, session.id) {
                    Ok(findings) => {
                        self.file_manager.write_findings(task.id, &findings).await?;
                        if findings.approved || findings.findings.is_empty() {
                            ReviewResult::Approved
                        } else {
                            ReviewResult::FindingsDetected(findings.findings.len())
                        }
                    }
                    Err(_) => Self::parse_review_response(&response_content),
                }
            }
        };

        let success = matches!(review_result, ReviewResult::Approved);

        if let Some(ref store) = activity_store {
            store.push_finished(success, None);
        }

        info!(
            review_result = ?review_result,
            "AI review result processed"
        );

        self.emit_event(Event::SessionEnded {
            session_id: session.id,
            task_id: task.id,
            success,
        });

        match review_result {
            ReviewResult::Approved => {
                info!(task_id = %task.id, "AI review APPROVED, proceeding to human review");
                self.transition(task, TaskStatus::Review)?;
                Ok(PhaseResult::ReviewPassed {
                    session_id: session_id_str,
                })
            }
            ReviewResult::FindingsDetected(count) => {
                info!(
                    task_id = %task.id,
                    finding_count = count,
                    "AI review found issues, waiting for user action"
                );
                // Stay in ai_review state - user must choose to fix or skip
                Ok(PhaseResult::ReviewFailed {
                    session_id: session_id_str,
                    feedback: format!("{} issues found. Review findings and choose to fix or skip.", count),
                    iteration,
                })
            }
            ReviewResult::ChangesRequested(feedback) => {
                warn!(
                    task_id = %task.id,
                    iteration = iteration,
                    feedback_preview = %feedback.chars().take(200).collect::<String>(),
                    "AI review REJECTED (legacy format), changes requested"
                );
                // Legacy behavior - auto transition to InProgress
                self.transition(task, TaskStatus::InProgress)?;
                Ok(PhaseResult::ReviewFailed {
                    session_id: session_id_str,
                    feedback,
                    iteration,
                })
            }
        }
    }

    /// Run a fix session to address findings from AI review
    #[instrument(skip(self, task), fields(task_id = %task.id))]
    async fn run_fix_session(&self, task: &mut Task) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            "Starting FIX session with MCP"
        );

        let mut session = Session::new(task.id, SessionPhase::Fix);

        debug!("Creating OpenCode session for fix");
        let opencode_session = self.create_opencode_session().await?;
        let session_id_str = opencode_session.id.to_string();

        info!(
            opencode_session_id = %session_id_str,
            "OpenCode session created for fix"
        );

        session.start(session_id_str.clone());
        self.persist_session(&session).await?;

        let activity_store = self.get_activity_store(session.id);

        self.emit_event(Event::SessionStarted {
            session_id: session.id,
            task_id: task.id,
            phase: session.phase.as_str().to_string(),
            status: session.status.as_str().to_string(),
            opencode_session_id: session.opencode_session_id.clone(),
            created_at: session.created_at,
        });

        // Get workspace path for MCP server
        let workspace_path = task
            .workspace_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.config.repo_path.clone());

        // Add MCP findings server to OpenCode
        if let Err(e) = self
            .add_mcp_findings_server(task.id, session.id, &workspace_path)
            .await
        {
            warn!(error = %e, "Failed to add MCP server for fix session");
            session.fail();
            self.update_session(&session).await?;

            if let Some(ref store) = activity_store {
                store.push_finished(false, Some(e.to_string()));
            }

            return Err(OrchestratorError::ExecutionFailed(format!(
                "MCP server required for fix session: {}",
                e
            )));
        }

        // Use fix prompt with MCP
        let prompt = PhasePrompts::fix_with_mcp(task);
        debug!(prompt_length = prompt.len(), "Sending fix prompt to OpenCode");

        let response_content = self
            .send_opencode_message_with_activity(
                &session_id_str,
                &prompt,
                activity_store.as_deref(),
            )
            .await;

        // Disconnect MCP server after fix (ignore errors)
        let _ = self.remove_mcp_findings_server(&workspace_path).await;

        let response_content = match response_content {
            Ok(content) => content,
            Err(e) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(false, Some(e.to_string()));
                }
                return Err(e);
            }
        };

        info!(
            response_length = response_content.len(),
            "Received fix session response"
        );

        session.complete();
        self.update_session(&session).await?;

        if let Some(ref store) = activity_store {
            store.push_finished(true, None);
        }

        self.emit_event(Event::SessionEnded {
            session_id: session.id,
            task_id: task.id,
            success: true,
        });

        // After fix, transition back to AI Review for re-evaluation
        info!(task_id = %task.id, "Fix session completed, transitioning to AI Review");
        self.transition(task, TaskStatus::AiReview)?;

        Ok(PhaseResult::FixCompleted {
            session_id: session_id_str,
        })
    }

    /// Fallback method for AI review without MCP (uses JSON parsing)
    async fn run_ai_review_json_fallback(
        &self,
        task: &mut Task,
        mut session: Session,
        session_id_str: String,
        activity_store: Option<Arc<SessionActivityStore>>,
        iteration: u32,
    ) -> Result<PhaseResult> {
        debug!("Getting workspace diff for review");
        let diff = self.get_workspace_diff(task).await?;
        debug!(diff_length = diff.len(), "Workspace diff retrieved");

        let prompt = PhasePrompts::review(task, &diff);
        debug!(prompt_length = prompt.len(), "Sending review prompt to OpenCode");

        let response_content = self
            .send_opencode_message_with_activity(
                &session_id_str,
                &prompt,
                activity_store.as_deref(),
            )
            .await;

        let response_content = match response_content {
            Ok(content) => content,
            Err(e) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(false, Some(e.to_string()));
                }
                return Err(e);
            }
        };

        info!(
            response_length = response_content.len(),
            "Received AI review response"
        );

        let _review_path = self
            .file_manager
            .write_review(task.id, &response_content)
            .await?;

        session.complete();
        self.update_session(&session).await?;

        let review_result = match Self::parse_review_json(&response_content, task.id, session.id) {
            Ok(findings) => {
                self.file_manager.write_findings(task.id, &findings).await?;
                if findings.approved || findings.findings.is_empty() {
                    ReviewResult::Approved
                } else {
                    ReviewResult::FindingsDetected(findings.findings.len())
                }
            }
            Err(_) => {
                warn!("Falling back to legacy text-based review parsing");
                Self::parse_review_response(&response_content)
            }
        };

        let success = matches!(review_result, ReviewResult::Approved);

        if let Some(ref store) = activity_store {
            store.push_finished(success, None);
        }

        self.emit_event(Event::SessionEnded {
            session_id: session.id,
            task_id: task.id,
            success,
        });

        match review_result {
            ReviewResult::Approved => {
                self.transition(task, TaskStatus::Review)?;
                Ok(PhaseResult::ReviewPassed {
                    session_id: session_id_str,
                })
            }
            ReviewResult::FindingsDetected(count) => {
                Ok(PhaseResult::ReviewFailed {
                    session_id: session_id_str,
                    feedback: format!("{} issues found. Review findings and choose to fix or skip.", count),
                    iteration,
                })
            }
            ReviewResult::ChangesRequested(feedback) => {
                self.transition(task, TaskStatus::InProgress)?;
                Ok(PhaseResult::ReviewFailed {
                    session_id: session_id_str,
                    feedback,
                    iteration,
                })
            }
        }
    }

    async fn get_workspace_diff(&self, task: &Task) -> Result<String> {
        if let Some(ref wm) = self.workspace_manager {
            if let Some(ref workspace_path) = task.workspace_path {
                let workspace = Workspace::new(
                    task.id.to_string(),
                    PathBuf::from(workspace_path),
                    format!("task-{}", task.id),
                );
                return wm
                    .get_diff(&workspace)
                    .await
                    .map_err(|e| OrchestratorError::ExecutionFailed(format!("VCS error: {}", e)));
            }
        }
        Ok("(no workspace configured - diff unavailable)".to_string())
    }

    /// Parse JSON review response and create ReviewFindings
    fn parse_review_json(
        content: &str,
        task_id: Uuid,
        session_id: Uuid,
    ) -> Result<ReviewFindings> {
        // Try to extract JSON from markdown code blocks or raw content
        let json_str = Self::extract_json_from_response(content);

        let raw: RawReviewResponse = serde_json::from_str(&json_str).map_err(|e| {
            warn!(
                error = %e,
                content_preview = %content.chars().take(500).collect::<String>(),
                "Failed to parse review JSON, falling back to text parsing"
            );
            OrchestratorError::ExecutionFailed(format!("Failed to parse review JSON: {}", e))
        })?;

        // Convert raw findings to ReviewFinding
        let findings: Vec<ReviewFinding> = raw
            .findings
            .into_iter()
            .enumerate()
            .map(|(i, f)| ReviewFinding {
                id: format!("finding-{}", i + 1),
                file_path: f.file_path,
                line_start: f.line_start,
                line_end: f.line_end,
                title: f.title,
                description: f.description,
                severity: match f.severity.to_lowercase().as_str() {
                    "error" => FindingSeverity::Error,
                    "info" => FindingSeverity::Info,
                    _ => FindingSeverity::Warning,
                },
                status: FindingStatus::Pending,
            })
            .collect();

        Ok(ReviewFindings::with_findings(
            task_id,
            session_id,
            raw.summary,
            findings,
        ))
    }

    /// Extract JSON from response that might be wrapped in markdown code blocks
    fn extract_json_from_response(content: &str) -> String {
        // Try to find JSON in ```json ... ``` blocks
        if let Some(start) = content.find("```json") {
            if let Some(end) = content[start..].find("```\n").or(content[start..].rfind("```")) {
                let json_start = start + 7; // length of "```json"
                let json_content = &content[json_start..start + end];
                return json_content.trim().to_string();
            }
        }

        // Try to find JSON in ``` ... ``` blocks
        if let Some(start) = content.find("```\n{") {
            if let Some(end) = content[start + 4..].find("\n```") {
                return content[start + 4..start + 4 + end].trim().to_string();
            }
        }

        // Try to find raw JSON (starts with { and ends with })
        if let Some(start) = content.find('{') {
            if let Some(end) = content.rfind('}') {
                return content[start..=end].to_string();
            }
        }

        // Return as-is, let JSON parser handle the error
        content.to_string()
    }

    /// Legacy text-based review parsing (fallback)
    fn parse_review_response(content: &str) -> ReviewResult {
        let content_upper = content.to_uppercase();

        if content_upper.contains("APPROVED") && !content_upper.contains("NOT APPROVED") {
            ReviewResult::Approved
        } else if content_upper.contains("CHANGES_REQUESTED")
            || content_upper.contains("CHANGES REQUESTED")
            || content_upper.contains("REJECTED")
        {
            let feedback = content
                .lines()
                .skip_while(|line| {
                    let upper = line.to_uppercase();
                    !upper.contains("CHANGES_REQUESTED")
                        && !upper.contains("CHANGES REQUESTED")
                        && !upper.contains("REJECTED")
                        && !upper.contains("FEEDBACK")
                        && !upper.contains("ISSUES")
                })
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();

            if feedback.is_empty() {
                ReviewResult::ChangesRequested(content.to_string())
            } else {
                ReviewResult::ChangesRequested(feedback)
            }
        } else {
            ReviewResult::ChangesRequested(
                "Review response unclear. Manual review required.".to_string(),
            )
        }
    }

    #[instrument(skip(self, task, feedback), fields(task_id = %task.id))]
    pub async fn run_fix_iteration(&self, task: &mut Task, feedback: &str) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            feedback_length = feedback.len(),
            "Starting FIX iteration based on review feedback"
        );

        let mut session = Session::new(task.id, SessionPhase::Implementation);

        debug!("Creating OpenCode session for fix iteration");
        let opencode_session = self.create_opencode_session().await?;
        let session_id_str = opencode_session.id.to_string();

        info!(
            opencode_session_id = %session_id_str,
            "OpenCode session created for fix iteration"
        );

        session.start(session_id_str.clone());
        self.persist_session(&session).await?;

        let activity_store = self.get_activity_store(session.id);

        self.emit_event(Event::SessionStarted {
            session_id: session.id,
            task_id: task.id,
            phase: session.phase.as_str().to_string(),
            status: session.status.as_str().to_string(),
            opencode_session_id: session.opencode_session_id.clone(),
            created_at: session.created_at,
        });

        let prompt = PhasePrompts::fix_issues(task, feedback);
        debug!(prompt_length = prompt.len(), "Sending fix prompt to OpenCode");

        let response = self
            .send_opencode_message_with_activity(
                &session_id_str,
                &prompt,
                activity_store.as_deref(),
            )
            .await;

        match response {
            Ok(_) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(true, None);
                }
            }
            Err(e) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(false, Some(e.to_string()));
                }
                return Err(e);
            }
        }

        info!("Fix iteration response received from OpenCode");

        session.complete();
        self.update_session(&session).await?;

        self.emit_event(Event::SessionEnded {
            session_id: session.id,
            task_id: task.id,
            success: true,
        });

        self.transition(task, TaskStatus::AiReview)?;

        info!(
            task_id = %task.id,
            "FIX iteration completed, returning to AI review"
        );

        Ok(PhaseResult::SessionCreated {
            session_id: session_id_str,
        })
    }

    #[instrument(skip(self, task), fields(task_id = %task.id))]
    pub async fn approve_plan(&self, task: &mut Task) -> Result<()> {
        info!(task_id = %task.id, "Plan APPROVED by human reviewer");

        if task.status != TaskStatus::PlanningReview {
            warn!(
                current_status = %task.status.as_str(),
                "Cannot approve plan - task not in PlanningReview state"
            );
            return Err(OrchestratorError::InvalidTransition {
                from: task.status.as_str().to_string(),
                to: "InProgress (plan approval)".to_string(),
            });
        }
        self.transition(task, TaskStatus::InProgress)?;
        info!(task_id = %task.id, "Task ready for implementation");
        Ok(())
    }

    #[instrument(skip(self, task, feedback), fields(task_id = %task.id))]
    pub async fn reject_plan(&self, task: &mut Task, feedback: &str) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            feedback_length = feedback.len(),
            "Plan REJECTED by human reviewer, re-planning"
        );

        if task.status != TaskStatus::PlanningReview {
            warn!(
                current_status = %task.status.as_str(),
                "Cannot reject plan - task not in PlanningReview state"
            );
            return Err(OrchestratorError::InvalidTransition {
                from: task.status.as_str().to_string(),
                to: "Planning (plan rejection)".to_string(),
            });
        }
        self.transition(task, TaskStatus::Planning)?;

        let mut session = Session::new(task.id, SessionPhase::Planning);
        let opencode_session = self.create_opencode_session().await?;
        let session_id_str = opencode_session.id.to_string();

        info!(
            opencode_session_id = %session_id_str,
            "OpenCode session created for re-planning"
        );

        session.start(session_id_str.clone());
        self.persist_session(&session).await?;

        let activity_store = self.get_activity_store(session.id);

        let prompt = PhasePrompts::replan(task, feedback);
        let response_content = self
            .send_opencode_message_with_activity(
                &session_id_str,
                &prompt,
                activity_store.as_deref(),
            )
            .await;

        let response_content = match response_content {
            Ok(content) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(true, None);
                }
                content
            }
            Err(e) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(false, Some(e.to_string()));
                }
                return Err(e);
            }
        };

        let plan_path = self
            .file_manager
            .write_plan(task.id, &response_content)
            .await?;

        info!(plan_path = %plan_path.display(), "New plan saved");

        session.complete();
        self.update_session(&session).await?;

        self.transition(task, TaskStatus::PlanningReview)?;

        info!(task_id = %task.id, "Re-planning completed, awaiting review");

        Ok(PhaseResult::PlanCreated {
            session_id: session_id_str,
            plan_path: plan_path.to_string_lossy().to_string(),
        })
    }

    #[instrument(skip(self, task), fields(task_id = %task.id))]
    pub async fn approve_review(&self, task: &mut Task) -> Result<()> {
        info!(task_id = %task.id, "Implementation APPROVED by human reviewer");

        if task.status != TaskStatus::Review {
            warn!(
                current_status = %task.status.as_str(),
                "Cannot approve review - task not in Review state"
            );
            return Err(OrchestratorError::InvalidTransition {
                from: task.status.as_str().to_string(),
                to: "Done (review approval)".to_string(),
            });
        }
        self.transition(task, TaskStatus::Done)?;
        info!(task_id = %task.id, "Task COMPLETED successfully");
        Ok(())
    }

    #[instrument(skip(self, task, feedback), fields(task_id = %task.id))]
    pub async fn reject_review(&self, task: &mut Task, feedback: &str) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            feedback_length = feedback.len(),
            "Implementation REJECTED by human reviewer, running fix iteration"
        );

        if task.status != TaskStatus::Review {
            warn!(
                current_status = %task.status.as_str(),
                "Cannot reject review - task not in Review state"
            );
            return Err(OrchestratorError::InvalidTransition {
                from: task.status.as_str().to_string(),
                to: "InProgress (review rejection)".to_string(),
            });
        }
        self.run_fix_iteration(task, feedback).await
    }
}

#[derive(Debug, Clone)]
pub enum PhaseResult {
    SessionCreated {
        session_id: String,
    },
    PlanCreated {
        session_id: String,
        plan_path: String,
    },
    AwaitingApproval {
        phase: SessionPhase,
    },
    ReviewPassed {
        session_id: String,
    },
    ReviewFailed {
        session_id: String,
        feedback: String,
        iteration: u32,
    },
    /// Fix session completed - will auto-transition back to AI Review
    FixCompleted {
        session_id: String,
    },
    MaxIterationsExceeded {
        iterations: u32,
    },
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewResult {
    Approved,
    ChangesRequested(String),
    /// Structured findings detected - task stays in ai_review waiting for user action
    FindingsDetected(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_review_approved() {
        let content = "## Review\n\nThe code looks good.\n\nAPPROVED";
        let result = TaskExecutor::parse_review_response(content);
        assert_eq!(result, ReviewResult::Approved);
    }

    #[test]
    fn test_parse_review_approved_lowercase() {
        let content = "The implementation is correct. Approved!";
        let result = TaskExecutor::parse_review_response(content);
        assert_eq!(result, ReviewResult::Approved);
    }

    #[test]
    fn test_parse_review_changes_requested() {
        let content = "## Review\n\nCHANGES_REQUESTED\n\n- Fix the error handling\n- Add tests";
        let result = TaskExecutor::parse_review_response(content);
        match result {
            ReviewResult::ChangesRequested(feedback) => {
                assert!(feedback.contains("Fix the error handling"));
            }
            _ => panic!("Expected ChangesRequested"),
        }
    }

    #[test]
    fn test_parse_review_rejected() {
        let content = "REJECTED\n\nThe code has critical issues:\n1. Security vulnerability";
        let result = TaskExecutor::parse_review_response(content);
        match result {
            ReviewResult::ChangesRequested(feedback) => {
                assert!(feedback.contains("Security vulnerability"));
            }
            _ => panic!("Expected ChangesRequested"),
        }
    }

    #[test]
    fn test_parse_review_unclear() {
        let content = "I'm not sure about this implementation.";
        let result = TaskExecutor::parse_review_response(content);
        match result {
            ReviewResult::ChangesRequested(feedback) => {
                assert!(feedback.contains("Manual review required"));
            }
            _ => panic!("Expected ChangesRequested"),
        }
    }

    #[test]
    fn test_parse_review_not_approved() {
        let content = "This is NOT APPROVED due to issues.";
        let result = TaskExecutor::parse_review_response(content);
        match result {
            ReviewResult::ChangesRequested(_) => {}
            ReviewResult::Approved => panic!("Should not be approved when NOT APPROVED is present"),
        }
    }

    #[test]
    fn test_executor_config_builder() {
        let config = ExecutorConfig::new("/repo")
            .with_plan_approval(false)
            .with_human_review(false)
            .with_max_iterations(5);

        assert_eq!(config.repo_path, PathBuf::from("/repo"));
        assert!(!config.require_plan_approval);
        assert!(!config.require_human_review);
        assert_eq!(config.max_review_iterations, 5);
    }
}
