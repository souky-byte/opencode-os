//! Git operations for wiki indexing
//!
//! Provides utilities for cloning remote repositories and getting commit info.

use std::path::Path;
use std::process::Command;
use tracing::{debug, info, warn};
use url::Url;

use crate::error::{WikiError, WikiResult};

/// Detect repository type from URL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoType {
    GitHub,
    GitLab,
    Bitbucket,
    Generic,
}

impl RepoType {
    /// Detect repo type from URL
    pub fn from_url(url: &str) -> Self {
        let url_lower = url.to_lowercase();
        if url_lower.contains("github.com") {
            RepoType::GitHub
        } else if url_lower.contains("gitlab.com") || url_lower.contains("gitlab") {
            RepoType::GitLab
        } else if url_lower.contains("bitbucket.org") || url_lower.contains("bitbucket") {
            RepoType::Bitbucket
        } else {
            RepoType::Generic
        }
    }
}

/// Inject authentication token into repository URL
///
/// Supports GitHub, GitLab, and Bitbucket authentication patterns.
pub fn inject_token_into_url(url: &str, token: &str, repo_type: RepoType) -> WikiResult<String> {
    let parsed =
        Url::parse(url).map_err(|e| WikiError::InvalidConfig(format!("Invalid URL: {}", e)))?;

    let scheme = parsed.scheme();
    let host = parsed
        .host_str()
        .ok_or_else(|| WikiError::InvalidConfig("URL missing host".to_string()))?;
    let path = parsed.path();

    // URL-encode the token to handle special characters
    let encoded_token = urlencoding::encode(token);

    let auth_url = match repo_type {
        RepoType::GitHub => {
            // GitHub: https://<token>@github.com/owner/repo
            format!("{}://{}@{}{}", scheme, encoded_token, host, path)
        }
        RepoType::GitLab => {
            // GitLab: https://oauth2:<token>@gitlab.com/owner/repo
            format!("{}://oauth2:{}@{}{}", scheme, encoded_token, host, path)
        }
        RepoType::Bitbucket => {
            // Bitbucket: https://x-token-auth:<token>@bitbucket.org/owner/repo
            format!(
                "{}://x-token-auth:{}@{}{}",
                scheme, encoded_token, host, path
            )
        }
        RepoType::Generic => {
            // Generic: Use GitHub-style auth
            format!("{}://{}@{}{}", scheme, encoded_token, host, path)
        }
    };

    Ok(auth_url)
}

/// Perform a shallow clone of a remote repository
///
/// Clones only the specified branch with depth=1 for efficiency.
/// Returns the commit SHA of the cloned repository.
pub fn shallow_clone(
    repo_url: &str,
    branch: &str,
    access_token: Option<&str>,
    target_dir: &Path,
) -> WikiResult<String> {
    info!(
        repo_url = %repo_url,
        branch = %branch,
        target_dir = %target_dir.display(),
        "Starting shallow clone"
    );

    // Prepare the clone URL (with or without auth)
    let clone_url = if let Some(token) = access_token {
        let repo_type = RepoType::from_url(repo_url);
        inject_token_into_url(repo_url, token, repo_type)?
    } else {
        repo_url.to_string()
    };

    // Ensure target directory exists
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir).map_err(|e| {
            WikiError::IoError(format!(
                "Failed to create target directory {}: {}",
                target_dir.display(),
                e
            ))
        })?;
    }

    // Run git clone with shallow options
    let output = Command::new("git")
        .args([
            "clone",
            "--depth=1",
            "--single-branch",
            "-b",
            branch,
            &clone_url,
            ".",
        ])
        .current_dir(target_dir)
        .output()
        .map_err(|e| WikiError::IoError(format!("Failed to execute git clone: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Sanitize error message to avoid leaking tokens
        let sanitized_error = if access_token.is_some() {
            stderr.replace(access_token.unwrap_or(""), "[REDACTED]")
        } else {
            stderr.to_string()
        };
        warn!(error = %sanitized_error, "Git clone failed");
        return Err(WikiError::GitError(format!(
            "Git clone failed: {}",
            sanitized_error
        )));
    }

    info!("Shallow clone completed successfully");

    // Get the commit SHA
    get_head_sha(target_dir)
}

/// Get the HEAD commit SHA from a git repository
pub fn get_head_sha(repo_path: &Path) -> WikiResult<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| WikiError::IoError(format!("Failed to execute git rev-parse: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WikiError::GitError(format!(
            "Failed to get HEAD SHA: {}",
            stderr
        )));
    }

    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    debug!(sha = %sha, "Got HEAD SHA");
    Ok(sha)
}

