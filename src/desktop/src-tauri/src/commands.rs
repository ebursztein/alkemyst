use crate::{LocalConnection, NativeRuntime};
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime, State, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;

pub(crate) fn main_window<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    if window.label() == "main" {
        Ok(())
    } else {
        Err("Native integration is only available to the main window.".into())
    }
}

#[tauri::command]
fn connection_bootstrap<R: Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, Arc<NativeRuntime>>,
) -> Result<LocalConnection, String> {
    main_window(&window)?;
    runtime
        .bootstrap()
        .map_err(|_| "Local services are stopping.".into())
}

#[derive(Serialize)]
struct NativeSettings {
    launch_at_login: bool,
    tracking_paused: bool,
    tracking: alkemyst_core::ingest::TrackingHealth,
}

#[tauri::command]
async fn native_settings<R: Runtime>(
    window: WebviewWindow<R>,
    app: AppHandle<R>,
    runtime: State<'_, Arc<NativeRuntime>>,
) -> Result<NativeSettings, String> {
    main_window(&window)?;
    let launch_at_login = app
        .autolaunch()
        .is_enabled()
        .map_err(|_| "Could not read launch-at-login settings.")?;
    let tracking = runtime.tracking_status().await;
    Ok(NativeSettings {
        launch_at_login,
        tracking_paused: tracking.paused,
        tracking: tracking.health,
    })
}

#[tauri::command]
fn set_launch_at_login<R: Runtime>(
    window: WebviewWindow<R>,
    app: AppHandle<R>,
    enabled: bool,
) -> Result<(), String> {
    main_window(&window)?;
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    result.map_err(|_| "Could not change launch-at-login settings.".into())
}

#[tauri::command]
async fn set_tracking_paused<R: Runtime>(
    window: WebviewWindow<R>,
    runtime: State<'_, Arc<NativeRuntime>>,
    paused: bool,
) -> Result<(), String> {
    main_window(&window)?;
    runtime
        .set_tracking_paused(paused)
        .await
        .map_err(|_| "Could not change Arena tracking.".to_owned())?;
    crate::menus::sync_pause(window.app_handle(), runtime.tracking_paused().await);
    Ok(())
}

pub fn configure_commands<R: Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
        connection_bootstrap,
        native_settings,
        set_launch_at_login,
        set_tracking_paused,
        crate::backup_commands::create_backup,
        crate::backup_commands::select_restore,
        crate::backup_commands::cancel_restore,
        crate::backup_commands::restore_and_restart,
        crate::backup_commands::native_restore_status,
    ])
}
