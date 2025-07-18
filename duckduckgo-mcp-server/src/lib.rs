pub mod auth;
pub mod auth_routes;
pub mod client;
pub mod config;
pub mod duckduckgo;
pub mod mcp_handler;
pub mod mcp_types;

// Re-export functions from main for tests
pub use crate::auth_routes::health_check;
