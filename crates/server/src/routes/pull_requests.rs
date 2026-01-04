use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use events::{Event, EventEnvelope};
use github::{
    CreateReviewCommentRequest, DiffSide, PrFile, PrIssueComment, PrReview, PrReviewComment,
    PrState, PullRequestDetail,
};
use opencode_core::Task;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument};
use utoipa::ToSchema;

use crate::error::AppError;
use crate::state::AppState;

// =============================================================================
// Query Parameters
// =============================================================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListPullRequestsQuery {
    /// Filter by state: open, closed, all
    #[serde(default = "default_state")]
    pub state: String,
}

fn default_state() -> String {
    "open".to_string()
}

// =============================================================================
// Response Types
// =============================================================================

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PullRequestsListResponse {
    pub pull_requests: Vec<PullRequestDetail>,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PrDiffResponse {
    pub diff: String,
    pub files: Vec<PrFile>,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PrCommentsResponse {
    pub review_comments: Vec<PrReviewComment>,
    pub issue_comments: Vec<PrIssueComment>,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PrReviewsResponse {
    pub reviews: Vec<PrReview>,
}

// =============================================================================
// Request Types
// =============================================================================

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct CreatePrCommentRequest {
    pub path: String,
    pub line: u32,
    pub side: String,
    pub body: String,
    pub commit_id: String,
    pub in_reply_to: Option<u64>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ReplyToCommentRequest {
    pub body: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct FixFromCommentsRequest {
    /// List of comment IDs to fix
    pub comment_ids: Vec<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct FixFromCommentsResponse {
    pub task: Task,
    pub comments_included: usize,
}

// =============================================================================
// List Pull Requests
// =============================================================================

#[utoipa::path(
    get,
    path = "/api/pull-requests",
    params(
        ("state" = Option<String>, Query, description = "Filter by state: open, closed, all")
    ),
    responses(
        (status = 200, description = "List of pull requests", body = PullRequestsListResponse),
        (status = 500, description = "GitHub API error")
    ),
    tag = "pull-requests"
)]
pub async fn list_pull_requests(
    State(state): State<AppState>,
    Query(query): Query<ListPullRequestsQuery>,
) -> Result<Json<PullRequestsListResponse>, AppError> {
    debug!("Listing pull requests with state: {}", query.state);

    let github = state.github_client().await.map_err(|e| {
        error!("Failed to get GitHub client: {}", e);
        AppError::Internal(format!("GitHub client error: {}", e))
    })?;

    let pr_state = match query.state.as_str() {
        "open" => Some(PrState::Open),
        "closed" => Some(PrState::Closed),
        "all" => None,
        _ => Some(PrState::Open),
    };

    let pull_requests = github
        .list_pull_requests_detail(pr_state)
        .await
        .map_err(|e| {
            error!("Failed to list pull requests: {}", e);
            AppError::Internal(format!("GitHub API error: {}", e))
        })?;

    Ok(Json(PullRequestsListResponse { pull_requests }))
}

// =============================================================================
// Get Pull Request Detail
// =============================================================================

#[utoipa::path(
    get,
    path = "/api/pull-requests/{number}",
    params(
        ("number" = u64, Path, description = "Pull request number")
    ),
    responses(
        (status = 200, description = "Pull request details", body = PullRequestDetail),
        (status = 404, description = "Pull request not found"),
        (status = 500, description = "GitHub API error")
    ),
    tag = "pull-requests"
)]
pub async fn get_pull_request(
    State(state): State<AppState>,
    Path(number): Path<u64>,
) -> Result<Json<PullRequestDetail>, AppError> {
    debug!("Getting pull request #{}", number);

    let github = state.github_client().await.map_err(|e| {
        error!("Failed to get GitHub client: {}", e);
        AppError::Internal(format!("GitHub client error: {}", e))
    })?;

    let pr = github.get_pull_request_detail(number).await.map_err(|e| {
        error!("Failed to get pull request #{}: {}", number, e);
        AppError::Internal(format!("GitHub API error: {}", e))
    })?;

    Ok(Json(pr))
}

// =============================================================================
// Get Pull Request Diff
// =============================================================================

#[utoipa::path(
    get,
    path = "/api/pull-requests/{number}/diff",
    params(
        ("number" = u64, Path, description = "Pull request number")
    ),
    responses(
        (status = 200, description = "Pull request diff", body = PrDiffResponse),
        (status = 500, description = "GitHub API error")
    ),
    tag = "pull-requests"
)]
pub async fn get_pull_request_diff(
    State(state): State<AppState>,
    Path(number): Path<u64>,
) -> Result<Json<PrDiffResponse>, AppError> {
    debug!("Getting diff for pull request #{}", number);

    let github = state.github_client().await.map_err(|e| {
        error!("Failed to get GitHub client: {}", e);
        AppError::Internal(format!("GitHub client error: {}", e))
    })?;

    let diff = github.get_pr_diff(number).await.map_err(|e| {
        error!("Failed to get diff for PR #{}: {}", number, e);
        AppError::Internal(format!("GitHub API error: {}", e))
    })?;

    let files = github.get_pr_files(number).await.map_err(|e| {
        error!("Failed to get files for PR #{}: {}", number, e);
        AppError::Internal(format!("GitHub API error: {}", e))
    })?;

    Ok(Json(PrDiffResponse { diff, files }))
}

