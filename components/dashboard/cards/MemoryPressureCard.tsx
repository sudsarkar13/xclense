import { cn } from "@/lib/utils";

import { formatGb } from "@/components/dashboard/shared";

interface MemoryPressureCardProps {
  pressurePercent: number;
  memoryUsedBytes: number;
  memoryFreeBytes: number;
  className?: string;
}

const SERIES = [35, 33, 32, 32, 31, 31, 33, 34, 33, 32, 35, 38, 36, 39, 44, 48, 46, 43, 45, 47, 52, 55, 49, 46, 44, 48, 50, 51, 49, 47];

export function MemoryPressureCard({
  pressurePercent,
  memoryUsedBytes,
  memoryFreeBytes,
  className,
}: MemoryPressureCardProps): React.JSX.Element {
  const clamped = Math.max(0, Math.min(100, pressurePercent));
  const sweep = 220;
  const start = 160;
  const pointer = start + (clamped / 100) * sweep;

  const chartPoints = SERIES.map((value, index) => {
    const x = (index / (SERIES.length - 1)) * 100;
    const y = 100 - value;
    return `${x},${y}`;
  }).join(" ");

  return (
    <section className={cn("h-full rounded-xl border border-white/15 bg-white/5 p-4", className)}>
      <h2 className="text-xl font-semibold">RAM Usage</h2>

      <div className="mt-4 flex flex-col items-center">
        <div className="relative h-36 w-64">
          <svg viewBox="0 0 260 150" className="h-full w-full">
            <defs>
              <linearGradient id="ram-good" x1="0" x2="1">
                <stop offset="0%" stopColor="#22c55e" />
                <stop offset="100%" stopColor="#84cc16" />
              </linearGradient>
              <linearGradient id="ram-mid" x1="0" x2="1">
                <stop offset="0%" stopColor="#eab308" />
                <stop offset="100%" stopColor="#fb923c" />
              </linearGradient>
              <linearGradient id="ram-high" x1="0" x2="1">
                <stop offset="0%" stopColor="#f97316" />
                <stop offset="100%" stopColor="#f43f5e" />
              </linearGradient>
            </defs>

            <path d="M30 120 A100 100 0 0 1 95 30" stroke="url(#ram-good)" strokeWidth="20" fill="none" strokeLinecap="round" />
            <path d="M95 30 A100 100 0 0 1 165 30" stroke="url(#ram-mid)" strokeWidth="20" fill="none" strokeLinecap="round" />
            <path d="M165 30 A100 100 0 0 1 230 120" stroke="url(#ram-high)" strokeWidth="20" fill="none" strokeLinecap="round" />

            <g transform={`rotate(${pointer} 130 120)`}>
              <rect x="126" y="26" width="8" height="96" rx="4" fill="#e5e7eb" />
            </g>
            <circle cx="130" cy="120" r="8" fill="#e5e7eb" />
          </svg>
        </div>

        <p className="-mt-1 text-4xl font-semibold leading-none">{clamped.toFixed(0)}%</p>
        <p className="mt-1 text-lg text-emerald-300">{clamped < 80 ? "(Good)" : "(High)"}</p>
        <p className="mt-1 text-sm text-zinc-300">
          Available: {formatGb(memoryFreeBytes)}, Used: {formatGb(memoryUsedBytes)}
        </p>
      </div>

      <div className="mt-4 rounded-md border border-white/10 bg-cyan-500/5 p-2">
        <svg viewBox="0 0 100 100" className="h-16 w-full" preserveAspectRatio="none">
          <polyline fill="none" stroke="#34d399" strokeWidth="2.5" points={chartPoints} />
          <polygon points={`0,100 ${chartPoints} 100,100`} fill="rgba(52, 211, 153, 0.15)" />
        </svg>
        <div className="mt-1 flex items-center justify-between text-xs text-zinc-400">
          <span>Last</span>
          <span>30 minutes</span>
        </div>
      </div>
    </section>
  );
}
