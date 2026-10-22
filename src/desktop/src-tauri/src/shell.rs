use crate::{CloseAction, ExitAction, Lifecycle, NativeRuntime, RuntimeOptions, Tracking};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

pub fn show(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn open_settings(app: &AppHandle) {
    show(app);
    let _ = app.emit_to("main", "navigate-settings", ());
}

fn build(options: Option<RuntimeOptions>, background: bool) -> tauri::Result<tauri::App> {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if !args.iter().any(|arg| arg == "--background") {
                show(app);
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .manage(Arc::new(Lifecycle::default()))
        .setup(move |app| {
            let options = options.unwrap_or(RuntimeOptions {
                database: app.path().app_data_dir()?.join("alkemyst.sqlite3"),
                tracking: Tracking::Discover,
                port: 47831,
                development: cfg!(debug_assertions),
            });
            let runtime = tauri::async_runtime::block_on(NativeRuntime::start(options))?;
            app.manage(Arc::new(runtime));
            crate::menus::install(app.handle())?;
            if !background {
                show(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main"
                && let WindowEvent::CloseRequested { api, .. } = event
                && window.state::<Arc<Lifecycle>>().close_action() == CloseAction::Hide
            {
                api.prevent_close();
                let _ = window.hide();
            }
        });
    crate::configure_commands(builder).build(tauri::generate_context!())
}

pub fn run() -> tauri::Result<()> {
    let background = std::env::args().any(|arg| arg == "--background");
    let app = build(None, background)?;
    app.run(|app, event| handle_event(app, &event));
    Ok(())
}

#[cfg(feature = "native-smoke")]
pub fn run_smoke(
    options: RuntimeOptions,
    mut hook: impl FnMut(&AppHandle, RunEvent) + 'static,
) -> tauri::Result<()> {
    let app = build(Some(options), false)?;
    app.run_return(move |app, event| {
        handle_event(app, &event);
        hook(app, event);
    });
    Ok(())
}

fn handle_event(app: &AppHandle, event: &RunEvent) {
    match event {
        RunEvent::ExitRequested { api, code, .. } => {
            let lifecycle = app.state::<Arc<Lifecycle>>().inner().clone();
            match lifecycle.exit_action() {
                ExitAction::BeginShutdown => {
                    api.prevent_exit();
                    let runtime = app.state::<Arc<NativeRuntime>>().inner().clone();
                    let app = app.clone();
                    let code = *code;
                    tauri::async_runtime::spawn(async move {
                        if runtime.stop().await.is_err() {
                            eprintln!("Alkemyst could not cleanly stop all local services.");
                        }
                        lifecycle.complete_shutdown();
                        app.exit(code.unwrap_or(0));
                    });
                }
                ExitAction::Wait => api.prevent_exit(),
                ExitAction::Exit => {}
            }
        }
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => show(app),
        _ => {}
    }
}
