# Xclense

Xclense is a macOS-focused advanced system utility application designed to help users manage storage, monitor applications/processes, understand RAM pressure, and maintain overall system health through actionable insights.

The project follows a dual-layer paradigm:

- **Next.js App Router** for the visualization and interaction layer (UI shell)
- **Rust + Tauri** for secure, performant system-level logic

This separation keeps UI iteration fast while centralizing all privileged operations and system introspection in a strongly typed backend.

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