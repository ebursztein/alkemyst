//! Shared application core. Transports consume the same operations.
pub mod catalog;
pub mod db;
pub mod domain;
mod health;
mod statements;
pub use health::{ApiDoc, BuildInfo, build_info};
