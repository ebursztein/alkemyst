use alkemyst_core::{Application, db::Engine};
mod client;
pub use client::ClientHarness;
pub fn app() -> (Engine, Application) {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    (db.clone(), Application::new(db))
}
