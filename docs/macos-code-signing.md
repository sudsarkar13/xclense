# macOS Code Signing, Notarization & Gatekeeper

## The error

> **"Xclense" Not Opened**
> Apple could not verify "Xclense" is free of malware that may harm your Mac or
> compromise your privacy.
> **[ Done ]  [ Move to Bin ]**

This is Gatekeeper, and it is not a bug in Xclense. It says exactly one thing: the
bundle is not signed with an Apple **Developer ID Application** certificate and has not
been **notarized** by Apple.

What the current builds actually carry:

```console
$ codesign -dv --verbose=4 /Applications/Xclense.app
Signature=adhoc
TeamIdentifier=not set
CodeDirectory ... flags=0x20002(adhoc,linker-signed)

$ spctl -a -vvv /Applications/Xclense.app
/Applications/Xclense.app: rejected
source=no usable signature
```

`Signature=adhoc` is what the Rust linker produces on Apple Silicon so the binary can
execute at all. It proves nothing about origin, so Gatekeeper refuses it.

## Why the old workaround stopped working

Gatekeeper only enforces this on files carrying the `com.apple.quarantine` extended
attribute, which browsers attach to every download. The historic fix was to
right-click ➔ **Open**. That path has been closed progressively:

| macOS | Behaviour for an unsigned, quarantined app |
| --- | --- |
| 14 Sonoma and earlier | Right-click ➔ **Open** ➔ confirm. Done. |
| 15 Sequoia | Right-click no longer works. Must use System Settings ➔ Privacy & Security ➔ **Open Anyway**, then authenticate. |
| 26 Tahoe | Same as Sequoia, and the first dialog offers only **Done** / **Move to Bin** — there is no affordance suggesting the app can be opened at all. |

The dialog is deliberately dead-ended. Nothing in it hints that Privacy & Security is
where you go next, which is why this reads as "the app is broken" rather than "the app
is unsigned".

## Unblocking a build today (testers)

**Option A — System Settings.** Launch Xclense, click **Done** on the warning, then
open **System Settings ➔ Privacy & Security**, scroll to the Security section, and click
**Open Anyway** next to the Xclense message. Authenticate, then launch again and
confirm. The message only appears for about an hour after the blocked launch attempt,
so do it straight away.

**Option B — remove the quarantine flag.** One command, and no dialog at all:

```bash
xattr -dr com.apple.quarantine /Applications/Xclense.app
```

This strips the "downloaded from the internet" marker, so Gatekeeper stops evaluating
the bundle. Only run it against software you actually trust — it is the same command
malware distributors ask victims to run.

**Note on OTA updates:** this only affects DMG installs. Updates delivered by the
in-app updater are downloaded by Xclense itself rather than by a browser, so they never
receive a quarantine attribute and never trigger the dialog. That is why alpha.2 ➔
alpha.3 ➔ alpha.4 updated silently while a fresh DMG download is blocked.

## Project position: unsigned is the intended distribution mode

Xclense is free, open source, and distributed outside the App Store. **We do not sign or
notarize releases**, and that is a decision rather than a gap.

A common misconception is worth clearing up first: **Developer ID signing has nothing to
do with the App Store.** It is Apple's mechanism for distributing apps *outside* the
store — the "Developer ID Application" certificate exists precisely for direct download.
Staying off the App Store does not remove the need for it; the thing that removes the
need is accepting that users perform a one-time override.

What that choice actually costs, honestly:

| | Unsigned (today) | Developer ID + notarized |
| --- | --- | --- |
| Price | Free | $99/year, forever |
| First launch | One-time override, per install | Opens normally |
| OTA updates | Work — never blocked | Work |
| Update payload integrity | Verified by Ed25519 signature | Same |
| Full Disk Access after an update | May need re-granting | Persists |
| User trust signal | "unidentified developer" | Named publisher |

The trade is real but small for an open-source tool where users can read and build the
source. The override is one command or four clicks, and it happens once per install.

Two consequences to be aware of:

- **The bundle carries no real signature at all.** `codesign -d -r-` reports *"code
  object is not signed at all"* — the ad-hoc, linker-signed marker covers only the inner
  executable, so `Sealed Resources=none` and the `Info.plist` is unbound.
