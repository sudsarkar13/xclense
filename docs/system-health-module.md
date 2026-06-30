# System Health Module — Plan & Specification

Status: **Plan / Spec only** (no implementation yet)
Scope: [components/dashboard/cards/SystemHealthCard.tsx](components/dashboard/cards/SystemHealthCard.tsx), related cards, and supporting data flow.

---

## 1. What the module reflects (meaning)

The **Overall System Health** card is the dashboard's top-level status surface. It is meant to answer one user question fast: _"Is my Mac okay right now, and if not, what should I do?"_

It does **not** show raw telemetry — that is the job of the sibling cards (RAM Usage, Storage Overview). Its role is to **synthesise** signals into a single, explainable verdict:

| Signal it combines            | Source                                           | Meaning                                              |
| ----------------------------- | ------------------------------------------------ | ---------------------------------------------------- |
| Issue severity counts         | `AnalysisReport.issues`                          | How many actionable problems exist right now         |
| Memory pressure               | `SystemHealth.memoryPressurePercent`             | Whether RAM is under sustained pressure              |
| Load averages (1m / 5m / 15m) | `SystemHealth.loadAverage*`                      | Whether the system is CPU-saturated (vs. just spiky) |
| Available memory              | `SystemHealth.memoryUsedBytes / memoryFreeBytes` | Hard floor for new apps / updates                    |
| Last checkpoint time          | `SystemHealth.scannedAtEpochMs`                  | How fresh the verdict is                             |

### What the verdict is _not_

- It is **not** a Windows-style "% health number" with hidden magic. The current score (`100 - critical*25 - warning*10`, clamped to 20–98) is a **placeholder heuristic**, not a model.
- It is **not** a duplicate of the RAM card — RAM is one input, not the whole thing.
- It is **not** a notification engine. The card surfaces a status; the Health page handles deep actions.

---

## 2. Current problems (audit)

Found by reading the existing code:

1. **Health score is fake / opaque.**
   `app/page.tsx:152-160` computes `100 - critical*25 - warning*10`, ignoring memory pressure and load entirely. The 20–98 clamp masks the inaccuracy.
