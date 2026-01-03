//! Unified execution engine for all phases.
//!
//! This module provides the `ExecutionEngine` which handles resource acquisition,
//! session execution, and cleanup for any phase implementation.

use opencode_core::Task;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::error::Result;
use crate::services::ExecutorContext;

use super::phase::{Phase, PhaseConfig, PhaseOutcome, SessionOutput};

/// Resources acquired for phase execution.
///
/// These resources are held for the duration of the session and
/// automatically cleaned up when dropped.
pub struct AcquiredResources {
    /// MCP guard for automatic cleanup (if MCP was requested)
    pub mcp_guard: Option<crate::resources::McpGuard>,
    /// Workspace path being used
    pub workspace_path: PathBuf,
    /// Task ID for this execution
    pub task_id: Uuid,
    /// Session ID for this execution
    pub session_id: Uuid,
}

impl AcquiredResources {
    /// Create a new resources container.
    pub fn new(workspace_path: PathBuf, task_id: Uuid, session_id: Uuid) -> Self {
        Self {
            mcp_guard: None,
            workspace_path,
            task_id,
            session_id,
        }
    }

    /// Set the MCP guard.
    pub fn with_mcp_guard(mut self, guard: crate::resources::McpGuard) -> Self {
        self.mcp_guard = Some(guard);
        self
    }
}

/// Unified execution engine that runs any Phase implementation.
///
/// The engine handles:
/// - Resource acquisition (workspace, MCP servers)
/// - Session creation and execution
/// - Activity streaming
/// - Automatic cleanup via RAII guards
pub struct ExecutionEngine {
    ctx: Arc<ExecutorContext>,
}

impl ExecutionEngine {
    /// Create a new execution engine with the given context.
    pub fn new(ctx: Arc<ExecutorContext>) -> Self {
        Self { ctx }
    }

    /// Get a reference to the executor context.
    pub fn context(&self) -> &ExecutorContext {
        &self.ctx
    }

    /// Execute a phase synchronously and return the outcome.
    ///
    /// This method:
    /// 1. Acquires required resources (with RAII guards)
    /// 2. Builds phase configuration
    /// 3. Executes the session
    /// 4. Processes the result
    /// 5. Cleans up resources (via Drop)
    pub async fn execute<P: Phase>(&self, phase: &P, task: &mut Task) -> Result<PhaseOutcome> {
        info!(
            task_id = %task.id,
            phase = ?phase.phase_type(),
            "Executing phase"
        );

        // Build phase configuration
        let config = phase.build_config(&self.ctx, task).await?;
        debug!(
            prompt_length = config.prompt.len(),
            working_dir = %config.working_dir.display(),
            mcp_servers = config.mcp_servers.len(),
            "Phase configuration built"
        );

        // Acquire resources
        let resources = self.acquire_resources(phase, task, &config).await?;

        // Execute session
        let output = self.run_session(task, &config, &resources).await?;

        // Process result (resources will be cleaned up when dropped)
        let outcome = phase.process_result(&self.ctx, task, &output).await?;

        info!(
            task_id = %task.id,
            phase = ?phase.phase_type(),
            outcome = ?outcome,
            "Phase completed"
        );

        Ok(outcome)
    }

    /// Acquire resources required by the phase.
    async fn acquire_resources<P: Phase>(
        &self,
        phase: &P,
        task: &Task,
        config: &PhaseConfig,
    ) -> Result<AcquiredResources> {
        let requirements = phase.required_resources();
        let session_id = Uuid::new_v4();

        let mut resources = AcquiredResources::new(config.working_dir.clone(), task.id, session_id);

        // Setup MCP servers if needed
        if requirements.needs_mcp_findings && !config.mcp_servers.is_empty() {
            let guard = crate::resources::McpGuard::connect(
                self.ctx.mcp_manager.clone(),
                config.working_dir.clone(),
                &config.mcp_servers,
                task.id,
                session_id,
            )
            .await?;
            resources = resources.with_mcp_guard(guard);
        }

        Ok(resources)
    }

    /// Run a session with the given configuration.
    async fn run_session(
        &self,
        task: &Task,
        config: &PhaseConfig,
        _resources: &AcquiredResources,
    ) -> Result<SessionOutput> {
        use opencode_core::Session;

        // Create session
        let mut session = Session::new(task.id, config.metadata.phase_type());

        // Create OpenCode session
        let opencode_session = self
            .ctx
            .opencode_client
            .create_session(&config.working_dir)
            .await?;

        let opencode_session_id = opencode_session.id.to_string();
        session.start(opencode_session_id.clone());

        // Persist session
        self.ctx.persist_session(&session).await?;

        // Emit session started event
        self.ctx.emit_session_started(&session, task.id);

        // Get activity store for streaming
        let activity_store = self.ctx.get_activity_store(session.id);

        // Send prompt
        let response = self
            .ctx
            .opencode_client
            .send_prompt(
                &opencode_session_id,
                &config.prompt,
                &config.working_dir,
                activity_store.as_deref(),
            )
            .await;

        let (success, response_text, error) = match response {
            Ok(text) => (true, text, None),
            Err(e) => (false, String::new(), Some(e.to_string())),
        };

        // Update session status
        if success {
            session.complete();
        } else {
            session.fail();
        }
        self.ctx.update_session(&session).await?;

        // Emit session ended event
        self.ctx.emit_session_ended(session.id, task.id, success);

        // Push finished activity
        if let Some(ref store) = activity_store {
            store.push_finished(success, error.clone());
        }

        Ok(SessionOutput {
            session_id: session.id,
            opencode_session_id,
            response_text,
            success,
            error,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acquired_resources_creation() {
        let resources =
            AcquiredResources::new(PathBuf::from("/tmp/test"), Uuid::new_v4(), Uuid::new_v4());
        assert!(resources.mcp_guard.is_none());
        assert_eq!(resources.workspace_path, PathBuf::from("/tmp/test"));
    }
}
