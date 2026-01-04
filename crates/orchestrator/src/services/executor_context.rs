use db::{SessionRepository, TaskRepository};
use events::{Event, EventBus, EventEnvelope};
use opencode_client::apis::configuration::Configuration;
use opencode_core::{Session, SessionPhase, Task, TaskStatus, UpdateTaskRequest};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;
use vcs::WorkspaceManager;

use crate::activity_store::{SessionActivityRegistry, SessionActivityStore};
use crate::error::{OrchestratorError, Result};
use crate::files::FileManager;
use crate::services::{McpManager, OpenCodeClient, WikiMcpConfig};
use crate::state_machine::TaskStateMachine;

#[derive(Debug, Clone, Default)]
pub struct ModelSelection {
    pub provider_id: String,
    pub model_id: String,
}

impl ModelSelection {
    pub fn new(provider_id: impl Into<String>, model_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            model_id: model_id.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PhaseModels {
    pub planning: Option<ModelSelection>,
    pub implementation: Option<ModelSelection>,
    pub review: Option<ModelSelection>,
    pub fix: Option<ModelSelection>,
}

#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    pub require_plan_approval: bool,
    pub require_human_review: bool,
    pub max_review_iterations: u32,
    pub repo_path: PathBuf,
    pub phase_models: PhaseModels,
    pub wiki_config: Option<WikiMcpConfig>,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            require_plan_approval: true,
            require_human_review: true,
            max_review_iterations: 3,
            repo_path: PathBuf::from("."),
            phase_models: PhaseModels::default(),
            wiki_config: None,
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

    pub fn with_phase_models(mut self, phase_models: PhaseModels) -> Self {
        self.phase_models = phase_models;
        self
    }

    pub fn with_wiki_config(mut self, wiki_config: WikiMcpConfig) -> Self {
        self.wiki_config = Some(wiki_config);
        self
    }
}

pub struct ExecutorContext {
    pub opencode_config: Arc<Configuration>,
    pub config: ExecutorConfig,
    pub file_manager: FileManager,
    pub workspace_manager: Option<Arc<WorkspaceManager>>,
    pub session_repo: Option<Arc<SessionRepository>>,
    pub task_repo: Option<Arc<TaskRepository>>,
    pub event_bus: Option<EventBus>,
    pub activity_registry: Option<SessionActivityRegistry>,
    pub mcp_manager: McpManager,
    pub opencode_client: OpenCodeClient,
}

impl ExecutorContext {
    pub fn new(opencode_config: Arc<Configuration>, config: ExecutorConfig) -> Self {
        let file_manager = FileManager::new(&config.repo_path);
        let mcp_manager = McpManager::new(Arc::clone(&opencode_config));
        let opencode_client = OpenCodeClient::new(Arc::clone(&opencode_config));
        Self {
            opencode_config,
            config,
            file_manager,
            workspace_manager: None,
            session_repo: None,
            task_repo: None,
            event_bus: None,
            activity_registry: None,
            mcp_manager,
            opencode_client,
        }
    }

    pub fn with_model(mut self, provider_id: &str, model_id: &str) -> Self {
        self.opencode_client = self.opencode_client.with_model(provider_id, model_id);
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

    pub fn file_manager(&self) -> &FileManager {
        &self.file_manager
    }

    pub fn opencode_client_for_phase(&self, phase: SessionPhase) -> OpenCodeClient {
        let model = match phase {
            SessionPhase::Planning => &self.config.phase_models.planning,
            SessionPhase::Implementation => &self.config.phase_models.implementation,
            SessionPhase::Review => &self.config.phase_models.review,
            SessionPhase::Fix => &self.config.phase_models.fix,
        };

        match model {
            Some(m) => self
                .opencode_client
                .clone()
                .with_model(&m.provider_id, &m.model_id),
            None => self.opencode_client.clone(),
        }
    }

    pub fn opencode_client_for_fix(&self) -> OpenCodeClient {
        self.opencode_client_for_phase(SessionPhase::Fix)
    }

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

    pub fn emit_event(&self, event: Event) {
        if let Some(ref bus) = self.event_bus {
            bus.publish(EventEnvelope::new(event));
        }
    }

    pub async fn persist_session(&self, session: &Session) -> Result<()> {
        if let Some(ref repo) = self.session_repo {
            repo.create(session).await?;
        }
        Ok(())
    }

    pub async fn update_session(&self, session: &Session) -> Result<()> {
        if let Some(ref repo) = self.session_repo {
            repo.update(session).await?;
        }
        Ok(())
    }

    pub fn get_activity_store(&self, session_id: Uuid) -> Option<Arc<SessionActivityStore>> {
        self.activity_registry
            .as_ref()
            .map(|reg| reg.get_or_create(session_id))
    }

    pub fn working_dir_for_task(&self, task: &Task) -> PathBuf {
        task.workspace_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.config.repo_path.clone())
    }

    pub async fn setup_workspace(&self, task: &mut Task) -> Result<()> {
        if task.workspace_path.is_some() {
            return Ok(());
        }

        if let Some(ref wm) = self.workspace_manager {
            debug!("Setting up VCS workspace for task");
            let workspace = wm
                .setup_workspace(&task.id.to_string())
                .await
                .map_err(|e| {
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

            if let Some(ref repo) = self.task_repo {
                let update = UpdateTaskRequest {
                    workspace_path: task.workspace_path.clone(),
                    ..Default::default()
                };
                if let Err(e) = repo.update(task.id, &update).await {
                    tracing::error!(error = %e, "Failed to persist workspace_path to database");
                }
            }
        }
        Ok(())
    }

    pub fn emit_session_started(&self, session: &Session, task_id: Uuid) {
        self.emit_event(Event::SessionStarted {
            session_id: session.id,
            task_id,
            phase: session.phase.as_str().to_string(),
            status: session.status.as_str().to_string(),
            opencode_session_id: session.opencode_session_id.clone(),
            created_at: session.created_at,
        });
    }

    pub fn emit_session_ended(&self, session_id: Uuid, task_id: Uuid, success: bool) {
        self.emit_event(Event::SessionEnded {
            session_id,
            task_id,
            success,
        });
    }

    /// Commit changes after a phase completes
    pub async fn commit_phase_changes(
        &self,
        task: &Task,
        phase: &str,
        description: &str,
    ) -> Result<()> {
        if let Some(ref wm) = self.workspace_manager {
            if task.workspace_path.is_some() {
                // Find workspace by task ID from list
                let task_id_str = task.id.to_string();
                let workspaces = wm.list_workspaces().await.map_err(|e| {
                    OrchestratorError::ExecutionFailed(format!("Failed to list workspaces: {}", e))
                })?;

                let workspace = workspaces.into_iter().find(|ws| ws.task_id == task_id_str);

                if let Some(ws) = workspace {
                    // Build commit message
                    let commit_message = format!("[{}] {}", phase, description);

                    match wm.commit(&ws, &commit_message).await {
                        Ok(commit_id) => {
                            info!(
                                task_id = %task.id,
                                phase = %phase,
                                commit_id = %commit_id,
                                "Phase changes committed"
                            );
                        }
                        Err(e) => {
                            // Don't fail the phase if commit fails - just log warning
                            tracing::warn!(
                                task_id = %task.id,
                                phase = %phase,
                                error = %e,
                                "Failed to commit phase changes (continuing anyway)"
                            );
                        }
                    }
                } else {
                    debug!(
                        task_id = %task.id,
                        "No workspace found for commit"
                    );
                }
            }
        }
        Ok(())
    }
}