- **Code identity changes on every build.** Two builds of the identical version produce
  different `CDHash` values (verified: `300179e6…` vs `9faeae61…`). macOS has nothing
  stable to anchor a TCC grant to, so a Full Disk Access grant may need re-applying
  after an update. This is the one place the missing certificate has a real, recurring
  user cost — see [docs/macos-permissions.md](macos-permissions.md).

Everything below is what to do **if that calculus ever changes** — a sponsor, donations,
or enough users that repeated FDA re-grants become the top complaint. It is wired up and
inactive, not missing.

## What signing would require

Gatekeeper acceptance requires **all** of:

1. A **Developer ID Application** certificate — Apple Developer Program, **$99/year**.
   There is no free tier that produces a distributable certificate. A free Apple ID
   only issues "Apple Development" certificates, which Gatekeeper rejects for
   distribution just as firmly as an ad-hoc signature.
2. **Hardened Runtime** enabled — a hard prerequisite for notarization.
3. **Notarization** — the bundle is uploaded to Apple, scanned, and a ticket is issued.
4. **Stapling** — the ticket is attached to the bundle so first launch works offline.

Steps 2–4 are already wired up in this repo and dormant. Step 1 is the purchase we have
chosen not to make.

### What is already in place

| Piece | Where | Status |
| --- | --- | --- |
| Hardened runtime | `src-tauri/tauri.conf.json` ➔ `bundle.macOS.hardenedRuntime` | ✅ |
| Entitlements | `src-tauri/Entitlements.plist` | ✅ |
| Usage descriptions | `src-tauri/Info.plist` | ✅ |
| Minimum system version | `bundle.macOS.minimumSystemVersion: "11.0"` | ✅ |
| Signing + notarization in CI | `.github/workflows/release.yml` | ✅ (dormant — activates on secrets) |
| Post-build signature assertion | `.github/workflows/release.yml` | ✅ |
| Developer ID certificate | Apple Developer Program | ➖ **deliberately not purchased** |

Adding the six `APPLE_*` secrets is the entire activation step — no code change. Until
then the workflow falls back to an ad-hoc build and emits a `::warning::` recording that
the release is unsigned.

> **Why `hardenedRuntime: true` does not affect today's builds.** Tauri only runs
> `codesign` when a signing identity is present, so with no certificate the setting is
> inert and the bundle keeps its linker-signed ad-hoc marker
> (`flags=0x20002(adhoc,linker-signed)`, verified after the change). This is deliberate:
> applying the hardened runtime to an *ad-hoc* signature would be the worst of both
> worlds, since the system does not honour privileged entitlements from an untrusted
> signature and Apple Events — the mechanism behind every Trash operation — could be
> denied. Hardened runtime is only correct alongside a real certificate, which is
> exactly when it switches on.

### Entitlements, and why they are minimal

`src-tauri/Entitlements.plist` grants exactly one thing:

```xml
<key>com.apple.security.automation.apple-events</key>
<true/>
```

Xclense trashes files by sending an Apple Event to Finder (`move_to_trash` in
`src-tauri/src/lib.rs`), which keeps every deletion recoverable. The hardened runtime
blocks Apple Events unless this entitlement is present — so without it, a *correctly
signed* build would build, install, launch, scan, and then silently fail to delete
anything. It is the least obvious way this could break.

Deliberately excluded:

- **`com.apple.security.app-sandbox`** — a sandboxed app can never hold Full Disk
  Access, which is precisely what the scanner needs (see
  [docs/macos-permissions.md](macos-permissions.md)).
- **`cs.allow-jit`, `cs.disable-library-validation`** — commonly copy-pasted into Tauri
  entitlements, but WKWebView runs its JIT in a separate Apple-signed process. The host
  binary needs neither, and unnecessary entitlements only widen the attack surface.

## If signing is ever adopted (maintainer)

Not currently planned. Kept here so the decision stays reversible without re-research.

### 1. Join the Apple Developer Program

<https://developer.apple.com/programs/> — $99/year. Enrolment takes anywhere from a few
hours to a couple of days.

### 2. Create the certificate

