# CopyTrack Sync Draft

This document captures the current recommendation for future sync support after the local-first product is stable.

## Current Position

- Version `1.0` remains local-first
- No account system is required today
- Sync should stay optional and additive, not foundational

## Architecture Direction

### Local Store Stays Canonical

Each device should keep SQLite as the source of truth for live UI, search, tags, favorites, and retention behavior. Sync should replicate changes between local stores instead of replacing local persistence with a remote-first model.

### Add A Sync Boundary

When sync is introduced, it should sit behind a dedicated transport boundary:

- `history store`
- `change journal`
- `sync adapter`
- `conflict resolver`

That keeps the current clipboard capture and search layers mostly unchanged.

### Change Journal

Clipboard entries should gain a lightweight sync identity and change metadata:

- `sync_id`
- `updated_at`
- `deleted_at`
- `device_id`
- `content_hash`

This makes merge, deletion propagation, and conflict handling much simpler than diffing raw local tables later.

## Recommended Rollout

1. Add export/import first and observe real portability use
2. Add internal change journal without network sync enabled
3. Add a single-provider sync adapter
4. Add optional account sign-in
5. Add encrypted multi-device sync

## Conflict Strategy

- Clipboard items: dedupe by content hash where possible
- Tags/favorites/pins: last-write-wins is acceptable for the first sync release
- Deletes: tombstones should beat older updates
- Settings: sync only explicitly opted-in settings, not all local preferences

## Security Notes

- Sensitive apps should still respect local exclusions before anything is synced
- End-to-end encryption should be the target for any cloud-backed history
- A future vault mode should remain local even if regular history sync exists

## Good First Sync Candidates

- Pinned snippets
- Favorites
- Tags
- Text and link entries

## Later Sync Candidates

- Images
- Large binary payloads
- File references with OS-specific path translation

## Why This Matters

This path lets CopyTrack stay fast and trustworthy now, while still leaving room for cloud sync later without forcing a rewrite of the current local product.
