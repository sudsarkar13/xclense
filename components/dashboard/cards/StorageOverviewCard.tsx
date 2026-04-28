import { cn } from "@/lib/utils";

import { formatGb } from "@/components/dashboard/shared";

interface StorageOverviewCardProps {
  totalBytes: number;
  freeBytes: number;
  usedPercent: number;
  className?: string;
}

export function StorageOverviewCard({
  totalBytes,
  freeBytes,
  usedPercent,
  className,
}: StorageOverviewCardProps): React.JSX.Element {
  const clamped = Math.max(0, Math.min(100, usedPercent));

  return (
    <section className={cn("h-full rounded-xl border border-white/15 bg-white/5 p-4", className)}>
      <h2 className="text-xl font-semibold">Storage Overview</h2>
      <div className="mt-4 grid grid-cols-2 gap-2 text-sm">
        <p className="text-zinc-300">{formatGb(totalBytes)} Total</p>
        <p className="text-right text-zinc-300">{formatGb(freeBytes)} Free</p>
      </div>
      <div className="mt-3 h-3 overflow-hidden rounded-full bg-white/15">
        <div
          className="h-full rounded-full bg-gradient-to-r from-sky-500 via-violet-500 to-emerald-400 transition-all duration-500"
          style={{ width: `${Math.max(8, clamped)}%` }}
        />
      </div>
      <div className="mt-3 text-sm text-zinc-300">Used: {clamped.toFixed(1)}%</div>
    </section>
  );
}
