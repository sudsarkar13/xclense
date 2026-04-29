"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

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

type CheckpointData = {
  analysis: AnalysisReport;
  health: SystemHealth;
};

type RealtimeData = {
  storage: StorageSummary;
  health: SystemHealth;
  processes: ProcessInfo[];
  issues: AnalysisReport["issues"];
};

export default function Home(): React.JSX.Element {
  const [checkpointData, setCheckpointData] = useState<CheckpointData | null>(null);
  const [realtimeData, setRealtimeData] = useState<RealtimeData | null>(null);
  const [memoryTrend, setMemoryTrend] = useState<number[]>([]);
  const [isCheckpointLoading, setIsCheckpointLoading] = useState<boolean>(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const isRealtimeLoadingRef = useRef<boolean>(false);

  const appendMemoryTrend = useCallback((value: number) => {
    const nextValue = Math.max(0, Math.min(100, value));
    setMemoryTrend((previous) => {
      const updated = [...previous, nextValue];
      return updated.slice(-30);
    });
  }, []);

  const loadCheckpointData = useCallback(async () => {
    if (!isTauriRuntime()) {
      setErrorMessage("Tauri runtime is not available. Open this page in the Xclense desktop app.");
      setCheckpointData(null);
      return;
    }

    try {
      setIsCheckpointLoading(true);
      setErrorMessage(null);

      const [analysis, health] = await Promise.all([analyzeIssues(), getSystemHealth()]);

      setCheckpointData({ analysis, health });
      appendMemoryTrend(health.memoryPressurePercent);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unknown dashboard error";
      setErrorMessage(`Failed to load checkpoint data: ${message}`);
      setCheckpointData(null);
    } finally {
      setIsCheckpointLoading(false);
    }
  }, [appendMemoryTrend]);

  const loadRealtimeData = useCallback(async () => {
    if (!isTauriRuntime()) {
      return;
    }

    if (isRealtimeLoadingRef.current) {
      return;
    }

    try {
      isRealtimeLoadingRef.current = true;
      setErrorMessage(null);

      const [storage, health, processes, analysis] = await Promise.all([
        scanStorage(),
        getSystemHealth(),
        listProcesses(),
        analyzeIssues(),
      ]);

      setRealtimeData({
        storage,
        health,
        processes: processes.slice(0, 6),
        issues: analysis.issues,
      });

      appendMemoryTrend(health.memoryPressurePercent);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unknown realtime dashboard error";
      setErrorMessage(`Failed to load realtime dashboard data: ${message}`);
    } finally {
      isRealtimeLoadingRef.current = false;
    }
  }, [appendMemoryTrend]);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void loadCheckpointData();
      void loadRealtimeData();
    }, 0);

    return () => {
      window.clearTimeout(timer);
    };
  }, [loadCheckpointData, loadRealtimeData]);

  useEffect(() => {
    const POLL_INTERVAL_MS = 3000;

    const poll = (): void => {
      if (document.visibilityState !== "visible") {
        return;
      }

      void loadRealtimeData();
    };

    const intervalId = window.setInterval(poll, POLL_INTERVAL_MS);

    const handleVisibilityChange = (): void => {
      if (document.visibilityState === "visible") {
        void loadRealtimeData();
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      window.clearInterval(intervalId);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, [loadRealtimeData]);

  const healthScore = useMemo(() => {
    if (!checkpointData) return 0;

    const criticalCount = checkpointData.analysis.issues.filter((issue) => issue.severity === "critical").length;
    const warningCount = checkpointData.analysis.issues.filter((issue) => issue.severity === "warning").length;
    const score = 100 - criticalCount * 25 - warningCount * 10;

    return Math.max(20, Math.min(98, score));
  }, [checkpointData]);

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

          {!checkpointData && !realtimeData && !errorMessage ? (
            <div className="rounded-lg border border-white/20 bg-white/5 p-4 text-sm text-zinc-300">
              Waiting for diagnostics data...
            </div>
          ) : null}

          {checkpointData && realtimeData ? (
            <div className="grid gap-3 md:gap-4 xl:grid-cols-12">
              <div className="xl:col-span-4">
                <SystemHealthCard
                  className="min-h-[175px]"
                  score={healthScore}
                  totalIssues={checkpointData.analysis.totalIssues}
                  memoryUsedBytes={checkpointData.health.memoryUsedBytes}
                  memoryFreeBytes={checkpointData.health.memoryFreeBytes}
                  onReload={() => void loadCheckpointData()}
                  isReloading={isCheckpointLoading}
                />
              </div>

              <div className="xl:col-span-4">
                <MemoryPressureCard
                  className="min-h-[175px]"
                  pressurePercent={realtimeData.health.memoryPressurePercent}
                  memoryUsedBytes={realtimeData.health.memoryUsedBytes}
                  memoryFreeBytes={realtimeData.health.memoryFreeBytes}
                  trendSeries={memoryTrend}
                />
              </div>

              <div className="xl:col-span-4">
                <StorageOverviewCard
                  className="min-h-[175px]"
                  totalBytes={realtimeData.storage.totalBytes}
                  freeBytes={realtimeData.storage.freeBytes}
                  usedPercent={realtimeData.storage.usedPercent}
                />
              </div>

              <div className="xl:col-span-7">
                <TopProcessesSection className="min-h-[310px]" processes={realtimeData.processes} />
              </div>

              <div className="xl:col-span-5">
                <IssueLogsSection className="min-h-[310px]" issues={realtimeData.issues} />
              </div>
            </div>
          ) : null}
        </main>
      </div>
    </div>
  );
}
