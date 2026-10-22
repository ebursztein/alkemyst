use super::api_support::TOKEN;
use alkemyst_core::{
    Application,
    api::{AuthConfig, router},
    db::Engine,
};
use axum::Router;

pub fn setup() -> (Engine, Application, Router) {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let app = Application::new(db.clone());
    let routes = router(app.clone(), AuthConfig::new(TOKEN.into(), false).unwrap());
    (db, app, routes)
}
