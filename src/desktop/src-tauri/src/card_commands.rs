use crate::{CardCacheHealth, NativeRuntime, commands::main_window};
use std::sync::Arc;
use tauri::{Runtime, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn native_card_cache_status<R: Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, Arc<NativeRuntime>>,
) -> Result<CardCacheHealth, String> {
    main_window(&window)?;
    Ok(runtime.card_cache_status().await)
}

#[tauri::command]
pub async fn retry_card_cache<R: Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, Arc<NativeRuntime>>,
) -> Result<(), String> {
    main_window(&window)?;
    runtime
        .retry_card_cache()
        .await
        .map_err(|_| "Card discovery is stopping.".into())
}

#[tauri::command]
pub async fn choose_card_cache<R: Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, Arc<NativeRuntime>>,
) -> Result<bool, String> {
    main_window(&window)?;
    let (sender, receiver) = tokio::sync::oneshot::channel();
    window
        .dialog()
        .file()
        .set_parent(&window)
        .set_title("Choose Arena's installed card database")
        .add_filter("Arena card database", &["mtga", "sqlite", "sqlite3", "db"])
        .pick_file(move |file| {
            let _ = sender.send(file);
        });
    let Some(file) = receiver
        .await
        .map_err(|_| "Could not open the file picker.")?
    else {
        return Ok(false);
    };
    let path = file
        .into_path()
        .map_err(|_| "Choose a local card database file.")?;
    runtime
        .choose_card_cache(path)
        .await
        .map_err(|_| "Card discovery is stopping.")?;
    Ok(true)
}
