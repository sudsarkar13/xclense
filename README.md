# Xclense

Xclense is a macOS-focused advanced system utility application designed to help users manage storage, monitor applications/processes, understand RAM pressure, and maintain overall system health through actionable insights.

The project follows a dual-layer paradigm:

- **Next.js App Router** for the visualization and interaction layer (UI shell)
- **Rust + Tauri** for secure, performant system-level logic

This separation keeps UI iteration fast while centralizing all privileged operations and system introspection in a strongly typed backend.

## Install

Download the latest `.dmg` from [Releases](https://github.com/sudsarkar13/xclense/releases),
open it, and drag **Xclense** into Applications. macOS 11+, Apple Silicon and Intel.

### First launch: "Apple could not verify Xclense"

Xclense is free and open source, distributed outside the App Store, and **deliberately
not code-signed** — a Developer ID certificate costs $99/year, and this project has no
revenue to justify it. macOS therefore blocks the first launch:

> **"Xclense" Not Opened** — Apple could not verify "Xclense" is free of malware…
> **[ Done ] [ Move to Bin ]**

Nothing is wrong with the app. macOS is reporting that it cannot identify the publisher,
which is exactly what it should say about an app that has not paid to be identified.
Since macOS 15 Sequoia, **right-click ➔ Open no longer works** — use one of these:

**Terminal (one command):**

```bash
xattr -dr com.apple.quarantine /Applications/Xclense.app
```

**Or System Settings:** click **Done** on the warning, open **System Settings ➔
Privacy & Security**, scroll to Security, click **Open Anyway** next to the Xclense
message, authenticate, then launch again and confirm.

You only do this once per install. Updates delivered by Xclense's own updater are never
blocked, because Gatekeeper only checks apps downloaded by a browser.

> **Read the command before running it.** `xattr -dr com.apple.quarantine` tells macOS
> to stop vetting an app, and it is the same instruction malware distributors give their
> victims. It is safe here because you can read every line of this repository and build
> the app yourself — but the habit of checking is worth more than this app is.

If you would rather trust nothing: `yarn install && yarn tauri build` produces the same
bundle from source. See [Local Development](#local-development).

Details and the reasoning: [docs/macos-code-signing.md](docs/macos-code-signing.md).

### Installed, but not showing up in app search?

If Xclense is in `/Applications` but does not appear in Spotlight or the Applications
view, macOS is almost certainly resolving `com.xclense.app` to a *different* copy of the
bundle. This happens when another `Xclense.app` exists somewhere else — most often a
build output under `src-tauri/target/` on a machine that also develops Xclense.
LaunchServices keys apps by bundle identifier and prefers whichever copy ran most
recently, so the installed one gets shadowed.

Check which copy macOS resolves to:

```bash
osascript -e 'tell application "Finder" to get POSIX path of (application file id "com.xclense.app" as alias)'
mdfind "kMDItemFSName == 'Xclense.app'"        # every copy on disk
```

If it points anywhere other than `/Applications/Xclense.app`, point it back:

```bash
LSR=/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister
$LSR -u /path/to/the/other/Xclense.app     # unregister the shadowing copy
$LSR -f /Applications/Xclense.app          # re-register the installed one
```

Deleting the stray copy works just as well. This cannot happen on a machine that only
ever installed from the DMG — there is only one bundle to resolve to.

## Product Goals

Xclense is intended to provide:

- Storage scanning and disk usage breakdowns
- Process-level visibility (what is running and resource impact)
- RAM and system health diagnostics
- Recommendation reports with clear severity and evidence
- Safe process control workflows (with safeguards and auditability)
- Timely notifications for proactive maintenance

## Architecture Overview

## Frontend (Current)

- Framework: Next.js (App Router)
- UI Components: shadcn/ui-based component set
- Responsibility: render dashboards, reports, tables, and user actions

## Backend (Planned/In Progress)

- Runtime: Tauri
- Language: Rust
- Responsibility: filesystem/process/system analysis and controlled action execution

Planned Rust service domains:

- `storage_service`
- `process_service`
- `health_service`
- `report_service`
- `notification_service`

## Repository Layout (Current + Planned)

```text
xclense/
├── app/                    # Next.js App Router UI shell
├── components/             # UI components
├── hooks/                  # Frontend hooks
├── lib/                    # Frontend utilities
├── shellscript/            # Existing macOS scan scripts (transition reference)
├── plan.md                 # Product + technical execution roadmap
└── src-tauri/              # Planned Rust + Tauri backend layer
```

## Development Status

Current status is an early foundation stage:

- Next.js frontend scaffold exists
- Disk scan shell script exists at `shellscript/mac_disk_scanner.sh`
- Full Rust/Tauri backend integration is the active build direction

## Roadmap Snapshot

Detailed breakdown is maintained in [`plan.md`](./plan.md). High-level phases:

1. Baseline docs and architecture alignment
2. Tauri bootstrap and command bridge
3. Read-only observability (storage, process, RAM)
4. Analysis/reporting engine
5. Safe control actions
6. Notifications, hardening, and release readiness

## Local Development

Use `yarn` as package manager for this repository.

```bash
yarn install
yarn dev
```

The frontend shell runs on `http://localhost:3000` by default.

## Logic Integration Direction

As backend commands are introduced, the UI should consume only typed Tauri command responses for system data. Direct shell execution from React components is intentionally avoided to preserve safety, testability, and portability.

## Safety Principles

Because Xclense includes process management capabilities, backend logic will enforce safety controls:

- protected process restrictions
- explicit confirmation for high-risk actions
- auditable action logs
- transparent failure reasons for users

## Next Step

The immediate implementation target is to scaffold Tauri and expose the first three backend commands:

- storage summary
- process list snapshot
- RAM/system health snapshot

Once these are live, UI can be iterated rapidly against real runtime data.
