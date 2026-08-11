# v0.2.0-alpha.4 — Alpha Release

## 🔄 What's Changed (v0.2.0-alpha.3 ➔ v0.2.0-alpha.4)

- **Channel**: Alpha Release (Preview Channel)
- **Platform**: macOS 11+ · Apple Silicon and Intel (universal binary)
- **Install**: Already on `v0.2.0-alpha.2` or later? Do nothing — this installs itself.

### 🐛 Fixed Bugs & Issues

- **No more permission prompt storm during scans.** macOS raises its "would like to
  access data from other apps" consent dialog once **per app container**, and a scan
  walks every one of them — roughly 1,000 on a typical machine, each blocking the scan
  until answered. Xclense now detects whether it holds Full Disk Access and, when it
  does not, skips those protected locations entirely instead of triggering prompts.
  A scan now raises **zero** dialogs.

### ✨ New Features & Enhancements

- **Full Disk Access status and one-click grant**: The Scan & clean dialog shows how
  many protected locations are being skipped and opens System Settings at the correct
  pane. Granting it once unlocks app container caches and app support data with no
  prompts at all — macOS Full Disk Access supersedes the per-app consent requirement.
- **Access re-checked automatically**: Permission state is re-read at the start of
  every scan, so granting access mid-session takes effect on the next scan without
  restarting Xclense.

### 📄 Documentation

- Added `docs/macos-permissions.md` explaining the TCC behaviour, what is skipped
  without Full Disk Access, and how detection works.
- Added a portable `tauri-ota-updates` skill capturing the full OTA setup procedure so
  it can be applied to other Tauri projects.

### ⚠️ Alpha Channel Notes

- Without Full Disk Access a scan finds ~72 items instead of ~77; the difference is app
  container caches and app support data, which are the largest single wins on most
  machines. Granting access is worth it.
- Cleanup always routes through Finder's Trash — Xclense never deletes permanently.

### 🔓 First launch on macOS 15 Sequoia and macOS 26 Tahoe

This build is **unsigned and un-notarized**, so macOS blocks the first launch with:

> **"Xclense" Not Opened** — Apple could not verify "Xclense" is free of malware…
> **[ Done ] [ Move to Bin ]**

The app is fine; it just has no Apple Developer ID yet. **Right-click ➔ Open no longer
works** — Apple removed that bypass in Sequoia. Do one of these instead:

1. Click **Done**, then open **System Settings ➔ Privacy & Security**, scroll to
   Security, and click **Open Anyway** next to the Xclense message. Authenticate, launch
   again, confirm. (The message expires about an hour after the blocked launch.)
2. Or, in Terminal: `xattr -dr com.apple.quarantine /Applications/Xclense.app`

This only affects DMG installs. **In-app OTA updates are never blocked**, because the
updater downloads them directly rather than through a browser.

Signing and notarization are already wired into the release pipeline and switch on as
soon as an Apple Developer ID certificate is available. See
[docs/macos-code-signing.md](docs/macos-code-signing.md).

Report issues at <https://github.com/sudsarkar13/xclense/issues> with the version
shown in the sidebar.
