# CopyTrack QA Checklist

## Scope

This checklist covers the current macOS-first desktop release.

## Functional Checks

- Copy plain text and verify it appears in history
- Copy a link and verify it is detected as `Link`
- Copy an image and verify it appears with an image preview
- Copy a file in Finder and verify it appears as a file entry
- Click an old history item and verify it is copied back into the clipboard
- Favorite, pin, tag, and delete entries
- Clear unpinned history and verify pinned entries remain
- Change history limit and verify cleanup respects pinned items

## Search And Organization

- Search by snippet text
- Search by source app name
- Search by tags
- Filter by content type
- Toggle `Favorites` and `Pinned`
- Sort by recent, oldest, favorites, and type

## Quick Access

- Open Quick Access with the default shortcut
- Navigate results with `ArrowUp` and `ArrowDown`
- Press `Enter` to copy the selected entry
- Press `Escape` to close the popup
- Change the shortcut in Settings and verify the new shortcut works

## Portability

- Export history to a JSON snapshot from Settings
- Import a JSON snapshot in `merge` mode and verify existing history remains
- Import a JSON snapshot in `replace` mode and verify current history is replaced
- Verify imported tags, favorites, pins, and settings behave as expected

## macOS Integration

- Verify the first-run onboarding appears on a fresh install and can be completed
- Verify the menu bar icon opens a menu with the latest four copied items
- Verify tray menu actions open history, quick access, settings, pause or resume capture, clear unpinned history, and quit
- Verify clicking a recent tray item copies it back into the clipboard
- Toggle launch at login and confirm the setting persists
- Hide the main window and confirm the app remains reachable from the menu bar

## Permissions And Privacy

- Confirm clipboard capture continues after macOS permission prompts are accepted
- Add `1Password` or another excluded app and verify copied data from that app is not stored
- Confirm history remains local and no network account is required

## Release Hygiene

- `npm test`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `npm run tauri build`
- Confirm `.env`, local caches, IDE folders, logs, `dist/`, and `target/` are not staged
