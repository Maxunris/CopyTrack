use tauri::menu::{Menu, MenuEvent, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager};

use crate::clipboard::copy_history_item;
use crate::history::{HistoryQuery, SettingsPatch};
use crate::SharedState;

const TRAY_ID: &str = "menu-bar";
const RECENT_PREFIX: &str = "recent::";

struct TrayCopy<'a> {
    open_app: &'a str,
    recent_empty: &'a str,
    pause_resume: &'a str,
    clear_unpinned: &'a str,
    quit: &'a str,
}

pub fn setup_tray(app: &AppHandle, state: SharedState) -> tauri::Result<()> {
    let menu = build_tray_menu(app, &state)?;

    TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event: MenuEvent| {
            let _ = handle_menu_event(app, event, &state);
        })
        .build(app)?;

    Ok(())
}

pub fn refresh_tray_menu(app: &AppHandle, state: &SharedState) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(build_tray_menu(app, state)?))?;
    }
    Ok(())
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent, state: &SharedState) -> tauri::Result<()> {
    let event_id = event.id().as_ref().to_string();

    match event_id.as_str() {
        "show" => {
            show_main_window(app)?;
        }
        "toggle_capture" => {
            if let Ok(settings) = state.store.load_settings() {
                let _ = state.store.save_settings(SettingsPatch {
                    capture_enabled: Some(!settings.capture_enabled),
                    history_limit: None,
                    shortcut: None,
                    theme: None,
                    language: None,
                    excluded_apps: None,
                    launch_at_login: None,
                });
                let _ = app.emit("history-changed", serde_json::json!({ "reason": "capture-toggle" }));
                refresh_tray_menu(app, state)?;
            }
        }
        "clear_history" => {
            let _ = state.store.clear_unpinned();
            let _ = app.emit("history-changed", serde_json::json!({ "reason": "clear-history" }));
            refresh_tray_menu(app, state)?;
        }
        "quit" => app.exit(0),
        _ if event_id.starts_with(RECENT_PREFIX) => {
            let id = event_id.trim_start_matches(RECENT_PREFIX);
            if let Some(item) = state.store.get_entry(id).ok().flatten() {
                let _ = copy_history_item(&item, &state.store);
            }
        }
        _ => {}
    }

    Ok(())
}

fn build_tray_menu(app: &AppHandle, state: &SharedState) -> tauri::Result<Menu<tauri::Wry>> {
    let settings = state.store.load_settings().unwrap_or_default();
    let copy = tray_copy(&settings.language);
    let recent_entries = state
        .store
        .list_entries(&HistoryQuery {
            search: None,
            content_type: None,
            only_favorites: None,
            only_pinned: None,
        })
        .unwrap_or_default()
        .into_iter()
        .take(4)
        .collect::<Vec<_>>();

    let menu = Menu::new(app)?;
    menu.append(&MenuItemBuilder::with_id("show", copy.open_app).build(app)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    if recent_entries.is_empty() {
        menu.append(
            &MenuItemBuilder::with_id("recent_empty", copy.recent_empty)
                .enabled(false)
                .build(app)?,
        )?;
    } else {
        for entry in recent_entries {
            menu.append(
                &MenuItemBuilder::with_id(
                    format!("{RECENT_PREFIX}{}", entry.id),
                    recent_label(&entry.preview_text),
                )
                .build(app)?,
            )?;
        }
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItemBuilder::with_id("toggle_capture", copy.pause_resume).build(app)?)?;
    menu.append(&MenuItemBuilder::with_id("clear_history", copy.clear_unpinned).build(app)?)?;
    menu.append(&MenuItemBuilder::with_id("quit", copy.quit).build(app)?)?;
    Ok(menu)
}

fn tray_copy(language: &str) -> TrayCopy<'static> {
    let is_ru = language == "ru"
        || (language == "system"
            && std::env::var("LANG")
                .unwrap_or_default()
                .to_lowercase()
                .starts_with("ru"));
    TrayCopy {
        open_app: if is_ru {
            "Открыть CopyTrack"
        } else {
            "Open CopyTrack"
        },
        recent_empty: if is_ru {
            "История пока пуста"
        } else {
            "No history yet"
        },
        pause_resume: if is_ru {
            "Пауза или возобновление"
        } else {
            "Pause or Resume Capture"
        },
        clear_unpinned: if is_ru {
            "Очистить незакрепленное"
        } else {
            "Clear Unpinned History"
        },
        quit: if is_ru { "Выйти" } else { "Quit" },
    }
}

fn recent_label(value: &str) -> String {
    let trimmed = value.trim().replace('\n', " ");
    let mut preview = trimmed.chars().take(42).collect::<String>();
    if trimmed.chars().count() > 42 {
        preview.push('…');
    }
    preview
}

pub fn show_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
    }

    Ok(())
}
