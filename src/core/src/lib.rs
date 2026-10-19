//! Shared application core. Transports consume the same operations.
mod health;
pub use health::{ApiDoc, BuildInfo, build_info};
