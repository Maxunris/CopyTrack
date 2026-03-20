#[cfg(target_os = "macos")]
use objc2::rc::autoreleasepool;
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeFileURL, NSWorkspace};

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

#[derive(Debug, Clone)]
pub struct FrontmostApp {
    pub name: Option<String>,
    pub bundle_id: Option<String>,
}

#[cfg(target_os = "macos")]
pub fn frontmost_app() -> Option<FrontmostApp> {
    autoreleasepool(|pool| {
        let workspace = NSWorkspace::sharedWorkspace();
        let app = workspace.frontmostApplication()?;

        Some(FrontmostApp {
            name: app.localizedName().map(|value| unsafe { value.to_str(pool) }.to_string()),
            bundle_id: app.bundleIdentifier().map(|value| unsafe { value.to_str(pool) }.to_string()),
        })
    })
}

#[cfg(not(target_os = "macos"))]
pub fn read_file_paths() -> Vec<String> {
    Vec::new()
}

#[cfg(not(target_os = "macos"))]
pub fn frontmost_app() -> Option<FrontmostApp> {
    None
}
