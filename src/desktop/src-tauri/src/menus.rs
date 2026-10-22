use crate::{NativeRuntime, shell};
use std::sync::Arc;
use tauri::{
    AppHandle, Manager,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
};

struct Controls<R: tauri::Runtime> {
    pause: tauri::menu::CheckMenuItem<R>,
}

pub fn sync_pause<R: tauri::Runtime>(app: &AppHandle<R>, paused: bool) {
    if let Some(controls) = app.try_state::<Controls<R>>() {
        let _ = controls.pause.set_checked(paused);
    }
}

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::default(app)?;
    let controls = Submenu::with_items(
        app,
        "Alkemyst controls",
        true,
        &[
            &MenuItem::with_id(
                app,
                "show",
                "Show Alkemyst",
                true,
                Some("CmdOrCtrl+Shift+A"),
            )?,
            &MenuItem::with_id(app, "settings", "Settings…", true, Some("CmdOrCtrl+,"))?,
        ],
    )?;
    menu.append(&controls)?;
    app.set_menu(menu)?;
    app.on_menu_event(|app, event| match event.id().as_ref() {
        "show" => shell::show(app),
        "settings" => shell::open_settings(app),
        "quit" => app.exit(0),
        _ => {}
    });
    let pause = tauri::menu::CheckMenuItem::with_id(
        app,
        "pause",
        "Pause Arena tracking",
        true,
        false,
        None::<&str>,
    )?;
    app.manage(Controls {
        pause: pause.clone(),
    });
    let tray_menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "show", "Show Alkemyst", true, None::<&str>)?,
            &MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?,
            &pause,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", "Quit Alkemyst", true, None::<&str>)?,
        ],
    )?;
    let icon = if cfg!(target_os = "macos") {
        tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?
    } else {
        tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))?
    };
    TrayIconBuilder::with_id("alkemyst")
        .icon(icon)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Alkemyst — local deck laboratory")
        .menu(&tray_menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| {
            if event.id().as_ref() == "pause" {
                let app = app.clone();
                let requested = pause.is_checked().unwrap_or(false);
                let runtime = app.state::<Arc<NativeRuntime>>().inner().clone();
                tauri::async_runtime::spawn(async move {
                    let _ = runtime.set_tracking_paused(requested).await;
                    sync_pause(&app, runtime.tracking_paused().await);
                });
            }
        })
        .build(app)?;
    Ok(())
}
