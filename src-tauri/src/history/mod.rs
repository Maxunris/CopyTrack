use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
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
pub const EXPORT_FORMAT_VERSION: u32 = 1;

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
    pub language: String,
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
            language: "system".to_string(),
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
    pub language: Option<String>,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagsPatch {
    pub id: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub entries: Vec<HistoryItem>,
    pub settings: AppSettings,
    pub supported_history_limits: Vec<u32>,
    pub default_shortcut: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSummary {
    pub path: String,
    pub entry_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    pub path: String,
    pub imported_count: usize,
    pub skipped_count: usize,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HistoryArchive {
    version: u32,
    exported_at: String,
    settings: AppSettings,
    entries: Vec<PortableHistoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortableHistoryItem {
    id: String,
    content_type: String,
    preview_text: String,
    full_text: Option<String>,
    image_data_base64: Option<String>,
    file_paths: Vec<String>,
    source_app: Option<String>,
    created_at: String,
    favorite: bool,
    pinned: bool,
    tags: Vec<String>,
    size_bytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImportMode {
    Merge,
    Replace,
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
        Self::new_in_dir(base_dir)
    }

    fn new_in_dir(base_dir: PathBuf) -> Result<Self, StoreError> {
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
              language TEXT NOT NULL DEFAULT 'system',
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
            CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
              entry_id UNINDEXED,
              preview_text,
              full_text,
              source_app,
              file_paths,
              tags
            );
            ",
        )?;

        let _ = connection.execute(
            "ALTER TABLE settings ADD COLUMN language TEXT NOT NULL DEFAULT 'system'",
            [],
        );

        let default_settings = AppSettings::default();
        connection.execute(
            "INSERT OR IGNORE INTO settings (id, capture_enabled, history_limit, shortcut, theme, language, excluded_apps_json, launch_at_login)
             VALUES (1, ?, ?, ?, ?, ?, ?, ?)",
            params![
                bool_to_int(default_settings.capture_enabled),
                default_settings.history_limit,
                default_settings.shortcut,
                default_settings.theme,
                default_settings.language,
                serde_json::to_string(&default_settings.excluded_apps)?,
                bool_to_int(default_settings.launch_at_login),
            ],
        )?;

        rebuild_search_index(&connection)?;

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
            "SELECT capture_enabled, history_limit, shortcut, theme, language, excluded_apps_json, launch_at_login FROM settings WHERE id = 1",
            [],
            |row| {
                let excluded_apps_json: String = row.get(5)?;
                let excluded_apps = serde_json::from_str(&excluded_apps_json).unwrap_or_default();
                Ok(AppSettings {
                    capture_enabled: row.get::<_, i64>(0)? != 0,
                    history_limit: row.get::<_, u32>(1)?,
                    shortcut: row.get(2)?,
                    theme: row.get(3)?,
                    language: row.get(4)?,
                    excluded_apps,
                    launch_at_login: row.get::<_, i64>(6)? != 0,
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
            language: patch.language.unwrap_or(current.language),
            excluded_apps: patch.excluded_apps.unwrap_or(current.excluded_apps),
            launch_at_login: patch.launch_at_login.unwrap_or(current.launch_at_login),
        };

        let connection = self.connection()?;
        connection.execute(
            "UPDATE settings
             SET capture_enabled = ?, history_limit = ?, shortcut = ?, theme = ?, language = ?, excluded_apps_json = ?, launch_at_login = ?
             WHERE id = 1",
            params![
                bool_to_int(updated.capture_enabled),
                updated.history_limit,
                updated.shortcut,
                updated.theme,
                updated.language,
                serde_json::to_string(&updated.excluded_apps)?,
                bool_to_int(updated.launch_at_login),
            ],
        )?;

        self.cleanup_unpinned(updated.history_limit)?;
        Ok(updated)
    }

    pub fn export_to_path(&self, path: &Path) -> Result<ExportSummary, StoreError> {
        let path = normalize_export_path(path);
        let archive = HistoryArchive {
            version: EXPORT_FORMAT_VERSION,
            exported_at: Utc::now().to_rfc3339(),
            settings: self.load_settings()?,
            entries: self
                .list_entries(&HistoryQuery {
                    search: None,
                    content_type: None,
                    only_favorites: None,
                    only_pinned: None,
                })?
                .into_iter()
                .map(|entry| self.export_item(&entry))
                .collect::<Result<Vec<_>, _>>()?,
        };

        fs::write(&path, serde_json::to_vec_pretty(&archive)?)?;
        Ok(ExportSummary {
            path: path.to_string_lossy().to_string(),
            entry_count: archive.entries.len(),
        })
    }

    pub fn import_from_path(&self, path: &Path, mode: ImportMode) -> Result<ImportSummary, StoreError> {
        let archive = serde_json::from_slice::<HistoryArchive>(&fs::read(path)?)?;
        let connection = self.connection()?;
        let mut known_fingerprints = self.known_fingerprints()?;
        let mut imported_count = 0usize;
        let mut skipped_count = 0usize;

        if matches!(mode, ImportMode::Replace) {
            self.clear_all_entries()?;
            known_fingerprints.clear();
            self.save_settings(SettingsPatch {
                capture_enabled: Some(archive.settings.capture_enabled),
                history_limit: Some(archive.settings.history_limit),
                shortcut: Some(archive.settings.shortcut.clone()),
                theme: Some(archive.settings.theme.clone()),
                language: Some(archive.settings.language.clone()),
                excluded_apps: Some(archive.settings.excluded_apps.clone()),
                launch_at_login: Some(archive.settings.launch_at_login),
            })?;
        }

        for portable in archive.entries {
            let Some((entry, fingerprint)) = self.import_item(portable)? else {
                skipped_count += 1;
                continue;
            };

            if known_fingerprints.contains(&fingerprint) {
                skipped_count += 1;
                continue;
            }

            persist_history_item(&connection, &entry, &fingerprint)?;
            upsert_search_index(&connection, &entry)?;
            known_fingerprints.insert(fingerprint);
            imported_count += 1;
        }

        let history_limit = self.load_settings()?.history_limit;
        self.cleanup_unpinned(history_limit)?;

        Ok(ImportSummary {
            path: path.to_string_lossy().to_string(),
            imported_count,
            skipped_count,
            mode: match mode {
                ImportMode::Merge => "merge".to_string(),
                ImportMode::Replace => "replace".to_string(),
            },
        })
    }

    fn search_matching_ids(&self, search: Option<&str>) -> Result<Option<HashSet<String>>, StoreError> {
        let Some(search) = search else {
            return Ok(None);
        };

        let Some(query) = build_fts_query(search.trim()) else {
            return Ok(None);
        };

        let connection = self.connection()?;
        let mut statement = connection.prepare("SELECT entry_id FROM entries_fts WHERE entries_fts MATCH ?")?;
        let rows = statement.query_map(params![query], |row| row.get::<_, String>(0))?;

        let mut matches = HashSet::new();
        for row in rows {
            matches.insert(row?);
        }

        Ok(Some(matches))
    }

    fn known_fingerprints(&self) -> Result<HashSet<String>, StoreError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("SELECT fingerprint FROM entries")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut fingerprints = HashSet::new();
        for row in rows {
            fingerprints.insert(row?);
        }
        Ok(fingerprints)
    }

    fn clear_all_entries(&self) -> Result<(), StoreError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("SELECT image_path FROM entries WHERE image_path IS NOT NULL")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;

        for row in rows {
            let target = PathBuf::from(row?);
            if target.exists() {
                let _ = fs::remove_file(target);
            }
        }

        connection.execute("DELETE FROM entries", [])?;
        connection.execute("DELETE FROM entries_fts", [])?;
        Ok(())
    }

    pub fn list_entries(&self, query: &HistoryQuery) -> Result<Vec<HistoryItem>, StoreError> {
        let matching_ids = self.search_matching_ids(query.search.as_deref())?;
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
            .filter(|entry| matches_query(entry, query, matching_ids.as_ref()))
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

    pub fn insert_capture(
        &self,
        capture: &ClipboardCapture,
        source_app: Option<String>,
    ) -> Result<Option<HistoryItem>, StoreError> {
        if self.latest_fingerprint()?.as_deref() == Some(capture.fingerprint().as_str()) {
            return Ok(None);
        }

        let item = self.capture_to_item(capture, source_app)?;
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

        upsert_search_index(&connection, &item)?;
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

    pub fn set_tags(&self, id: &str, tags: Vec<String>) -> Result<(), StoreError> {
        let normalized = tags
            .into_iter()
            .map(|tag| tag.trim().to_lowercase())
            .filter(|tag| !tag.is_empty())
            .collect::<Vec<_>>();

        let connection = self.connection()?;
        connection.execute(
            "UPDATE entries SET tags_json = ? WHERE id = ?",
            params![serde_json::to_string(&normalized)?, id],
        )?;
        if let Some(entry) = self.get_entry(id)? {
            upsert_search_index(&connection, &entry)?;
        }
        Ok(())
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
            remove_search_index(&connection, id)?;
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
        for id in ids {
            remove_search_index(&connection, &id)?;
        }
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
            remove_search_index(&connection, &id)?;
        }

        Ok(())
    }

    fn capture_to_item(
        &self,
        capture: &ClipboardCapture,
        source_app: Option<String>,
    ) -> Result<HistoryItem, StoreError> {
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
                    source_app,
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
                    source_app,
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
                    source_app,
                    created_at,
                    favorite: false,
                    pinned: false,
                    tags: Vec::new(),
                    size_bytes: paths.join("\n").len() as u64,
                })
            }
        }
    }

    fn export_item(&self, entry: &HistoryItem) -> Result<PortableHistoryItem, StoreError> {
        let image_data_base64 = if let Some(image_path) = &entry.image_path {
            let bytes = fs::read(image_path)?;
            Some(BASE64.encode(bytes))
        } else {
            None
        };

        Ok(PortableHistoryItem {
            id: entry.id.clone(),
            content_type: entry.content_type.clone(),
            preview_text: entry.preview_text.clone(),
            full_text: entry.full_text.clone(),
            image_data_base64,
            file_paths: entry.file_paths.clone(),
            source_app: entry.source_app.clone(),
            created_at: entry.created_at.clone(),
            favorite: entry.favorite,
            pinned: entry.pinned,
            tags: entry.tags.clone(),
            size_bytes: entry.size_bytes,
        })
    }

    fn import_item(&self, portable: PortableHistoryItem) -> Result<Option<(HistoryItem, String)>, StoreError> {
        let id = Uuid::new_v4().to_string();
        let normalized_tags = portable
            .tags
            .into_iter()
            .map(|tag| tag.trim().to_lowercase())
            .filter(|tag| !tag.is_empty())
            .collect::<Vec<_>>();

        match portable.content_type.as_str() {
            "image" => {
                let Some(image_data_base64) = portable.image_data_base64 else {
                    return Ok(None);
                };

                let image_bytes = BASE64
                    .decode(image_data_base64)
                    .map_err(|error| std::io::Error::other(error.to_string()))?;
                let image = image::load_from_memory(&image_bytes)?;
                let rgba = image.to_rgba8();
                let width = rgba.width() as usize;
                let height = rgba.height() as usize;
                let raw = rgba.into_raw();
                let image_path = self.asset_dir.join(format!("{id}.png"));
                persist_png(&image_path, &raw, width, height)?;

                Ok(Some((
                    HistoryItem {
                        id,
                        content_type: "image".to_string(),
                        preview_text: portable.preview_text,
                        full_text: None,
                        image_path: Some(image_path.to_string_lossy().to_string()),
                        file_paths: portable.file_paths,
                        source_app: portable.source_app,
                        created_at: portable.created_at,
                        favorite: portable.favorite,
                        pinned: portable.pinned,
                        tags: normalized_tags,
                        size_bytes: portable.size_bytes.max(image_bytes.len() as u64),
                    },
                    fingerprint_for_image(&raw, width, height),
                )))
            }
            "file" => Ok(Some((
                HistoryItem {
                    id,
                    content_type: "file".to_string(),
                    preview_text: portable.preview_text,
                    full_text: portable.full_text.clone(),
                    image_path: None,
                    file_paths: portable.file_paths.clone(),
                    source_app: portable.source_app,
                    created_at: portable.created_at,
                    favorite: portable.favorite,
                    pinned: portable.pinned,
                    tags: normalized_tags,
                    size_bytes: portable.size_bytes,
                },
                fingerprint_for_files(&portable.file_paths),
            ))),
            _ => {
                let text = portable
                    .full_text
                    .clone()
                    .unwrap_or_else(|| portable.preview_text.clone());
                Ok(Some((
                    HistoryItem {
                        id,
                        content_type: portable.content_type,
                        preview_text: portable.preview_text,
                        full_text: Some(text.clone()),
                        image_path: None,
                        file_paths: portable.file_paths,
                        source_app: portable.source_app,
                        created_at: portable.created_at,
                        favorite: portable.favorite,
                        pinned: portable.pinned,
                        tags: normalized_tags,
                        size_bytes: portable.size_bytes.max(text.len() as u64),
                    },
                    fingerprint_for_text(&text),
                )))
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

fn persist_history_item(connection: &Connection, item: &HistoryItem, fingerprint: &str) -> Result<(), StoreError> {
    connection.execute(
        "INSERT INTO entries (
            id, content_type, preview_text, full_text, image_path, file_paths_json, source_app, created_at, favorite, pinned, tags_json, size_bytes, fingerprint
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            item.id,
            item.content_type,
            item.preview_text,
            item.full_text,
            item.image_path,
            serde_json::to_string(&item.file_paths)?,
            item.source_app,
            item.created_at,
            bool_to_int(item.favorite),
            bool_to_int(item.pinned),
            serde_json::to_string(&item.tags)?,
            item.size_bytes,
            fingerprint,
        ],
    )?;
    Ok(())
}

fn normalize_export_path(path: &Path) -> PathBuf {
    if path.extension().is_some() {
        path.to_path_buf()
    } else {
        path.with_extension("json")
    }
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

fn rebuild_search_index(connection: &Connection) -> Result<(), StoreError> {
    connection.execute("DELETE FROM entries_fts", [])?;

    let mut statement = connection.prepare(
        "SELECT id, preview_text, full_text, source_app, file_paths_json, tags_json FROM entries",
    )?;

    let rows = statement.query_map([], |row| {
        let file_paths_json: String = row.get(4)?;
        let tags_json: String = row.get(5)?;

        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            serde_json::from_str::<Vec<String>>(&file_paths_json).unwrap_or_default(),
            serde_json::from_str::<Vec<String>>(&tags_json).unwrap_or_default(),
        ))
    })?;

    for row in rows {
        let (entry_id, preview_text, full_text, source_app, file_paths, tags) = row?;
        connection.execute(
            "INSERT INTO entries_fts (entry_id, preview_text, full_text, source_app, file_paths, tags)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                entry_id,
                preview_text,
                full_text.unwrap_or_default(),
                source_app.unwrap_or_default(),
                file_paths.join(" "),
                tags.join(" "),
            ],
        )?;
    }

    Ok(())
}

