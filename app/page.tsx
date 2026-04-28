"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import {
  analyzeIssues,
  isTauriRuntime,
  type AnalysisReport,
  type IssueReport,
} from "@/lib/tauri-client";

type Severity = IssueReport["severity"];

const SEVERITY_ORDER: Severity[] = ["critical", "warning", "info"];

const severityClassMap: Record<Severity, string> = {
  critical: "border-red-500/40 bg-red-50 text-red-900 dark:bg-red-950/30 dark:text-red-100",
  warning:
    "border-amber-500/40 bg-amber-50 text-amber-900 dark:bg-amber-950/30 dark:text-amber-100",
  info: "border-blue-500/40 bg-blue-50 text-blue-900 dark:bg-blue-950/30 dark:text-blue-100",
};

const severityBadgeMap: Record<Severity, string> = {
  critical: "CRITICAL",
  warning: "WARNING",
  info: "INFO",
};

const actionLabelMap: Record<string, string> = {
  run_storage_cleanup: "Run storage cleanup",
  review_storage_usage: "Review storage usage",
  reduce_memory_pressure: "Reduce memory pressure",
  inspect_memory_consumers: "Inspect memory consumers",
  review_process_candidates: "Review process candidates",
};

export default function Home() {
  const [report, setReport] = useState<AnalysisReport | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const groupedIssues = useMemo(() => {
    if (!report) {
      return {
        critical: [] as IssueReport[],
        warning: [] as IssueReport[],
        info: [] as IssueReport[],
      };
    }

    return report.issues.reduce(
      (accumulator, issue) => {
        accumulator[issue.severity].push(issue);
        return accumulator;
      },
      {
        critical: [] as IssueReport[],
        warning: [] as IssueReport[],
        info: [] as IssueReport[],
      }
    );
  }, [report]);

  const loadAnalysisReport = useCallback(async () => {
    if (!isTauriRuntime()) {
      setErrorMessage(
        "Tauri runtime is not available. Run this page inside the desktop app to view live diagnostics."
      );
      setReport(null);
      return;
    }

    try {
      setIsLoading(true);
      setErrorMessage(null);
      const result = await analyzeIssues();
      setReport(result);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unknown diagnostics error";
      setErrorMessage(`Failed to load diagnostics report: ${message}`);
      setReport(null);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void loadAnalysisReport();
    }, 0);

    return () => {
      window.clearTimeout(timer);
    };
  }, [loadAnalysisReport]);

  return (
    <div className="min-h-screen bg-zinc-50 px-4 py-8 text-zinc-900 dark:bg-zinc-950 dark:text-zinc-100 sm:px-8">
      <main className="mx-auto flex w-full max-w-6xl flex-col gap-6">
        <header className="rounded-xl border border-zinc-200 bg-white p-6 shadow-sm dark:border-zinc-800 dark:bg-zinc-900">
          <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">Xclense Diagnostics</h1>
          <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-400">
            Phase 3 issue analysis output from Rust/Tauri telemetry.
          </p>
          <div className="mt-4 flex flex-wrap items-center gap-3 text-xs sm:text-sm">
            <button
              type="button"
              onClick={() => void loadAnalysisReport()}
              className="rounded-md bg-zinc-900 px-3 py-2 font-medium text-white transition hover:bg-zinc-700 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-zinc-300"
            >
              Refresh Report
            </button>
            {report ? (
              <span className="text-zinc-600 dark:text-zinc-300">
                Total issues: <strong>{report.totalIssues}</strong>
              </span>
            ) : null}
            {isLoading ? <span className="text-zinc-500">Loading…</span> : null}
          </div>
        </header>

        {errorMessage ? (
          <section className="rounded-xl border border-red-500/40 bg-red-50 p-4 text-sm text-red-900 dark:bg-red-950/30 dark:text-red-100">
            {errorMessage}
          </section>
        ) : null}

        {!errorMessage && !isLoading && report && report.totalIssues === 0 ? (
          <section className="rounded-xl border border-emerald-500/40 bg-emerald-50 p-6 text-sm text-emerald-900 dark:bg-emerald-950/30 dark:text-emerald-100">
            No active issues detected. System appears healthy with current heuristics.
          </section>
        ) : null}

        {SEVERITY_ORDER.map((severity) => {
          const issues = groupedIssues[severity];
          if (issues.length === 0) {
            return null;
          }

          return (
            <section key={severity} className="flex flex-col gap-4">
              <h2 className="text-lg font-semibold tracking-tight">{severityBadgeMap[severity]}</h2>
              <div className="grid gap-4 md:grid-cols-2">
                {issues.map((issue) => (
                  <article
                    key={issue.id}
                    className={`rounded-xl border p-5 shadow-sm ${severityClassMap[issue.severity]}`}
                  >
                    <div className="flex items-start justify-between gap-3">
                      <h3 className="text-base font-semibold">{issue.title}</h3>
                      <span className="rounded-md border border-current/30 px-2 py-1 text-[10px] font-bold tracking-wide">
                        {severityBadgeMap[issue.severity]}
                      </span>
                    </div>

                    <p className="mt-2 text-sm opacity-90">
                      Confidence: <strong>{(issue.confidence * 100).toFixed(0)}%</strong>
                    </p>

                    <div className="mt-4">
                      <p className="text-xs font-semibold uppercase tracking-wide opacity-80">Evidence</p>
                      <ul className="mt-2 list-disc space-y-1 pl-5 text-sm">
                        {issue.evidence.map((line, index) => (
                          <li key={`${issue.id}-evidence-${index}`}>{line}</li>
                        ))}
                      </ul>
                    </div>

                    <div className="mt-4">
                      <p className="text-xs font-semibold uppercase tracking-wide opacity-80">Recommendation</p>
                      <p className="mt-1 text-sm">{issue.recommendation}</p>
                    </div>

                    <button
                      type="button"
                      className="mt-4 rounded-md border border-current/40 px-3 py-2 text-xs font-semibold transition hover:bg-black/5 dark:hover:bg-white/10"
                    >
                      {actionLabelMap[issue.suggestedAction] ?? issue.suggestedAction}
                    </button>
                  </article>
                ))}
              </div>
            </section>
          );
        })}
      </main>
    </div>
  );
}