1. Xcode ➔ **Settings ➔ Accounts ➔ Manage Certificates ➔ + ➔ Developer ID Application**
   (or via <https://developer.apple.com/account/resources/certificates/list>)
2. Confirm it landed:

   ```bash
   security find-identity -v -p codesigning
   # 1) ABC123... "Developer ID Application: Your Name (TEAMID)"
   ```

3. Export from **Keychain Access** as `.p12` with a password, then base64 it:

   ```bash
   openssl base64 -A -in certificate.p12 -out certificate-base64.txt
   ```

> Back the `.p12` up the same way as the updater signing key (see
> [docs/release-process.md](release-process.md)). Losing it is recoverable — you can
> issue a new certificate — but every user then sees the developer identity change.

### 3. Create an App Store Connect API key for notarization

<https://appstoreconnect.apple.com/access/integrations/api> ➔ **Keys** ➔ **+**, role
**Developer**. Download the `AuthKey_XXXX.p8` — it can only be downloaded **once**.

### 4. Set the repository secrets

```bash
gh secret set APPLE_CERTIFICATE          --repo sudsarkar13/xclense < certificate-base64.txt
gh secret set APPLE_CERTIFICATE_PASSWORD --repo sudsarkar13/xclense
gh secret set APPLE_SIGNING_IDENTITY     --repo sudsarkar13/xclense   # "Developer ID Application: Your Name (TEAMID)"
gh secret set APPLE_API_ISSUER           --repo sudsarkar13/xclense   # UUID above the keys table
gh secret set APPLE_API_KEY              --repo sudsarkar13/xclense   # the 10-char Key ID
gh secret set APPLE_API_KEY_CONTENT      --repo sudsarkar13/xclense < AuthKey_XXXX.p8
```

That is the entire change. The next tagged release signs, notarizes, staples, and
verifies itself — and now **fails** rather than shipping if any of that does not hold.

### 5. Verify

```bash
spctl -a -vvv -t exec /Applications/Xclense.app     # expect: accepted, source=Notarized Developer ID
codesign -dv --verbose=4 /Applications/Xclense.app  # expect: TeamIdentifier=<TEAMID>, flags=...runtime
xcrun stapler validate /Applications/Xclense.app    # expect: The validate action worked!
```

Local signed build, for testing before touching CI:

```bash
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
export APPLE_API_ISSUER="..." APPLE_API_KEY="..." APPLE_API_KEY_PATH="$HOME/private_keys/AuthKey_XXXX.p8"
yarn tauri build --target universal-apple-darwin
```

## Gotchas

| Symptom | Cause |
| --- | --- |
| Build succeeds, bundle still ad-hoc | `APPLE_CERTIFICATE` set but `APPLE_CERTIFICATE_PASSWORD` wrong — import fails quietly. The workflow now asserts `TeamIdentifier` and fails. |
| Notarization rejected: "not signed with hardened runtime" | `hardenedRuntime` disabled, or a nested binary signed without it. |
| Signed build cleans nothing | Missing `com.apple.security.automation.apple-events`. Apple Events are denied, not errored. |
| First launch blocked offline, fine online | Ticket not stapled. Tauri staples automatically; check the notarize step ran. |
| Notarization hangs | Apple-side queue. Minutes normally, occasionally much longer. |
| "App is damaged and can't be opened" | Quarantined **and** the signature does not match the bytes — usually an app modified after signing. |

## Sources

- [Safely open apps on your Mac — Apple Support](https://support.apple.com/en-us/102445)
- [macOS Code Signing — Tauri v2](https://v2.tauri.app/distribute/sign/macos/)
- [macOS Sequoia removes the Control-click Gatekeeper bypass](https://www.idownloadblog.com/2024/08/07/apple-macos-sequoia-gatekeeper-change-install-unsigned-apps-mac/)
- [Apple forces the signing of applications in macOS Sequoia 15.1 — Hackaday](https://hackaday.com/2024/11/01/apple-forces-the-signing-of-applications-in-macos-sequoia-15-1/)
- [Allow downloaded apps to open in macOS Tahoe](https://swissmacuser.ch/fix-macos-tahoe-app-is-damaged-and-cant-be-opened-move-trash/)
