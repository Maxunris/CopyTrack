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
4. Confirm signing and notarization secrets are configured in GitHub.
5. Create or update a `v*` tag for the release version.
6. Let the GitHub Actions release workflow build the signed and notarized macOS bundle.
7. Verify the generated `.dmg`, zipped `.app`, and checksum file in the GitHub Release.
8. Publish release notes describing features, fixes, known limitations, and minimum supported macOS version.

## GitHub Release Assets

For public macOS releases, publish at least:

- `CopyTrack_<version>_aarch64.dmg`
- `CopyTrack_<version>_aarch64.app.zip`
- `CopyTrack_<version>_SHA256.txt`

Release notes should live in `docs/releases/<version>.md` when available.

## Signing and Notarization

The repository now includes a macOS release workflow at `.github/workflows/macos-release.yml`.

It expects Apple credentials through GitHub Secrets and performs:

- code signing with `Developer ID Application`
- notarization through App Store Connect API credentials
- stapling and validation for the built `.app` and `.dmg`
- upload to GitHub Releases on `v*` tags

Setup details, required secrets, and local verification commands are documented in [docs/SIGNING.md](./docs/SIGNING.md).
