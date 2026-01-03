//! RAII resource guards for automatic cleanup.
//!
//! This module provides guards that ensure resources are properly
//! cleaned up even in error scenarios:
//!
//! - [`McpGuard`] - Automatic MCP server disconnection
//! - [`SessionGuard`] - Automatic session failure handling

mod mcp_guard;
mod session_guard;

pub use mcp_guard::McpGuard;
pub use session_guard::SessionGuard;
