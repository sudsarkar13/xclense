import { Search, Eye } from "lucide-react";

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
  const usedBytes = totalBytes - freeBytes;

  const applicationPercent = Math.max(8, clamped * 0.45);
  const systemPercent = Math.max(6, clamped * 0.28);
  const photosPercent = Math.max(4, clamped * 0.15);
  const otherPercent = Math.max(3, clamped * 0.12);

  return (
    <section className={cn("h-full rounded-xl border border-white/15 bg-white/5 p-3", className)}>
      <h2 className="text-lg font-semibold">Storage Overview</h2>

      <div className="mt-3 flex items-center justify-between text-xs">
        <p className="text-zinc-300">{formatGb(totalBytes)} Total</p>
        <p className="text-zinc-300">{formatGb(freeBytes)} Free</p>
      </div>

      <div className="mt-2 flex h-6 overflow-hidden rounded-md bg-white/10">
        <div className="bg-sky-500" style={{ width: `${applicationPercent}%` }} />
        <div className="bg-violet-500" style={{ width: `${systemPercent}%` }} />
        <div className="bg-emerald-500" style={{ width: `${photosPercent}%` }} />
        <div className="bg-amber-400" style={{ width: `${otherPercent}%` }} />
      </div>

      <div className="mt-2 grid grid-cols-2 gap-x-2 gap-y-1.5 text-xs text-zinc-300">
        <p className="inline-flex items-center gap-1.5"><span className="h-2.5 w-2.5 rounded-full bg-sky-500" />Applications</p>
        <p className="inline-flex items-center gap-1.5"><span className="h-2.5 w-2.5 rounded-full bg-violet-500" />System Data</p>
        <p className="inline-flex items-center gap-1.5"><span className="h-2.5 w-2.5 rounded-full bg-emerald-500" />Photos</p>
        <p className="inline-flex items-center gap-1.5"><span className="h-2.5 w-2.5 rounded-full bg-amber-400" />Other</p>
      </div>

      <div className="mt-2 border-t border-white/10 pt-2">
        <div className="flex items-center justify-between">
          <p className="text-xs text-zinc-200">Duplicates: {formatGb(usedBytes * 0.08)}</p>
          <button
            type="button"
            className="inline-flex items-center gap-1 rounded-md border border-white/15 bg-white/10 px-2.5 py-0.5 text-xs font-medium text-zinc-100 transition hover:bg-white/15"
          >
            <Search className="h-3 w-3" />
            Scan
          </button>
        </div>

        <div className="mt-2 flex items-center justify-between border-t border-white/10 pt-2">
          <p className="text-xs text-zinc-200">Large Files: {formatGb(usedBytes * 0.18)}</p>
          <button
            type="button"
            className="inline-flex items-center gap-1 rounded-md border border-white/15 bg-white/10 px-2.5 py-0.5 text-xs font-medium text-zinc-100 transition hover:bg-white/15"
          >
            <Eye className="h-3 w-3" />
            View
          </button>
        </div>
      </div>
    </section>
  );
}
