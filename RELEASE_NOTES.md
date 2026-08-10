# v0.2.0-alpha.3 — Alpha Release

## 🔄 What's Changed (v0.2.0-alpha.2 ➔ v0.2.0-alpha.3)

- **Channel**: Alpha Release (Preview Channel)
- **Platform**: macOS 11+ · **Apple Silicon and Intel** (universal binary)
- **Install**: If you already run `v0.2.0-alpha.2`, do nothing — this update installs
  itself. Otherwise download the `.dmg`, drag Xclense to Applications, and on first
  launch use **right-click ➔ Open** to get past Gatekeeper.

### ✨ New Features & Enhancements

- **Intel Mac support**: Xclense now ships as a universal binary carrying both `arm64`
  and `x86_64` slices. One download serves every Mac from 2016 onward.
- **Intel Macs receive OTA updates**: The update manifest previously advertised only
  `darwin-aarch64`, so an Intel install found no matching entry and silently never
  updated — no error, no notice. The manifest now declares both architectures.

### 🔧 Build & Release Infrastructure

- Release builds use `--target universal-apple-darwin`, and the pipeline runs
  `lipo -archs` to fail the build if either architecture slice is missing. A
  single-architecture bundle looks entirely normal on the release page and simply
  fails to launch for half the audience, so this is checked rather than assumed.
- The update signing key is now backed up in three independent places with tested
  recovery procedures, instead of existing only on one machine.

### ⚠️ Alpha Channel Notes

- The build remains **unsigned and un-notarized**; Gatekeeper warns on first launch.
- The DMG is larger (~14 MB, was ~8 MB) because it now carries two architectures.
- A full storage scan takes roughly 30-40 seconds.
- Cleanup always routes through Finder's Trash — Xclense never deletes permanently.

Report issues at <https://github.com/sudsarkar13/xclense/issues> with the version
shown in the sidebar.
