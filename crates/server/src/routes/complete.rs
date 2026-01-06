use axum::extract::{Path, State};
use axum::Json;
use github::{CreatePrRequest, GhCli, RepoConfig};
use opencode_core::{TaskStatus, UpdateTaskRequest};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use vcs::DiffSummary;

use crate::config::UserMode;
use crate::error::AppError;
use crate::state::AppState;

// ============================================================================
// Complete Preview Endpoint
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct CompletePreviewResponse {
    pub task_id: String,
    pub branch_name: String,
    pub base_branch: String,
    pub diff_summary: DiffSummary,
    pub suggested_pr_title: String,
    pub suggested_pr_body: String,
    pub github_available: bool,
    pub has_uncommitted_changes: bool,
}

#[utoipa::path(
    get,
    path = "/api/tasks/{task_id}/complete/preview",
    params(
        ("task_id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Complete preview data", body = CompletePreviewResponse),
        (status = 404, description = "Task or workspace not found"),
        (status = 400, description = "Task not in review status")
    ),
    tag = "complete"
)]
pub async fn get_complete_preview(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<CompletePreviewResponse>, AppError> {
    let project = state.project().await?;

    // Get task
    let task = project
        .task_repository
        .find_by_id(task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task not found: {}", task_id)))?;

    // Validate task status
    if task.status != TaskStatus::Review {
        return Err(AppError::BadRequest(
            "Task must be in review status to complete".to_string(),
        ));
    }

    // Find workspace
    let workspaces = project.workspace_manager.list_workspaces().await?;
    let workspace = workspaces
        .into_iter()
        .find(|ws| ws.task_id == task_id.to_string())
        .ok_or_else(|| AppError::NotFound(format!("Workspace not found for task: {}", task_id)))?;

    // Get diff summary
    let diff_summary = project
        .workspace_manager
        .vcs()
        .get_diff_summary(&workspace)
        .await?;

    // Check for uncommitted changes
    let has_uncommitted_changes = project
        .workspace_manager
        .vcs()
        .has_uncommitted_changes(&workspace)
        .await?;

    // Check if GitHub is available (via token OR gh CLI)
    let github_available = state.github_client().await.is_ok() || GhCli::is_available().await;

    // Get main branch
    let base_branch = project.workspace_manager.vcs().main_branch().to_string();

    // Generate suggested PR content
    let suggested_pr_title = task.title.clone();
    let suggested_pr_body =
        generate_pr_body(&task.title, Some(task.description.as_str()), &diff_summary);

    Ok(Json(CompletePreviewResponse {
        task_id: task_id.to_string(),
        branch_name: workspace.branch_name,
        base_branch,
        diff_summary,
        suggested_pr_title,
        suggested_pr_body,
        github_available,
        has_uncommitted_changes,
    }))
}

fn generate_pr_body(title: &str, description: Option<&str>, summary: &DiffSummary) -> String {
    let mut body = String::new();

    body.push_str("## Summary\n\n");
    if let Some(desc) = description {
        body.push_str(desc);
        body.push_str("\n\n");
    } else {
        body.push_str(&format!("{}\n\n", title));
    }

    body.push_str("## Changes\n\n");
    body.push_str(&format!("- **{}** files changed\n", summary.files_changed));
    body.push_str(&format!("- **+{}** additions\n", summary.additions));
    body.push_str(&format!("- **-{}** deletions\n", summary.deletions));

    body
}

// ============================================================================
// Complete Task Endpoint
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum CompleteAction {
    /// Create a pull request on GitHub
    CreatePr,
    /// Merge changes to local main branch
    MergeLocal,
    /// Just mark as complete without merging
    CompleteOnly,
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PrOptions {
    pub title: String,
    pub body: String,
    pub base_branch: String,
    pub draft: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct MergeOptions {
    pub commit_message: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct CompleteTaskRequest {
    pub action: CompleteAction,
    pub pr_options: Option<PrOptions>,
    pub merge_options: Option<MergeOptions>,
    pub cleanup_worktree: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PrInfo {
    pub number: u64,
    pub url: String,
    pub title: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MergeResultInfo {
    Success { commit_sha: Option<String> },
    Conflicts { files: Vec<String> },
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct CompleteTaskResponse {
    pub success: bool,
    pub pr: Option<PrInfo>,
    pub merge_result: Option<MergeResultInfo>,
    pub worktree_cleaned: bool,
}

#[utoipa::path(
    post,
    path = "/api/tasks/{task_id}/complete",
    params(
        ("task_id" = Uuid, Path, description = "Task ID")
    ),
    request_body = CompleteTaskRequest,
    responses(
        (status = 200, description = "Task completed successfully", body = CompleteTaskResponse),
        (status = 404, description = "Task or workspace not found"),
        (status = 400, description = "Invalid request or task not in review status"),
        (status = 409, description = "Merge conflicts")
    ),
    tag = "complete"
)]
pub async fn complete_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<CompleteTaskRequest>,
) -> Result<Json<CompleteTaskResponse>, AppError> {
    let project = state.project().await?;

    // Get task
    let task = project
        .task_repository
        .find_by_id(task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task not found: {}", task_id)))?;

    // Validate task status
    if task.status != TaskStatus::Review {
        return Err(AppError::BadRequest(
            "Task must be in review status to complete".to_string(),
        ));
    }

    // Find workspace
    let workspaces = project.workspace_manager.list_workspaces().await?;
    let workspace = workspaces
        .into_iter()
        .find(|ws| ws.task_id == task_id.to_string())
        .ok_or_else(|| AppError::NotFound(format!("Workspace not found for task: {}", task_id)))?;

    let mut response = CompleteTaskResponse {
        success: false,
        pr: None,
        merge_result: None,
        worktree_cleaned: false,
    };

    match payload.action {
        CompleteAction::CreatePr => {
            let pr_opts = payload.pr_options.ok_or_else(|| {
                AppError::BadRequest("PR options required for create_pr action".to_string())
            })?;

            // Commit any uncommitted changes
            let has_changes = project
                .workspace_manager
                .vcs()
                .has_uncommitted_changes(&workspace)
                .await?;

            if has_changes {
                let commit_msg = format!("{}\n\n{}", pr_opts.title, pr_opts.body);
                project
                    .workspace_manager
                    .vcs()
                    .commit(&workspace, &commit_msg)
                    .await?;
            }

            // Build PR request
            let pr_request =
                CreatePrRequest::new(&pr_opts.title, &workspace.branch_name, &pr_opts.base_branch)
                    .with_body(&pr_opts.body);

            let pr_request = if pr_opts.draft {
                pr_request.as_draft()
            } else {
                pr_request
            };

            // Try GitHub API first, fall back to gh CLI
            let pr = if let Ok(github_client) = state.github_client().await {
                // Push branch to remote first
                project
                    .workspace_manager
                    .vcs()
                    .push(&workspace, "origin")
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to push branch: {}", e)))?;

                // Create PR via GitHub API (with token)
                github_client
                    .create_pull_request(pr_request)
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to create PR: {}", e)))?
            } else if GhCli::is_available().await {
                // Use gh CLI (uses user's local authentication)
                let repo_config = RepoConfig::from_git_remote(&project.path)
                    .await
                    .ok_or_else(|| {
                        AppError::BadRequest(
                            "Could not detect GitHub repository from git remote".to_string(),
                        )
                    })?;

                let gh_cli = GhCli::new(repo_config, &workspace.path);

                // gh CLI handles push + PR creation
                gh_cli
                    .push_and_create_pr(pr_request)
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to create PR via gh: {}", e)))?
            } else {
                return Err(AppError::BadRequest(
                    "GitHub not available. Please set GITHUB_TOKEN or install and authenticate gh CLI.".to_string(),
                ));
            };

            response.pr = Some(PrInfo {
                number: pr.number,
                url: pr.html_url,
                title: pr.title,
            });
        }

        CompleteAction::MergeLocal => {
            let merge_opts = payload.merge_options.unwrap_or_else(|| MergeOptions {
                commit_message: format!("Merge task: {}", task.title),
            });

            let merge_result = project
                .workspace_manager
                .merge_workspace(&workspace, &merge_opts.commit_message)
                .await?;

            match merge_result {
                vcs::MergeResult::Success => {
                    response.merge_result = Some(MergeResultInfo::Success { commit_sha: None });
                }
                vcs::MergeResult::Conflicts { files } => {
                    let conflict_paths: Vec<String> =
                        files.iter().map(|f| f.path.display().to_string()).collect();
                    response.merge_result = Some(MergeResultInfo::Conflicts {
                        files: conflict_paths.clone(),
                    });
                    return Err(AppError::Conflict(format!(
                        "Merge conflicts in: {}",
                        conflict_paths.join(", ")
                    )));
                }
            }
        }

        CompleteAction::CompleteOnly => {
            // Just transition to done, no merge/PR
        }
    }

    // Cleanup worktree if requested
    if payload.cleanup_worktree {
        project
            .workspace_manager
            .cleanup_workspace(&workspace)
            .await?;
        response.worktree_cleaned = true;
    }

    // Transition task to done
    let update_request = UpdateTaskRequest {
        title: None,
        description: None,
        status: Some(TaskStatus::Done),
        workspace_path: Some(String::new()), // Clear workspace path
    };
    project
        .task_repository
        .update(task_id, &update_request)
        .await?;

    response.success = true;
    Ok(Json(response))
}

// ============================================================================
// User Mode Endpoints
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct UserModeResponse {
    pub mode: UserMode,
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct UpdateUserModeRequest {
    pub mode: UserMode,
}

#[utoipa::path(
    get,
    path = "/api/settings/user-mode",
    responses(
        (status = 200, description = "Current user mode", body = UserModeResponse)
    ),
    tag = "settings"
)]
pub async fn get_user_mode(
    State(state): State<AppState>,
) -> Result<Json<UserModeResponse>, AppError> {
    let project = state.project().await?;
    let config = project.get_config().await;

    Ok(Json(UserModeResponse {
        mode: config.user_mode,
    }))
}

#[utoipa::path(
    put,
    path = "/api/settings/user-mode",
    request_body = UpdateUserModeRequest,
    responses(
        (status = 200, description = "User mode updated", body = UserModeResponse)
    ),
    tag = "settings"
)]
pub async fn update_user_mode(
    State(state): State<AppState>,
    Json(payload): Json<UpdateUserModeRequest>,
) -> Result<Json<UserModeResponse>, AppError> {
    let project = state.project().await?;
    let mut config = project.get_config().await;

    config.user_mode = payload.mode;
    project.save_config(&config).await?;

    Ok(Json(UserModeResponse {
        mode: config.user_mode,
    }))
}
