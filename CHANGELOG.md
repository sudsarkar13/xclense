# Changelog

All notable changes to the **Xclense** project are documented in this file.

Versions follow [Semantic Versioning](https://semver.org/). Pre-release builds use
the `-alpha.N` and `-beta.N` suffixes described in
[docs/release-process.md](docs/release-process.md).

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
