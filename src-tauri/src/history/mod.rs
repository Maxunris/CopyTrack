use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::clipboard::ClipboardCapture;

pub const DEFAULT_SHORTCUT: &str = "CommandOrControl+Shift+V";
pub const DEFAULT_HISTORY_LIMIT: u32 = 100;
pub const SUPPORTED_HISTORY_LIMITS: [u32; 5] = [50, 100, 500, 1000, 10000];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub id: String,
    pub content_type: String,
    pub preview_text: String,
    pub full_text: Option<String>,
    pub image_path: Option<String>,
    pub file_paths: Vec<String>,
    pub source_app: Option<String>,
    pub created_at: String,
    pub favorite: bool,
    pub pinned: bool,
    pub tags: Vec<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub capture_enabled: bool,
    pub history_limit: u32,
    pub shortcut: String,
    pub theme: String,
    pub excluded_apps: Vec<String>,
    pub launch_at_login: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            capture_enabled: true,
            history_limit: DEFAULT_HISTORY_LIMIT,
            shortcut: DEFAULT_SHORTCUT.to_string(),
            theme: "system".to_string(),
            excluded_apps: Vec::new(),
            launch_at_login: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    pub capture_enabled: Option<bool>,
    pub history_limit: Option<u32>,
    pub shortcut: Option<String>,
    pub theme: Option<String>,
    pub excluded_apps: Option<Vec<String>>,
    pub launch_at_login: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQuery {
    pub search: Option<String>,
    pub content_type: Option<String>,
    pub only_favorites: Option<bool>,
    pub only_pinned: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub entries: Vec<HistoryItem>,
    pub settings: AppSettings,
    pub supported_history_limits: Vec<u32>,
    pub default_shortcut: String,
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct HistoryStore {
    db_path: PathBuf,
    asset_dir: PathBuf,
}

impl HistoryStore {
    pub fn new(app_name: &str) -> Result<Self, StoreError> {
        let base_dir = dirs::data_local_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join(app_name);
        let asset_dir = base_dir.join("assets");
        fs::create_dir_all(&asset_dir)?;

        let db_path = base_dir.join("copytrack.sqlite3");
        let store = Self { db_path, asset_dir };
        store.initialize()?;
        Ok(store)
    }

    fn connection(&self) -> Result<Connection, rusqlite::Error> {
        Connection::open(&self.db_path)
    }

    fn initialize(&self) -> Result<(), StoreError> {
        let connection = self.connection()?;
        connection.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS settings (
              id INTEGER PRIMARY KEY CHECK (id = 1),
              capture_enabled INTEGER NOT NULL,
              history_limit INTEGER NOT NULL,
              shortcut TEXT NOT NULL,
              theme TEXT NOT NULL,
              excluded_apps_json TEXT NOT NULL,
              launch_at_login INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS entries (
              id TEXT PRIMARY KEY,
              content_type TEXT NOT NULL,
              preview_text TEXT NOT NULL,
              full_text TEXT,
              image_path TEXT,
              file_paths_json TEXT NOT NULL,
              source_app TEXT,
              created_at TEXT NOT NULL,
              favorite INTEGER NOT NULL DEFAULT 0,
              pinned INTEGER NOT NULL DEFAULT 0,
              tags_json TEXT NOT NULL,
              size_bytes INTEGER NOT NULL,
              fingerprint TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_entries_created_at ON entries(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_entries_content_type ON entries(content_type);
            CREATE INDEX IF NOT EXISTS idx_entries_pinned ON entries(pinned);
            CREATE INDEX IF NOT EXISTS idx_entries_favorite ON entries(favorite);
            ",
        )?;

        let default_settings = AppSettings::default();
        connection.execute(
            "INSERT OR IGNORE INTO settings (id, capture_enabled, history_limit, shortcut, theme, excluded_apps_json, launch_at_login)
             VALUES (1, ?, ?, ?, ?, ?, ?)",
            params![
                bool_to_int(default_settings.capture_enabled),
                default_settings.history_limit,
                default_settings.shortcut,
                default_settings.theme,
                serde_json::to_string(&default_settings.excluded_apps)?,
                bool_to_int(default_settings.launch_at_login),
            ],
        )?;

        Ok(())
    }

    pub fn bootstrap(&self) -> Result<BootstrapPayload, StoreError> {
        let settings = self.load_settings()?;
        let entries = self.list_entries(&HistoryQuery {
            search: None,
            content_type: None,
            only_favorites: None,
            only_pinned: None,
        })?;

        Ok(BootstrapPayload {
            entries,
            settings,
            supported_history_limits: SUPPORTED_HISTORY_LIMITS.to_vec(),
            default_shortcut: DEFAULT_SHORTCUT.to_string(),
        })
    }

    pub fn load_settings(&self) -> Result<AppSettings, StoreError> {
        let connection = self.connection()?;
        let settings = connection.query_row(
            "SELECT capture_enabled, history_limit, shortcut, theme, excluded_apps_json, launch_at_login FROM settings WHERE id = 1",
            [],
            |row| {
                let excluded_apps_json: String = row.get(4)?;
                let excluded_apps = serde_json::from_str(&excluded_apps_json).unwrap_or_default();
                Ok(AppSettings {
                    capture_enabled: row.get::<_, i64>(0)? != 0,
                    history_limit: row.get::<_, u32>(1)?,
                    shortcut: row.get(2)?,
                    theme: row.get(3)?,
                    excluded_apps,
                    launch_at_login: row.get::<_, i64>(5)? != 0,
                })
            },
        )?;

        Ok(settings)
    }

    pub fn save_settings(&self, patch: SettingsPatch) -> Result<AppSettings, StoreError> {
        let current = self.load_settings()?;
        let updated = AppSettings {
            capture_enabled: patch.capture_enabled.unwrap_or(current.capture_enabled),
            history_limit: patch
                .history_limit
                .filter(|value| SUPPORTED_HISTORY_LIMITS.contains(value))
                .unwrap_or(current.history_limit),
            shortcut: patch.shortcut.unwrap_or(current.shortcut),
            theme: patch.theme.unwrap_or(current.theme),
            excluded_apps: patch.excluded_apps.unwrap_or(current.excluded_apps),
            launch_at_login: patch.launch_at_login.unwrap_or(current.launch_at_login),
        };

        let connection = self.connection()?;
        connection.execute(
            "UPDATE settings
             SET capture_enabled = ?, history_limit = ?, shortcut = ?, theme = ?, excluded_apps_json = ?, launch_at_login = ?
             WHERE id = 1",
            params![
                bool_to_int(updated.capture_enabled),
                updated.history_limit,
                updated.shortcut,
                updated.theme,
                serde_json::to_string(&updated.excluded_apps)?,
                bool_to_int(updated.launch_at_login),
            ],
        )?;

        self.cleanup_unpinned(updated.history_limit)?;
        Ok(updated)
    }

    pub fn list_entries(&self, query: &HistoryQuery) -> Result<Vec<HistoryItem>, StoreError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, content_type, preview_text, full_text, image_path, file_paths_json, source_app, created_at, favorite, pinned, tags_json, size_bytes
             FROM entries
             ORDER BY pinned DESC, created_at DESC",
        )?;

        let rows = statement.query_map([], |row| {
            let file_paths_json: String = row.get(5)?;
            let tags_json: String = row.get(10)?;

            Ok(HistoryItem {
                id: row.get(0)?,
                content_type: row.get(1)?,
                preview_text: row.get(2)?,
                full_text: row.get(3)?,
                image_path: row.get(4)?,
                file_paths: serde_json::from_str(&file_paths_json).unwrap_or_default(),
                source_app: row.get(6)?,
                created_at: row.get(7)?,
                favorite: row.get::<_, i64>(8)? != 0,
                pinned: row.get::<_, i64>(9)? != 0,
                tags: serde_json::from_str(&tags_json).unwrap_or_default(),
                size_bytes: row.get::<_, u64>(11)?,
            })
        })?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }

        Ok(entries
            .into_iter()
            .filter(|entry| matches_query(entry, query))
            .collect())
    }

    pub fn get_entry(&self, id: &str) -> Result<Option<HistoryItem>, StoreError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, content_type, preview_text, full_text, image_path, file_paths_json, source_app, created_at, favorite, pinned, tags_json, size_bytes
             FROM entries
             WHERE id = ?",
        )?;

        let mut rows = statement.query(params![id])?;
        if let Some(row) = rows.next()? {
            let file_paths_json: String = row.get(5)?;
            let tags_json: String = row.get(10)?;
            return Ok(Some(HistoryItem {
                id: row.get(0)?,
                content_type: row.get(1)?,
                preview_text: row.get(2)?,
                full_text: row.get(3)?,
                image_path: row.get(4)?,
                file_paths: serde_json::from_str(&file_paths_json).unwrap_or_default(),
                source_app: row.get(6)?,
                created_at: row.get(7)?,
                favorite: row.get::<_, i64>(8)? != 0,
                pinned: row.get::<_, i64>(9)? != 0,
                tags: serde_json::from_str(&tags_json).unwrap_or_default(),
                size_bytes: row.get::<_, u64>(11)?,
            }));
        }

        Ok(None)
    }

    pub fn latest_fingerprint(&self) -> Result<Option<String>, StoreError> {
        let connection = self.connection()?;
        let fingerprint = connection.query_row(
            "SELECT fingerprint FROM entries ORDER BY created_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        );

        match fingerprint {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(StoreError::Database(error)),
        }
    }

    pub fn insert_capture(&self, capture: &ClipboardCapture) -> Result<Option<HistoryItem>, StoreError> {
        if self.latest_fingerprint()?.as_deref() == Some(capture.fingerprint().as_str()) {
            return Ok(None);
        }

        let item = self.capture_to_item(capture)?;
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO entries (
                id, content_type, preview_text, full_text, image_path, file_paths_json, source_app, created_at, favorite, pinned, tags_json, size_bytes, fingerprint
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, 0, ?, ?, ?)",
            params![
                item.id,
                item.content_type,
                item.preview_text,
                item.full_text,
                item.image_path,
                serde_json::to_string(&item.file_paths)?,
                item.source_app,
                item.created_at,
                serde_json::to_string(&item.tags)?,
                item.size_bytes,
                capture.fingerprint(),
            ],
        )?;

        let settings = self.load_settings()?;
        self.cleanup_unpinned(settings.history_limit)?;
        Ok(self.get_entry(&item.id)?)
    }

    pub fn set_pinned(&self, id: &str, pinned: bool) -> Result<(), StoreError> {
        self.update_boolean_field(id, "pinned", pinned)
    }

    pub fn set_favorite(&self, id: &str, favorite: bool) -> Result<(), StoreError> {
        self.update_boolean_field(id, "favorite", favorite)
    }

    fn update_boolean_field(&self, id: &str, field: &str, value: bool) -> Result<(), StoreError> {
        let connection = self.connection()?;
        let sql = format!("UPDATE entries SET {field} = ? WHERE id = ?");
        connection.execute(&sql, params![bool_to_int(value), id])?;
        Ok(())
    }

    pub fn delete_entries(&self, ids: &[String]) -> Result<(), StoreError> {
        let connection = self.connection()?;
        for id in ids {
            if let Some(entry) = self.get_entry(id)? {
                if let Some(image_path) = entry.image_path {
                    let path = PathBuf::from(image_path);
                    if path.exists() {
                        let _ = fs::remove_file(path);
                    }
                }
            }
            connection.execute("DELETE FROM entries WHERE id = ?", params![id])?;
        }
        Ok(())
    }

    pub fn clear_unpinned(&self) -> Result<(), StoreError> {
        let connection = self.connection()?;
        let mut statement =
            connection.prepare("SELECT id, image_path FROM entries WHERE pinned = 0 ORDER BY created_at DESC")?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })?;

        let mut ids = Vec::new();
        let mut image_paths = Vec::new();
        for row in rows {
            let (id, image_path) = row?;
            ids.push(id);
            if let Some(path) = image_path {
                image_paths.push(path);
            }
        }

        for path in image_paths {
            let target = PathBuf::from(path);
            if target.exists() {
                let _ = fs::remove_file(target);
            }
        }

        connection.execute("DELETE FROM entries WHERE pinned = 0", [])?;
        Ok(())
    }

    pub fn cleanup_unpinned(&self, history_limit: u32) -> Result<(), StoreError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, image_path FROM entries WHERE pinned = 0 ORDER BY created_at DESC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })?;

        let mut stale_ids = Vec::new();
        let mut stale_paths = Vec::new();
        for (index, row) in rows.enumerate() {
            let (id, image_path) = row?;
            if index >= history_limit as usize {
                stale_ids.push(id);
                if let Some(path) = image_path {
                    stale_paths.push(path);
                }
            }
        }

        for path in stale_paths {
            let target = PathBuf::from(path);
            if target.exists() {
                let _ = fs::remove_file(target);
            }
        }

        for id in stale_ids {
            connection.execute("DELETE FROM entries WHERE id = ?", params![id])?;
        }

        Ok(())
    }

    fn capture_to_item(&self, capture: &ClipboardCapture) -> Result<HistoryItem, StoreError> {
        let created_at = Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();

        match capture {
            ClipboardCapture::Text { value } => {
                let preview = summarize_text(value);
                Ok(HistoryItem {
                    id,
                    content_type: detect_text_type(value).to_string(),
                    preview_text: preview,
                    full_text: Some(value.clone()),
                    image_path: None,
                    file_paths: Vec::new(),
                    source_app: None,
                    created_at,
                    favorite: false,
                    pinned: false,
                    tags: Vec::new(),
                    size_bytes: value.len() as u64,
                })
            }
            ClipboardCapture::Image {
                bytes,
                width,
                height,
            } => {
                let image_path = self.asset_dir.join(format!("{id}.png"));
                persist_png(&image_path, bytes, *width, *height)?;
                Ok(HistoryItem {
                    id,
                    content_type: "image".to_string(),
                    preview_text: format!("Image {}x{}", width, height),
                    full_text: None,
                    image_path: Some(image_path.to_string_lossy().to_string()),
                    file_paths: Vec::new(),
                    source_app: None,
                    created_at,
                    favorite: false,
                    pinned: false,
                    tags: Vec::new(),
                    size_bytes: bytes.len() as u64,
                })
            }
            ClipboardCapture::Files { paths } => {
                let preview = if paths.len() == 1 {
                    format!("File: {}", basename(&paths[0]))
                } else {
                    format!("{} files copied", paths.len())
                };
                Ok(HistoryItem {
                    id,
                    content_type: "file".to_string(),
                    preview_text: preview,
                    full_text: Some(paths.join("\n")),
                    image_path: None,
                    file_paths: paths.clone(),
                    source_app: None,
                    created_at,
                    favorite: false,
                    pinned: false,
                    tags: Vec::new(),
                    size_bytes: paths.join("\n").len() as u64,
                })
            }
        }
    }
}

