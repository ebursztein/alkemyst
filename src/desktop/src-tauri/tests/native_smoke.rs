use alkemyst_desktop::{NativeRuntime, RuntimeOptions, Tracking, run_smoke};
use std::sync::{Arc, Mutex};
use tauri::{Manager, RunEvent};

fn main() {
    let dir = tempfile::tempdir().unwrap();
    let current = dir.path().join("Player.log");
    std::fs::write(&current, "").unwrap();
    let outcome = Arc::new(Mutex::new(None));
    let result = outcome.clone();
    run_smoke(
        RuntimeOptions {
            database: dir.path().join("smoke.sqlite3"),
            tracking: Tracking::Files {
                previous: dir.path().join("Player-prev.log"),
                current: current.clone(),
            },
            port: 0,
            development: false,
        },
        move |app, event| {
            if matches!(event, RunEvent::Ready) {
                let app = app.clone();
                let current = current.clone();
                let result = result.clone();
                tauri::async_runtime::spawn(async move {
                    let checked = tokio::time::timeout(
                        std::time::Duration::from_secs(30),
                        exercise(&app, &current),
                    )
                    .await;
                    *result.lock().unwrap() = Some(
                        checked
                            .map_err(|_| "Native smoke timed out".to_owned())
                            .and_then(|v| v),
                    );
                    app.exit(0);
                });
            }
        },
    )
    .unwrap();
    outcome
        .lock()
        .unwrap()
        .take()
        .expect("Native app never reached Ready")
        .unwrap();
    let url = CONNECTION
        .lock()
        .unwrap()
        .take()
        .expect("Smoke did not reach MCP check");
    let stopped = tauri::async_runtime::block_on(async {
        reqwest::Client::new()
            .get(format!("{url}/v1/health"))
            .send()
            .await
            .is_err()
    });
    assert!(stopped, "Quit left the local API running");
    println!("Native smoke passed: window, tray, background capture, MCP, graceful Quit.");
}

async fn exercise(app: &tauri::AppHandle, current: &std::path::Path) -> Result<(), String> {
    let runtime = app.state::<Arc<NativeRuntime>>().inner().clone();
    let config = runtime.bootstrap().map_err(|e| e.to_string())?;
    let window = app
        .get_webview_window("main")
        .ok_or("Missing main window")?;
    if !window.is_visible().map_err(|e| e.to_string())? {
        return Err("Main window was not visible".into());
    }
    if app.tray_by_id("alkemyst").is_none() {
        return Err("Missing system tray".into());
    }
    // The signal requires the bundled React UI, a successful SDK deck read,
    // native settings, and an actual IPC mutation from the main webview.
    window.eval(r#"
        const ready = setInterval(() => {
            if (document.querySelector('h1')?.textContent === 'Decks' &&
                document.body.textContent.includes('Your next brew starts here')) {
                clearInterval(ready);
                window.__TAURI_INTERNALS__.invoke('native_settings').then(settings => {
                    if (typeof settings.launch_at_login !== 'boolean') throw Error('Invalid settings');
                    return window.__TAURI_INTERNALS__.invoke('set_tracking_paused', { paused: true });
                });
            }
        }, 100);
    "#).map_err(|e| e.to_string())?;
    while !runtime.tracking_paused().await {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    window
        .eval("window.__TAURI_INTERNALS__.invoke('set_tracking_paused', { paused: false })")
        .map_err(|e| e.to_string())?;
    while runtime.tracking_paused().await {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let client = reqwest::Client::new();
    window.close().map_err(|e| e.to_string())?;
    while window.is_visible().map_err(|e| e.to_string())? {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    if app.get_webview_window("main").is_none() {
        return Err("Close destroyed the window".into());
    }
    std::fs::write(
        current,
        include_str!("../../../core/tests/fixtures/arena-game.txt"),
    )
    .unwrap();
    loop {
        let capture: serde_json::Value = client
            .get(format!("{}/v1/capture", config.base_url))
            .bearer_auth(&config.token)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        if capture["games"].as_u64().unwrap_or(0) > 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    let initialize = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": { "name": "native-test", "version": "1" }
        }
    });
    let response = client
        .post(format!("{}/mcp", config.base_url))
        .bearer_auth(&config.token)
        .header("Accept", "application/json, text/event-stream")
        .json(&initialize)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err("MCP failed after closing the window".into());
    }
    // Keep a clone so the test can check shutdown after the real event loop exits.
    *CONNECTION.lock().unwrap() = Some(config.base_url);
    Ok(())
}

static CONNECTION: Mutex<Option<String>> = Mutex::new(None);
