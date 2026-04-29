import { RotateCw, Wrench } from "lucide-react";

import { cn } from "@/lib/utils";

import { AnimatedCircularProgressBar } from "@/components/dashboard/magic/AnimatedCircularProgressBar";
import { formatGb, statusColorClass, statusTone } from "@/components/dashboard/shared";

interface SystemHealthCardProps {
  score: number;
  totalIssues: number;
  memoryUsedBytes: number;
  memoryFreeBytes: number;
  onReload: () => void;
  isReloading: boolean;
  className?: string;
}

export function SystemHealthCard({
  score,
  totalIssues,
  memoryUsedBytes,
  memoryFreeBytes,
  onReload,
  isReloading,
  className,
}: SystemHealthCardProps): React.JSX.Element {
  const recommendationText =
    totalIssues > 0
      ? `${totalIssues} issue(s) can be reviewed and fixed.`
      : "No urgent issues found. System is running clean.";

  return (
    <section className={cn("h-full rounded-xl border border-white/15 bg-white/5 p-2", className)}>
      <div className="flex items-start justify-between gap-2.5">
        <h2 className="text-xs font-semibold">Overall System Health</h2>
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

      <div className="mt-2 border-t border-white/10 pt-1">
        <p className="text-sm font-medium text-zinc-100">Recommendations</p>
        <p className="mt-0.5 text-[11px] text-zinc-300">{recommendationText}</p>

        <div className="mt-1.5 flex items-center justify-between gap-2">
          <p className="text-xs text-zinc-300">
            Memory snapshot: {formatGb(memoryUsedBytes)} used / {formatGb(memoryFreeBytes)} free
          </p>
          <button
            type="button"
            className="inline-flex items-center gap-1.5 rounded-md bg-amber-400 px-3 py-1 text-xs font-semibold text-amber-950 transition hover:bg-amber-300"
          >
            <Wrench className="h-3 w-3" />
            Fix
          </button>
        </div>
      </div>
    </section>
  );
}
