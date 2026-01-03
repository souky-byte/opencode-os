use axum::extract::Query;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use utoipa::ToSchema;

use crate::project_manager::detect_vcs;

#[derive(Debug, Deserialize, ToSchema)]
pub struct BrowseQuery {
    #[serde(default = "default_path")]
    pub path: String,
}

fn default_path() -> String {
    dirs::home_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "/".to_string())
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_vcs_root: bool,
    pub vcs: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BrowseResponse {
    pub current_path: String,
    pub parent_path: Option<String>,
    pub entries: Vec<DirectoryEntry>,
    pub is_vcs_root: bool,
    pub vcs: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/filesystem/browse",
    params(
        ("path" = Option<String>, Query, description = "Directory path to browse")
    ),
    responses(
        (status = 200, description = "Directory listing", body = BrowseResponse),
        (status = 400, description = "Invalid path")
    ),
    tag = "filesystem"
)]
pub async fn browse_directory(Query(query): Query<BrowseQuery>) -> Json<BrowseResponse> {
    let path = PathBuf::from(&query.path);

    if !path.exists() || !path.is_dir() {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        return Json(browse_path(&home));
    }

    Json(browse_path(&path))
}

fn browse_path(path: &PathBuf) -> BrowseResponse {
    let current_vcs = detect_vcs(path);
    let is_vcs_root = current_vcs != "none";

    let parent_path = path.parent().map(|p| p.display().to_string());

    let mut entries: Vec<DirectoryEntry> = std::fs::read_dir(path)
        .ok()
        .map(|read_dir| {
            read_dir
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();

                    if name.starts_with('.') {
                        return None;
                    }

                    let is_dir = path.is_dir();
                    if !is_dir {
                        return None;
                    }

                    let vcs = detect_vcs(&path);
                    let is_vcs_root = vcs != "none";

                    Some(DirectoryEntry {
                        name,
                        path: path.display().to_string(),
                        is_dir,
                        is_vcs_root,
                        vcs: is_vcs_root.then_some(vcs.to_string()),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    BrowseResponse {
        current_path: path.display().to_string(),
        parent_path,
        entries,
        is_vcs_root,
        vcs: is_vcs_root.then_some(current_vcs.to_string()),
    }
}