// =============================================================================
// Get Pull Request Files
// =============================================================================

#[utoipa::path(
    get,
    path = "/api/pull-requests/{number}/files",
    params(
        ("number" = u64, Path, description = "Pull request number")
    ),
    responses(
        (status = 200, description = "Changed files", body = Vec<PrFile>),
        (status = 500, description = "GitHub API error")
    ),
    tag = "pull-requests"
)]
pub async fn get_pull_request_files(
    State(state): State<AppState>,
    Path(number): Path<u64>,
) -> Result<Json<Vec<PrFile>>, AppError> {
    debug!("Getting files for pull request #{}", number);

    let github = state.github_client().await.map_err(|e| {
        error!("Failed to get GitHub client: {}", e);
        AppError::Internal(format!("GitHub client error: {}", e))
    })?;

    let files = github.get_pr_files(number).await.map_err(|e| {
        error!("Failed to get files for PR #{}: {}", number, e);
        AppError::Internal(format!("GitHub API error: {}", e))
    })?;

    Ok(Json(files))
}

// =============================================================================
// Get Pull Request Comments
// =============================================================================

#[utoipa::path(
    get,
    path = "/api/pull-requests/{number}/comments",
    params(
        ("number" = u64, Path, description = "Pull request number")
    ),
    responses(
        (status = 200, description = "All PR comments", body = PrCommentsResponse),
        (status = 500, description = "GitHub API error")
    ),
    tag = "pull-requests"
)]
pub async fn get_pull_request_comments(
    State(state): State<AppState>,
    Path(number): Path<u64>,
) -> Result<Json<PrCommentsResponse>, AppError> {
    debug!("Getting comments for pull request #{}", number);

    let github = state.github_client().await.map_err(|e| {
        error!("Failed to get GitHub client: {}", e);
        AppError::Internal(format!("GitHub client error: {}", e))
    })?;

    let review_comments = github.get_pr_review_comments(number).await.map_err(|e| {
        error!("Failed to get review comments for PR #{}: {}", number, e);
        AppError::Internal(format!("GitHub API error: {}", e))
    })?;

    let issue_comments = github.get_pr_issue_comments(number).await.map_err(|e| {
        error!("Failed to get issue comments for PR #{}: {}", number, e);
        AppError::Internal(format!("GitHub API error: {}", e))
    })?;

    Ok(Json(PrCommentsResponse {
        review_comments,
        issue_comments,
    }))
}

// =============================================================================
// Create Review Comment
// =============================================================================

