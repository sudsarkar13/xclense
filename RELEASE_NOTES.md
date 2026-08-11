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
- The build remains **unsigned and un-notarized**; Gatekeeper warns on first launch.
- Cleanup always routes through Finder's Trash — Xclense never deletes permanently.

Report issues at <https://github.com/sudsarkar13/xclense/issues> with the version
shown in the sidebar.
