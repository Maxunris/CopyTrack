mod clipboard;
mod history;
mod tray;

pub mod platform {
    pub mod macos;
}

use std::sync::{Arc, Mutex};

use tauri::{Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::clipboard::copy_history_item;
use crate::history::{
    BootstrapPayload, ExportSummary, HistoryItem, HistoryQuery, HistoryStore, ImportMode,
    ImportSummary, SettingsPatch, TagsPatch,
};

#[derive(Clone)]
pub struct SharedState {
    pub store: Arc<HistoryStore>,
    pub last_seen_fingerprint: Arc<Mutex<Option<String>>>,
    pub registered_shortcut: Arc<Mutex<Option<String>>>,
}

impl SharedState {
    fn new() -> Result<Self, String> {
        Ok(Self {
            store: Arc::new(HistoryStore::new("CopyTrack").map_err(|error| error.to_string())?),
            last_seen_fingerprint: Arc::new(Mutex::new(None)),
            registered_shortcut: Arc::new(Mutex::new(None)),
        })
    }
}

#[tauri::command]
fn bootstrap_app(state: State<SharedState>) -> Result<BootstrapPayload, String> {
    state.store.bootstrap().map_err(|error| error.to_string())
}

#[tauri::command]
fn list_history(query: HistoryQuery, state: State<SharedState>) -> Result<Vec<HistoryItem>, String> {
    state.store.list_entries(&query).map_err(|error| error.to_string())
}

#[tauri::command]
fn save_settings(
    patch: SettingsPatch,
    app: tauri::AppHandle,
    state: State<SharedState>,
) -> Result<crate::history::AppSettings, String> {
    let updated = state.store.save_settings(patch).map_err(|error| error.to_string())?;
    if !updated.shortcut.trim().is_empty() {
        apply_global_shortcut(&app, &state, &updated.shortcut)?;
    }
    tray::refresh_tray_menu(&app, &state).map_err(|error| error.to_string())?;
    Ok(updated)
}

#[tauri::command]
fn toggle_pin(id: String, pinned: bool, state: State<SharedState>) -> Result<(), String> {
    state.store.set_pinned(&id, pinned).map_err(|error| error.to_string())
}

#[tauri::command]
fn toggle_favorite(id: String, favorite: bool, state: State<SharedState>) -> Result<(), String> {
    state
        .store
        .set_favorite(&id, favorite)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_history_items(
    ids: Vec<String>,
    app: tauri::AppHandle,
    state: State<SharedState>,
) -> Result<(), String> {
    state.store.delete_entries(&ids).map_err(|error| error.to_string())
        .and_then(|_| tray::refresh_tray_menu(&app, &state).map_err(|error| error.to_string()))
}

#[tauri::command]
fn clear_unpinned_history(app: tauri::AppHandle, state: State<SharedState>) -> Result<(), String> {
    state.store.clear_unpinned().map_err(|error| error.to_string())
        .and_then(|_| tray::refresh_tray_menu(&app, &state).map_err(|error| error.to_string()))
}

#[tauri::command]
fn save_tags(patch: TagsPatch, state: State<SharedState>) -> Result<(), String> {
    state
        .store
        .set_tags(&patch.id, patch.tags)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn copy_entry(id: String, state: State<SharedState>) -> Result<(), String> {
    let item = state
        .store
        .get_entry(&id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "History item not found".to_string())?;

    copy_history_item(&item, &state.store)?;
    Ok(())
}

#[tauri::command]
fn open_quick_access(app: tauri::AppHandle) -> Result<(), String> {
    show_quick_access_window(&app).map_err(|error| error.to_string())
}

#[tauri::command]
fn hide_quick_access(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("quick-access") {
        window.hide().map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn export_history(path: String, state: State<SharedState>) -> Result<ExportSummary, String> {
    state
        .store
        .export_to_path(std::path::Path::new(&path))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn import_history(
    path: String,
    mode: ImportMode,
    app: tauri::AppHandle,
    state: State<SharedState>,
) -> Result<ImportSummary, String> {
    state
        .store
        .import_from_path(std::path::Path::new(&path), mode)
        .map_err(|error| error.to_string())
        .and_then(|summary| {
            tray::refresh_tray_menu(&app, &state).map_err(|error| error.to_string())?;
            Ok(summary)
        })
}

fn ensure_quick_access_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("quick-access").is_some() {
        return Ok(());
    }

    #[allow(unused_mut)]
    let mut builder = WebviewWindowBuilder::new(
        app,
        "quick-access",
        WebviewUrl::App("index.html#quick-access".into()),
    )
    .title("CopyTrack Quick Access")
    .inner_size(760.0, 560.0)
    .resizable(false)
    .center()
    .visible(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .focused(false)
    .decorations(false);

    #[cfg(target_os = "macos")]
    {
        builder = builder.hidden_title(true);
    }

    builder.build()?;
    Ok(())
}

pub(crate) fn show_quick_access_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    ensure_quick_access_window(app)?;

    if let Some(window) = app.get_webview_window("quick-access") {
        window.show()?;
        window.unminimize()?;
        window.center()?;
        window.set_focus()?;
    }

    Ok(())
}

fn apply_global_shortcut(app: &tauri::AppHandle, state: &SharedState, shortcut: &str) -> Result<(), String> {
    let manager = app.global_shortcut();
    let mut registered = state
        .registered_shortcut
        .lock()
        .map_err(|_| "shortcut lock poisoned".to_string())?;

    if let Some(existing) = registered.clone() {
        if existing == shortcut {
            return Ok(());
        }
        manager
            .unregister(existing.as_str())
            .map_err(|error| error.to_string())?;
    }

    let shortcut_value = shortcut.trim().to_string();
    manager
        .on_shortcut(shortcut_value.as_str(), move |app_handle, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                let _ = show_quick_access_window(app_handle);
            }
        })
        .map_err(|error| error.to_string())?;

    *registered = Some(shortcut_value);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shared_state = SharedState::new().expect("failed to initialize CopyTrack store");

    tauri::Builder::default()
        .setup({
            let shared_state = shared_state.clone();
            move |app| {
                #[cfg(desktop)]
                {
                    use tauri_plugin_autostart::MacosLauncher;

                    app.handle()
                        .plugin(tauri_plugin_global_shortcut::Builder::new().build())?;
                    app.handle().plugin(tauri_plugin_dialog::init())?;
                    app.handle()
                        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))?;
                }

                ensure_quick_access_window(app.handle())?;
                let shortcut = shared_state
                    .store
                    .load_settings()
                    .map_err(|error| tauri::Error::Anyhow(anyhow::anyhow!(error.to_string())))?
                    .shortcut;
                apply_global_shortcut(app.handle(), &shared_state, &shortcut)
                    .map_err(|error| tauri::Error::Anyhow(anyhow::anyhow!(error)))?;
                tray::setup_tray(app.handle(), shared_state.clone())?;
                clipboard::start_monitor(app.handle().clone(), shared_state.clone());
                Ok(())
            }
        })
        .manage(shared_state)
        .invoke_handler(tauri::generate_handler![
            bootstrap_app,
            list_history,
            save_settings,
            toggle_pin,
            toggle_favorite,
            delete_history_items,
            clear_unpinned_history,
            save_tags,
            copy_entry,
            open_quick_access,
            hide_quick_access,
            export_history,
            import_history
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
