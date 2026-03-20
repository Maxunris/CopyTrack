# CopyTrack

<p align="center">
  <img src="./public/app-icon.svg" alt="CopyTrack icon" width="120" />
</p>

<p align="center">
  A local-first clipboard history app for macOS with quick recall, one-click re-copy, menu bar access, and a glass-inspired desktop UI.
</p>

![CopyTrack main window](./docs/screenshots/main-window.png)

## Why CopyTrack

CopyTrack keeps the things you copy close at hand. Instead of losing useful snippets, links, images, or file references after the next copy, you get a searchable local history with favorites, pins, tags, and a keyboard-first quick popup.

Version `0.1.0` is focused on a strong macOS-first foundation. The long-term plan is cross-platform, but the product starts where menu bar workflows, global shortcuts, and polished utility-app ergonomics matter most.

## Current Features

- Local-only clipboard history
- Text, links, images, and file references
- One-click re-copy from the main window and quick popup
- Global shortcut with editable keybind
- Menu bar presence and launch at login
- Favorites, pins, tags, filters, and sorting
- Import and export as JSON snapshots
- Retention presets: `50`, `100`, `500`, `1000`, `10000`
- Excluded apps for sensitive workflows
- Indexed local search backed by SQLite `FTS5`

## Screens

### Main Window

![CopyTrack main window](./docs/screenshots/main-window.png)

### Quick Access

![CopyTrack quick access popup](./docs/screenshots/quick-access.png)

### Settings

![CopyTrack settings sheet](./docs/screenshots/settings-sheet.png)

## How It Works

1. CopyTrack watches the macOS clipboard locally.
2. Each supported clipboard item is normalized into a history entry.
3. Entries are saved in SQLite, while larger image payloads live on disk inside the app data folder.
4. Search, filters, tags, and sorting help you find something fast.
5. Clicking an item copies it back into the clipboard immediately.

## Quick Start

### Run From Source

```bash
npm install
npm run tauri dev
```

### Verify The Project

```bash
npm test
cargo test --manifest-path src-tauri/Cargo.toml
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

### Build A Desktop Bundle

```bash
npm run tauri build
```

## First-Launch Tips

- Default shortcut: `Cmd+Shift+V`
- Open an item in history to copy it back instantly
- Use the menu bar icon if the main window is hidden
- Add password managers or other sensitive apps to the exclusion list
- If macOS asks about clipboard access, allow CopyTrack so background capture keeps working

More onboarding details live in [docs/ONBOARDING.md](./docs/ONBOARDING.md).

## Stack

- `Tauri 2`
- `React 19`
- `TypeScript`
- `Rust`
- `SQLite + FTS5`

Architecture notes are in [ARCHITECTURE.md](./ARCHITECTURE.md).

## Release Notes

- Current release process notes: [RELEASE.md](./RELEASE.md)
- Future ideas and post-`1.0` expansion list: [NEXT.md](./NEXT.md)
- Deferred sync architecture draft: [SYNC.md](./SYNC.md)

For the future public `v1.0` release, the plan is to upload macOS release artifacts like `.dmg` bundles to GitHub Releases.

## Quality

The project already includes frontend and Rust tests for history filtering, sorting, duplicate handling, indexed search behavior, and excluded-app rules. A macOS QA checklist is maintained in [docs/QA.md](./docs/QA.md).

## Roadmap

- Finish the dedicated production popup flow polish
- Improve file handling depth on macOS pasteboard
- Add encryption and cloud sync only after the local-first core is stable
- Expand to Windows and Linux later
