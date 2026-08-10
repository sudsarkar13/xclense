---
name: release-manager
description: Standard operating procedure for cutting Xclense releases — version bumping across every manifest, static frontend export, macOS Tauri bundle build, version-specific release notes generation (previous ➔ new delta), release channel tagging (Stable/Beta/Alpha), and GitHub Release publishing. ONLY activate when the maintainer explicitly requests a new version release or version bump.
---

# Release Manager Skill — Xclense

> **ACTIVATION RULE**: Activate this skill ONLY when the maintainer explicitly asks for
> a new version release, release update, or version bump. Do NOT auto-trigger it for
> routine bug fixes, feature work, or minor edits.

Xclense ships as a **macOS Tauri desktop app**: a Next.js static export bundled inside
a Rust binary, distributed as a `.dmg` attached to a GitHub Release. There is no
package registry to publish to — the GitHub Release *is* the distribution channel.

Repository: `sudsarkar13/xclense` (private) · Default branch: `main`

---

## 📌 Release Naming & Versioning Rules

### 1. Release Channel Categorization

Every release MUST fall into one of three channels:

| Channel | Tag format | Release title | GitHub flag |
| --- | --- | --- | --- |
| **Stable** | `vX.Y.Z` | `vX.Y.Z — Stable Release` | Latest |
| **Beta** | `vX.Y.Z-beta.N` | `vX.Y.Z-beta.N — Beta Release` | `--prerelease` |
| **Alpha** | `vX.Y.Z-alpha.N` | `vX.Y.Z-alpha.N — Alpha Release` | `--prerelease` |

> Keep titles clean and standardized. Do **NOT** write verbose custom titles like
> `v0.3.0 Release — Big Storage Overhaul`. The highlights belong in the notes, not the
> title.

Channel semantics and the promotion path are defined in
[docs/release-process.md](../../../docs/release-process.md). In short:
`X.Y.Z-alpha.N` ➔ `X.Y.Z-beta.N` ➔ `X.Y.Z`, where the target `X.Y.Z` is chosen once at
the start of the cycle and never moves backwards.

### 2. Version-Specific Release Notes (NO Full Historical Changelog)

When publishing a GitHub Release, **DO NOT** attach the whole `CHANGELOG.md`. The
release body MUST be limited to the **delta between the previous version and the new
version**, written to `RELEASE_NOTES.md` and passed via `--notes-file`.

Every release note MUST contain:

1. **Comparison header**: `What's Changed (v<PREVIOUS> ➔ v<NEW>)`
2. **Channel line**: `Channel: Alpha Release (Preview Channel)` (or Beta / Stable)
3. **New Features & Enhancements**: bulleted list of what was added
4. **Fixed Bugs & Issues**: bulleted list of what was fixed
5. **Alpha/Beta notes** (pre-releases only): known limitations, signing status,
   and where to report issues

---

## 🎯 The Version Bump Surface

`X.Y.Z` must be updated consistently in **four** places. Missing any one produces a
bundle whose reported version disagrees with its tag.

| File | Field | Consumed by |
| --- | --- | --- |
| `package.json` | `"version"` | npm/yarn metadata, tooling |
| `src-tauri/tauri.conf.json` | `"version"` | `.app`/`.dmg` bundle version, `CFBundleShortVersionString` |
| `src-tauri/Cargo.toml` | `version` under `[package]` | `env!("CARGO_PKG_VERSION")`, which is written into every audit record via `source_version` |
| `src-tauri/Cargo.lock` | `Xclense` package entry | Regenerated automatically — run `cargo check` after bumping `Cargo.toml`, then commit the lock |

Plus the two narrative files:

- `CHANGELOG.md` — prepend a new `## [vX.Y.Z] - YYYY-MM-DD` section (full history lives here)
- `RELEASE_NOTES.md` — **overwrite** with the notes for this release only

There is no version string hardcoded in the UI. `ping_backend` returns
`env!("CARGO_PKG_VERSION")`, so the app reports its version from `Cargo.toml` alone —
never hardcode a version in TSX.

Verify nothing was missed before tagging:

```bash
grep -rn "0\.2\.0-alpha\.1" package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
```

---

## 📋 Standard Operating Procedure (SOP)

### 1. Bump every manifest

Update the four files above to the target version, then refresh the lock:

```bash
cd src-tauri && cargo check && cd ..
```

### 2. Quality gates — all must pass before building

```bash
yarn tsc --noEmit                      # TypeScript
yarn eslint .                          # Lint
yarn build                             # Static export — REQUIRED before any cargo step
cd src-tauri && cargo clippy && cd ..  # Rust lints
```

> `yarn build` must come before `cargo check`/`cargo clippy`. `tauri-build` resolves
> `frontendDist` at compile time and panics with
> `The 'frontendDist' configuration is set to "../out" but this path doesn't exist`
> when the frontend has not been exported yet. This bites on a clean checkout and in
> CI, not locally where `out/` usually already exists.

