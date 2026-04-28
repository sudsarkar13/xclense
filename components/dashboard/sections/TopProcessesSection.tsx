import { processBarClass } from "@/components/dashboard/shared";
import { type DashboardProcess } from "@/components/dashboard/types";

interface TopProcessesSectionProps {
  processes: DashboardProcess[];
}

export function TopProcessesSection({ processes }: TopProcessesSectionProps): React.JSX.Element {
  return (
    <section className="rounded-xl border border-white/15 bg-white/5 p-4">
      <h2 className="text-xl font-semibold">Top Resource Consuming Apps</h2>
      <p className="text-xs text-zinc-400">by CPU &amp; Memory</p>

      <div className="mt-3 space-y-2.5">
        {processes.length === 0 ? (
          <p className="text-sm text-zinc-300">No process data available for this cycle.</p>
        ) : (
          processes.slice(0, 4).map((process, index) => (
            <div key={`${process.pid}-${process.name}`} className="rounded-lg border border-white/10 p-2.5">
              <div className="grid grid-cols-12 items-center gap-2 text-xs">
                <div className="col-span-5 truncate font-medium">{process.name}</div>
                <div className="col-span-3 h-2.5 rounded-full bg-white/10">
                  <div
                    className={`h-full rounded-full bg-gradient-to-r ${processBarClass(index)} transition-all duration-500`}
                    style={{ width: `${Math.min(100, Math.max(10, process.cpuPercent * 2))}%` }}
                  />
                </div>
                <div className="col-span-2 text-zinc-300">CPU {process.cpuPercent.toFixed(1)}%</div>
                <div className="col-span-2 text-zinc-300">MEM {process.memoryPercent.toFixed(1)}%</div>
              </div>
            </div>
          ))
        )}
      </div>
    </section>
  );
}
