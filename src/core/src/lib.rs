//! Shared application core. Transports consume the same operations.
pub mod api;
mod application;
pub use application::Application;
pub mod catalog;
pub mod db;
pub mod domain;
mod health;
mod statements;
pub use api::ApiDoc;
pub use health::{BuildInfo, build_info};

pub mod mcp;

pub mod ingest;

pub mod history;
