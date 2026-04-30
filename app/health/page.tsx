"use client";

import { useCallback, useEffect, useState } from "react";

import { DashboardHeader } from "@/components/dashboard/layout/DashboardHeader";
import { DashboardNav } from "@/components/dashboard/layout/DashboardNav";
import {
  analyzeIssues,
  createReportSnapshot,
  exportReportSnapshot,
  getReportSnapshot,
  isTauriRuntime,
  listReportSnapshots,
  type AnalysisReport,
  type ReportSnapshot,
  type ReportSnapshotMeta,
} from "@/lib/tauri-client";

type ExportFormat = "json" | "txt";

export default function HealthPage(): React.JSX.Element {
  const [analysis, setAnalysis] = useState<AnalysisReport | null>(null);
  const [snapshots, setSnapshots] = useState<ReportSnapshotMeta[]>([]);
  const [selectedSnapshot, setSelectedSnapshot] = useState<ReportSnapshot | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [reportMessage, setReportMessage] = useState<string | null>(null);
  const [isSavingSnapshot, setIsSavingSnapshot] = useState<boolean>(false);
  const [isLoadingSnapshots, setIsLoadingSnapshots] = useState<boolean>(false);

  const loadAnalysis = useCallback(async (): Promise<void> => {
    if (!isTauriRuntime()) {
      return;
    }

    try {
      const result = await analyzeIssues();
      setAnalysis(result);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unknown analysis load error";
      setErrorMessage(`Failed to load analysis: ${message}`);
    }
  }, []);

  const loadSnapshots = useCallback(async (): Promise<void> => {
    if (!isTauriRuntime()) {
      return;
    }

    try {
      setIsLoadingSnapshots(true);
      const result = await listReportSnapshots(20);
      setSnapshots(result);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unknown snapshot load error";
      setReportMessage(`Failed to load report snapshots: ${message}`);
    } finally {
      setIsLoadingSnapshots(false);
    }
  }, []);

  const handleSaveSnapshot = useCallback(async (): Promise<void> => {
    if (!analysis) {
      setReportMessage("No analysis report available yet to snapshot.");
      return;
    }

    if (!isTauriRuntime()) {
      setReportMessage("Tauri runtime is not available. Snapshot feature works only in desktop app.");
      return;
    }

    try {
      setIsSavingSnapshot(true);
      setReportMessage(null);
      const meta = await createReportSnapshot(analysis);
      setReportMessage(`Snapshot saved: ${meta.snapshotId}`);
      await loadSnapshots();
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unknown snapshot save error";
      setReportMessage(`Failed to save report snapshot: ${message}`);
    } finally {
      setIsSavingSnapshot(false);
    }
  }, [analysis, loadSnapshots]);

  const handleOpenSnapshot = useCallback(async (snapshotId: string): Promise<void> => {
    try {
      setReportMessage(null);
      const snapshot = await getReportSnapshot(snapshotId);
      setSelectedSnapshot(snapshot);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unknown snapshot read error";
      setReportMessage(`Failed to open snapshot: ${message}`);
    }
  }, []);

  const handleExportSnapshot = useCallback(
    async (snapshotId: string, format: ExportFormat): Promise<void> => {
      try {
        setReportMessage(null);
        const result = await exportReportSnapshot(snapshotId, format);
        setReportMessage(`Exported ${result.format.toUpperCase()} report to: ${result.filePath}`);
      } catch (error) {
        const message = error instanceof Error ? error.message : "Unknown snapshot export error";
        setReportMessage(`Failed to export snapshot: ${message}`);
      }
    },
    [],
  );

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void loadAnalysis();
      void loadSnapshots();
    }, 0);

    return () => {
      window.clearTimeout(timer);
    };
  }, [loadAnalysis, loadSnapshots]);

  return (
    <div className="h-screen w-screen overflow-hidden bg-[radial-gradient(circle_at_top_right,#3347ad_0%,#11152f_40%,#0a0f24_100%)] font-sans text-zinc-100">
      <div className="grid h-full min-h-0 w-full grid-cols-12 border border-white/15 bg-[#0d1226]/90 shadow-2xl">
        <DashboardNav />

        <main className="col-span-12 min-h-0 overflow-auto p-3 md:col-span-9 lg:col-span-10 md:p-4">
          <DashboardHeader />

          {errorMessage ? (
            <div className="mb-3 rounded-lg border border-red-400/40 bg-red-500/10 p-3 text-sm text-red-200">
              {errorMessage}
            </div>
          ) : null}

          <section className="mb-3 rounded-lg border border-white/15 bg-white/5 p-3">
            <div className="flex flex-wrap items-center gap-2">
              <button
                type="button"
                onClick={() => void handleSaveSnapshot()}
                disabled={isSavingSnapshot}
                className="rounded-md border border-cyan-300/40 bg-cyan-400/15 px-3 py-1 text-xs font-semibold text-cyan-100 transition hover:bg-cyan-400/25 disabled:cursor-not-allowed disabled:opacity-60"
              >
                {isSavingSnapshot ? "Saving..." : "Save Snapshot"}
              </button>
              <button
                type="button"
                onClick={() => void loadSnapshots()}
                disabled={isLoadingSnapshots}
                className="rounded-md border border-white/20 bg-white/10 px-3 py-1 text-xs font-medium text-zinc-200 transition hover:bg-white/15 disabled:cursor-not-allowed disabled:opacity-60"
              >
                {isLoadingSnapshots ? "Refreshing..." : "Refresh Snapshots"}
              </button>
            </div>

            {reportMessage ? (
              <p className="mt-2 text-xs text-zinc-300">{reportMessage}</p>
            ) : null}

            <div className="mt-2 grid gap-2 lg:grid-cols-2">
              <div className="max-h-56 overflow-auto rounded-md border border-white/10 bg-white/5 p-2">
                {snapshots.length === 0 ? (
                  <p className="text-xs text-zinc-400">No snapshots yet.</p>
                ) : (
                  snapshots.map((snapshot) => (
                    <div key={snapshot.snapshotId} className="mb-1.5 rounded-md border border-white/10 p-2 text-xs last:mb-0">
                      <p className="font-semibold text-zinc-100">{snapshot.snapshotId}</p>
                      <p className="text-zinc-400">
                        {snapshot.highestSeverity.toUpperCase()} · {snapshot.issueCount} issue(s)
                      </p>
                      <div className="mt-1 flex flex-wrap gap-1.5">
                        <button
                          type="button"
                          onClick={() => void handleOpenSnapshot(snapshot.snapshotId)}
                          className="rounded border border-white/20 px-2 py-0.5 text-[11px] text-zinc-200 hover:bg-white/10"
                        >
                          View
                        </button>
                        <button
                          type="button"
                          onClick={() => void handleExportSnapshot(snapshot.snapshotId, "json")}
                          className="rounded border border-white/20 px-2 py-0.5 text-[11px] text-zinc-200 hover:bg-white/10"
                        >
                          Export JSON
                        </button>
                        <button
                          type="button"
                          onClick={() => void handleExportSnapshot(snapshot.snapshotId, "txt")}
                          className="rounded border border-white/20 px-2 py-0.5 text-[11px] text-zinc-200 hover:bg-white/10"
                        >
                          Export TXT
                        </button>
                      </div>
                    </div>
                  ))
                )}
              </div>

              <div className="max-h-56 overflow-auto rounded-md border border-white/10 bg-white/5 p-2 text-xs">
                {selectedSnapshot ? (
                  <>
                    <p className="font-semibold text-zinc-100">Selected: {selectedSnapshot.meta.snapshotId}</p>
                    <p className="mt-0.5 text-zinc-400">
                      Severity: {selectedSnapshot.meta.highestSeverity.toUpperCase()} · Issues: {selectedSnapshot.meta.issueCount}
                    </p>
                    <ul className="mt-2 space-y-1 text-zinc-300">
                      {selectedSnapshot.report.issues.length === 0 ? (
                        <li>No issues in this snapshot.</li>
                      ) : (
                        selectedSnapshot.report.issues.map((issue) => (
                          <li key={issue.id}>
                            <span className="font-semibold uppercase">{issue.severity}</span> — {issue.title}
                          </li>
                        ))
                      )}
                    </ul>
                  </>
                ) : (
                  <p className="text-zinc-400">Select a snapshot to preview report details.</p>
                )}
              </div>
            </div>
          </section>
        </main>
      </div>
    </div>
  );
}
