# CopyTrack Architecture Draft

## Recommended Stack

- App shell: Tauri 2
- Frontend: React 19 + TypeScript + Vite
- Styling: Tailwind CSS 4 plus a small design-token layer for macOS-inspired translucent surfaces
- Desktop backend: Rust
- Local database: SQLite with FTS5 for search
- State and data fetching: TanStack Query plus lightweight local state where needed
- Validation: Zod
- Icons and asset pipeline: SVG source files with generated app icons for release builds

## Why This Stack

Tauri is the best balance for this project right now. It keeps the app lightweight like a desktop utility, gives us a strong path to Windows and Linux later, and still lets us build a polished macOS-first interface. Rust is a good fit for clipboard monitoring, tray integration, global shortcuts, storage, and background work. React plus TypeScript gives us fast UI iteration and a mature ecosystem without locking the product into an Electron-sized runtime.

If the only goal were macOS polish with no concern for future platforms, SwiftUI would be the strongest native alternative. For this project, the future cross-platform path matters enough that Tauri is the more practical long-term choice.

## Proposed Modules

### Frontend

- `src/app/`: app bootstrap, routing, providers
- `src/features/history/`: list, item cards, item preview, recopy actions
- `src/features/search/`: search input, filters, tags, sort controls
- `src/features/settings/`: retention presets, startup, theme, exclusions, shortcut editor
- `src/features/quick-access/`: popup UI and keyboard navigation
- `src/shared/ui/`: reusable macOS-style components
- `src/shared/lib/`: formatters, type guards, frontend helpers

### Tauri and Native Layer

- `src-tauri/src/clipboard/`: clipboard watcher and content normalization
- `src-tauri/src/history/`: database models and repository layer
- `src-tauri/src/search/`: FTS-backed queries and filter composition
- `src-tauri/src/settings/`: persisted preferences and startup settings
- `src-tauri/src/tray/`: menu bar icon and tray actions
- `src-tauri/src/shortcuts/`: global shortcut registration and updates
- `src-tauri/src/platform/macos/`: macOS-specific clipboard and window behavior

## Data Model

Each clipboard record should contain:

- `id`
- `content_type`
- `created_at`
- `source_app`
- `preview_text`
- `payload_path` for large binary data
- `text_content` for searchable text
- `favorite`
- `pinned`
- `tags`
- `is_sensitive`
- `size_bytes`

Images and large payloads should live on disk with metadata in SQLite. Text, links, and lightweight content can stay directly in the database.

## Version 1.0 Priorities

1. macOS-first desktop release
2. Menu bar presence and launch at login
3. Global shortcut with editable keybind
4. Local-only storage
5. Retention presets: 50, 100, 500, 1000, 10000
6. One-click re-copy from history
7. Search and fast recent access
8. Glass-like macOS utility styling

## Deferred for Later

- Cloud sync
- Encryption and vault mode
- Team features
- Windows and Linux platform packages
- Import and export tooling
