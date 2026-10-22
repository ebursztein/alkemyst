use alkemyst_desktop::{NativeRuntime, RuntimeOptions, Tracking, configure_commands};
use std::sync::Arc;
use tauri::{WebviewWindowBuilder, ipc::InvokeBody, test::*};

#[test]
fn only_the_main_window_can_bootstrap_and_no_data_or_sql_commands_are_exposed() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = Arc::new(
        tauri::async_runtime::block_on(NativeRuntime::start(RuntimeOptions {
            database: dir.path().join("commands.sqlite3"),
            tracking: Tracking::Disabled,
            port: 0,
            development: false,
        }))
        .unwrap(),
    );
    let expected = runtime.bootstrap().unwrap();
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
    let invoke = |window: &tauri::WebviewWindow<tauri::test::MockRuntime>,
                  command: &str,
                  body: serde_json::Value| {
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
                body: InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
    };
    let connection: serde_json::Value =
        invoke(&main, "connection_bootstrap", serde_json::json!({}))
            .unwrap()
            .deserialize()
            .unwrap();
    assert_eq!(connection["base_url"], expected.base_url);
    assert_eq!(connection["token"], expected.token);
    assert!(invoke(&other, "connection_bootstrap", serde_json::json!({})).is_err());
    assert!(invoke(&main, "execute_sql", serde_json::json!({})).is_err());
    assert!(invoke(&main, "deck_list", serde_json::json!({})).is_err());
    for command in [
        "native_settings",
        "set_launch_at_login",
        "set_tracking_paused",
    ] {
        assert!(
            invoke(
                &other,
                command,
                serde_json::json!({"paused":true,"enabled":true})
            )
            .is_err()
        );
    }
    assert!(
        invoke(
            &main,
            "set_tracking_paused",
            serde_json::json!({"paused":true})
        )
        .is_ok()
    );
    assert!(tauri::async_runtime::block_on(runtime.tracking_paused()));
    assert!(
        invoke(
            &main,
            "set_tracking_paused",
            serde_json::json!({"paused":false})
        )
        .is_ok()
    );
    assert!(!tauri::async_runtime::block_on(runtime.tracking_paused()));
    tauri::async_runtime::block_on(runtime.stop()).unwrap();
    assert!(invoke(&main, "connection_bootstrap", serde_json::json!({})).is_err());
}
