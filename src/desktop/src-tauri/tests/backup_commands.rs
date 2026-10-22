use alkemyst_desktop::{NativeRuntime, RuntimeOptions, Tracking, configure_commands};
use std::sync::Arc;
use tauri::{WebviewWindowBuilder, ipc::InvokeBody, test::*};

#[test]
fn backup_commands_reject_other_windows_before_opening_any_native_dialog() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Arc::new(
        tauri::async_runtime::block_on(NativeRuntime::start(RuntimeOptions {
            database: dir.path().join("alkemyst.sqlite3"),
            tracking: Tracking::Disabled,
            port: 0,
            development: false,
        }))
        .unwrap(),
    );
    let app = configure_commands(mock_builder())
        .manage(runtime.clone())
        .build(mock_context(noop_assets()))
        .unwrap();
    let main = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let other = WebviewWindowBuilder::new(&app, "other", Default::default())
        .build()
        .unwrap();
    let invoke = |window: &tauri::WebviewWindow<tauri::test::MockRuntime>, command: &str| {
        get_ipc_response(
            window,
            tauri::webview::InvokeRequest {
                cmd: command.into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: if cfg!(windows) {
                    "http://tauri.localhost"
                } else {
                    "tauri://localhost"
                }
                .parse()
                .unwrap(),
                body: InvokeBody::Json(serde_json::json!({})),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
    };
    for command in [
        "create_backup",
        "select_restore",
        "cancel_restore",
        "restore_and_restart",
        "native_restore_status",
    ] {
        let error = invoke(&other, command).expect_err("other window was allowed");
        assert!(
            error.to_string().contains("main window"),
            "{command}: {error}"
        );
    }
    let result: serde_json::Value = invoke(&main, "native_restore_status")
        .unwrap()
        .deserialize()
        .unwrap();
    assert_eq!(result["state"], "none");
    tauri::async_runtime::block_on(runtime.stop()).unwrap();
}
