//! A goldfish turn model, not a complete Magic rules engine. The caller supplies
//! explicit modeled abilities; card text is never interpreted optimistically.
mod actions;
mod models;
mod resources;
mod trial;
pub use models::*;
pub use trial::Trial;
