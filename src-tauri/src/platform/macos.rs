#[cfg(target_os = "macos")]
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeFileURL};

#[cfg(target_os = "macos")]
pub fn read_file_paths() -> Vec<String> {
    let pasteboard = NSPasteboard::generalPasteboard();
    let mut paths = Vec::new();

    if let Some(items) = pasteboard.pasteboardItems() {
        for item in items.iter() {
            if let Some(file_url) = item.stringForType(unsafe { &NSPasteboardTypeFileURL }) {
                paths.push(file_url.to_string());
            }
        }
    }

    paths
}

#[cfg(not(target_os = "macos"))]
pub fn read_file_paths() -> Vec<String> {
    Vec::new()
}
