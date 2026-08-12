# v0.2.0-alpha.6 — Alpha Release

## 🔄 What's Changed (v0.2.0-alpha.5 ➔ v0.2.0-alpha.6)

- **Channel**: Alpha Release (Preview Channel)
- **Platform**: macOS 11+ · Apple Silicon and Intel (universal binary)
- **Install**: Already on `v0.2.0-alpha.2` or later? Do nothing — this installs itself.

### 🐛 Fixed Bugs & Issues

- **Scans no longer interrupt you with permission prompts.** The Full Disk Access gate
  added in `alpha.4` covered app containers but missed the personal folders, so a scan
  still walked `~/Desktop`, `~/Documents`, `~/Downloads`, `~/Movies` and `~/Pictures`.
  macOS guards each of those with its own consent dialog, so you could be interrupted
  up to five times per scan while the app claimed protected locations were being
  skipped. All five are now skipped unless Full Disk Access is held.

  Verified against a real scan without Full Disk Access: **zero** guarded paths read,
  64 items still found.

- **Old downloads are reported as skipped**, rather than being scanned and prompting.

### ✨ New Features & Enhancements

- **Clearer Full Disk Access notice.** The Scan & clean dialog now names what is being
  left out — app locations plus Downloads, Desktop, Documents, Movies and Pictures — and
  frames Full Disk Access as the single approval that replaces every prompt, which is
  what it actually is.
- **Partial scans still run.** The project and large-file walks drop only the guarded
  roots and keep scanning `~/Developer`, `~/Projects`, `~/Sites` and similar. A partial
  scan is worth more than one that interrogates you before it will start.

### 🧪 Quality

- **First test in the project**, guarding exactly this. It asserts on the paths a scan
  actually reads rather than on the permission gate's own bookkeeping, so it fails when
  a new scan location is added without being filtered — which is how this regressed
  twice. CI runs it on a machine with no Full Disk Access, which is the case under test.

### 💡 About permissions, in short

macOS provides **no way for an app to request Full Disk Access, and no way to batch
consent** — Apple made it a manual toggle on purpose. So there are only two possible
designs, and Xclense now does the second:

1. Prompt for each folder, mid-scan — what was happening
2. Skip everything guarded, and offer one manual grant — what it does now

**Full Disk Access is the single permission.** One toggle covers every app container and
all five personal folders, and nothing is asked again. The button in the Scan & clean
dialog opens the exact settings pane.

### ⚠️ Alpha Channel Notes

- Without Full Disk Access a scan finds ~64 items instead of ~77. The difference is app
  containers, app support data, and your personal folders — often the largest wins.
- **Full Disk Access may need re-granting after an update.** An unsigned app's code
  identity changes with every build, so macOS has nothing stable to bind the grant to.
  If your grant does not survive this update, please report it — that is useful signal.
- Cleanup always routes through Finder's Trash — Xclense never deletes permanently.

### 🔓 First launch on macOS 15 Sequoia and macOS 26 Tahoe

This build is **unsigned and un-notarized**, so macOS blocks the first launch with:

> **"Xclense" Not Opened** — Apple could not verify "Xclense" is free of malware…
> **[ Done ] [ Move to Bin ]**

The app is fine; it just has no Apple Developer ID. **Right-click ➔ Open no longer
works** — Apple removed that bypass in Sequoia. Do one of these instead:

1. Click **Done**, then open **System Settings ➔ Privacy & Security**, scroll to
   Security, and click **Open Anyway** next to the Xclense message. Authenticate, launch
   again, confirm. (The message expires about an hour after the blocked launch.)
2. Or, in Terminal: `xattr -dr com.apple.quarantine /Applications/Xclense.app`

This only affects DMG installs. **In-app OTA updates are never blocked**, because the
updater downloads them directly rather than through a browser.

Report issues at <https://github.com/sudsarkar13/xclense/issues> with the version
shown in the sidebar.
