"use client";

import { CircleHelp } from "lucide-react";

import { cn } from "@/lib/utils";

import { processBarClass } from "@/components/dashboard/shared";
import { type DashboardProcess } from "@/components/dashboard/types";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "@/components/ui/hover-card";

interface TopProcessesSectionProps {
  processes: DashboardProcess[];
  className?: string;
}

export function TopProcessesSection({ processes, className }: TopProcessesSectionProps): React.JSX.Element {
  return (
    <section className={cn("h-full rounded-xl border border-white/15 bg-white/5 p-4", className)}>
      <div className="inline-flex items-center gap-1.5">
        <h2 className="text-xs font-semibold">Top Resource Consuming Apps</h2>
        <HoverCard openDelay={120} closeDelay={80}>
          <HoverCardTrigger asChild>
            <button
              type="button"
              className="inline-flex h-5 w-5 items-center justify-center rounded-full border border-white/20 bg-white/5 text-zinc-300 transition hover:bg-white/10 hover:text-zinc-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-300/60"
              aria-label="What this top resource consuming apps section shows"
            >
              <CircleHelp className="h-3.5 w-3.5" />
            </button>
          </HoverCardTrigger>
          <HoverCardContent align="start" className="w-72 border border-white/15 bg-zinc-900/95 text-zinc-100">
            <p className="text-xs font-semibold">Top Resource Consuming Apps</p>
            <p className="mt-1 text-[11px] leading-relaxed text-zinc-300">
              This section ranks currently active apps by CPU and memory usage so you can quickly identify heavy
              processes and close or optimize them if system performance drops.
            </p>
          </HoverCardContent>
        </HoverCard>
      </div>
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
