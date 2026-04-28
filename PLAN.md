# Xclense Implementation Plan (Next.js UI + Rust/Tauri Logic)

## 1) Product Direction

Xclense is a macOS-focused advanced system utility tool that helps users understand and optimize system health through storage analysis, process visibility, RAM monitoring, actionable reports, and timely notifications. The UI is owned and evolved in Next.js, while privileged and system-level logic is implemented in Rust through Tauri commands.

The architectural crux is separation of concerns: **UI should remain visualization-first and interaction-first**, while **Rust services own all filesystem/process/system introspection and control**.

---

## 2) Target Architecture

### 2.1 Frontend Layer (Next.js App Router)

- Keep existing `app/` structure as primary UI shell.
- Use components for dashboards, process tables, storage views, alerts, and report views.
- UI should call Tauri command endpoints only (no direct shell/process invocation from React).
- Keep domain-aligned frontend models for:
  - Storage summary
  - Process snapshot
  - System health metrics
  - Recommendation/report cards

### 2.2 Backend Layer (Tauri + Rust)

Add `src-tauri/` with command handlers and service modules:

- `commands/`:
  - `scan_storage`
  - `list_processes`
  - `get_system_health`
  - `analyze_issues`
  - `manage_process_action` (safe gated action)

- `services/`:
  - `storage_service`
  - `process_service`
  - `health_service`
  - `report_service`
  - `notification_service`

- `adapters/`:
  - `macos_disk_adapter` (migrate logic from shell script gradually)
  - `macos_process_adapter`
  - `macos_memory_adapter`

- `models/`:
  - shared response and error models
  - severity + recommendation enums

---

## 3) Implementation Phases

### Phase 0 — Baseline and Alignment

Goal: Establish repository direction and conventions for dual-stack development.

- Keep Next.js app as-is for UI ownership.
- Add project docs (this plan + README architecture rewrite).
- Define naming conventions for command payloads and response types.
- Define safety policy for destructive actions.

Deliverables:
- `PLAN.md`
- Updated `README.md`

### Phase 1 — Tauri Bootstrap and Command Bridge

Goal: Introduce Tauri app infrastructure without disrupting Next.js UI flow.

Tasks:
- Scaffold `src-tauri` with Rust entrypoints and command registration.
- Add a basic health/ping command from Rust to Next.js.
- Create typed command wrapper in frontend (`lib/tauri-client.ts`).

Deliverables:
- Working Rust command invocation from UI shell.
- Initial error handling contract between frontend and backend.

### Phase 2 — Read-Only Observability (Storage + Processes + RAM)

Goal: Provide core insights before any control actions.

Tasks:
- Implement storage scanner service in Rust.
  - Start by integrating/parsing existing shell script behavior where practical.
  - Gradually replace with native Rust system calls/crates.
- Implement process list snapshot:
  - pid, name, cpu%, mem%, state, parent process, risk tier.
- Implement RAM and pressure metrics endpoint.

Deliverables:
- UI-ready JSON contracts for all read-only system metrics.
- Stable backend commands with predictable error surfaces.

### Phase 3 — Analysis and Recommendation Engine

Goal: Convert raw metrics into meaningful, user-friendly decisions.

Tasks:
- Build `analyze_issues` pipeline:
  - High RAM pressure heuristics
  - Disk saturation heuristics
  - Suspicious long-running/idle-heavy process heuristics
- Generate recommendation objects with severity and confidence.
- Create report snapshots that can be stored/exported.

Deliverables:
- Structured report model
- Actionable recommendations surfaced to UI

### Phase 4 — Safe Control Actions

Goal: Enable process operations with strict safeguards.

Tasks:
- Implement safe action command (`manage_process_action`) with allow/deny policy.
- Require confirmation payload for risky operations.
- Add backend audit logging for control actions.

Deliverables:
- Controlled process stop/kill pathways
- Audit trail records for critical actions

### Phase 5 — Notifications, Hardening, and Release Readiness

Goal: Production-grade behavior for recurring system utility usage.

Tasks:
- Background periodic checks and notification triggers.
- Tune performance for frequent scans.
- Add test matrix (unit/integration for Rust services).
- Validate permissions and failure behavior under macOS constraints.
- Prepare packaging strategy for Tauri app builds.

Deliverables:
- Stable recurring monitoring
- Test and reliability baseline
- Packaging-ready app architecture

---

## 4) Data Contracts (Initial Draft)

- `StorageSummary`
  - total_bytes, used_bytes, free_bytes, used_percent
  - disks[], volumes[], hotspot_paths[]

- `ProcessInfo`
  - pid, name, user, cpu_percent, memory_percent, uptime_seconds, state, risk_level

- `SystemHealth`
  - memory_total, memory_used, memory_pressure, cpu_load, swap_used

- `IssueReport`
  - id, title, severity, confidence, evidence[], recommendation, suggested_action

- `ActionResult`
  - action, target_pid, status, message, performed_at, audit_id

---

## 5) Safety Framework

- Never allow blind process termination.
- Maintain protected process rules (system-critical denylist).
- Enforce explicit user confirmation for force actions.
- Record all critical actions with timestamp and context.
- Return transparent, explainable error messages.

---

## 6) Development Workflow (Logic-First + UI-Owned)

- UI design/UX flow remains user-owned.
- Backend capability tickets are implemented incrementally.
- Each backend command is considered complete only when:
  - Type-safe contract is stable
  - Error paths are handled
  - Response supports UI visualization needs

---

## 7) Immediate Next Build Slice Recommendation

Start with **Phase 1 + first part of Phase 2**:

1. Tauri bootstrap and command bridge
2. Storage summary command
3. Process list command
4. RAM health command

This sequence delivers early real data for your UI explorations while preserving a robust backend foundation.
