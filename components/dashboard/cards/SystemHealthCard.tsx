import { cn } from "@/lib/utils";

import { AnimatedCircularProgressBar } from "@/components/dashboard/magic/AnimatedCircularProgressBar";
import { formatGb, statusColorClass, statusTone } from "@/components/dashboard/shared";

interface SystemHealthCardProps {
  score: number;
  totalIssues: number;
  memoryUsedBytes: number;
  memoryFreeBytes: number;
  className?: string;
}

export function SystemHealthCard({
  score,
  totalIssues,
  memoryUsedBytes,
  memoryFreeBytes,
  className,
}: SystemHealthCardProps): React.JSX.Element {
  return (
    <section className={cn("h-full rounded-xl border border-white/15 bg-white/5 p-4", className)}>
      <h2 className="text-xl font-semibold">Overall System Health</h2>
      <div className="mt-4 flex items-center gap-4">
        <AnimatedCircularProgressBar
          value={score}
          gaugePrimaryColor="rgb(6 182 212)"
          gaugeSecondaryColor="rgba(148, 163, 184, 0.35)"
          className="shrink-0"
        />
        <div>
          <p className={`text-2xl font-semibold ${statusColorClass(score)}`}>{statusTone(score)}</p>
          <p className="mt-1 text-sm text-zinc-300">Mac stability appears normal in this cycle.</p>
        </div>
      </div>

      <div className="mt-4 border-t border-white/10 pt-3 text-sm text-zinc-300">
        <p>{totalIssues > 0 ? `${totalIssues} issue(s) need attention.` : "No urgent issues found."}</p>
        <p className="mt-1 text-xs text-zinc-400">
          Memory snapshot: {formatGb(memoryUsedBytes)} used / {formatGb(memoryFreeBytes)} free
        </p>
      </div>
    </section>
  );
}