#[utoipa::path(
    post,
    path = "/api/pull-requests/{number}/comments",
    params(
        ("number" = u64, Path, description = "Pull request number")
    ),
    request_body = CreatePrCommentRequest,
    responses(
        (status = 201, description = "Comment created", body = PrReviewComment),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "GitHub API error")
    ),
    tag = "pull-requests"
)]
#[instrument(skip(state), fields(pr_number = %number))]
pub async fn create_review_comment(
    State(state): State<AppState>,
    Path(number): Path<u64>,
    Json(payload): Json<CreatePrCommentRequest>,
) -> Result<(StatusCode, Json<PrReviewComment>), AppError> {
    info!(
        "Creating review comment on PR #{} at {}:{}",
        number, payload.path, payload.line
    );

    let github = state.github_client().await.map_err(|e| {
        error!("Failed to get GitHub client: {}", e);
        AppError::Internal(format!("GitHub client error: {}", e))
    })?;

    let side = match payload.side.to_uppercase().as_str() {
        "LEFT" => DiffSide::Left,
        _ => DiffSide::Right,
    };

    let request = CreateReviewCommentRequest {
        path: payload.path,
        line: payload.line,
        side,
        body: payload.body,
        commit_id: payload.commit_id,
        in_reply_to: payload.in_reply_to,
    };

    let comment = github
        .create_review_comment(number, request)
        .await
        .map_err(|e| {
            error!("Failed to create review comment on PR #{}: {}", number, e);
            AppError::Internal(format!("GitHub API error: {}", e))
        })?;

    Ok((StatusCode::CREATED, Json(comment)))
}

// =============================================================================
// Reply to Review Comment
// =============================================================================

#[utoipa::path(
    post,
    path = "/api/pull-requests/{number}/comments/{comment_id}/reply",
    params(
        ("number" = u64, Path, description = "Pull request number"),
        ("comment_id" = u64, Path, description = "Comment ID to reply to")
    ),
    request_body = ReplyToCommentRequest,
    responses(
        (status = 201, description = "Reply created", body = PrReviewComment),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "GitHub API error")
    ),
    tag = "pull-requests"
)]
#[instrument(skip(state), fields(pr_number = %number, comment_id = %comment_id))]
pub async fn reply_to_comment(
    State(state): State<AppState>,
    Path((number, comment_id)): Path<(u64, u64)>,
    Json(payload): Json<ReplyToCommentRequest>,
) -> Result<(StatusCode, Json<PrReviewComment>), AppError> {
    info!("Replying to comment {} on PR #{}", comment_id, number);

    let github = state.github_client().await.map_err(|e| {
        error!("Failed to get GitHub client: {}", e);
        AppError::Internal(format!("GitHub client error: {}", e))
    })?;

    let comment = github
        .reply_to_review_comment(number, comment_id, &payload.body)
        .await
        .map_err(|e| {
            error!(
                "Failed to reply to comment {} on PR #{}: {}",
                comment_id, number, e
            );
            AppError::Internal(format!("GitHub API error: {}", e))
        })?;

    Ok((StatusCode::CREATED, Json(comment)))
}

// =============================================================================
// Get Pull Request Reviews
// =============================================================================

#[utoipa::path(
    get,
    path = "/api/pull-requests/{number}/reviews",
    params(
        ("number" = u64, Path, description = "Pull request number")
    ),
    responses(
        (status = 200, description = "PR reviews", body = PrReviewsResponse),
        (status = 500, description = "GitHub API error")
    ),
    tag = "pull-requests"
)]
pub async fn get_pull_request_reviews(
    State(state): State<AppState>,
    Path(number): Path<u64>,
) -> Result<Json<PrReviewsResponse>, AppError> {
    debug!("Getting reviews for pull request #{}", number);

    let github = state.github_client().await.map_err(|e| {
        error!("Failed to get GitHub client: {}", e);
        AppError::Internal(format!("GitHub client error: {}", e))
    })?;

    let reviews = github.get_pr_reviews(number).await.map_err(|e| {
        error!("Failed to get reviews for PR #{}: {}", number, e);
        AppError::Internal(format!("GitHub API error: {}", e))
    })?;

    Ok(Json(PrReviewsResponse { reviews }))
}

