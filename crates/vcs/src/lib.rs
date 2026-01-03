pub mod error;
pub mod git;
pub mod jj;
pub mod traits;
pub mod workspace;

pub use error::{Result, VcsError};
pub use git::GitVcs;
pub use jj::JujutsuVcs;
pub use traits::{
    ConflictFile, ConflictType, MergeResult, VersionControl, Workspace, WorkspaceStatus,
};
pub use workspace::{WorkspaceConfig, WorkspaceManager};
