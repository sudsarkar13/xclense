# v0.2.0-alpha.1 — Alpha Release

## 🔄 What's Changed (v0.1.0 ➔ v0.2.0-alpha.1)

- **Channel**: Alpha Release (Preview Channel)
- **Platform**: macOS 11+ · Apple Silicon (aarch64)
- **Install**: Download the `.dmg`, drag Xclense to Applications. The build is
  unsigned, so on first launch use **right-click ➔ Open** and confirm the prompt.

### ✨ New Features & Enhancements

- **Storage Scan & Clean Engine**: Scans user caches, app logs, browser caches,
  developer artifacts, package manager caches, app container caches, Downloads, Trash,
  and system temporary folders. Cleaned items go to Finder's Trash, so every action is
  recoverable.
- **Hidden File & Folder Discovery**: Lists every dot-file and dot-folder in the home
  root at any size, plus hidden folders under `~/Library` and `Application Support`,
  and hidden project build caches (`.next`, `.turbo`, `target`, `dist`, `__pycache__`,
  `.venv`, `Pods`, `DerivedData`). On a representative machine this surfaces 77 items
  versus the 26 found by category scanning alone.
- **Deletion Risk Analysis**: Each item shows its owning tool, a 0-99 safety score,
  whether the data rebuilds itself, and what actually breaks if it is missing —
  resolved from a catalogue of ~90 known developer tools. Unrecognised entries are
  labelled `unidentified` instead of being guessed at.
- **Protected Paths**: `.ssh`, `.gnupg`, `.aws`, `.kube`, `.docker`, `.config`,
  `.local`, Keychains, MobileSync, shell rc files, and Cline directories are visible
  for auditing but can never be selected for cleanup.
- **Live Progress Overlay**: Real-time scan and cleanup progress showing the current
  stage, the exact path being read, locations counted, items found, and space
  reclaimed.
- **System Health Dashboard**: Health score with severity breakdown, RAM pressure
  gauge with 30-minute trend, storage overview, top resource-consuming processes, and
  a live issue log.
- **Guided Remediation & Process Control**: Remediation plans with per-step risk
  levels and an auto-runnable safe subset; terminate/force-kill workflows gated behind
  explicit confirmation and written to an append-only audit trail.
- **Report Snapshots**: Save, reload, and export health snapshots as JSON or TXT.

### 🐛 Fixed Bugs & Issues

- Corrected storage overview byte math so used, free, and percentage figures agree
  with physical volume totals.
- Fixed nested paths being double-counted in the reclaimable total.
- Excluded protected and high-risk entries from the headline "Reclaimable" figure so
  it reflects only selectable space.
- Fixed a dialog backdrop flash caused by mismatched animation timing.
- Moved long-running scan and cleanup commands off the main thread; progress events
  previously could not reach the UI until the whole operation had finished.

### 🔧 Build & Release Infrastructure

- Frontend now builds as a true static export (`output: "export"` ➔ `out/`), which is
  what allows Tauri to bundle a working `.app`/`.dmg`.
- Added the `release-manager` skill, CI pipeline (typecheck, lint, clippy, export
  build), and a tag-triggered macOS release pipeline.

### ⚠️ Alpha Channel Notes

This is an early preview build. Known limitations:

- The build is **unsigned and un-notarized**; Gatekeeper will warn on first launch.
- A full scan takes roughly 30-40 seconds because directory sizing shells out to
  `du -sk` per candidate path.
- Apple Silicon only; no Intel (x86_64) bundle is published in this build.
- Cleanup always routes through Finder's Trash — Xclense never deletes permanently.

Please report issues at <https://github.com/sudsarkar13/xclense/issues> and include
the version string from the release title.
