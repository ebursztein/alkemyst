//! Development bootstrap. Credentials are supplied out of band, never printed.
#[path = "support/fixture_catalog.rs"]
mod fixture_catalog;
use alkemyst_core::{
    api::{AuthConfig, RunningServer},
    db::Engine,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let database = args.next().ok_or("database path required")?;
    let port = args
        .next()
        .map(|port| port.parse::<u16>())
        .transpose()?
        .unwrap_or(47831);
    let token = std::env::var("ALKEMYST_TOKEN")
        .map_err(|_| "set ALKEMYST_TOKEN to a local credential of at least 32 characters")?;
    let app = fixture_catalog::application(Engine::open(
        std::path::Path::new(&database),
        8 * 1024 * 1024,
    )?);
    let server = RunningServer::start(
        app,
        AuthConfig::new(token, true)?,
        ([127, 0, 0, 1], port).into(),
    )
    .await?;
    println!(
        "{}",
        serde_json::json!({"base_url":format!("http://{}",server.address()),"pid":std::process::id()})
    );
    tokio::signal::ctrl_c().await?;
    server.stop().await?;
    Ok(())
}
