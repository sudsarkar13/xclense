"use client";

import { useMemo, useState } from "react";
import {
	AlertTriangle,
	CheckCircle2,
	ChevronRight,
	Info,
	Loader2,
	Sparkles,
} from "lucide-react";

import { cn } from "@/lib/utils";

import { healthToneClass } from "@/components/dashboard/shared";
import { type HealthScoreBreakdown } from "@/components/dashboard/types";
import {
	type RemediationPlan,
	type RemediationStep,
	type RemediationStepResult,
} from "@/lib/tauri-client";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";

interface FixOverlayProps {
	open: boolean;
	onOpenChange: (next: boolean) => void;
	plan: RemediationPlan | null;
	score: HealthScoreBreakdown;
	topIssueTitle?: string;
	isPlanLoading: boolean;
	isExecuting: boolean;
	results: RemediationStepResult[] | null;
	onRunAllSafe: () => void;
	onRunOne: (stepId: string) => void;
	onRescan: () => void;
	errorMessage?: string | null;
}

const riskBadgeClass = (risk: RemediationStep["riskLevel"]): string => {
	if (risk === "low")
		return "border-emerald-400/35 bg-emerald-500/10 text-emerald-200";
	if (risk === "medium")
		return "border-amber-400/35 bg-amber-500/10 text-amber-200";
	return "border-red-400/35 bg-red-500/10 text-red-200";
};

const statusBadgeClass = (status: RemediationStepResult["status"]): string => {
	if (status === "succeeded")
		return "border-emerald-400/35 bg-emerald-500/10 text-emerald-200";
	if (status === "failed")
		return "border-red-400/35 bg-red-500/10 text-red-200";
	if (status === "skipped")
		return "border-amber-400/35 bg-amber-500/10 text-amber-200";
	return "border-white/20 bg-white/5 text-zinc-300";
};

const statusLabel = (status: RemediationStepResult["status"]): string => {
	switch (status) {
		case "succeeded":
			return "Done";
		case "failed":
			return "Failed";
		case "skipped":
			return "Manual";
		default:
			return "Unknown";
	}
};

