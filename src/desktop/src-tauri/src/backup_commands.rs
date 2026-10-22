use crate::{NativeRuntime, RestoreStatus, commands::main_window};
use std::{path::PathBuf, sync::Arc};
use tauri::{AppHandle, Runtime, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

async fn choose<R: Runtime>(
    window: &WebviewWindow<R>,
    saving: bool,
) -> Result<Option<PathBuf>, String> {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let dialog = window
        .dialog()
        .file()
        .set_parent(window)
        .add_filter("Alkemyst backup", &["sqlite3"]);
    if saving {
        dialog
            .set_title("Save a new Alkemyst backup")
            .set_file_name("alkemyst-backup.sqlite3")
            .save_file(move |path| {
                let _ = sender.send(path);
            });
    } else {
        dialog
            .set_title("Choose an Alkemyst backup")
            .pick_file(move |path| {
                let _ = sender.send(path);
            });
    }
    receiver
        .await
        .map_err(|_| "The file dialog could not be opened.".to_owned())?
        .map(|path| {
            path.into_path()
                .map_err(|_| "Choose a local file.".to_owned())
        })
        .transpose()
}

#[tauri::command]
pub async fn create_backup<R: Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, Arc<NativeRuntime>>,
) -> Result<Option<String>, String> {
    main_window(&window)?;
    let Some(path) = choose(&window, true).await? else {
        return Ok(None);
    };
    runtime.backup_to(&path).await.map_err(|_| {
        "Could not save the backup. Choose a new file name and check available disk space."
            .to_owned()
    })?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[tauri::command]
pub async fn select_restore<R: Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, Arc<NativeRuntime>>,
) -> Result<Option<String>, String> {
    main_window(&window)?;
    let Some(path) = choose(&window, false).await? else {
        return Ok(None);
    };
    runtime.prepare_restore(&path).await.map_err(|_| {
        "Could not prepare this backup. Select a valid backup from the same Alkemyst version and check available disk space.".to_owned()
    })?;
    Ok(Some(
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
    ))
}

#[tauri::command]
pub async fn cancel_restore<R: Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, Arc<NativeRuntime>>,
) -> Result<(), String> {
    main_window(&window)?;
    runtime.cancel_restore().await;
    Ok(())
}

#[tauri::command]
pub async fn restore_and_restart<R: Runtime>(
    window: WebviewWindow<R>,
    app: AppHandle<R>,
    runtime: State<'_, Arc<NativeRuntime>>,
) -> Result<(), String> {
    main_window(&window)?;
    runtime.confirm_restore().await.map_err(|_| {
        "Could not schedule the restore. Your current library is unchanged.".to_owned()
    })?;
    // request_restart delivers ExitRequested so the existing shutdown handler
    // stops capture, HTTP/MCP, simulation jobs and SQLite before process exit.
    app.request_restart();
    Ok(())
}

#[tauri::command]
pub fn native_restore_status<R: Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, Arc<NativeRuntime>>,
) -> Result<RestoreStatus, String> {
    main_window(&window)?;
    Ok(runtime.restore_status())
}