// =============================================================================
// Fix from PR Comments (Create Task from Comments)
// =============================================================================

#[utoipa::path(
    post,
    path = "/api/pull-requests/{number}/fix",
    params(
        ("number" = u64, Path, description = "Pull request number")
    ),
    request_body = FixFromCommentsRequest,
    responses(
        (status = 201, description = "Task created from comments", body = FixFromCommentsResponse),
        (status = 400, description = "Invalid request - no comments provided"),
        (status = 500, description = "GitHub API error or task creation error")
    ),
    tag = "pull-requests"
)]
#[instrument(skip(state), fields(pr_number = %number))]
pub async fn fix_from_pr_comments(
    State(state): State<AppState>,
    Path(number): Path<u64>,
    Json(payload): Json<FixFromCommentsRequest>,
) -> Result<(StatusCode, Json<FixFromCommentsResponse>), AppError> {
    info!(
        "Creating fix task from {} comments on PR #{}",
        payload.comment_ids.len(),
        number
    );

    if payload.comment_ids.is_empty() {
        return Err(AppError::BadRequest(
            "At least one comment ID is required".to_string(),
        ));
    }

    // Get GitHub client
    let github = state.github_client().await.map_err(|e| {
        error!("Failed to get GitHub client: {}", e);
        AppError::Internal(format!("GitHub client error: {}", e))
    })?;

    // Get PR details for context
    let pr = github.get_pull_request_detail(number).await.map_err(|e| {
        error!("Failed to get PR #{}: {}", number, e);
        AppError::Internal(format!("GitHub API error: {}", e))
    })?;

    // Get all review comments
    let all_comments = github.get_pr_review_comments(number).await.map_err(|e| {
        error!("Failed to get comments for PR #{}: {}", number, e);
        AppError::Internal(format!("GitHub API error: {}", e))
    })?;

    // Filter to selected comments
    let selected_comments: Vec<_> = all_comments
        .into_iter()
        .filter(|c| payload.comment_ids.contains(&c.id))
        .collect();

    if selected_comments.is_empty() {
        return Err(AppError::BadRequest(
            "No matching comments found for the provided IDs".to_string(),
        ));
    }

    // Build task description from comments
    let mut description = format!(
        "Fix issues from PR #{} ({})\n\n## Review Comments to Address:\n\n",
        number, pr.title
    );

    for comment in &selected_comments {
        description.push_str(&format!(
            "### `{}` (line {})\n**@{}**: {}\n\n",
            comment.path,
            comment.line.unwrap_or(0),
            comment.user.login,
            comment.body
        ));
    }

    description.push_str(&format!(
        "\n---\n*Branch: {} → {}*\n*PR URL: {}*",
        pr.head_branch, pr.base_branch, pr.html_url
    ));

    // Create task
    let task_title = if selected_comments.len() == 1 {
        format!(
            "Fix: {} in {}",
            truncate_string(&selected_comments[0].body, 50),
            selected_comments[0]
                .path
                .split('/')
                .next_back()
                .unwrap_or("file")
        )
    } else {
        format!(
            "Fix {} review comments from PR #{}",
            selected_comments.len(),
            number
        )
    };

    let project = state.project().await?;
    let task = Task::new(task_title.clone(), description);
    let created = project.task_repository.create(&task).await?;

    info!(
        task_id = %created.id,
        comments_count = selected_comments.len(),
        "Created fix task from PR comments"
    );

    // Publish event
    state
        .event_bus
        .publish(EventEnvelope::new(Event::TaskCreated {
            task_id: created.id,
            title: task_title,
        }));

    Ok((
        StatusCode::CREATED,
        Json(FixFromCommentsResponse {
            task: created,
            comments_included: selected_comments.len(),
        }),
    ))
}

fn truncate_string(s: &str, max_len: usize) -> String {
    let s = s.replace('\n', " ").trim().to_string();
    if s.len() <= max_len {
        s
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