export function FixOverlay({
	open,
	onOpenChange,
	plan,
	score,
	topIssueTitle,
	isPlanLoading,
	isExecuting,
	results,
	onRunAllSafe,
	onRunOne,
	onRescan,
	errorMessage,
}: FixOverlayProps): React.JSX.Element {
	const resultsByStepId = useMemo(() => {
		const map = new Map<string, RemediationStepResult>();
		for (const result of results ?? []) {
			map.set(result.stepId, result);
		}
		return map;
	}, [results]);

	const safeAutoCount = plan?.autoSafeSteps.length ?? 0;
	const manualCount =
		plan ? plan.steps.filter((step) => !step.autoRunnable).length : 0;

	const [acknowledgedSafe, setAcknowledgedSafe] = useState<boolean>(false);

	const handleOpenChange = (next: boolean): void => {
		if (!next) {
			setAcknowledgedSafe(false);
		}
		onOpenChange(next);
	};

	const handleRunAllSafe = (): void => {
		if (plan && plan.autoSafeSteps.length > 0) {
			onRunAllSafe();
		}
	};

	const handleRunOne = (stepId: string): void => {
		onRunOne(stepId);
	};

	const showResults = Boolean(results && results.length > 0);

	return (
		<Dialog open={open} onOpenChange={handleOpenChange}>
			<DialogContent
				showCloseButton
				className="!max-w-none w-[1000px] max-w-[92vw] max-h-[80vh] overflow-y-auto border-white/15 bg-zinc-900/95 text-zinc-100 data-[state=open]:!translate-x-[calc(-50%+8.3333vw)] data-[state=closed]:!translate-x-[calc(-50%+8.3333vw)]">
				<DialogHeader className="space-y-1.5">
					<div className="flex items-center gap-2">
						<Sparkles className="h-4 w-4 text-cyan-300" />
						<DialogTitle className="text-base">Fix overall health</DialogTitle>
					</div>
					<DialogDescription className="text-xs text-zinc-300">
						{topIssueTitle ?
							`Recommended actions for: ${topIssueTitle}.`
						:	"Recommended actions to bring your system back to a healthy state."
						}
					</DialogDescription>
					<div className="mt-1 flex items-center gap-3 text-[11px] text-zinc-300">
						<span>
							<span className="text-zinc-400">Current score:</span>{" "}
							<span
								className={cn("font-semibold", healthToneClass(score.tone))}>
								{score.final}
							</span>
						</span>
						<span>
							<span className="text-zinc-400">Tone:</span> {score.tone}
						</span>
						<span>
							<span className="text-zinc-400">Steps:</span>{" "}
							{plan?.steps.length ?? 0} ({safeAutoCount} safe / {manualCount}{" "}
							manual)
						</span>
					</div>
				</DialogHeader>

				{errorMessage ?
					<div className="rounded-md border border-red-400/40 bg-red-500/10 p-2.5 text-xs text-red-200">
						{errorMessage}
					</div>
				:	null}

				{isPlanLoading ?
					<div className="flex items-center gap-2 rounded-md border border-white/15 bg-white/5 p-3 text-xs text-zinc-300">
						<Loader2 className="h-3.5 w-3.5 animate-spin" />
						Building remediation plan…
					</div>
				: !plan ?
					<p className="text-xs text-zinc-300">
						No plan is available yet. Try reopening this overlay after the next
						scan.
					</p>
				: plan.steps.length === 0 ?
					<p className="text-xs text-zinc-300">No actions to suggest.</p>
				:	<div className="space-y-3">
						<div className="rounded-md border border-white/10 bg-white/5 p-3 text-[11px] leading-relaxed text-zinc-300">
							<div className="flex items-start gap-2">
								<Info className="mt-0.5 h-3.5 w-3.5 shrink-0 text-cyan-300" />
								<p>
									Safe steps run automatically and only perform non-destructive
									actions (for example, asking macOS to release inactive
									memory). Manual steps require you to follow the guidance below
									— nothing here deletes files or kills processes for you.
								</p>
							</div>
						</div>

						<ol className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
							{plan.steps.map((step, index) => {
								const result = resultsByStepId.get(step.id);
								const alreadySucceeded = result?.status === "succeeded";
								return (
									<li
										key={step.id}
										className={cn(
											"rounded-lg border p-3",
											alreadySucceeded ?
												"border-emerald-400/30 bg-emerald-500/5"
											:	"border-white/10 bg-white/5",
										)}>
										<div className="flex items-start justify-between gap-2">
											<div className="flex min-w-0 flex-1 items-start gap-2">
												<span className="mt-0.5 inline-flex h-5 w-5 shrink-0 items-center justify-center rounded-full border border-white/15 bg-white/5 text-[10px] font-semibold text-zinc-200">
													{index + 1}
												</span>
												<div className="min-w-0">
													<div className="flex flex-wrap items-center gap-1.5">
														<p className="text-sm font-semibold text-zinc-100">
															{step.title}
														</p>
														<span
															className={cn(
																"rounded-full border px-1.5 py-0.5 text-[10px]",
																riskBadgeClass(step.riskLevel),
															)}>
															{step.riskLevel} risk
														</span>
														{step.autoRunnable ?
															<span className="rounded-full border border-cyan-400/35 bg-cyan-500/10 px-1.5 py-0.5 text-[10px] text-cyan-200">
																auto
															</span>
														:	<span className="rounded-full border border-white/15 bg-white/5 px-1.5 py-0.5 text-[10px] text-zinc-300">
																manual
															</span>
														}
														{result ?
															<span
																className={cn(
																	"rounded-full border px-1.5 py-0.5 text-[10px]",
																	statusBadgeClass(result.status),
																)}>
																{result.status === "succeeded" ?
																	<CheckCircle2 className="mr-0.5 inline h-3 w-3" />
																: result.status === "failed" ?
																	<AlertTriangle className="mr-0.5 inline h-3 w-3" />
																:	null}
																{statusLabel(result.status)}
															</span>
														:	null}
													</div>
													<p className="mt-1 text-[11px] leading-relaxed text-zinc-300">
														{step.description}
													</p>

													<details className="mt-1.5 group">
														<summary className="cursor-pointer text-[11px] text-cyan-300 hover:text-cyan-200">
															Show step-by-step guidance
														</summary>
														<ol className="mt-1.5 space-y-1 text-[11px] text-zinc-300">
															{step.guidance.map((line) => (
																<li
																	key={line}
																	className="flex items-start gap-1.5">
																	<ChevronRight className="mt-0.5 h-3 w-3 shrink-0 text-zinc-400" />
																	<span>{line}</span>
																</li>
															))}
														</ol>
													</details>

													{result ?
														<p className="mt-1.5 text-[11px] text-zinc-300">
															<span className="text-zinc-400">Result:</span>{" "}
															{result.message}
														</p>
													:	null}
												</div>
											</div>
										</div>

										{step.autoRunnable && !alreadySucceeded ?
											<div className="mt-2 flex items-center justify-end">
												<Button
													size="sm"
													variant="default"
													disabled={isExecuting}
													onClick={() => handleRunOne(step.id)}>
													{isExecuting ?
														<Loader2 className="mr-1 h-3.5 w-3.5 animate-spin" />
													:	<Sparkles className="mr-1 h-3.5 w-3.5" />}
													Run this step
												</Button>
											</div>
										:	null}
									</li>
								);
							})}
						</ol>
					</div>
				}

				<DialogFooter className="mt-2 flex flex-wrap items-center justify-between gap-2 sm:justify-between">
					<div className="flex items-center gap-2 text-[11px] text-zinc-300">
						<label className="inline-flex items-center gap-1.5">
							<input
								type="checkbox"
								checked={acknowledgedSafe}
								onChange={(event) => setAcknowledgedSafe(event.target.checked)}
								className="h-3.5 w-3.5 rounded border-white/20 bg-white/5"
								aria-label="I understand safe steps only run non-destructive actions"
							/>
							<span>
								I understand safe steps only run non-destructive actions
							</span>
						</label>
					</div>
					<div className="flex flex-wrap items-center gap-2">
						<Button
							variant="ghost"
							size="sm"
							onClick={onRescan}
							disabled={isExecuting || isPlanLoading}>
							<Loader2
								className={cn(
									"mr-1 h-3.5 w-3.5",
									isPlanLoading ? "animate-spin" : "hidden",
								)}
							/>
							Re-scan
						</Button>
						{safeAutoCount > 0 ?
							<Button
								size="sm"
								onClick={handleRunAllSafe}
								disabled={
									!acknowledgedSafe ||
									isExecuting ||
									!plan ||
									safeAutoCount === 0 ||
									showResults
								}
								title={
									acknowledgedSafe ?
										`Run ${safeAutoCount} safe step${safeAutoCount === 1 ? "" : "s"}`
									:	"Tick the acknowledgement first"
								}>
								{isExecuting ?
									<Loader2 className="mr-1 h-3.5 w-3.5 animate-spin" />
								:	<Sparkles className="mr-1 h-3.5 w-3.5" />}
								Run {safeAutoCount} safe step{safeAutoCount === 1 ? "" : "s"}
							</Button>
						:	null}
					</div>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}
