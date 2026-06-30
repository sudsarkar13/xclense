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

export interface IssueCategory {
	id: string;
	label: string;
	severity: SeverityLevel;
	count: number;
	firstIssueId: string;
}

export interface HealthScoreBreakdown {
	base: number;
	criticalPenalty: number;
	warningPenalty: number;
	pressurePenalty: number;
	loadPenalty: number;
	final: number;
	tone: "Healthy" | "Watch" | "Action needed";
}
