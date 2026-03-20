use std::borrow::Cow;
use std::thread;
use std::time::Duration;

use arboard::{Clipboard, ImageData};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::history::{
    fingerprint_for_files, fingerprint_for_image, fingerprint_for_text, AppSettings, HistoryItem,
    HistoryStore,
};
use crate::platform::macos::FrontmostApp;
use crate::SharedState;

#[derive(Debug, Clone)]
pub enum ClipboardCapture {
    Text { value: String },
    Image { bytes: Vec<u8>, width: usize, height: usize },
    Files { paths: Vec<String> },
}

impl ClipboardCapture {
    pub fn fingerprint(&self) -> String {
        match self {
            Self::Text { value } => fingerprint_for_text(value),
            Self::Image {
                bytes,
                width,
                height,
            } => fingerprint_for_image(bytes, *width, *height),
            Self::Files { paths } => fingerprint_for_files(paths),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryChangedEvent {
    pub reason: String,
}

pub fn start_monitor(app: AppHandle, state: SharedState) {
    thread::spawn(move || loop {
        let settings = match state.store.load_settings() {
            Ok(settings) => settings,
            Err(_) => {
                thread::sleep(Duration::from_millis(900));
                continue;
            }
        };

        if !settings.capture_enabled {
            thread::sleep(Duration::from_millis(900));
            continue;
        }

        if let Some(capture) = read_clipboard_capture() {
            let fingerprint = capture.fingerprint();
            let source_app = crate::platform::macos::frontmost_app();
            let mut last_seen = state.last_seen_fingerprint.lock().expect("clipboard lock poisoned");

            if is_excluded_app(&settings, source_app.as_ref()) {
                *last_seen = Some(fingerprint);
                thread::sleep(Duration::from_millis(700));
                continue;
            }

            if last_seen.as_deref() != Some(fingerprint.as_str()) {
                match state
                    .store
                    .insert_capture(&capture, source_app.and_then(|app| app.name))
                {
                    Ok(Some(_)) => {
                        *last_seen = Some(fingerprint);
                        let _ = app.emit(
                            "history-changed",
                            HistoryChangedEvent {
                                reason: "captured".to_string(),
                            },
                        );
                    }
                    Ok(None) => {
                        *last_seen = Some(fingerprint);
                    }
                    Err(_) => {}
                }
            }
        }

        thread::sleep(Duration::from_millis(700));
    });
}

fn is_excluded_app(settings: &AppSettings, source_app: Option<&FrontmostApp>) -> bool {
    let Some(source_app) = source_app else {
        return false;
    };

    let excluded = settings
        .excluded_apps
        .iter()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    if excluded.is_empty() {
        return false;
    }

    let app_keys = [source_app.bundle_id.as_deref(), source_app.name.as_deref()]
        .into_iter()
        .flatten()
        .map(|value| value.trim().to_lowercase())
        .collect::<Vec<_>>();

    app_keys.iter().any(|value| excluded.contains(value))
}

pub fn read_clipboard_capture() -> Option<ClipboardCapture> {
    let files = crate::platform::macos::read_file_paths();
    if !files.is_empty() {
        return Some(ClipboardCapture::Files { paths: files });
    }

    let mut clipboard = Clipboard::new().ok()?;
    if let Ok(text) = clipboard.get_text() {
        if !text.trim().is_empty() {
            return Some(ClipboardCapture::Text { value: text });
        }
    }

    if let Ok(image) = clipboard.get_image() {
        return Some(ClipboardCapture::Image {
            bytes: image.bytes.into_owned(),
            width: image.width,
            height: image.height,
        });
    }

    None
}

pub fn copy_history_item(item: &HistoryItem, _store: &HistoryStore) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|error| error.to_string())?;

    match item.content_type.as_str() {
        "image" => {
            let path = item
                .image_path
                .clone()
                .ok_or_else(|| "Image path not found".to_string())?;
            let image = image::open(path).map_err(|error| error.to_string())?;
            let rgba = image.to_rgba8();
            let width = rgba.width() as usize;
            let height = rgba.height() as usize;
            clipboard
                .set_image(ImageData {
                    width,
                    height,
                    bytes: Cow::Owned(rgba.into_raw()),
                })
                .map_err(|error| error.to_string())
        }
        "file" => clipboard
            .set_text(item.file_paths.join("\n"))
            .map_err(|error| error.to_string()),
        _ => clipboard
            .set_text(item.full_text.clone().unwrap_or_else(|| item.preview_text.clone()))
            .map_err(|error| error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excluded_apps_match_bundle_id_or_name() {
        let settings = AppSettings {
            capture_enabled: true,
            history_limit: 100,
            shortcut: "CommandOrControl+Shift+V".to_string(),
            theme: "system".to_string(),
            excluded_apps: vec!["com.apple.keychainaccess".to_string(), "1password".to_string()],
            launch_at_login: false,
        };

        assert!(is_excluded_app(
            &settings,
            Some(&FrontmostApp {
                name: Some("1Password".to_string()),
                bundle_id: Some("com.1password.1password".to_string()),
            }),
        ));

        assert!(is_excluded_app(
            &settings,
            Some(&FrontmostApp {
                name: Some("Keychain Access".to_string()),
                bundle_id: Some("com.apple.keychainaccess".to_string()),
            }),
        ));

        assert!(!is_excluded_app(
            &settings,
            Some(&FrontmostApp {
                name: Some("Notes".to_string()),
                bundle_id: Some("com.apple.Notes".to_string()),
            }),
        ));
    }
}
