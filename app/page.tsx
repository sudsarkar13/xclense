"use client";

import { useCallback, useEffect, useMemo, useState } from "react";

import { MemoryPressureCard } from "@/components/dashboard/cards/MemoryPressureCard";
import { StorageOverviewCard } from "@/components/dashboard/cards/StorageOverviewCard";
import { SystemHealthCard } from "@/components/dashboard/cards/SystemHealthCard";
import { DashboardHeader } from "@/components/dashboard/layout/DashboardHeader";
import { DashboardNav } from "@/components/dashboard/layout/DashboardNav";
import { IssueLogsSection } from "@/components/dashboard/sections/IssueLogsSection";
import { TopProcessesSection } from "@/components/dashboard/sections/TopProcessesSection";
import {
  analyzeIssues,
  getSystemHealth,
  isTauriRuntime,
  listProcesses,
  scanStorage,
  type AnalysisReport,
  type ProcessInfo,
  type StorageSummary,
  type SystemHealth,
} from "@/lib/tauri-client";

type DashboardData = {
  analysis: AnalysisReport;
  storage: StorageSummary;
  health: SystemHealth;
  processes: ProcessInfo[];
};

export default function Home(): React.JSX.Element {
  const [data, setData] = useState<DashboardData | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const loadDashboard = useCallback(async () => {
    if (!isTauriRuntime()) {
      setErrorMessage("Tauri runtime is not available. Open this page in the Xclense desktop app.");
      setData(null);
      return;
    }

    try {
      setIsLoading(true);
      setErrorMessage(null);

      const [analysis, storage, health, processes] = await Promise.all([
        analyzeIssues(),
        scanStorage(),
        getSystemHealth(),
        listProcesses(),
      ]);

      setData({
        analysis,
        storage,
        health,
        processes: processes.slice(0, 6),
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unknown dashboard error";
      setErrorMessage(`Failed to load dashboard data: ${message}`);
      setData(null);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void loadDashboard();
    }, 0);

    return () => {
      window.clearTimeout(timer);
    };
  }, [loadDashboard]);

  const healthScore = useMemo(() => {
    if (!data) return 0;

    const criticalCount = data.analysis.issues.filter((issue) => issue.severity === "critical").length;
    const warningCount = data.analysis.issues.filter((issue) => issue.severity === "warning").length;
    const score = 100 - criticalCount * 25 - warningCount * 10;

    return Math.max(20, Math.min(98, score));
  }, [data]);

  return (
    <div className="h-screen w-screen overflow-hidden bg-[radial-gradient(circle_at_top_right,#3347ad_0%,#11152f_40%,#0a0f24_100%)] font-sans text-zinc-100">
      <div className="grid h-full min-h-0 w-full grid-cols-12 border border-white/15 bg-[#0d1226]/90 shadow-2xl">
        <DashboardNav />

        <main className="col-span-12 min-h-0 overflow-auto p-3 md:col-span-9 lg:col-span-10 md:p-4">
          <DashboardHeader isLoading={isLoading} onRefresh={() => void loadDashboard()} />

          {errorMessage ? (
            <div className="mb-3 rounded-lg border border-red-400/40 bg-red-500/10 p-3 text-sm text-red-200">
              {errorMessage}
            </div>
          ) : null}

          {!data && !errorMessage ? (
            <div className="rounded-lg border border-white/20 bg-white/5 p-4 text-sm text-zinc-300">
              Waiting for diagnostics data...
            </div>
          ) : null}

          {data ? (
            <div className="grid auto-rows-fr gap-3 md:gap-4 xl:grid-cols-12">
              <div className="xl:col-span-4">
                <SystemHealthCard
                  className="min-h-[300px]"
                  score={healthScore}
                  totalIssues={data.analysis.totalIssues}
                  memoryUsedBytes={data.health.memoryUsedBytes}
                  memoryFreeBytes={data.health.memoryFreeBytes}
                />
              </div>

              <div className="xl:col-span-4">
                <MemoryPressureCard
                  className="min-h-[300px]"
                  pressurePercent={data.health.memoryPressurePercent}
                  memoryUsedBytes={data.health.memoryUsedBytes}
                  memoryFreeBytes={data.health.memoryFreeBytes}
                />
              </div>

              <div className="xl:col-span-4">
                <StorageOverviewCard
                  className="min-h-[300px]"
                  totalBytes={data.storage.totalBytes}
                  freeBytes={data.storage.freeBytes}
                  usedPercent={data.storage.usedPercent}
                />
              </div>

              <div className="xl:col-span-7">
                <TopProcessesSection className="min-h-[310px]" processes={data.processes} />
              </div>

              <div className="xl:col-span-5">
                <IssueLogsSection className="min-h-[310px]" issues={data.analysis.issues} />
              </div>
            </div>
          ) : null}
        </main>
      </div>
    </div>
  );
}
