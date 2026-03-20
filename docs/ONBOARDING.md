# CopyTrack Onboarding

## First Launch On macOS

1. Open CopyTrack.
2. Keep the app running so the menu bar icon stays available.
3. Copy a few test items from Notes, Safari, or Finder.
4. Press `Cmd+Shift+V` to open Quick Access.
5. Click any history item to copy it back into the clipboard.

## Permissions And System Behavior

### Clipboard Access

macOS may prompt for pasteboard access depending on the context. If it does, allow CopyTrack. Without that approval, clipboard history can appear incomplete or stop updating.

### Launch At Login

CopyTrack uses the system login-item flow. You can enable or disable it from Settings at any time.

### Menu Bar

Closing the window hides it instead of quitting the app. This keeps the menu bar icon available as the main recovery point.

## Recommended Setup

- Keep history limit at `100` or `500` for a balanced default
- Add sensitive apps to the exclusion list
- Use tags for snippets you reuse often
- Pin long-lived entries you never want cleaned up

## Troubleshooting

### Items stop appearing

- Check that capture is still enabled in Settings
- Confirm CopyTrack is still running from the menu bar
- Re-open the app after any clipboard permission prompt

### Shortcut does not open the popup

- Check the shortcut value in Settings
- Try the default `Cmd+Shift+V`
- Make sure another app is not intercepting the same global shortcut

### Sensitive data should not be stored

Add the app's bundle identifier or visible app name to the excluded-apps list in Settings.
