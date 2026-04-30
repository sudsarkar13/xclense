import { CircleHelp, RotateCw, Wrench } from "lucide-react";

import { cn } from "@/lib/utils";

import { AnimatedCircularProgressBar } from "@/components/dashboard/magic/AnimatedCircularProgressBar";
import { formatGb, statusColorClass, statusTone } from "@/components/dashboard/shared";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "@/components/ui/hover-card";

interface SystemHealthCardProps {
  score: number;
  totalIssues: number;
  memoryTotalBytes: number;
  memoryUsedBytes: number;
  memoryFreeBytes: number;
  memoryPressurePercent: number;
  loadAverage1m: number;
  loadAverage5m: number;
  loadAverage15m: number;
  lastCheckpointEpochMs: number;
  criticalCount: number;
  warningCount: number;
  infoCount: number;
  autoRefreshSeconds: number;
  onReload: () => void;
  isReloading: boolean;
  className?: string;
}

export function SystemHealthCard({
  score,
  totalIssues,
  memoryTotalBytes,
  memoryUsedBytes,
  memoryFreeBytes,
  memoryPressurePercent,
  loadAverage1m,
  loadAverage5m,
  loadAverage15m,
  lastCheckpointEpochMs,
  criticalCount,
  warningCount,
  infoCount,
  autoRefreshSeconds,
  onReload,
  isReloading,
  className,
}: SystemHealthCardProps): React.JSX.Element {
  const recommendationText =
    totalIssues > 0
      ? `${totalIssues} issue(s) can be reviewed and fixed.`
      : "No urgent issues found. System is running clean.";

  const checkpointTime = new Date(lastCheckpointEpochMs).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });

  const pressureLabel = `${Math.max(0, Math.min(100, memoryPressurePercent)).toFixed(0)}%`;

  return (
    <section className={cn("flex h-full flex-col rounded-xl border border-white/15 bg-white/5 p-2", className)}>
      <div className="flex items-start justify-between gap-2.5">
        <div className="inline-flex items-center gap-1.5">
          <h2 className="text-xs font-semibold">Overall System Health</h2>
          <HoverCard openDelay={120} closeDelay={80}>
            <HoverCardTrigger asChild>
              <button
                type="button"
                className="inline-flex h-5 w-5 items-center justify-center rounded-full border border-white/20 bg-white/5 text-zinc-300 transition hover:bg-white/10 hover:text-zinc-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-300/60"
                aria-label="What this system health card shows"
              >
                <CircleHelp className="h-3.5 w-3.5" />
              </button>
            </HoverCardTrigger>
            <HoverCardContent align="start" className="w-72 border border-white/15 bg-zinc-900/95 text-zinc-100">
              <p className="text-xs font-semibold">Overall System Health</p>
              <p className="mt-1 text-[11px] leading-relaxed text-zinc-300">
                This widget combines issue severity, memory pressure, and system load to estimate your Mac&apos;s health score.
                Use it as a quick status snapshot before drilling into individual alerts.
              </p>
            </HoverCardContent>
          </HoverCard>
        </div>
        <button
          type="button"
          onClick={onReload}
          disabled={isReloading}
          className="inline-flex h-7 w-7 items-center justify-center rounded-md border border-white/15 bg-white/5 text-zinc-300 transition hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-50"
          aria-label="Reload overall system health"
          title="Reload"
        >
          <RotateCw className={cn("h-4 w-4", isReloading && "animate-spin")} />
        </button>
      </div>

      <div className="mt-2 flex items-center gap-2">
        <AnimatedCircularProgressBar
          value={score}
          gaugePrimaryColor="rgb(6 182 212)"
          gaugeSecondaryColor="rgba(148, 163, 184, 0.35)"
          className="w-20 shrink-0 text-xl"
        />
        <div>
          <p className={cn("text-2xl font-semibold leading-none", statusColorClass(score))}>{statusTone(score)}</p>
          <p className="mt-1 text-xs text-zinc-300">Your Mac is running smoothly.</p>
        </div>
      </div>

      <div className="mt-2 flex flex-1 flex-col border-t border-white/10 pt-1">
        <p className="text-xs font-medium text-zinc-100">Recommendations</p>
        <p className="mt-0.5 text-[11px] text-zinc-300">{recommendationText}</p>

        <div className="mt-1.5 grid grid-cols-2 gap-x-3 gap-y-1 text-[10px] leading-tight text-zinc-300">
          <p>
            <span className="text-zinc-400">Load:</span> {loadAverage1m.toFixed(2)} / {loadAverage5m.toFixed(2)} / {loadAverage15m.toFixed(2)}
          </p>
          <p>
            <span className="text-zinc-400">Scan:</span> {checkpointTime}
          </p>
          <p>
            <span className="text-zinc-400">Mem:</span> {formatGb(memoryTotalBytes)} total
          </p>
          <p>
            <span className="text-zinc-400">Pressure:</span> {pressureLabel}
          </p>
          <p>
            <span className="text-zinc-400">Used/Free:</span> {formatGb(memoryUsedBytes)} / {formatGb(memoryFreeBytes)}
          </p>
          <p>
            <span className="text-zinc-400">Mode:</span> Manual + {autoRefreshSeconds}s live
          </p>
        </div>

        <div className="mt-auto flex items-center justify-between gap-2 pt-1">
          <div className="flex flex-wrap items-center gap-1 text-[10px]">
            <span
              className="rounded-full border border-red-400/35 bg-red-500/10 px-1.5 py-0.5 text-red-200"
              title={`C = Critical issues. ${criticalCount} critical issue(s) need immediate attention.`}
              aria-label={`Critical issues: ${criticalCount}`}
            >
              C {criticalCount}
            </span>
            <span
              className="rounded-full border border-amber-400/35 bg-amber-500/10 px-1.5 py-0.5 text-amber-200"
              title={`W = Warning issues. ${warningCount} warning issue(s) should be reviewed soon.`}
              aria-label={`Warning issues: ${warningCount}`}
            >
              W {warningCount}
            </span>
            <span
              className="rounded-full border border-cyan-400/35 bg-cyan-500/10 px-1.5 py-0.5 text-cyan-200"
              title={`I = Informational findings. ${infoCount} informational item(s) are for awareness.`}
              aria-label={`Informational issues: ${infoCount}`}
            >
              I {infoCount}
            </span>
          </div>

          <button
            type="button"
            className="inline-flex items-center gap-1.5 rounded-md bg-amber-400 px-3 py-1 text-xs font-semibold text-amber-950 transition hover:bg-amber-300"
            title="Open recommended quick fixes"
            aria-label="Open recommended quick fixes"
          >
            <Wrench className="h-3 w-3" />
            Fix
          </button>
        </div>

        <p className="mt-1 text-[10px] text-zinc-400">
          Hover C/W/I badges to see abbreviation meaning and severity context.
        </p>
      </div>
    </section>
  );
}