/// Get the current branch name from a git repository
pub fn get_current_branch(repo_path: &Path) -> WikiResult<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| WikiError::IoError(format!("Failed to execute git rev-parse: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WikiError::GitError(format!(
            "Failed to get current branch: {}",
            stderr
        )));
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    debug!(branch = %branch, "Got current branch");
    Ok(branch)
}

/// Check if a directory is a git repository
pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

/// Get the remote URL (origin) from a git repository
pub fn get_remote_url(repo_path: &Path) -> WikiResult<Option<String>> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| WikiError::IoError(format!("Failed to execute git remote: {}", e)))?;

    if !output.status.success() {
        return Ok(None);
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        return Ok(None);
    }

    debug!(url = %url, "Got remote URL");
    Ok(Some(url))
}

/// List all remote branches from the repository
pub fn list_remote_branches(repo_path: &Path) -> WikiResult<Vec<String>> {
    let output = Command::new("git")
        .args(["ls-remote", "--heads", "origin"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| WikiError::IoError(format!("Failed to execute git ls-remote: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!(error = %stderr, "Failed to list remote branches");
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            line.split('\t')
                .nth(1)
                .and_then(|ref_name| ref_name.strip_prefix("refs/heads/"))
                .map(|s| s.to_string())
        })
        .collect();

    debug!(count = branches.len(), "Listed remote branches");
    Ok(branches)
}

/// List local branches from the repository
pub fn list_local_branches(repo_path: &Path) -> WikiResult<Vec<String>> {
    let output = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| WikiError::IoError(format!("Failed to execute git branch: {}", e)))?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let branches: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(branches)
}

/// Clean up a cloned repository directory
pub fn cleanup_clone(target_dir: &Path) -> WikiResult<()> {
    if target_dir.exists() {
        std::fs::remove_dir_all(target_dir).map_err(|e| {
            WikiError::IoError(format!(
                "Failed to cleanup directory {}: {}",
                target_dir.display(),
                e
            ))
        })?;
        debug!(dir = %target_dir.display(), "Cleaned up clone directory");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_type_detection() {
        assert_eq!(
            RepoType::from_url("https://github.com/owner/repo"),
            RepoType::GitHub
        );
        assert_eq!(
            RepoType::from_url("https://gitlab.com/owner/repo"),
            RepoType::GitLab
        );
        assert_eq!(
            RepoType::from_url("https://bitbucket.org/owner/repo"),
            RepoType::Bitbucket
        );
        assert_eq!(
            RepoType::from_url("https://example.com/owner/repo"),
            RepoType::Generic
        );
    }

    #[test]
    fn test_inject_token_github() {
        let url = "https://github.com/owner/repo.git";
        let token = "ghp_abc123";
        let result = inject_token_into_url(url, token, RepoType::GitHub).unwrap();
        assert_eq!(result, "https://ghp_abc123@github.com/owner/repo.git");
    }

    #[test]
    fn test_inject_token_gitlab() {
        let url = "https://gitlab.com/owner/repo.git";
        let token = "glpat-abc123";
        let result = inject_token_into_url(url, token, RepoType::GitLab).unwrap();
        assert_eq!(
            result,
            "https://oauth2:glpat-abc123@gitlab.com/owner/repo.git"
        );
    }

    #[test]
    fn test_inject_token_bitbucket() {
        let url = "https://bitbucket.org/owner/repo.git";
        let token = "abc123";
        let result = inject_token_into_url(url, token, RepoType::Bitbucket).unwrap();
        assert_eq!(
            result,
            "https://x-token-auth:abc123@bitbucket.org/owner/repo.git"
        );
    }

    #[test]
    fn test_inject_token_special_chars() {
        let url = "https://github.com/owner/repo.git";
        let token = "token@with/special=chars";
        let result = inject_token_into_url(url, token, RepoType::GitHub).unwrap();
        // Token should be URL-encoded
        assert!(result.contains("token%40with%2Fspecial%3Dchars"));
    }
}