fn upsert_search_index(connection: &Connection, entry: &HistoryItem) -> Result<(), StoreError> {
    remove_search_index(connection, &entry.id)?;
    connection.execute(
        "INSERT INTO entries_fts (entry_id, preview_text, full_text, source_app, file_paths, tags)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![
            entry.id,
            entry.preview_text,
            entry.full_text.clone().unwrap_or_default(),
            entry.source_app.clone().unwrap_or_default(),
            entry.file_paths.join(" "),
            entry.tags.join(" "),
        ],
    )?;
    Ok(())
}

fn remove_search_index(connection: &Connection, id: &str) -> Result<(), StoreError> {
    connection.execute("DELETE FROM entries_fts WHERE entry_id = ?", params![id])?;
    Ok(())
}

fn build_fts_query(search: &str) -> Option<String> {
    let tokens = search
        .split_whitespace()
        .map(|token| token.trim_matches('"'))
        .filter(|token| !token.is_empty())
        .map(|token| format!("\"{}\"", token.replace('"', "\"\"")))
        .collect::<Vec<_>>();

    if tokens.is_empty() {
        None
    } else {
        Some(tokens.join(" AND "))
    }
}

fn matches_query(
    entry: &HistoryItem,
    query: &HistoryQuery,
    matching_ids: Option<&HashSet<String>>,
) -> bool {
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

    if let Some(matching_ids) = matching_ids {
        if !matching_ids.contains(&entry.id) {
            return false;
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
    use std::fs;

    #[test]
    fn text_summary_keeps_preview_short() {
        let preview = summarize_text(
            "This is a very long line that should be shortened into a preview that does not exceed the UI limit for a single list row and keeps going well beyond the visible content size that the history list should render in the first release",
        );

        assert!(preview.chars().count() <= 121);
        assert!(preview.ends_with('…'));
    }

    #[test]
    fn query_filters_by_flags_and_matching_ids() {
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
            ,
            Some(&HashSet::from(["1".to_string()])),
        ));
        assert!(!matches_query(
            &entry,
            &HistoryQuery {
                search: Some("image".to_string()),
                content_type: Some("text".to_string()),
                only_favorites: Some(true),
                only_pinned: Some(false),
            },
            Some(&HashSet::from(["2".to_string()])),
        ));
    }

    #[test]
    fn build_fts_query_quotes_tokens() {
        assert_eq!(
            build_fts_query("release notes"),
            Some("\"release\" AND \"notes\"".to_string())
        );
    }

    #[test]
    fn store_deduplicates_and_searches_indexed_content() {
        let base_dir = std::env::temp_dir().join(format!("copytrack-test-{}", Uuid::new_v4()));
        let store = HistoryStore::new_in_dir(base_dir.clone()).expect("store should initialize");

        let first = store
            .insert_capture(
                &ClipboardCapture::Text {
                    value: "Release checklist for CopyTrack".to_string(),
                },
                Some("Notes".to_string()),
            )
            .expect("first insert should succeed");
        assert!(first.is_some());

        let duplicate = store
            .insert_capture(
                &ClipboardCapture::Text {
                    value: "Release checklist for CopyTrack".to_string(),
                },
                Some("Notes".to_string()),
            )
            .expect("duplicate insert should not fail");
        assert!(duplicate.is_none());

        let entry = first.expect("entry should exist");
        store
            .set_tags(&entry.id, vec!["release".to_string(), "docs".to_string()])
            .expect("tags should save");

        let search_results = store
            .list_entries(&HistoryQuery {
                search: Some("release docs".to_string()),
                content_type: Some("text".to_string()),
                only_favorites: Some(false),
                only_pinned: Some(false),
            })
            .expect("search should succeed");

        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].source_app.as_deref(), Some("Notes"));

        fs::remove_dir_all(base_dir).expect("temp store should be removed");
    }

    #[test]
    fn store_exports_and_imports_round_trip() {
        let source_dir = std::env::temp_dir().join(format!("copytrack-export-source-{}", Uuid::new_v4()));
        let target_dir = std::env::temp_dir().join(format!("copytrack-export-target-{}", Uuid::new_v4()));

        let source_store = HistoryStore::new_in_dir(source_dir.clone()).expect("source store should initialize");
        source_store
            .save_settings(SettingsPatch {
                capture_enabled: Some(true),
                history_limit: Some(500),
                shortcut: Some("CommandOrControl+Shift+V".to_string()),
                theme: Some("dark".to_string()),
                language: Some("ru".to_string()),
                excluded_apps: Some(vec!["com.1password.1password".to_string()]),
                launch_at_login: Some(true),
            })
            .expect("settings should save");
        let inserted = source_store
            .insert_capture(
                &ClipboardCapture::Text {
                    value: "Ship the public beta".to_string(),
                },
                Some("Notes".to_string()),
            )
            .expect("insert should succeed")
            .expect("entry should be created");
        source_store
            .set_tags(&inserted.id, vec!["release".to_string(), "beta".to_string()])
            .expect("tags should save");

        let export_path = source_dir.join("copytrack-history.json");
        let export_summary = source_store
            .export_to_path(&export_path)
            .expect("export should succeed");
        assert_eq!(export_summary.entry_count, 1);
        assert!(export_path.exists());

        let target_store = HistoryStore::new_in_dir(target_dir.clone()).expect("target store should initialize");
        let import_summary = target_store
            .import_from_path(&export_path, ImportMode::Replace)
            .expect("import should succeed");

        assert_eq!(import_summary.imported_count, 1);
        assert_eq!(import_summary.skipped_count, 0);
        assert_eq!(target_store.load_settings().expect("settings should load").theme, "dark");
        assert_eq!(
            target_store.load_settings().expect("settings should load").language,
            "ru"
        );

        let imported_entries = target_store
            .list_entries(&HistoryQuery {
                search: Some("release beta".to_string()),
                content_type: Some("text".to_string()),
                only_favorites: Some(false),
                only_pinned: Some(false),
            })
            .expect("search should succeed");

        assert_eq!(imported_entries.len(), 1);
        assert_eq!(imported_entries[0].source_app.as_deref(), Some("Notes"));
        assert_eq!(imported_entries[0].tags, vec!["release".to_string(), "beta".to_string()]);

        fs::remove_dir_all(source_dir).expect("source temp store should be removed");
        fs::remove_dir_all(target_dir).expect("target temp store should be removed");
    }
}
