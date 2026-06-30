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

export type SeverityLevel = "critical" | "warning" | "info";

export interface IssueReport {
	id: string;
	title: string;
	severity: SeverityLevel;
	confidence: number;
	evidence: string[];
	recommendation: string;
	suggestedAction: string;
}

export interface IssueCategory {
	id: string;
	label: string;
	severity: SeverityLevel;
	count: number;
	firstIssueId: string;
}

export interface AnalysisReport {
	generatedAtEpochMs: number;
	totalIssues: number;
	issues: IssueReport[];
	categories: IssueCategory[];
}

export interface ReportSnapshotMeta {
	snapshotId: string;
	createdAtEpochMs: number;
	issueCount: number;
	highestSeverity: SeverityLevel | "none";
	sourceVersion: string;
}

export interface ReportSnapshot {
	meta: ReportSnapshotMeta;
	report: AnalysisReport;
}

export interface ExportResult {
	snapshotId: string;
	format: "json" | "txt";
	exportedAtEpochMs: number;
	filePath: string;
}

export type ProcessActionType = "terminate" | "force_kill";

export interface ProcessActionConfirmation {
	acknowledgedRisk: boolean;
	reason: string;
	typedToken?: string;
}

export interface ManageProcessActionRequest {
	pid: number;
	action: ProcessActionType;
	processNameHint?: string;
	confirmation?: ProcessActionConfirmation;
	sourceContext?: "health_page" | "dashboard" | "auto_suggestion";
}

export type ActionStatus = "executed" | "blocked" | "denied" | "failed";

export interface ActionResult {
	action: ProcessActionType;
	targetPid: number;
	status: ActionStatus;
	message: string;
	decisionCode: string;
	performedAtEpochMs: number;
	auditId: string;
	riskLevel: "low" | "medium" | "high" | "critical";
}

export interface ActionAuditRecord {
	auditId: string;
	action: ProcessActionType;
	pid: number;
	processName: string;
	decision: ActionStatus;
	decisionCode: string;
	reason: string;
	riskLevel: "low" | "medium" | "high" | "critical";
	requestedAtEpochMs: number;
	completedAtEpochMs?: number;
	sourceVersion: string;
	sourceContext?: string;
}

export type RemediationRiskLevel = "low" | "medium" | "high";

export interface RemediationStep {
	id: string;
	title: string;
	description: string;
	riskLevel: RemediationRiskLevel;
	autoRunnable: boolean;
	guidance: string[];
}

export interface RemediationPlan {
	generatedAtEpochMs: number;
	issueCount: number;
	steps: RemediationStep[];
	autoSafeSteps: string[];
}

export type RemediationStatus = "succeeded" | "failed" | "skipped" | "unknown";

export interface RemediationStepResult {
	stepId: string;
	status: RemediationStatus;
	message: string;
	performedAtEpochMs: number;
}

export interface RemediationExecution {
	requestedStepIds: string[];
	results: RemediationStepResult[];
	allSucceeded: boolean;
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

export const createReportSnapshot = async (
	report?: AnalysisReport,
): Promise<ReportSnapshotMeta> => {
	ensureTauriRuntime();
	return invoke<ReportSnapshotMeta>("create_report_snapshot", {
		report,
	});
};

export const listReportSnapshots = async (
	limit?: number,
): Promise<ReportSnapshotMeta[]> => {
	ensureTauriRuntime();
	return invoke<ReportSnapshotMeta[]>("list_report_snapshots", {
		limit,
	});
};

export const getReportSnapshot = async (
	snapshotId: string,
): Promise<ReportSnapshot> => {
	ensureTauriRuntime();
	return invoke<ReportSnapshot>("get_report_snapshot", {
		snapshotId,
	});
};

export const exportReportSnapshot = async (
	snapshotId: string,
	format: "json" | "txt",
): Promise<ExportResult> => {
	ensureTauriRuntime();
	return invoke<ExportResult>("export_report_snapshot", {
		snapshotId,
		format,
	});
};

export const manageProcessAction = async (
	request: ManageProcessActionRequest,
): Promise<ActionResult> => {
	ensureTauriRuntime();
	return invoke<ActionResult>("manage_process_action", {
		request,
	});
};

export const listProcessActionAudits = async (
	limit?: number,
): Promise<ActionAuditRecord[]> => {
	ensureTauriRuntime();
	return invoke<ActionAuditRecord[]>("list_process_action_audits", {
		limit,
	});
};

export const getRemediationPlan = async (
	report?: AnalysisReport,
): Promise<RemediationPlan> => {
	ensureTauriRuntime();
	return invoke<RemediationPlan>("get_remediation_plan", {
		report,
	});
};

export const runSafeRemediation = async (
	stepIds: string[],
): Promise<RemediationExecution> => {
	ensureTauriRuntime();
	return invoke<RemediationExecution>("run_safe_remediation", {
		stepIds,
	});
};
