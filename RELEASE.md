# Release Notes

## Current Release Outputs

For macOS, the current debug bundle is generated at:

- `src-tauri/target/debug/bundle/macos/CopyTrack.app`
- `src-tauri/target/debug/bundle/dmg/CopyTrack_1.0.0_aarch64.dmg`

For production releases, use the release build instead of debug:

```bash
npm run tauri build
```

## Version 1.0 Release Checklist

1. Run verification locally:
   - `npm test`
   - `cargo test --manifest-path src-tauri/Cargo.toml`
   - `npm run build`
   - `npm run tauri build`
2. Confirm the README is up to date with screenshots and install instructions.
3. Confirm no secrets, local caches, logs, `dist/`, or `node_modules/` are staged.
4. Build macOS release artifacts.
5. Upload the generated `.dmg` and, if useful, the `.app` archive to GitHub Releases.
6. Publish release notes describing features, fixes, known limitations, and minimum supported macOS version.

## GitHub Release Assets

For `v1.0.0`, upload at least:

- `CopyTrack_<version>_aarch64.dmg`

Optional extras:

- zipped `CopyTrack.app`
- checksum file
- changelog text

## Signing and Notarization

Not configured yet. Before the first public macOS release, add:

- Apple Developer signing identity
- notarization credentials
- stapling step for the final DMG

These steps should be added before tagging a public `1.0.0` release.
