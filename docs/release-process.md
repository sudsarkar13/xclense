# Xclense Release Process — Channels, Versioning, and the Alpha/Beta Programs

This document defines how Xclense versions are numbered, how the Alpha, Beta, and
Stable channels work, and what each channel promises to the people running it.

The mechanical, step-by-step publishing procedure lives in the
[`release-manager` skill](../.claude/skills/release-manager/SKILL.md).

---

## 1. Versioning

Xclense follows [Semantic Versioning](https://semver.org/): `MAJOR.MINOR.PATCH`, with
optional pre-release suffixes.

| Component | Increments when |
| --- | --- |
| `MAJOR` | Breaking change to stored data, audit formats, or the IPC contract |
| `MINOR` | New capability added in a backwards-compatible way |
| `PATCH` | Bug fixes only, no new capability |

While the app is pre-1.0, `MINOR` carries the weight of a feature release and breaking
changes may land in a `MINOR` bump.

### Pre-release suffixes

```
0.2.0-alpha.1   ➔   0.2.0-alpha.2   ➔   0.2.0-beta.1   ➔   0.2.0
└──────────────── same target version ────────────────────┘
```

The target version (`0.2.0`) is chosen once at the start of a cycle and never moves
backwards. Alpha builds harden into beta builds, and the final stable release drops the
suffix entirely.

Semver orders these correctly: `0.2.0-alpha.1 < 0.2.0-alpha.2 < 0.2.0-beta.1 < 0.2.0`.
A pre-release always sorts *before* the stable version it leads to, which is what lets
tooling reason about upgrade paths.

If a change during the cycle turns out to be breaking or much larger than planned,
abandon the target and restart at the next minor: `0.3.0-alpha.1`.

---

## 2. The three channels

### 🔴 Alpha — `vX.Y.Z-alpha.N`

**Audience**: the maintainer and a small circle of invited testers who are comfortable
with rough edges.

**Promise**: none. Features may be incomplete, may change shape between builds, or may
be withdrawn. Alpha builds are where a feature is proven against a real machine for the
first time.

- Published as a **GitHub pre-release** (`--prerelease`), so it never occupies the
  "Latest" slot.
- Expect frequent increments — `alpha.1`, `alpha.2`, `alpha.3` within days.
- Known limitations are listed explicitly in the release notes.
- Destructive operations must remain reversible (Xclense routes all cleanup through
  Finder's Trash, never permanent deletion) — this is a hard requirement for any alpha
  build that touches the filesystem.

**Exit criteria to reach Beta**: the feature set for the target version is complete, no
known data-loss paths, and a full scan/clean cycle completes without errors on the
maintainer's machine.

### 🟡 Beta — `vX.Y.Z-beta.N`

**Audience**: a wider tester group willing to run the app against their daily-driver
machine.

**Promise**: feature-complete for the target version. No new features are added during
beta — only bug fixes, polish, and documentation.

- Published as a **GitHub pre-release** (`--prerelease`).
- Every beta increment should fix reported issues, not add scope. If a new feature is
  genuinely required, the target version moves and the cycle restarts at alpha.
- Release notes must list which reported issues each beta closes.

**Exit criteria to reach Stable**: no open bugs classed as blocking, the release notes'
"Known limitations" list is empty or acceptable, and the bundle installs and runs on a
clean machine.

### 🟢 Stable — `vX.Y.Z`

**Audience**: everyone.

**Promise**: safe for regular use. Documented behaviour matches actual behaviour.

- Published as a normal release and becomes **Latest** on GitHub.
- Regressions found after a stable release are fixed in a `PATCH` release
  (`0.2.1`), not by re-tagging.

---

## 3. Channel mechanics on GitHub

The channel is derived from the tag itself — there is no separate configuration:

| Tag pattern | Title | `--prerelease` | Shown as "Latest" |
| --- | --- | --- | --- |
| `v0.2.0-alpha.1` | `v0.2.0-alpha.1 — Alpha Release` | yes | no |
| `v0.2.0-beta.1` | `v0.2.0-beta.1 — Beta Release` | yes | no |
| `v0.2.0` | `v0.2.0 — Stable Release` | no | yes |

`.github/workflows/release.yml` parses the tag and applies the correct flag
automatically, so pushing a tag is the only action needed to publish. The workflow also
refuses to publish if the tag disagrees with `package.json`, `tauri.conf.json`, or
`Cargo.toml`, which prevents a build labelled with the wrong version from ever reaching
a release page.

The in-app sidebar shows the running version and derives its channel label from the
same string, so a tester can always read their exact build off the screen.

---

## 4. Running the Alpha/Beta program

### Distributing builds

The repository is private, so GitHub Releases are visible only to collaborators. To
bring in an outside tester, either add them as a repository collaborator, or send them
the `.dmg` directly from the release page.

### What testers need to know

Every pre-release's notes must state:

1. **Install instructions** — the bundle is unsigned and un-notarized, so first launch
   requires **right-click ➔ Open** and confirming the Gatekeeper prompt.
2. **Known limitations** — what is expected to be broken or slow.
3. **How to report** — <https://github.com/sudsarkar13/xclense/issues>, including the
   version string shown in the sidebar.

### Signing and notarization

Unsigned builds are acceptable for a small invited alpha. Before a public beta, set up
Apple Developer signing so testers stop seeing Gatekeeper warnings. Tauri reads these
from the environment (add them as repository secrets for the release workflow):

```
APPLE_CERTIFICATE
APPLE_CERTIFICATE_PASSWORD
APPLE_SIGNING_IDENTITY
APPLE_ID
APPLE_PASSWORD
APPLE_TEAM_ID
```

### Architecture coverage

The current pipeline builds for Apple Silicon (`aarch64`) only. To also serve Intel
Macs, build a universal binary:

```bash
yarn tauri build --target universal-apple-darwin
```

This requires both `aarch64-apple-darwin` and `x86_64-apple-darwin` Rust targets to be
installed.

---

## 5. Release checklist

- [ ] Target version and channel decided
- [ ] Version bumped in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`
- [ ] `cargo check` run so `src-tauri/Cargo.lock` picks up the new version
- [ ] `yarn tsc --noEmit`, `yarn eslint .`, and `cargo clippy` all clean
- [ ] `CHANGELOG.md` has a new dated section
- [ ] `RELEASE_NOTES.md` overwritten with the previous ➔ new delta only
- [ ] `yarn tauri build` produces a `.dmg` whose filename carries the new version
- [ ] Bundle contains `Contents/Resources/icon.icns` and `CFBundleIconFile` in Info.plist
- [ ] Bundle smoke-tested: dashboard loads, scan completes, sidebar shows the new version
- [ ] Commit, tag `vX.Y.Z`, push both
- [ ] Release published with the correct pre-release flag and the `.dmg` attached
