# v0.2.0-alpha.7 — Alpha Release

## 🔄 What's Changed (v0.2.0-alpha.6 ➔ v0.2.0-alpha.7)

- **Channel**: Alpha Release (Preview Channel)
- **Platform**: macOS 11+ · Apple Silicon and Intel (universal binary)
- **Install**: Already on `v0.2.0-alpha.2` or later? Do nothing — this installs itself.

> Fixes the two problems reported against `alpha.6`: scans still raised permission
> prompts, and the "I've granted it" button did nothing.

### 🐛 Fixed Bugs & Issues

- **Scans no longer raise permission prompts — for real this time.** The `alpha.6` gate
  checked *categories*, but guarded paths are declared **inside** category definitions.
  Browser caches read `Library/Containers/com.apple.Safari/…` and
  `Library/Application Support/Google/Chrome/…` on every scan, and the category check
  waved them straight through. The guard is now applied to **every path** before it is
  read, so no category can reintroduce a prompt and a newly added scan location cannot
  bypass it.

  Browser caches were leaking on every scan and nothing reported it. They are now
  correctly listed among the skipped categories.

- **"Granted it" now actually applies the permission.** The old *"I've granted it —
  re-check"* button could never have worked: macOS decides a process's Full Disk Access
  when it launches and never revisits it, so a grant made while Xclense is open has no
  effect until it restarts. Re-checking in place was guaranteed to report the permission
  as still missing. The button now **restarts Xclense**, which is the only action that
  applies the grant. It is disabled while a scan or cleanup is running so nothing is cut
  off midway.

### 🧪 Quality

- The permission test now asserts against the **full** set of guarded locations rather
  than only the personal folders. The narrower version is exactly why it passed while
  Safari's container was being read on every scan.

### 💡 Granting Full Disk Access — the correct sequence

1. Open the Scan & clean dialog and click **Open Privacy Settings**
2. Enable **Xclense** in Privacy & Security ➔ Full Disk Access
3. Click **Granted it — restart Xclense**

Step 3 is not optional. macOS will keep reporting the permission as missing until the
app is relaunched, no matter how many times you re-check.

### ⚠️ Alpha Channel Notes

- Without Full Disk Access a scan finds ~65 items instead of ~77. The difference is app
  containers, app support data, browser caches, and your personal folders.
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
