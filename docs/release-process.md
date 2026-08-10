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

```text
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

The repository is **public**, so anyone with the release URL can download a build —
no collaborator invite needed. Point a tester at the releases page and they install
the `.dmg` once; every build after that arrives over the air.

The repository was made public specifically to serve OTA updates: release assets in a
private repo return `404` to unauthenticated clients, so an installed app could never
fetch its own update.

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

```text
APPLE_CERTIFICATE
APPLE_CERTIFICATE_PASSWORD
APPLE_SIGNING_IDENTITY
APPLE_ID
APPLE_PASSWORD
APPLE_TEAM_ID
```

### Architecture coverage

From `v0.2.0-alpha.3` the pipeline builds a **universal binary** carrying both `arm64`
and `x86_64` slices, so one `.dmg` serves Apple Silicon and Intel Macs alike:

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
yarn tauri build --target universal-apple-darwin
```

Artifacts move under `src-tauri/target/universal-apple-darwin/release/bundle/`, not
`target/release/bundle/`. Confirm both slices are present before publishing — a
single-architecture bundle looks completely normal on the release page and simply
fails to launch for half your users:

```bash
lipo -archs src-tauri/target/universal-apple-darwin/release/bundle/macos/Xclense.app/Contents/MacOS/Xclense
# expected: x86_64 arm64
```

`release.yml` runs this check and fails the build if either slice is missing.

---

## 5. Over-the-air (OTA) updates

Installed copies of Xclense update themselves. The user never downloads a DMG twice.

### How it works

```text
app launches
   ↓ (5s delay, then every 6 hours)
GET raw.githubusercontent.com/sudsarkar13/xclense/main/updates/latest.json
   ↓ manifest version > running version?
download github.com/.../releases/download/<tag>/Xclense.app.tar.gz
   ↓ verify Ed25519 signature against the public key compiled into the app
install (replaces the .app bundle)
   ↓ app idle?
relaunch   ──  busy?  ──▶ wait for the scan/cleanup to finish, then relaunch
```

### Why a static manifest and not the release URL

The obvious endpoint is
`https://github.com/<owner>/<repo>/releases/latest/download/latest.json`. It does not
work here: GitHub's `/releases/latest` **excludes pre-releases**, and every alpha and
beta build is published as one. An alpha tester would never be offered an alpha
update.

So the manifest is a committed file, `updates/latest.json`, served through
`raw.githubusercontent.com`. `release.yml` regenerates and commits it on every tag
push. That URL caches for around five minutes, which is the practical floor on how
fast an update reaches clients.

### Signing

Updates are signed with an Ed25519 (minisign) keypair. The **public** key is compiled
into the app via `plugins.updater.pubkey` in `tauri.conf.json`; the **private** key
signs the tarball at build time.

- Working copy: `~/.tauri/xclense-updater.key`, mode `600`, never in the repo
- macOS login Keychain, service `Xclense Updater Signing Key`
- iCloud Drive: `Xclense-Signing-Key/` (with a `README.txt` covering recovery)
- CI: `TAURI_SIGNING_PRIVATE_KEY` repository secret — **write-only**, GitHub will
  never reveal it again, so it cannot serve as a backup
- Password: `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (currently empty)

Restore from the Keychain (`security` appends a newline the key file does not have):

```bash
security find-generic-password -s "Xclense Updater Signing Key" -w \
  | perl -pe 'chomp if eof' > ~/.tauri/xclense-updater.key
chmod 600 ~/.tauri/xclense-updater.key
```

> **Losing the private key is unrecoverable.** Every installed copy only trusts
> signatures from this exact key. If it is lost, no future build can update any
> existing install — every user has to reinstall from a DMG by hand.
>
> Replacing the key is possible but requires planning: ship a release signed with the
> **old** key that carries the **new** public key, wait for users to install it, and
> only then start signing with the new key. Skipping that transitional build strands
> everyone.

An unsigned build is worse than a failed build: the release page looks correct, but
clients silently reject the update. `release.yml` fails the run if the `.sig` file is
missing, which is the guard against shipping one.

### What the user sees

Nothing, until there is something to see. A corner panel appears during download with
a progress bar, switches to "Installing", and the app relaunches itself. If a storage
scan or cleanup is running, the panel says the restart is waiting and the relaunch
happens the moment that work finishes — a cleanup moving files to Trash is never cut
off midway.

A failed check is silent by design. Testers are frequently offline or behind captive
portals, and a background check that cannot reach GitHub is not an error worth
interrupting anyone about.

### Architecture coverage caveat

The manifest declares only `darwin-aarch64`. An Intel Mac running Xclense finds no
matching platform entry and simply never updates — silently. Adding
`darwin-x86_64` (or a universal build) is a prerequisite for supporting Intel testers.

## 6. Release checklist

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