If formatting is needed, this repo has **no prettier config file** — the style comes
from the maintainer's editor. Match it explicitly or the diff explodes:

```bash
npx prettier --use-tabs --experimental-ternaries --bracket-same-line --write <files>
```

CSS files use 2-space indent — leave them out of the `--use-tabs` run.

### 3. Build the macOS bundle

`tauri build` runs `yarn build` first (via `beforeBuildCommand`), which must emit a
static tree into `out/` — this depends on `output: "export"` in `next.config.ts` and
`"frontendDist": "../out"` in `tauri.conf.json`. Do not change either without
re-verifying the bundle.

```bash
yarn tauri build
```

Artifacts land in:

```
src-tauri/target/release/bundle/macos/Xclense.app
src-tauri/target/release/bundle/dmg/Xclense_<version>_aarch64.dmg
```

> The build is **unsigned and un-notarized**. Gatekeeper will warn testers on first
> launch; the install instructions in `RELEASE_NOTES.md` must tell them to use
> right-click ➔ Open. To sign later, set `APPLE_CERTIFICATE`,
> `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`,
> `APPLE_PASSWORD`, and `APPLE_TEAM_ID` in the environment (or as repo secrets for
> `.github/workflows/release.yml`).

### 4. Smoke-test the bundle

```bash
open src-tauri/target/release/bundle/macos/Xclense.app
```

Confirm the dashboard loads, the storage scan runs to completion with live progress,
and the version in the About/`ping_backend` response matches the target version.

### 5. Write version-specific notes

Overwrite `RELEASE_NOTES.md` with the previous ➔ new delta (format above), and prepend
the matching section to `CHANGELOG.md`. Derive the delta from the commit range:

```bash
git log --oneline --no-merges <PREVIOUS_TAG>..HEAD
```

For the very first release, use the full history.

### 6. Commit, tag, and push

```bash
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock \
        CHANGELOG.md RELEASE_NOTES.md
git commit -m "release: vX.Y.Z — <Channel> Release — <short highlights>"

git push origin main

git tag -a vX.Y.Z -m "vX.Y.Z — <Channel> Release"
git push origin vX.Y.Z
```

`gh` is already authenticated for `sudsarkar13`; plain `git push origin` works — no
token interpolation in the URL is needed.

### 7. Publish the GitHub Release

**Stable:**

```bash
gh release create vX.Y.Z \
  "src-tauri/target/release/bundle/dmg/Xclense_X.Y.Z_aarch64.dmg" \
  --title "vX.Y.Z — Stable Release" \
  --notes-file RELEASE_NOTES.md \
  --repo sudsarkar13/xclense
```

**Alpha / Beta — add `--prerelease`:**

```bash
gh release create vX.Y.Z-alpha.N \
  "src-tauri/target/release/bundle/dmg/Xclense_X.Y.Z-alpha.N_aarch64.dmg" \
  --title "vX.Y.Z-alpha.N — Alpha Release" \
  --notes-file RELEASE_NOTES.md \
  --prerelease \
  --repo sudsarkar13/xclense
```

> The `.dmg` filename embeds the version — always resolve it with
> `ls src-tauri/target/release/bundle/dmg/` rather than assuming the exact string.
> Bundle artifacts are gitignored via `/src-tauri/target` and must NOT be committed.

> **Two publishing paths, pick one.** Pushing the tag also triggers
> `.github/workflows/release.yml`, which builds its own bundle and publishes. If you
> publish by hand from a local build (step 7), that workflow run is redundant —
> it detects the existing release and uploads its bundle to it with `--clobber`
> rather than failing, but it still burns macOS runner minutes. Either cancel the run
> with `gh run cancel <id>`, or skip step 7 entirely and let the workflow do the
> publishing.

### 8. Verify

```bash
gh release view vX.Y.Z --repo sudsarkar13/xclense
gh release list --repo sudsarkar13/xclense --limit 5
```

Confirm the pre-release flag is correct: alpha and beta releases must NOT be marked
`Latest`, or testers on the stable channel will be offered a preview build.

---

## 🚦 Choosing the Next Version

| Situation | Next version from `v0.2.0-alpha.1` |
| --- | --- |
| More alpha fixes, same target | `v0.2.0-alpha.2` |
| Feature-complete, ready for wider testing | `v0.2.0-beta.1` |
| Beta validated, shipping | `v0.2.0` |
| Breaking change mid-alpha | Restart at `v0.3.0-alpha.1` |
| Urgent fix on a shipped stable | `v0.2.1` |

Never re-tag or force-push a published tag. If a release is wrong, publish the next
increment and delete the bad release with
`gh release delete <tag> --cleanup-tag`.
