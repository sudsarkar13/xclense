import { cn } from "@/lib/utils";

import { formatGb } from "@/components/dashboard/shared";

interface MemoryPressureCardProps {
  pressurePercent: number;
  memoryUsedBytes: number;
  memoryFreeBytes: number;
  className?: string;
}

export function MemoryPressureCard({
  pressurePercent,
  memoryUsedBytes,
  memoryFreeBytes,
  className,
}: MemoryPressureCardProps): React.JSX.Element {
  const clamped = Math.max(0, Math.min(100, pressurePercent));

  return (
    <section className={cn("h-full rounded-xl border border-white/15 bg-white/5 p-4", className)}>
      <h2 className="text-xl font-semibold">RAM Pressure</h2>
      <div className="mt-4 text-center">
        <p className="text-4xl font-bold">{clamped.toFixed(0)}%</p>
        <p className="mt-1 text-sm text-emerald-300">{clamped < 80 ? "Good" : "High"}</p>
      </div>
      <p className="mt-4 text-sm text-zinc-300">Available: {formatGb(memoryFreeBytes)} · Used: {formatGb(memoryUsedBytes)}</p>
      <div className="mt-3 h-3 overflow-hidden rounded-full bg-white/15">
        <div
          className="h-full rounded-full bg-gradient-to-r from-emerald-500 via-cyan-400 to-blue-500 transition-all duration-500"
          style={{ width: `${clamped}%` }}
        />
      </div>
    </section>
  );
}
