# macOS Signing And Notarization

CopyTrack is set up to ship public macOS builds through GitHub Actions with Apple code signing and notarization.

## What The Workflow Does

The workflow at `.github/workflows/macos-release.yml` does the following on macOS:

1. installs Node and Rust dependencies;
2. imports your `Developer ID Application` certificate into a temporary keychain;
3. writes the App Store Connect API key to a temporary `.p8` file;
4. runs `npm run check`;
5. builds a signed and notarized Tauri bundle for `aarch64-apple-darwin`;
6. validates the `.app` and `.dmg`;
7. uploads the `.dmg`, zipped `.app`, and `SHA256` file to GitHub Releases for `v*` tags.

Manual `workflow_dispatch` runs are also supported for dry runs. They upload artifacts to the workflow run, but only tag builds publish a GitHub Release.

## Required Apple Side Setup

You need:

- an active Apple Developer membership;
- a `Developer ID Application` certificate exported as `.p12`;
- an App Store Connect API key with permission to submit for notarization.

## Required GitHub Secrets

Create these repository secrets before running the workflow:

- `APPLE_CERTIFICATE`: base64-encoded `.p12` certificate export
- `APPLE_CERTIFICATE_PASSWORD`: password used when exporting the `.p12`
- `KEYCHAIN_PASSWORD`: temporary keychain password for the CI runner
- `APPLE_API_ISSUER`: App Store Connect issuer ID
- `APPLE_API_KEY`: App Store Connect key ID
- `APPLE_API_KEY_CONTENT`: raw contents of the downloaded `AuthKey_<KEYID>.p8`

The workflow automatically detects the signing identity from the imported certificate, so you do not need to hardcode it in the repo.

## Preparing The Certificate

1. Open `Keychain Access` on your Mac.
2. Find the `Developer ID Application` certificate under `My Certificates`.
3. Export it as `.p12`.
4. Convert it to base64:

```bash
openssl base64 -A -in /path/to/copytrack-developer-id.p12 -out certificate-base64.txt
```

5. Put the contents of `certificate-base64.txt` into the `APPLE_CERTIFICATE` secret.

## Preparing The Notarization API Key

1. Open App Store Connect.
2. Go to `Users and Access` -> `Integrations`.
3. Create a key with access suitable for notarization.
4. Save:
   - the issuer ID into `APPLE_API_ISSUER`;
   - the key ID into `APPLE_API_KEY`;
   - the downloaded `.p8` file contents into `APPLE_API_KEY_CONTENT`.

The private key can only be downloaded once, so store it safely outside the repository.

## Release Flow

1. Make sure `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` all contain the same version.
2. Add release notes to `docs/releases/<version>.md`.
3. Commit and push the release changes.
4. Create and push a tag:

```bash
git tag v1.0.1
git push origin v1.0.1
```

5. Wait for the `macOS Release` workflow to finish.
6. Open the GitHub Release and verify the assets.

## Local Validation Commands

After downloading a release artifact, validate it with:

```bash
scripts/verify-macos-release.sh "/Applications/CopyTrack.app"
scripts/verify-macos-release.sh "/path/to/CopyTrack_1.0.0_aarch64.dmg"
```

The script checks code signing, Gatekeeper assessment, and stapled notarization tickets.

## Notes

- This setup signs and notarizes public builds without committing secrets to git.
- The current workflow ships Apple Silicon (`aarch64`) bundles. Intel or universal builds can be added later if needed.
