use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info};
use utoipa::ToSchema;

use crate::config::WikiConfig as ProjectWikiConfig;
use crate::config::ProjectConfig;
use crate::error::AppError;
use crate::state::AppState;

use wiki::{
    CodeIndexer, IndexStatus, SearchResult, WikiConfig as WikiEngineConfig, WikiEngine, WikiPage,
    WikiStructure, WikiTree,
};

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct WikiStatusResponse {
    pub enabled: bool,
    pub configured: bool,
    pub branches: Vec<BranchStatus>,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct BranchStatus {
    pub branch: String,
    pub state: String,
    pub file_count: u32,
    pub chunk_count: u32,
    pub page_count: u32,
    pub last_indexed_at: Option<String>,
    pub progress_percent: u8,
    pub error_message: Option<String>,
    pub current_phase: Option<String>,
    pub current_item: Option<String>,
}

impl From<IndexStatus> for BranchStatus {
    fn from(status: IndexStatus) -> Self {
        Self {
            branch: status.branch,
            state: status.state.as_str().to_string(),
            file_count: status.file_count,
            chunk_count: status.chunk_count,
            page_count: status.page_count,
            last_indexed_at: status.last_indexed_at.map(|dt| dt.to_rfc3339()),
            progress_percent: status.progress_percent,
            error_message: status.error_message,
            current_phase: status.current_phase,
            current_item: status.current_item,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct IndexRequest {
    pub branch: Option<String>,
    pub force: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct IndexResponse {
    pub started: bool,
    pub branch: String,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct WikiStructureResponse {
    pub branch: String,
    pub root: WikiTreeNode,
    pub page_count: u32,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[schema(no_recursion)]
pub struct WikiTreeNode {
    pub slug: String,
    pub title: String,
    pub page_type: String,
    pub order: u32,
    pub children: Vec<WikiTreeNode>,
}

impl From<WikiTree> for WikiTreeNode {
    fn from(tree: WikiTree) -> Self {
        Self {
            slug: tree.slug,
            title: tree.title,
            page_type: tree.page_type.as_str().to_string(),
            order: tree.order,
            children: tree.children.into_iter().map(WikiTreeNode::from).collect(),
        }
    }
}

impl From<WikiStructure> for WikiStructureResponse {
    fn from(structure: WikiStructure) -> Self {
        Self {
            branch: structure.branch,
            root: WikiTreeNode::from(structure.root),
            page_count: structure.page_count,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct WikiPageResponse {
    pub slug: String,
    pub title: String,
    pub content: String,
    pub page_type: String,
    pub parent_slug: Option<String>,
    pub file_paths: Vec<String>,
    pub has_diagrams: bool,
    pub updated_at: String,
}

impl From<WikiPage> for WikiPageResponse {
    fn from(page: WikiPage) -> Self {
        Self {
            slug: page.slug,
            title: page.title,
            content: page.content,
            page_type: page.page_type.as_str().to_string(),
            parent_slug: page.parent_slug,
            file_paths: page.file_paths,
            has_diagrams: page.has_diagrams,
            updated_at: page.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct WikiSearchResponse {
    pub query: String,
    pub results: Vec<WikiSearchResult>,
    pub total_count: u32,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct WikiSearchResult {
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub content: String,
    pub language: Option<String>,
    pub score: f32,
}

impl From<SearchResult> for WikiSearchResult {
    fn from(result: SearchResult) -> Self {
        Self {
            file_path: result.file_path,
            start_line: result.start_line,
            end_line: result.end_line,
            content: result.content,
            language: result.language,
            score: result.score,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct AskRequest {
    pub question: String,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct AskResponse {
    pub answer: String,
    pub sources: Vec<AskSource>,
    pub conversation_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct AskSource {
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub score: f32,
    pub snippet: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct WebhookPushRequest {
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub after: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct WebhookResponse {
    pub accepted: bool,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct WikiSettingsResponse {
    pub enabled: bool,
    pub branches: Vec<String>,
    pub has_api_key: bool,
    pub embedding_model: Option<String>,
    pub chat_model: Option<String>,
    pub auto_sync: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct UpdateWikiSettingsRequest {
    pub enabled: Option<bool>,
    pub branches: Option<Vec<String>>,
    pub openrouter_api_key: Option<String>,
    pub embedding_model: Option<String>,
    pub chat_model: Option<String>,
    pub auto_sync: Option<bool>,
}

fn get_wiki_db_path(project_path: &std::path::Path) -> PathBuf {
    project_path.join(".opencode-studio").join("wiki.db")
}

fn create_wiki_engine(
    project_path: &std::path::Path,
    wiki_config: &ProjectWikiConfig,
) -> Result<WikiEngine, AppError> {
    let api_key = wiki_config
        .openrouter_api_key
        .clone()
        .ok_or_else(|| AppError::BadRequest("Wiki API key not configured".to_string()))?;

    let engine_config = WikiEngineConfig {
        branches: wiki_config.branches.clone(),
        openrouter_api_key: api_key,
        embedding_model: wiki_config
            .embedding_model
            .clone()
            .unwrap_or_else(|| "openai/text-embedding-3-small".to_string()),
        chat_model: wiki_config
            .chat_model
            .clone()
            .unwrap_or_else(|| "anthropic/claude-3.5-sonnet".to_string()),
        db_path: get_wiki_db_path(project_path),
        auto_sync: wiki_config.auto_sync,
        ..Default::default()
    };

    WikiEngine::new(engine_config).map_err(|e| {
        error!(error = %e, "Failed to create wiki engine");
        AppError::Internal(format!("Failed to initialize wiki: {}", e))
    })
}

#[utoipa::path(
    get,
    path = "/api/wiki/status",
    responses(
        (status = 200, description = "Wiki status", body = WikiStatusResponse),
        (status = 500, description = "Failed to get status")
    ),
    tag = "wiki"
)]
pub async fn get_wiki_status(State(state): State<AppState>) -> Result<Json<WikiStatusResponse>, AppError> {
    debug!("Getting wiki status");

    let project = state.project().await?;
    let config = ProjectConfig::read(&project.project_path).await;

    if !config.wiki.enabled || config.wiki.openrouter_api_key.is_none() {
        return Ok(Json(WikiStatusResponse {
            enabled: config.wiki.enabled,
            configured: config.wiki.openrouter_api_key.is_some(),
            branches: Vec::new(),
        }));
    }

    let engine = create_wiki_engine(&project.project_path, &config.wiki)?;
    let mut branches = Vec::new();

    for branch_name in &config.wiki.branches {
        let status = engine
            .get_index_status(branch_name)
            .map_err(|e| AppError::Internal(format!("Failed to get index status: {}", e)))?
            .unwrap_or_else(|| IndexStatus::new(branch_name.clone()));
        branches.push(BranchStatus::from(status));
    }

    Ok(Json(WikiStatusResponse {
        enabled: config.wiki.enabled,
        configured: true,
        branches,
    }))
}

#[utoipa::path(
    post,
    path = "/api/wiki/index",
    request_body = IndexRequest,
    responses(
        (status = 200, description = "Indexing started", body = IndexResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Failed to start indexing")
    ),
    tag = "wiki"
)]
pub async fn start_indexing(
    State(state): State<AppState>,
    Json(payload): Json<IndexRequest>,
) -> Result<Json<IndexResponse>, AppError> {
    info!("Starting wiki indexing");

    let project = state.project().await?;
    let config = ProjectConfig::read(&project.project_path).await;

    if !config.wiki.enabled {
        return Err(AppError::BadRequest("Wiki is not enabled".to_string()));
    }

    let branch = payload
        .branch
        .unwrap_or_else(|| config.wiki.branches.first().cloned().unwrap_or_else(|| "main".to_string()));

    let force = payload.force.unwrap_or(false);
    let engine = create_wiki_engine(&project.project_path, &config.wiki)?;

    let status = engine
        .get_index_status(&branch)
        .map_err(|e| AppError::Internal(format!("Failed to get index status: {}", e)))?;

    if let Some(ref s) = status {
        if s.is_indexing() && !force {
            return Ok(Json(IndexResponse {
                started: false,
                branch,
                message: "Indexing already in progress. Use force=true to restart.".to_string(),
            }));
        }
    }

    let project_path = project.project_path.clone();
    let wiki_config = config.wiki.clone();
    let branch_clone = branch.clone();

    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        let db_path = get_wiki_db_path(&project_path);
        rt.block_on(async {
            if let Err(e) = run_indexing(project_path, wiki_config, branch_clone.clone(), force).await {
                error!(error = %e, "Indexing failed");
                if let Ok(vs) = wiki::VectorStore::new(&db_path) {
                    let mut status = wiki::IndexStatus::new(branch_clone);
                    status.state = wiki::IndexState::Failed;
                    status.error_message = Some(e.to_string());
                    let _ = vs.update_index_status(&status);
                }
            }
        });
    });

    Ok(Json(IndexResponse {
        started: true,
        branch,
        message: "Indexing started".to_string(),
    }))
}

#[allow(clippy::arc_with_non_send_sync)]
async fn run_indexing(
    project_path: PathBuf,
    wiki_config: ProjectWikiConfig,
    branch: String,
    force: bool,
) -> Result<(), wiki::WikiError> {
    use wiki::IndexState;
    
    info!(branch = %branch, force = force, "Running indexing");

    let api_key = wiki_config
        .openrouter_api_key
        .ok_or_else(|| wiki::WikiError::InvalidConfig("API key not configured".to_string()))?;

    let embedding_model = wiki_config
        .embedding_model
        .unwrap_or_else(|| "openai/text-embedding-3-small".to_string());
    let chat_model = wiki_config
        .chat_model
        .unwrap_or_else(|| "anthropic/claude-sonnet-4-20250514".to_string());
    let db_path = get_wiki_db_path(&project_path);

    let openrouter = Arc::new(wiki::OpenRouterClient::new(api_key, "https://openrouter.ai/api/v1".to_string()));
    let vector_store = Arc::new(wiki::VectorStore::new(&db_path)?);

    if force {
        info!(branch = %branch, "Force flag set, clearing existing data");
        vector_store.clear_branch(&branch)?;
    }
    
    let indexer = CodeIndexer::new(
        openrouter.clone(),
        vector_store.clone(),
        embedding_model,
        350,
        100,
    );

    let commit_sha = get_current_commit_sha(&project_path).unwrap_or_else(|| "unknown".to_string());
    
    indexer.index_branch(&project_path, &branch, &commit_sha, None).await?;
    info!(branch = %branch, "Code indexing completed, generating wiki pages...");

    let mut status = vector_store.get_index_status(&branch)?.unwrap_or_else(|| wiki::IndexStatus::new(branch.clone()));
    status.state = IndexState::Generating;
    status.current_phase = Some("generating_wiki".to_string());
    status.progress_percent = 50;
    vector_store.update_index_status(&status)?;

    let generator = wiki::WikiGenerator::new(
        openrouter,
        vector_store.clone(),
        chat_model,
        350,
        100,
    );

    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let result = generator.generate_wiki(&project_path, project_name, &branch, &commit_sha, None).await;

    let mut final_status = vector_store.get_index_status(&branch)?.unwrap_or_else(|| wiki::IndexStatus::new(branch.clone()));
    match &result {
        Ok(structure) => {
            final_status.state = IndexState::Indexed;
            final_status.page_count = structure.page_count;
            final_status.progress_percent = 100;
            final_status.current_phase = None;
            final_status.current_item = None;
            final_status.last_indexed_at = Some(chrono::Utc::now());
            vector_store.update_index_status(&final_status)?;
            info!(branch = %branch, pages = structure.page_count, "Wiki generation completed");
        }
        Err(e) => {
            final_status.state = IndexState::Failed;
            final_status.error_message = Some(e.to_string());
            final_status.current_phase = None;
            final_status.current_item = None;
            vector_store.update_index_status(&final_status)?;
            error!(branch = %branch, error = %e, "Wiki generation failed");
        }
    }

    result.map(|_| ())
}

fn get_current_commit_sha(project_path: &std::path::Path) -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_path)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

#[utoipa::path(
    get,
    path = "/api/wiki/structure",
    params(
        ("branch" = Option<String>, Query, description = "Branch name (default: first configured branch)")
    ),
    responses(
        (status = 200, description = "Wiki structure", body = WikiStructureResponse),
        (status = 404, description = "Structure not found"),
        (status = 500, description = "Failed to get structure")
    ),
    tag = "wiki"
)]
pub async fn get_wiki_structure(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<WikiStructureResponse>, AppError> {
    debug!("Getting wiki structure");

    let project = state.project().await?;
    let config = ProjectConfig::read(&project.project_path).await;

    if !config.wiki.enabled {
        return Err(AppError::BadRequest("Wiki is not enabled".to_string()));
    }

    let branch = params
        .get("branch")
        .cloned()
        .unwrap_or_else(|| config.wiki.branches.first().cloned().unwrap_or_else(|| "main".to_string()));

    let engine = create_wiki_engine(&project.project_path, &config.wiki)?;

    let structure = engine
        .get_structure(&branch)
        .map_err(|e| AppError::Internal(format!("Failed to get structure: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Wiki structure not found for branch: {}", branch)))?;

    Ok(Json(WikiStructureResponse::from(structure)))
}

#[utoipa::path(
    get,
    path = "/api/wiki/pages/{slug}",
    params(
        ("slug" = String, Path, description = "Page slug")
    ),
    responses(
        (status = 200, description = "Wiki page", body = WikiPageResponse),
        (status = 404, description = "Page not found"),
        (status = 500, description = "Failed to get page")
    ),
    tag = "wiki"
)]
pub async fn get_wiki_page(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<WikiPageResponse>, AppError> {
    debug!(slug = %slug, "Getting wiki page");

    let project = state.project().await?;
    let config = ProjectConfig::read(&project.project_path).await;

    if !config.wiki.enabled {
        return Err(AppError::BadRequest("Wiki is not enabled".to_string()));
    }

    let engine = create_wiki_engine(&project.project_path, &config.wiki)?;

    let page = engine
        .get_page(&slug)
        .map_err(|e| AppError::Internal(format!("Failed to get page: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Wiki page not found: {}", slug)))?;

    Ok(Json(WikiPageResponse::from(page)))
}

#[utoipa::path(
    post,
    path = "/api/wiki/search",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search results", body = WikiSearchResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Search failed")
    ),
    tag = "wiki"
)]
pub async fn search_wiki(
    State(state): State<AppState>,
    Json(payload): Json<SearchRequest>,
) -> Result<Json<WikiSearchResponse>, AppError> {
    info!(query = %payload.query, "Searching wiki");

    let project = state.project().await?;
    let config = ProjectConfig::read(&project.project_path).await;

    if !config.wiki.enabled {
        return Err(AppError::BadRequest("Wiki is not enabled".to_string()));
    }

    let api_key = config
        .wiki
        .openrouter_api_key
        .clone()
        .ok_or_else(|| AppError::BadRequest("Wiki API key not configured".to_string()))?;
    let embedding_model = config
        .wiki
        .embedding_model
        .clone()
        .unwrap_or_else(|| "openai/text-embedding-3-small".to_string());
    let db_path = get_wiki_db_path(&project.project_path);
    let query = payload.query.clone();
    let limit = payload.limit.unwrap_or(10);

    let start = Instant::now();
    
    let openrouter = wiki::OpenRouterClient::new(api_key, "https://openrouter.ai/api/v1".to_string());
    let query_embedding = openrouter
        .create_embedding(&query, &embedding_model)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create embedding: {}", e)))?;

    let results = tokio::task::spawn_blocking(move || {
        let vector_store = wiki::VectorStore::new(&db_path)
            .map_err(|e| AppError::Internal(format!("Failed to open vector store: {}", e)))?;
        vector_store
            .search_similar(&query_embedding, limit)
            .map_err(|e| AppError::Internal(format!("Search failed: {}", e)))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))??;

    let duration_ms = start.elapsed().as_millis() as u64;

    let total_count = results.len() as u32;
    let search_results: Vec<WikiSearchResult> = results.into_iter().map(WikiSearchResult::from).collect();

    Ok(Json(WikiSearchResponse {
        query: payload.query,
        results: search_results,
        total_count,
        duration_ms,
    }))
}

#[utoipa::path(
    post,
    path = "/api/wiki/ask",
    request_body = AskRequest,
    responses(
        (status = 200, description = "RAG response", body = AskResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Ask failed")
    ),
    tag = "wiki"
)]
pub async fn ask_wiki(
    State(state): State<AppState>,
    Json(payload): Json<AskRequest>,
) -> Result<Json<AskResponse>, AppError> {
    info!(question = %payload.question, "Asking wiki");

    let project = state.project().await?;
    let config = ProjectConfig::read(&project.project_path).await;

    if !config.wiki.enabled {
        return Err(AppError::BadRequest("Wiki is not enabled".to_string()));
    }

    let api_key = config
        .wiki
        .openrouter_api_key
        .clone()
        .ok_or_else(|| AppError::BadRequest("Wiki API key not configured".to_string()))?;
    let embedding_model = config
        .wiki
        .embedding_model
        .clone()
        .unwrap_or_else(|| "openai/text-embedding-3-small".to_string());
    let chat_model = config
        .wiki
        .chat_model
        .clone()
        .unwrap_or_else(|| "anthropic/claude-3.5-sonnet".to_string());
    let db_path = get_wiki_db_path(&project.project_path);
    let question = payload.question.clone();
    let conversation_id = payload.conversation_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let openrouter = wiki::OpenRouterClient::new(api_key, "https://openrouter.ai/api/v1".to_string());
    
    let query_embedding = openrouter
        .create_embedding(&question, &embedding_model)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create embedding: {}", e)))?;

    let search_results = tokio::task::spawn_blocking(move || {
        let vector_store = wiki::VectorStore::new(&db_path)
            .map_err(|e| AppError::Internal(format!("Failed to open vector store: {}", e)))?;
        vector_store
            .search_similar(&query_embedding, 10)
            .map_err(|e| AppError::Internal(format!("Search failed: {}", e)))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))??;

    if search_results.is_empty() {
        return Ok(Json(AskResponse {
            answer: "I couldn't find any relevant code in the indexed codebase to answer your question.".to_string(),
            sources: Vec::new(),
            conversation_id,
        }));
    }

    let context = build_rag_context(&search_results);
    let sources: Vec<AskSource> = search_results
        .iter()
        .map(|r| AskSource {
            file_path: r.file_path.clone(),
            start_line: r.start_line,
            end_line: r.end_line,
            score: r.score,
            snippet: truncate_string(&r.content, 200),
        })
        .collect();

    let messages = vec![
        wiki::ChatMessage::system(RAG_SYSTEM_PROMPT),
        wiki::ChatMessage::user(format_rag_prompt(&question, &context)),
    ];

    let answer = openrouter
        .chat_completion(messages, &chat_model, Some(0.3), Some(2048))
        .await
        .map_err(|e| AppError::Internal(format!("Chat completion failed: {}", e)))?;

    Ok(Json(AskResponse {
        answer,
        sources,
        conversation_id,
    }))
}

const RAG_SYSTEM_PROMPT: &str = r#"You are a knowledgeable code assistant helping developers understand a codebase.
When answering:
- Reference specific files and line numbers when relevant (format: `file_path:line_number`)
- Provide concise but complete explanations
- Include code examples when helpful
- If the context doesn't contain enough information, say so clearly
- Don't make up information that's not in the provided context"#;

fn build_rag_context(results: &[SearchResult]) -> String {
    let mut context = String::new();
    for (i, result) in results.iter().enumerate() {
        context.push_str(&format!(
            "\n--- Source {}: {} (lines {}-{}) ---\n",
            i + 1,
            result.file_path,
            result.start_line,
            result.end_line
        ));
        if let Some(ref lang) = result.language {
            context.push_str(&format!("```{}\n{}\n```\n", lang, result.content));
        } else {
            context.push_str(&format!("```\n{}\n```\n", result.content));
        }
    }
    context
}

fn format_rag_prompt(query: &str, context: &str) -> String {
    format!(
        r#"Based on the following code snippets from the codebase, please answer this question:

**Question:** {}

**Relevant Code:**
{}

Please provide a clear and helpful answer based on the code context above."#,
        query, context
    )
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let boundary = s
            .char_indices()
            .take_while(|(i, _)| *i < max_len)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        format!("{}...", &s[..boundary])
    }
}

#[utoipa::path(
    post,
    path = "/api/wiki/webhook/push",
    request_body = WebhookPushRequest,
    responses(
        (status = 200, description = "Webhook processed", body = WebhookResponse),
        (status = 400, description = "Invalid request")
    ),
    tag = "wiki"
)]
pub async fn handle_push_webhook(
    State(state): State<AppState>,
    Json(payload): Json<WebhookPushRequest>,
) -> Result<Json<WebhookResponse>, AppError> {
    info!(git_ref = %payload.git_ref, commit = %payload.after, "Received push webhook");

    let project = state.project().await?;
    let config = ProjectConfig::read(&project.project_path).await;

    if !config.wiki.enabled || !config.wiki.auto_sync {
        return Ok(Json(WebhookResponse {
            accepted: false,
            message: "Auto-sync is disabled".to_string(),
        }));
    }

    let branch = payload
        .git_ref
        .strip_prefix("refs/heads/")
        .unwrap_or(&payload.git_ref)
        .to_string();

    if !config.wiki.branches.contains(&branch) {
        return Ok(Json(WebhookResponse {
            accepted: false,
            message: format!("Branch '{}' is not configured for indexing", branch),
        }));
    }

    let project_path = project.project_path.clone();
    let wiki_config = config.wiki.clone();
    let branch_clone = branch.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        if let Err(e) = rt.block_on(run_indexing(project_path, wiki_config, branch_clone, true)) {
            error!(error = %e, "Auto-sync indexing failed");
        }
    });

    Ok(Json(WebhookResponse {
        accepted: true,
        message: format!("Indexing started for branch: {}", branch),
    }))
}

#[utoipa::path(
    get,
    path = "/api/settings/wiki",
    responses(
        (status = 200, description = "Wiki settings", body = WikiSettingsResponse)
    ),
    tag = "settings"
)]
pub async fn get_wiki_settings(State(state): State<AppState>) -> Result<Json<WikiSettingsResponse>, AppError> {
    debug!("Getting wiki settings");

    let project = state.project().await?;
    let config = ProjectConfig::read(&project.project_path).await;

    Ok(Json(WikiSettingsResponse {
        enabled: config.wiki.enabled,
        branches: config.wiki.branches,
        has_api_key: config.wiki.openrouter_api_key.is_some(),
        embedding_model: config.wiki.embedding_model,
        chat_model: config.wiki.chat_model,
        auto_sync: config.wiki.auto_sync,
    }))
}

#[utoipa::path(
    put,
    path = "/api/settings/wiki",
    request_body = UpdateWikiSettingsRequest,
    responses(
        (status = 200, description = "Settings updated", body = WikiSettingsResponse),
        (status = 500, description = "Failed to save settings")
    ),
    tag = "settings"
)]
pub async fn update_wiki_settings(
    State(state): State<AppState>,
    Json(payload): Json<UpdateWikiSettingsRequest>,
) -> Result<Json<WikiSettingsResponse>, AppError> {
    info!("Updating wiki settings");

    let project = state.project().await?;
    let mut config = ProjectConfig::read(&project.project_path).await;

    if let Some(enabled) = payload.enabled {
        config.wiki.enabled = enabled;
    }
    if let Some(branches) = payload.branches {
        config.wiki.branches = branches;
    }
    if let Some(api_key) = payload.openrouter_api_key {
        config.wiki.openrouter_api_key = if api_key.is_empty() { None } else { Some(api_key) };
    }
    if let Some(model) = payload.embedding_model {
        config.wiki.embedding_model = if model.is_empty() { None } else { Some(model) };
    }
    if let Some(model) = payload.chat_model {
        config.wiki.chat_model = if model.is_empty() { None } else { Some(model) };
    }
    if let Some(auto_sync) = payload.auto_sync {
        config.wiki.auto_sync = auto_sync;
    }

    config.write(&project.project_path).await.map_err(|e| {
        error!(error = %e, "Failed to save wiki config");
        AppError::Internal(format!("Failed to save settings: {}", e))
    })?;

    debug!("Wiki settings saved successfully");

    Ok(Json(WikiSettingsResponse {
        enabled: config.wiki.enabled,
        branches: config.wiki.branches,
        has_api_key: config.wiki.openrouter_api_key.is_some(),
        embedding_model: config.wiki.embedding_model,
        chat_model: config.wiki.chat_model,
        auto_sync: config.wiki.auto_sync,
    }))
}
