use tauri::menu::{MenuBuilder, MenuEvent, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::history::SettingsPatch;
use crate::SharedState;

pub fn setup_tray(app: &AppHandle, state: SharedState) -> tauri::Result<()> {
    let show = MenuItemBuilder::with_id("show", "Open CopyTrack").build(app)?;
    let toggle_capture = MenuItemBuilder::with_id("toggle_capture", "Pause or Resume Capture").build(app)?;
    let clear_history = MenuItemBuilder::with_id("clear_history", "Clear Unpinned History").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[&show, &toggle_capture, &clear_history, &quit])
        .build()?;

    TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event: MenuEvent| match event.id().as_ref() {
            "show" => {
                let _ = show_main_window(app);
            }
            "toggle_capture" => {
                if let Ok(settings) = state.store.load_settings() {
                    let _ = state.store.save_settings(SettingsPatch {
                        capture_enabled: Some(!settings.capture_enabled),
                        history_limit: None,
                        shortcut: None,
                        theme: None,
                        excluded_apps: None,
                        launch_at_login: None,
                    });
                    let _ = app.emit("history-changed", serde_json::json!({ "reason": "capture-toggle" }));
                }
            }
            "clear_history" => {
                let _ = state.store.clear_unpinned();
                let _ = app.emit("history-changed", serde_json::json!({ "reason": "clear-history" }));
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event: TrayIconEvent| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

pub fn show_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
    }

    Ok(())
}
