export type SeverityLevel = "critical" | "warning" | "info";

export interface DashboardIssue {
  id: string;
  title: string;
  severity: SeverityLevel;
}

export interface DashboardProcess {
  pid: number;
  name: string;
  cpuPercent: number;
  memoryPercent: number;
}
