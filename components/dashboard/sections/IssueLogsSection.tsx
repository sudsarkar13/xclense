import { Activity } from "lucide-react";

import { cn } from "@/lib/utils";

import { severityDotClass } from "@/components/dashboard/shared";
import { type DashboardIssue } from "@/components/dashboard/types";

interface IssueLogsSectionProps {
  issues: DashboardIssue[];
  className?: string;
}

export function IssueLogsSection({ issues, className }: IssueLogsSectionProps): React.JSX.Element {
  return (
    <section className={cn("h-full rounded-xl border border-white/15 bg-white/5 p-4", className)}>
      <h2 className="flex items-center gap-2 text-xl font-semibold">
        <Activity className="h-4 w-4" /> Recent Process Logs
      </h2>

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