2. **"Healthy / Moderate / Critical" is hard-coded against the same broken score** — see [shared.tsx:3-13](components/dashboard/shared.tsx#L3-L13).
3. **The `Fix` button does nothing.** No `onClick`, no destination, no intent. It is purely decorative. (SystemHealthCard.tsx, `Wrench` button block.)
4. **Hover badges C/W/I are unmemorable.**
   - They use `title` attributes only — no visible key, no consistent placement.
   - They show counts but not _what kind_ of issue, so a user seeing "C 2" has no clue whether that's a runaway process or a full disk.
   - They share a row with the decorative `Fix` button, so users cannot tell which is a count vs. a call to action.
5. **The big-circle "Healthy" tone never matches reality.**
   `statusTone(score)` always returns one of three strings, but `MemoryPressureCard` uses its own `pressureTone` ("Low / Moderate / High") and `StorageOverviewCard` has no tone at all — three different vocabularies for the same idea across the page.
6. **"Recommendations" text is generic.**
   `${totalIssues} issue(s) can be reviewed and fixed.` does not name the top issue or its severity, so it does not actually recommend anything.
7. **"Mode: Manual + 3s live"** is leaked implementation detail that adds no user value.
8. **`lastCheckpointEpochMs` is the health scan time, not the most recent issue time** — the "Scan:" label is technically correct but ambiguous.

---

## 3. Goals for the redesign

1. **Explainable score.** Every point in the score must trace back to a named input. The score itself can stay 0–100, but the breakdown must be visible on hover.
2. **Actionable `Fix` button.** When pressed, it must do one of:
   - Open the issue with the highest severity in the Issue Logs section (in-page anchor), or
   - Open the `/health` page filtered to that issue, or
   - Trigger a concrete remediation command via a new Tauri command (see §6).
3. **Meaningful badges.** Replace "C / W / I" with badges that carry the _actual_ issue title, severity color, and a click-to-jump action.
4. **Consistent status vocabulary** across the dashboard (`Healthy / Watch / Action needed`).
5. **Honest empty state.** When there is no checkpoint yet, say so explicitly instead of `0 issues can be reviewed`.
6. **Accessibility.** Replace `title=` tooltips with the existing `HoverCard` component (already used for the `?` icon) so screen readers and keyboard users get the same info.

---

## 4. Proposed score model (transparent)

Replace the current formula with a **weighted, capped, explainable** model. Inputs and weights:

```text
score = 100
  - 25 * min(criticalCount, 4)              # cap at 4 to avoid runaway penalties
  - 10 * min(warningCount, 6)               # cap at 6
  - pressurePenalty(pressurePercent)        # 0..25, see below
  - loadPenalty(loadAverage1m)               # 0..15, see below
  + clamp(score, 0, 100)
```

`pressurePenalty(p)`:

| Pressure | Penalty                              |
| -------- | ------------------------------------ |
| < 60%    | 0                                    |
| 60–79%   | `p - 60` (0..19)                     |
| ≥ 80%    | 20 + `(p - 80) / 2` (20..25, capped) |

`loadPenalty(load1m)` (per-core relative; assume logical cores `n`):

| Effective load (`load1m / n`) | Penalty                             |
| ----------------------------- | ----------------------------------- |
| < 0.7                         | 0                                   |
| 0.7–1.4                       | linear 0→10                         |
| ≥ 1.4                         | 10 + 2.5 per 1.0 over, capped at 15 |

Each component is exposed to the UI so the breakdown can be rendered on hover:

```ts
type ScoreBreakdown = {
  base: 100;
  criticalPenalty: number;
  warningPenalty: number;
  pressurePenalty: number;
  loadPenalty: number;
  final: number; // 0..100
  tone: "Healthy" | "Watch" | "Action needed";
};
```

`tone` thresholds: `>= 80 Healthy`, `60..79 Watch`, `< 60 Action needed`. Replace `statusTone` / `statusColorClass` in [shared.tsx](components/dashboard/shared.tsx) with a single function that returns both label and class, so every card speaks the same language.

---

## 5. Badge redesign (C/W/I → real badges)

Replace the three C/W/I pills with **one pill per issue category actually present in the latest report**, capped at 3 visible + "+N more":

- Each pill shows: `SeverityIcon  Issue category (count)`
  - Example: `🔴  Storage full (2)`, `🟡  Memory pressure (1)`, `🔵  Background noise (3)`
- Categories are derived from `IssueReport.id` / `title` prefix in the analysis service (Rust side — see §6.2).
- Hovering a pill opens a `HoverCard` listing the issue titles in that category.
- Clicking a pill jumps to that issue's row in the Issue Logs section (anchor link, not a new page).

Implementation sketch in [SystemHealthCard.tsx](components/dashboard/cards/SystemHealthCard.tsx):

```tsx
<button onClick={() => scrollToIssue(firstIssueId)}>
  <SeverityDot severity="critical" />
  <span>{label}</span>
  <span className="ml-1 opacity-70">{count}</span>
</button>
```

Remove the inline `title=` attributes. Keep the `Fix` button, but make it:

- Disabled when `totalIssues === 0`.
- Labelled with the top issue's title, e.g. `Fix top issue: "Storage 92% full"`.
- Wired to either scroll to the issue or invoke the new remediation command.

---

## 6. Required supporting changes

### 6.1 Frontend — types & shared helpers

[components/dashboard/types.ts](components/dashboard/types.ts)

```ts
export interface IssueCategory {
  id: string; // e.g. "storage_full", "memory_pressure"
  label: string; // e.g. "Storage full"
  severity: SeverityLevel;
  count: number;
  firstIssueId: string; // for scroll-to behaviour
}

export interface HealthScoreBreakdown {
  base: number;
  criticalPenalty: number;
  warningPenalty: number;
  pressurePenalty: number;
  loadPenalty: number;
  final: number;
  tone: "Healthy" | "Watch" | "Action needed";
}
```

[components/dashboard/shared.tsx](components/dashboard/shared.tsx)

- Replace `statusTone` / `statusColorClass` with one `healthTone(score): { label, className }` so all cards agree.
- Add `pressurePenalty()`, `loadPenalty()`, `computeHealthScore()` — pure functions, easy to unit-test.

### 6.2 Rust — issue categorisation

In [src-tauri/src/lib.rs](src-tauri/src/lib.rs) the `IssueReport` already has `id`, `title`, `severity`. Extend `analyze_issues` so the returned `AnalysisReport` also includes:

```rust
pub struct AnalysisReport {
  pub generated_at_epoch_ms: u128,
  pub total_issues: usize,
  pub issues: Vec<IssueReport>,
  pub categories: Vec<IssueCategory>,   // NEW
}

pub struct IssueCategory {
  pub id: String,
  pub label: String,
  pub severity: String,   // highest severity in this category
  pub count: usize,
  pub first_issue_id: String,
}
```

Rule of thumb for categorisation (heuristic, can grow):

| Title/ID substring                     | Category id       | Label           |
| -------------------------------------- | ----------------- | --------------- |
| `storage` / `disk` / `apfs snapshot`   | `storage_full`    | Storage full    |
| `memory` / `ram` / `swap` / `pressure` | `memory_pressure` | Memory pressure |
| `cpu` / `load average`                 | `cpu_load`        | CPU overload    |
| `zombie` / `orphan`                    | `process_zombie`  | Stuck processes |
| everything else                        | `other`           | Other           |

This keeps the **frontend dumb** (it just renders categories) and the **backend authoritative** (it decides what is "storage" vs. "memory").

### 6.3 New remediation command (optional but planned)

The current `Fix` button is decorative. A real "Fix" needs a backend command:

```rust
#[tauri::command]
async fn remediate_top_issue(report: AnalysisReport) -> Result<RemediationResult, String>
```

That returns one of:

- `ActionPerformed { audit_id }` — for safe, reversible fixes (e.g. `tmutil deletelocalsnapshots /`)
- `ConfirmationRequired { token, reason }` — for anything risky (e.g. killing a process), which then routes the user into the existing Health-page confirmation dialog
- `NotApplicable` — when there is nothing safe to auto-fix

This reuses the existing audit framework in [lib.rs](src-tauri/src/lib.rs) and is in line with **Phase 4 (Safe Control Actions)** in [PLAN.md](PLAN.md).

### 6.4 `app/page.tsx`

- Compute `healthScoreBreakdown` from the same inputs (memo).
- Pass `categories` from `AnalysisReport` into `SystemHealthCard`.
- Wire `Fix` button → either scroll to first issue (`document.getElementById(issueId)?.scrollIntoView()`) or call `remediate_top_issue` via the new tauri-client wrapper.
- Drop the "Mode: Manual + 3s live" line — it is internal noise.

---

## 7. Component prop changes (SystemHealthCard)

Replace the long prop list with two structured objects so the API stays sane as the score model grows:

```ts
interface SystemHealthCardProps {
  health: SystemHealth; // backend payload, unchanged
  issues: {
    total: number;
    critical: number;
    warning: number;
    info: number;
    categories: IssueCategory[]; // NEW
    topIssueId?: string; // NEW
  };
  score: HealthScoreBreakdown; // NEW: replaces `score: number`
  autoRefreshSeconds: number;
  onReload: () => void;
  isReloading: boolean;
  onFixTopIssue?: () => void; // NEW: wired by page
  className?: string;
}
```

This drops `score`, `totalIssues`, `criticalCount`, `warningCount`, `infoCount`, `lastCheckpointEpochMs` as flat props and groups them — easier to extend, less prop drilling.

---

## 8. Visual layout (proposed)

```text
┌─ Overall System Health ────── [? hover] ────── [⟳ reload] ─┐
│                                                             │
│   ╭───────╮   Healthy                                      │
│   │  82   │   Memory is comfortable. 1 warning to review.   │
│   │   %   │                                                │
│   ╰───────╯                                                │
│                                                             │
│ ────────────────────────────────────────────────────────── │
│ Recommendations                                             │
│ • Top: "APFS snapshots eating 14 GB" (warning)              │
│ • Memory: Low pressure                                      │
│                                                             │
│ Issues: [🟡 Memory pressure (1)] [🔵 Snapshots (3)] +2 more│
│                                                             │
│                       [ 🔧 Fix top: APFS snapshots ]       │
└─────────────────────────────────────────────────────────────┘
```

- Hovering the circular % reveals the score breakdown table (`base - penalties = final`).
- Hovering any issue badge lists the issue titles in that category.
- `Fix` is the only call-to-action in the row; badges are click-to-scroll.

---

## 9. Out of scope (explicit non-goals)

- Replacing the underlying Rust analysis engine. Only the categorisation output and (optionally) a `remediate_top_issue` command are added.
- Changing the RAM and Storage cards beyond vocabulary alignment.
- Adding notifications / alerts. That belongs to a separate module per [PLAN.md §3 Phase 5](PLAN.md).
- Redesigning the dashboard grid.

---

## 10. Build order

1. **Backend (Rust):** add `categories` to `AnalysisReport`; wire categorisation heuristic in `analyze_issues`.
2. **Shared helpers:** replace `statusTone` / `statusColorClass` with `healthTone`; add score breakdown helpers.
3. **Types:** add `IssueCategory`, `HealthScoreBreakdown` in [types.ts](components/dashboard/types.ts).
4. **SystemHealthCard:** new prop shape, breakdown hover card, category badges, real `Fix` handler, consistent tone vocabulary.
5. **page.tsx:** compute breakdown, pass categories, wire `Fix`.
6. **Verification:**
   - Unit-test the score helpers (pure functions, trivial to cover).
   - Manual: zero issues, 1 critical, 1 warning + memory at 75%, load 2.0 — confirm tone and breakdown.
7. **(Stretch)** Add `remediate_top_issue` Rust command and a thin `lib/tauri-client.ts` wrapper.

---

## 11. Acceptance criteria

- [ ] Hovering the big circular score shows a table of every penalty component and the final number.
- [ ] Issue badges show category labels and counts, not single letters; hovering lists the issues.
- [ ] `Fix` button is disabled when no issues exist; otherwise labelled with the top issue's title and actually navigates or invokes remediation.
- [ ] `Healthy / Watch / Action needed` is the only status vocabulary used on the dashboard.
- [ ] Score breakdown is unit-tested; tone mapping is unit-tested.
- [ ] No `title=` attribute remains on the System Health card.
