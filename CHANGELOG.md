# Changelog

All notable changes to the **Xclense** project are documented in this file.

Versions follow [Semantic Versioning](https://semver.org/). Pre-release builds use
the `-alpha.N` and `-beta.N` suffixes described in
[docs/release-process.md](docs/release-process.md).

---

## [v0.2.0-alpha.5] - 2026-08-12

### 🚀 Highlights & Features

- **Consent Dialogs Carry Reasons**: `src-tauri/Info.plist` declares
  `NSAppleEventsUsageDescription` plus Desktop/Documents/Downloads usage strings, so
  macOS explains why each permission is being requested instead of prompting blankly.

### 🔐 Signing & Distribution

- **Gatekeeper Readiness**: macOS 15+ hard-blocks unsigned apps with a dead-end
  *Done / Move to Bin* dialog — the historic right-click ➔ Open bypass was removed in
  Sequoia. The bundle now enables the **hardened runtime**, ships
  `src-tauri/Entitlements.plist`, and declares usage descriptions via
  `src-tauri/Info.plist`, so it is notarization-ready the moment a Developer ID
  certificate exists.
- **Apple Events Entitlement**: `com.apple.security.automation.apple-events` is granted
  because cleanup trashes files through Finder. Under the hardened runtime a *correctly
  signed* build without it would install, launch, scan, and then silently fail to delete
  anything.
- **Conditional Signing Pipeline**: `release.yml` signs, notarizes, and staples when the
  `APPLE_*` secrets are present, and falls back to an ad-hoc build with a warning when
  they are not — so the alpha pipeline is unaffected until the membership is purchased.
  A post-build gate asserts the team identifier, hardened-runtime flag, signature
  validity, and Gatekeeper's own verdict, because a failed import otherwise produces a
  perfectly normal-looking unsigned release.
- **Minimum System Version**: pinned to macOS 11.0, matching the arm64 slice of the
  universal binary rather than Tauri's 10.13 default.

### 🔧 Build & Release Infrastructure

- **Resilient Release Publishing**: a `gh release create` interrupted by a GitHub 5xx
  leaves a *draft* with assets attached; retrying then fails with "already exists" while
  the draft stays invisible to the updater. Publishing now retries with backoff, resumes
  onto an existing release in any state, and converges on published — then asserts
  draft/pre-release flags and asset count.
- **Update Payload Reachability Gate**: the release asset is fetched anonymously before
  the manifest is allowed to advertise it. A manifest pointing at a private or missing
  asset breaks every installed client at once and looks entirely normal.
- **Endpoint Propagation Wait**: `raw.githubusercontent.com` caches for about five
  minutes, so a run could finish green while clients still received the previous
  manifest. The workflow now waits for the endpoint to serve the new version.

### 📄 Documentation

- **Unsigned Distribution Is the Stated Policy**: Xclense is free, open source, and
  distributed outside the App Store, so it ships unsigned rather than paying $99/year
  for a Developer ID. Documented as a decision with its costs stated, not as a gap —
  including the correction that Developer ID signing is Apple's mechanism for
  distribution *outside* the App Store, so skipping the store does not remove it.
- **README install section**: first-launch override instructions, both routes, with an
  explicit caution about what `xattr -dr com.apple.quarantine` does and a pointer to
  building from source instead.
- `docs/macos-code-signing.md` — the Gatekeeper error explained, the per-macOS-version
  behaviour table, both workarounds, the cost/benefit table, and the dormant
  certificate/notarization setup should the decision change.
- Corrected the now-obsolete "right-click ➔ Open" instructions in
  `docs/release-process.md` and the release-manager skill; first-launch instructions are
  now permanent release-note boilerplate.

---

## [v0.2.0-alpha.4] - 2026-08-11

### 🐛 Fixed Bugs & Issues

- **macOS Permission Prompt Storm**: macOS 14+ raises its "access data from other apps"
  TCC consent dialog once **per app container**, and `scan_app_container_caches` walked
  every entry under `~/Library/Containers` (742), `~/Library/Group Containers` (149),
  and `~/Library/Application Support` (104) — roughly 1,000 blocking dialogs per scan.
  Xclense now probes for Full Disk Access and, when absent, skips those categories
  entirely. Verified: zero container paths are touched and 72 items are still found.

### 🚀 Highlights & Features

- **Full Disk Access Detection**: `check_full_disk_access` probes readability of
  `~/Library/Application Support/com.apple.TCC/TCC.db`, which only FDA-holding
  processes can open and which does not itself raise a prompt. Re-checked at the start
  of every scan, so granting access mid-session applies without restarting.
- **One-Click Grant Flow**: `open_full_disk_access_settings` deep-links to
  Privacy & Security ➔ Full Disk Access. The Scan & clean dialog reports how many
  protected locations were skipped and offers a re-check.
- **Scan Result Transparency**: `StorageScanResult` gained `fullDiskAccess` and
  `skippedCategories`, so the UI can state exactly what was not examined instead of
  silently under-reporting.

### 📄 Documentation

- `docs/macos-permissions.md` — TCC behaviour, what is skipped, how detection works,
  and why a single up-front prompt is not possible.
- Portable `tauri-ota-updates` skill capturing the whole OTA procedure for reuse on
  other Tauri projects.

---

## [v0.2.0-alpha.3] - 2026-08-11

### 🚀 Highlights & Features

- **Universal Binary (Intel + Apple Silicon)**: Xclense builds with
  `--target universal-apple-darwin`, producing a single bundle carrying both `arm64`
  and `x86_64` slices. Verified with `lipo -archs`, which the release pipeline now
  enforces — a single-architecture bundle is indistinguishable from a correct one on
  the release page and simply fails to launch for Intel users.
- **OTA Updates Reach Intel Macs**: The update manifest declared only
  `darwin-aarch64`, so an Intel install matched no platform entry and silently never
  updated. It now declares `darwin-x86_64` as well, both pointing at the universal
  payload.

### 🔧 Build & Release Infrastructure

- Bundle artifacts moved to `src-tauri/target/universal-apple-darwin/release/bundle/`;
  the release workflow, skill, and docs were updated to match.
- The updater signing key is backed up in three independent locations (macOS login
  Keychain, iCloud Drive, and the working copy) with recovery procedures that were
  executed and verified, not merely written down. The GitHub secret is explicitly
  documented as unusable for recovery, since GitHub never reveals a secret once set.

---

## [v0.2.0-alpha.2] - 2026-08-11

### 🚀 Highlights & Features

- **Over-the-air Updates**: Xclense updates itself. It checks shortly after launch and
  every six hours, downloads the new build, installs it, and relaunches. Update
  payloads are verified against an Ed25519 signature before install, so a tampered or
  unsigned payload is rejected.
- **Interruption-Safe Restarts**: A shared busy registry (`lib/app-busy.ts`) tracks
  work that must not be killed. If a storage scan or cleanup is in flight when an
  update is staged, the relaunch waits for it to finish rather than cutting a
  Trash operation in half.
- **Update Progress UI**: A corner panel reports download progress with byte counts,
  then install and restart state. Failed checks are silent by design — testers are
  regularly offline, and that is not an error worth surfacing.
- **In-App Version Display**: The sidebar shows the running version and derives its
  channel label from the version string, so neither can drift from `Cargo.toml`.

### 🔧 Build & Release Infrastructure

- Release builds emit and sign the `.app.tar.gz` updater payload; `release.yml` fails
  the run if the `.sig` is absent instead of publishing a release that existing
  installs would silently reject.
- Update manifest published to `updates/latest.json` on `main` and served via
  `raw.githubusercontent.com`. GitHub's "latest release" URL excludes pre-releases, so
  no alpha build could ever be discovered through it.
- Repository made public. Release assets in a private repo return `404` to
  unauthenticated clients, which made OTA impossible.
- GitHub Actions moved off the deprecated Node 20 runtime (checkout v7, setup-node v7,
  cache v6).
- CI now builds the frontend before the Rust checks — `tauri-build` resolves
  `frontendDist` at compile time and failed on a clean checkout without it.

### 🐛 Fixed Bugs & Issues

- Resolved four clippy lints (`unnecessary_sort_by` ×3, `collapsible_str_replace`) so
  the warning count sits at zero.

---

## [v0.2.0-alpha.1] - 2026-08-11

First published build of Xclense and the opening of the **Alpha channel**.

### 🚀 Highlights & Features

- **Storage Scan & Clean Engine**: Category-driven scanner covering user caches, app
  logs, browser caches, developer artifacts, package manager caches, app container
  caches, Downloads, Trash, and system temporary folders, with per-item sizing via
  `du -sk` and Finder-Trash-based removal so every action is recoverable.
- **Hidden File & Folder Discovery**: Full audit of dot-files and dot-folders in the
  home root at any size, hidden folders under `~/Library` and `Application Support`,
  and hidden project build caches (`.next`, `.turbo`, `target`, `dist`, `__pycache__`,
  `.venv`, `Pods`, `DerivedData`). A real scan surfaces 77 items against the 26 the
  earlier category-only pass reported.
- **Deletion Risk Analysis**: Every scan item carries the owning tool, a safety score
  (0-99), whether the data regenerates itself, and a plain-language statement of what
  breaks if the item is missing, resolved from a catalogue of ~90 known tools.
  Unrecognised entries are explicitly marked `unidentified` rather than guessed at.
- **Protected Path Enforcement**: Credential and configuration paths (`.ssh`, `.gnupg`,
  `.aws`, `.kube`, `.docker`, `.config`, `.local`, Keychains, MobileSync, shell rc
  files) and Cline directories are listed but never selectable, enforced at both scan
  and cleanup time.
- **Live Scan & Cleanup Progress**: `storage-scan-progress` and
  `storage-cleanup-progress` event streams drive a real-time overlay showing the
  current stage, the path being read, locations counted, items found, and space
  reclaimed. Long-running commands run off the main thread so events actually reach
  the UI mid-scan.
- **System Health Dashboard**: Overall health score with severity breakdown, RAM
  pressure gauge with a 30-minute trend chart, storage overview with category
  estimates, top resource-consuming processes, and a live issue log.
- **Guided Remediation**: Issue analysis produces a remediation plan with per-step risk
  levels, an auto-runnable safe subset, and a fix overlay for one-click application.
- **Process Control with Auditing**: Terminate and force-kill workflows gated behind
  explicit confirmation, with every decision written to an append-only audit record
  carrying a decision code, risk level, and source context.
- **Report Snapshots & Export**: Health snapshots can be saved, listed, reloaded, and
  exported to JSON or TXT for sharing.

### 🐛 Fixed Bugs & Issues

- Corrected byte math in the storage overview so used, free, and percentage figures
  agree with the physical volume totals.
- Fixed nested paths being counted twice in the reclaimable total; a parent and its
  child are now never both reported.
- Excluded protected and high-risk entries from the headline "Reclaimable" figure so
  it reflects only space that can actually be selected.
- Fixed dialog backdrop and animation timing that caused a visible flash on open.

### 🔧 Build & Release Infrastructure

- **Static Frontend Export**: `next.config.ts` now sets `output: "export"` and
  `tauri.conf.json` points `frontendDist` at `../out`, so `tauri build` bundles a
  servable static tree instead of Next.js internals. This is what makes a shippable
  `.app`/`.dmg` possible.
- **Release Manager Skill**: Added `.claude/skills/release-manager/SKILL.md` defining
  the version-bump surface, release channels, and publishing SOP.
- **CI/CD Pipelines**: Added `.github/workflows/ci.yml` (typecheck, lint, clippy,
  static export on every push and PR) and `.github/workflows/release.yml` (macOS
  bundle build and GitHub Release publication on `v*` tags).

---

## [v0.1.0] - 2026-04-28

### 🚀 Initial Scaffold

- **Dual-Layer Architecture**: Next.js App Router UI shell paired with a Rust + Tauri
  backend holding all privileged system operations.
- **Typed IPC Client**: `lib/tauri-client.ts` wrapping Tauri commands with runtime
  guards so the UI degrades safely outside the desktop shell.
- **Baseline System Commands**: `ping_backend`, storage summary scanning, process
  listing, and system health metrics.
- **Project Documentation**: Architecture overview, roadmap, and the APFS-safe macOS
  disk scanner shell script.
