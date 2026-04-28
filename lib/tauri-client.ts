import { invoke } from "@tauri-apps/api/core";

export interface PingResponse {
  service: string;
  status: string;
  version: string;
}

export interface StorageSummary {
  totalBytes: number;
  usedBytes: number;
  freeBytes: number;
  usedPercent: number;
  scannedAtEpochMs: number;
}

export interface ProcessInfo {
  pid: number;
  name: string;
  cpuPercent: number;
  memoryPercent: number;
  state: string;
}

export interface SystemHealth {
  memoryTotalBytes: number;
  memoryFreeBytes: number;
  memoryUsedBytes: number;
  memoryPressurePercent: number;
  loadAverage1m: number;
  loadAverage5m: number;
  loadAverage15m: number;
  scannedAtEpochMs: number;
}

export interface IssueReport {
  id: string;
  title: string;
  severity: "critical" | "warning" | "info";
  confidence: number;
  evidence: string[];
  recommendation: string;
  suggestedAction: string;
}

export interface AnalysisReport {
  generatedAtEpochMs: number;
  totalIssues: number;
  issues: IssueReport[];
}

/**
 * Determines whether the app is running inside a Tauri runtime.
 */
export const isTauriRuntime = (): boolean => {
  if (typeof window === "undefined") {
    return false;
  }

  return "__TAURI_INTERNALS__" in window;
};

const ensureTauriRuntime = (): void => {
  if (!isTauriRuntime()) {
    throw new Error("Tauri runtime is not available in this environment.");
  }
};

/**
 * Calls the Rust backend ping command.
 * Returns null when not running inside Tauri, allowing web-only UI work.
 */
export const pingBackend = async (): Promise<PingResponse | null> => {
  if (!isTauriRuntime()) {
    return null;
  }

  return invoke<PingResponse>("ping_backend");
};

export const scanStorage = async (): Promise<StorageSummary> => {
  ensureTauriRuntime();
  return invoke<StorageSummary>("scan_storage");
};

export const listProcesses = async (): Promise<ProcessInfo[]> => {
  ensureTauriRuntime();
  return invoke<ProcessInfo[]>("list_processes");
};

export const getSystemHealth = async (): Promise<SystemHealth> => {
  ensureTauriRuntime();
  return invoke<SystemHealth>("get_system_health");
};

export const analyzeIssues = async (): Promise<AnalysisReport> => {
  ensureTauriRuntime();
  return invoke<AnalysisReport>("analyze_issues");
};
