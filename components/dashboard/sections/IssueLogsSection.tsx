import { Activity, CircleHelp } from "lucide-react";

import { cn } from "@/lib/utils";

import { severityDotClass } from "@/components/dashboard/shared";
import { type DashboardIssue } from "@/components/dashboard/types";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "@/components/ui/hover-card";

interface IssueLogsSectionProps {
  issues: DashboardIssue[];
  className?: string;
}

export function IssueLogsSection({ issues, className }: IssueLogsSectionProps): React.JSX.Element {
  return (
    <section className={cn("h-full rounded-xl border border-white/15 bg-white/5 p-4", className)}>
      <div className="inline-flex items-center gap-1.5">
        <h2 className="flex items-center gap-2 text-xs font-semibold">
          <Activity className="h-4 w-4" /> Recent Process Logs
        </h2>
        <HoverCard openDelay={120} closeDelay={80}>
          <HoverCardTrigger asChild>
            <button
              type="button"
              className="inline-flex h-5 w-5 items-center justify-center rounded-full border border-white/20 bg-white/5 text-zinc-300 transition hover:bg-white/10 hover:text-zinc-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-300/60"
              aria-label="What this recent process logs section shows"
            >
              <CircleHelp className="h-3.5 w-3.5" />
            </button>
          </HoverCardTrigger>
          <HoverCardContent align="start" className="w-72 border border-white/15 bg-zinc-900/95 text-zinc-100">
            <p className="text-xs font-semibold">Recent Process Logs</p>
            <p className="mt-1 text-[11px] leading-relaxed text-zinc-300">
              This section lists the latest warnings and critical findings from monitoring checks.
              Each log helps you understand what changed and which process needs attention first.
            </p>
          </HoverCardContent>
        </HoverCard>
      </div>

      <div className="mt-3 space-y-2.5">
        {issues.length === 0 ? (
          <p className="text-sm text-zinc-300">No active warnings in the current cycle.</p>
        ) : (
          issues.map((issue) => (
            <div key={issue.id} className="flex items-center gap-2 rounded-lg border border-white/10 p-2.5">
              <span className={`h-2.5 w-2.5 rounded-full ${severityDotClass(issue.severity)}`} />
              <span className="text-xs font-semibold uppercase tracking-wide">{issue.severity}</span>
              <span className="text-sm text-zinc-300">{issue.title}</span>
            </div>
          ))
        )}
      </div>
    </section>
  );
}