fn persist_png(path: &Path, bytes: &[u8], width: usize, height: usize) -> Result<(), StoreError> {
    let image = image::RgbaImage::from_raw(width as u32, height as u32, bytes.to_vec())
        .ok_or_else(|| std::io::Error::other("invalid image buffer"))?;
    image.save(path)?;
    Ok(())
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_string()
}

fn detect_text_type(value: &str) -> &'static str {
    if value.starts_with("http://") || value.starts_with("https://") {
        "link"
    } else {
        "text"
    }
}

fn summarize_text(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "Empty text entry".to_string();
    }

    let single_line = trimmed.replace('\n', " ");
    let mut preview = single_line.chars().take(120).collect::<String>();
    if single_line.chars().count() > 120 {
        preview.push('…');
    }
    preview
}

fn bool_to_int(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn matches_query(entry: &HistoryItem, query: &HistoryQuery) -> bool {
    if query.only_favorites.unwrap_or(false) && !entry.favorite {
        return false;
    }

    if query.only_pinned.unwrap_or(false) && !entry.pinned {
        return false;
    }

    if let Some(content_type) = &query.content_type {
        if !content_type.is_empty() && content_type != "all" && entry.content_type != *content_type {
            return false;
        }
    }

    if let Some(search) = &query.search {
        let search = search.trim().to_lowercase();
        if !search.is_empty() {
            let haystack = [
                entry.preview_text.to_lowercase(),
                entry.full_text.clone().unwrap_or_default().to_lowercase(),
                entry.source_app.clone().unwrap_or_default().to_lowercase(),
                entry.file_paths.join(" ").to_lowercase(),
                entry.tags.join(" ").to_lowercase(),
            ]
            .join(" ");

            if !haystack.contains(&search) {
                return false;
            }
        }
    }

    true
}

pub fn fingerprint_for_text(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update("text");
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn fingerprint_for_image(bytes: &[u8], width: usize, height: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update("image");
    hasher.update(width.to_le_bytes());
    hasher.update(height.to_le_bytes());
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn fingerprint_for_files(paths: &[String]) -> String {
    let mut hasher = Sha256::new();
    hasher.update("file");
    hasher.update(paths.join("\n").as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_summary_keeps_preview_short() {
        let preview = summarize_text(
            "This is a very long line that should be shortened into a preview that does not exceed the UI limit for a single list row and keeps going well beyond the visible content size that the history list should render in the first release",
        );

        assert!(preview.chars().count() <= 121);
        assert!(preview.ends_with('…'));
    }

    #[test]
    fn query_filters_by_flags_and_search() {
        let entry = HistoryItem {
            id: "1".to_string(),
            content_type: "text".to_string(),
            preview_text: "Deploy checklist".to_string(),
            full_text: Some("Release build and changelog".to_string()),
            image_path: None,
            file_paths: Vec::new(),
            source_app: Some("Notes".to_string()),
            created_at: Utc::now().to_rfc3339(),
            favorite: true,
            pinned: false,
            tags: vec!["release".to_string()],
            size_bytes: 24,
        };

        assert!(matches_query(
            &entry,
            &HistoryQuery {
                search: Some("release".to_string()),
                content_type: Some("text".to_string()),
                only_favorites: Some(true),
                only_pinned: Some(false),
            }
        ));
        assert!(!matches_query(
            &entry,
            &HistoryQuery {
                search: Some("image".to_string()),
                content_type: Some("text".to_string()),
                only_favorites: Some(true),
                only_pinned: Some(false),
            }
        ));
    }
}
