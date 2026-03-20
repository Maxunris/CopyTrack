use std::borrow::Cow;
use std::thread;
use std::time::Duration;

use arboard::{Clipboard, ImageData};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::history::{
    fingerprint_for_files, fingerprint_for_image, fingerprint_for_text, HistoryItem, HistoryStore,
};
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
            let mut last_seen = state.last_seen_fingerprint.lock().expect("clipboard lock poisoned");
            if last_seen.as_deref() != Some(fingerprint.as_str()) {
                match state.store.insert_capture(&capture) {
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
