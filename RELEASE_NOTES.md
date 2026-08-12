# v0.2.0-alpha.5 — Alpha Release

## 🔄 What's Changed (v0.2.0-alpha.4 ➔ v0.2.0-alpha.5)

- **Channel**: Alpha Release (Preview Channel)
- **Platform**: macOS 11+ · Apple Silicon and Intel (universal binary)
- **Install**: Already on `v0.2.0-alpha.2` or later? Do nothing — this installs itself.

> A small, mostly-infrastructure release. No scanning or cleanup behaviour changed; the
> user-visible differences are the macOS permission dialogs and the install experience.

### ✨ New Features & Enhancements

- **Permission dialogs now explain themselves.** macOS consent prompts previously
  appeared without a reason string. Xclense now declares why it needs each kind of
  access, so the Apple Events prompt reads *"Xclense asks Finder to move the files you
  select into the Trash, so nothing is ever deleted permanently and every cleanup can be
  undone"* instead of nothing at all. Desktop, Documents, and Downloads carry
  descriptions too.
- **Correct minimum system version.** The bundle now declares macOS 11.0, matching the
  Apple Silicon slice of the universal binary, rather than Tauri's 10.13 default. Older
  systems get a clear refusal instead of an app that installs and fails.
- **Documented install path.** The README and every release now carry first-launch
  instructions for macOS 15+, where Apple removed the right-click ➔ Open bypass.

### 🐛 Fixed Bugs & Issues

- **Releases can no longer get stuck as invisible drafts.** A GitHub 5xx during asset
  upload leaves the release as a *draft* with assets attached; retrying then fails with
  "already exists" while the draft stays invisible to the updater. This happened during
  `v0.2.0-alpha.4`. Publishing now retries with backoff, resumes onto an existing
  release in any state, and verifies the draft flag, pre-release flag, and asset count
  before reporting success.
- **Update manifests can no longer advertise an unreachable payload.** The release asset
  is now fetched anonymously — the way an installed client fetches it — before the
  manifest points at it. A manifest referencing a private or missing asset breaks every
  installed copy at once while the release page looks perfectly normal.
- **Releases no longer report success before clients can see them.** The update endpoint
  is CDN-cached for about five minutes, so a release could complete while every client
  still received the previous version. The pipeline now waits for the endpoint to serve
  the new version.

### 🔐 Signing

Xclense ships **unsigned by design** — it is free, open source, and distributed outside
the App Store, and a Developer ID certificate costs $99/year. The hardened runtime,
entitlements, and a full sign ➔ notarize ➔ staple ➔ verify pipeline are in place but
dormant; they would activate only if that decision ever changes. Reasoning and costs:
[docs/macos-code-signing.md](docs/macos-code-signing.md).

### 📄 Documentation

- `docs/macos-code-signing.md` — the Gatekeeper error explained, behaviour per macOS
  version, both override routes, and the cost/benefit of signing.
- README gained an Install section, including an explicit caution about what
  `xattr -dr com.apple.quarantine` actually does and how to build from source instead.

### ⚠️ Alpha Channel Notes

- Without Full Disk Access a scan finds ~72 items instead of ~77; the difference is app
  container caches and app support data, which are the largest single wins on most
  machines. Granting access is worth it.
- **Full Disk Access may need re-granting after an update.** An unsigned app's code
  identity changes with every build, so macOS has nothing stable to bind the grant to.
  If your granted access does not survive this update, that is why — please report it.
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
