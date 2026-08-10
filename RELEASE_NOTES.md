# v0.2.0-alpha.2 — Alpha Release

## 🔄 What's Changed (v0.2.0-alpha.1 ➔ v0.2.0-alpha.2)

- **Channel**: Alpha Release (Preview Channel)
- **Platform**: macOS 11+ · Apple Silicon (aarch64)
- **Install**: Download the `.dmg`, drag Xclense to Applications. The build is
  unsigned, so on first launch use **right-click ➔ Open** and confirm the prompt.

### ✨ New Features & Enhancements

- **Over-the-air updates**: Xclense now updates itself. It checks for a new build
  shortly after launch and every six hours, downloads it in the background, installs
  it, and relaunches — no manual download after this release.
- **Signed update payloads**: Every update is verified against an Ed25519 signature
  before it is installed. A tampered or unsigned payload is rejected outright.
- **Updates never interrupt work**: If a storage scan or cleanup is running when an
  update is ready, the restart waits until that work finishes. A cleanup moving files
  to Trash is never cut off part-way.
- **Live update progress**: A corner panel shows download progress with byte counts,
  then the install and restart. Failed checks stay silent — being offline is not an
  error worth interrupting anyone about.
- **Version visible in-app**: The sidebar shows the running version and its release
  channel, so a bug report can always name the exact build.

### 🔧 Build & Release Infrastructure

- Release builds now emit and sign the `.app.tar.gz` updater payload alongside the
  `.dmg`, and the pipeline fails if the signature is missing rather than publishing a
  release that silently cannot be installed by existing clients.
- The update manifest is published to `updates/latest.json` on `main`. GitHub's
  "latest release" URL excludes pre-releases, so an alpha build could never have been
  discovered through it.
- Actions moved off the deprecated Node 20 runtime; CI builds the frontend before the
  Rust checks, which previously failed on a clean checkout.

### ⚠️ Alpha Channel Notes

- **This is the first build that can update itself.** If you are on `v0.2.0-alpha.1`,
  install this one manually — from `v0.2.0-alpha.2` onward, updates arrive
  automatically.
- The build remains **unsigned and un-notarized**; Gatekeeper warns on first launch.
- Apple Silicon only. An Intel Mac will not find a matching update and will silently
  stay on its installed version.
- A full storage scan takes roughly 30-40 seconds.
- Cleanup always routes through Finder's Trash — Xclense never deletes permanently.

Report issues at <https://github.com/sudsarkar13/xclense/issues> with the version
shown in the sidebar.
