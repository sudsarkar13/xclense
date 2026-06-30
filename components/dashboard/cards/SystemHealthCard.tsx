import { CircleHelp, RotateCw, Wrench } from "lucide-react";

import { cn } from "@/lib/utils";

import { AnimatedCircularProgressBar } from "@/components/dashboard/magic/AnimatedCircularProgressBar";
import {
	formatGb,
	healthToneClass,
	severityBadgeClass,
	severityDotClass,
} from "@/components/dashboard/shared";
import {
	type HealthScoreBreakdown,
	type IssueCategory,
} from "@/components/dashboard/types";
import {
	HoverCard,
	HoverCardContent,
	HoverCardTrigger,
} from "@/components/ui/hover-card";

interface SystemHealthCardProps {
	score: HealthScoreBreakdown;
	totalIssues: number;
	topIssueTitle?: string;
	topIssueId?: string;
	categories: IssueCategory[];
	memoryTotalBytes: number;
	memoryUsedBytes: number;
	memoryFreeBytes: number;
	memoryPressurePercent: number;
	loadAverage1m: number;
	loadAverage5m: number;
	loadAverage15m: number;
	lastCheckpointEpochMs: number;
	criticalCount: number;
	warningCount: number;
	infoCount: number;
	autoRefreshSeconds: number;
	onReload: () => void;
	isReloading: boolean;
	onFixTopIssue?: () => void;
	onJumpToIssue?: (issueId: string) => void;
	className?: string;
}

const VISIBLE_CATEGORIES = 3;

const formatCheckpointTime = (epochMs: number): string => {
	if (!Number.isFinite(epochMs) || epochMs <= 0) return "—";
	return new Date(epochMs).toLocaleTimeString([], {
		hour: "2-digit",
		minute: "2-digit",
	});
};

const pressureLabel = (pressurePercent: number): string =>
	`${Math.max(0, Math.min(100, pressurePercent)).toFixed(0)}%`;

const truncate = (text: string, max: number): string =>
	text.length <= max ? text : `${text.slice(0, max - 1).trimEnd()}…`;

export function SystemHealthCard({
	score,
	totalIssues,
	topIssueTitle,
	topIssueId,
	categories,
	memoryTotalBytes,
	memoryUsedBytes,
	memoryFreeBytes,
	memoryPressurePercent,
	loadAverage1m,
	loadAverage5m,
	loadAverage15m,
	lastCheckpointEpochMs,
	criticalCount,
	warningCount,
	infoCount,
	autoRefreshSeconds,
	onReload,
	isReloading,
	onFixTopIssue,
	onJumpToIssue,
	className,
}: SystemHealthCardProps): React.JSX.Element {
	const visibleCategories = categories.slice(0, VISIBLE_CATEGORIES);
	const hiddenCategoryCount = Math.max(
		0,
		categories.length - VISIBLE_CATEGORIES,
	);

	const recommendationLines: string[] = [];
	if (totalIssues === 0) {
		recommendationLines.push(
			"No urgent issues found. System is running clean.",
		);
	} else {
		if (topIssueTitle) {
			recommendationLines.push(`Top: “${topIssueTitle}”.`);
		}
		recommendationLines.push(
			memoryPressurePercent >= 80 ?
				"Memory: under sustained pressure — close heavy apps."
			:	"Memory: pressure is comfortable.",
		);
		recommendationLines.push(
			criticalCount + warningCount > 0 ?
				`${criticalCount} critical / ${warningCount} warning awaiting action.`
			:	"No critical or warning findings.",
		);
	}

	const handleFixClick = (): void => {
		if (totalIssues === 0) return;
		if (onFixTopIssue) {
			onFixTopIssue();
			return;
		}
		if (topIssueId && onJumpToIssue) {
			onJumpToIssue(topIssueId);
		}
	};

	const handleCategoryClick = (issueId: string): void => {
		if (onJumpToIssue) {
			onJumpToIssue(issueId);
		}
	};

	const hasFixHandler =
		Boolean(onFixTopIssue) || Boolean(onJumpToIssue && topIssueId);

	return (
		<section
			className={cn(
				"flex h-full flex-col rounded-xl border border-white/15 bg-white/5 p-2",
				className,
			)}>
			<div className="flex items-start justify-between gap-2.5">
				<div className="inline-flex items-center gap-1.5">
					<h2 className="text-xs font-semibold">Overall System Health</h2>
					<HoverCard openDelay={120} closeDelay={80}>
						<HoverCardTrigger asChild>
							<button
								type="button"
								className="inline-flex h-5 w-5 items-center justify-center rounded-full border border-white/20 bg-white/5 text-zinc-300 transition hover:bg-white/10 hover:text-zinc-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-300/60"
								aria-label="What this system health card shows">
								<CircleHelp className="h-3.5 w-3.5" />
							</button>
						</HoverCardTrigger>
						<HoverCardContent
							align="start"
							className="w-72 border border-white/15 bg-zinc-900/95 text-zinc-100">
							<p className="text-xs font-semibold">Overall System Health</p>
							<p className="mt-1 text-[11px] leading-relaxed text-zinc-300">
								Combines issue severity, memory pressure, and CPU load into a
								transparent 0–100 score. Hover the score circle to see every
								penalty that contributed.
							</p>
						</HoverCardContent>
					</HoverCard>
				</div>
				<button
					type="button"
					onClick={onReload}
					disabled={isReloading}
					className="inline-flex h-7 w-7 items-center justify-center rounded-md border border-white/15 bg-white/5 text-zinc-300 transition hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-50"
					aria-label="Reload overall system health"
					title="Reload">
					<RotateCw className={cn("h-4 w-4", isReloading && "animate-spin")} />
				</button>
			</div>

			<div className="mt-2 flex items-center gap-2">
				<HoverCard openDelay={120} closeDelay={80}>
					<HoverCardTrigger asChild>
						<button
							type="button"
							className="shrink-0 rounded-full focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-300/60"
							aria-label="View health score breakdown">
							<AnimatedCircularProgressBar
								value={score.final}
								gaugePrimaryColor="rgb(6 182 212)"
								gaugeSecondaryColor="rgba(148, 163, 184, 0.35)"
								className="w-20 text-xl"
							/>
						</button>
					</HoverCardTrigger>
					<HoverCardContent
						align="start"
						className="w-72 border border-white/15 bg-zinc-900/95 text-zinc-100">
						<p className="text-xs font-semibold">Score breakdown</p>
						<p className="mt-1 text-[11px] leading-relaxed text-zinc-300">
							Final score = 100 − each penalty below. Penalties are capped so a
							flood of low-severity issues cannot mask a single critical one.
						</p>
						<table className="mt-2 w-full text-[11px] text-zinc-200">
							<tbody>
								<tr className="border-b border-white/10">
									<td className="py-0.5">Base</td>
									<td className="py-0.5 text-right tabular-nums">
										+{score.base}
									</td>
								</tr>
								<tr className="border-b border-white/10">
									<td className="py-0.5">Critical issues</td>
									<td className="py-0.5 text-right tabular-nums text-red-200">
										−{score.criticalPenalty}
									</td>
								</tr>
								<tr className="border-b border-white/10">
									<td className="py-0.5">Warnings</td>
									<td className="py-0.5 text-right tabular-nums text-amber-200">
										−{score.warningPenalty}
									</td>
								</tr>
								<tr className="border-b border-white/10">
									<td className="py-0.5">Memory pressure</td>
									<td className="py-0.5 text-right tabular-nums text-amber-200">
										−{score.pressurePenalty}
									</td>
								</tr>
								<tr>
									<td className="py-0.5">CPU load</td>
									<td className="py-0.5 text-right tabular-nums text-amber-200">
										−{score.loadPenalty}
									</td>
								</tr>
								<tr className="border-t border-white/15">
									<td className="pt-1 font-semibold">Final</td>
									<td className="pt-1 text-right font-semibold tabular-nums">
										{score.final}
									</td>
								</tr>
							</tbody>
						</table>
					</HoverCardContent>
				</HoverCard>
				<div>
					<p
						className={cn(
							"text-2xl font-semibold leading-none",
							healthToneClass(score.tone),
						)}>
						{score.tone}
					</p>
					<p className="mt-1 text-xs text-zinc-300">
						{totalIssues === 0 ?
							"No issues require action."
						:	`${totalIssues} issue${totalIssues === 1 ? "" : "s"} can be reviewed.`
						}
					</p>
				</div>
			</div>

			<div className="mt-2 flex flex-1 flex-col border-t border-white/10 pt-1">
				<p className="text-xs font-medium text-zinc-100">Recommendations</p>
				<ul className="mt-0.5 space-y-0.5 text-[11px] text-zinc-300">
					{recommendationLines.map((line) => (
						<li key={line}>• {line}</li>
					))}
				</ul>

				<div className="mt-1.5 grid grid-cols-2 gap-x-3 gap-y-1 text-[10px] leading-tight text-zinc-300">
					<p>
						<span className="text-zinc-400">Load:</span>{" "}
						{loadAverage1m.toFixed(2)} / {loadAverage5m.toFixed(2)} /{" "}
						{loadAverage15m.toFixed(2)}
					</p>
					<p>
						<span className="text-zinc-400">Updated:</span>{" "}
						{formatCheckpointTime(lastCheckpointEpochMs)}
					</p>
					<p>
						<span className="text-zinc-400">Mem:</span>{" "}
						{formatGb(memoryTotalBytes)} total
					</p>
					<p>
						<span className="text-zinc-400">Pressure:</span>{" "}
						{pressureLabel(memoryPressurePercent)}
					</p>
					<p>
						<span className="text-zinc-400">Used/Free:</span>{" "}
						{formatGb(memoryUsedBytes)} / {formatGb(memoryFreeBytes)}
					</p>
					<p>
						<span className="text-zinc-400">Refresh:</span> every{" "}
						{autoRefreshSeconds}s
					</p>
				</div>

				{visibleCategories.length > 0 ?
					<div className="mt-1.5">
						<p className="text-[10px] uppercase tracking-wide text-zinc-400">
							Issues by category
						</p>
						<div className="mt-1 flex flex-wrap items-center gap-1 text-[10px]">
							{visibleCategories.map((category) => {
								const interactive = Boolean(
									category.firstIssueId && onJumpToIssue,
								);
								const inner = (
									<span className="inline-flex items-center gap-1">
										<span
											className={cn(
												"h-1.5 w-1.5 rounded-full",
												severityDotClass(category.severity),
											)}
										/>
										<span>{category.label}</span>
										<span className="opacity-70">({category.count})</span>
									</span>
								);

								if (!interactive) {
									return (
										<span
											key={category.id}
											className={cn(
												"rounded-full border px-1.5 py-0.5",
												severityBadgeClass(category.severity),
											)}>
											{inner}
										</span>
									);
								}

								return (
									<HoverCard key={category.id} openDelay={120} closeDelay={80}>
										<HoverCardTrigger asChild>
											<button
												type="button"
												onClick={() =>
													handleCategoryClick(category.firstIssueId)
												}
												className={cn(
													"inline-flex items-center gap-1 rounded-full border px-1.5 py-0.5 transition hover:brightness-110 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-300/60",
													severityBadgeClass(category.severity),
												)}
												aria-label={`Jump to first issue in ${category.label}`}>
												{inner}
											</button>
										</HoverCardTrigger>
										<HoverCardContent
											align="start"
											className="w-64 border border-white/15 bg-zinc-900/95 text-zinc-100">
											<p className="text-xs font-semibold">{category.label}</p>
											<p className="mt-1 text-[11px] text-zinc-300">
												Click to jump to the first issue in this category.
											</p>
										</HoverCardContent>
									</HoverCard>
								);
							})}
							{hiddenCategoryCount > 0 ?
								<span className="text-[10px] text-zinc-400">
									+{hiddenCategoryCount} more
								</span>
							:	null}
						</div>
					</div>
				:	<p className="mt-1.5 text-[10px] text-zinc-400">
						No active issues. Hover the score to see how it was calculated.
					</p>
				}

				<div className="mt-auto flex items-center justify-between gap-2 pt-1">
					<div className="flex flex-wrap items-center gap-1 text-[10px] text-zinc-400">
						<span className="rounded-full border border-red-400/35 bg-red-500/10 px-1.5 py-0.5 text-red-200">
							Critical {criticalCount}
						</span>
						<span className="rounded-full border border-amber-400/35 bg-amber-500/10 px-1.5 py-0.5 text-amber-200">
							Warning {warningCount}
						</span>
						<span className="rounded-full border border-cyan-400/35 bg-cyan-500/10 px-1.5 py-0.5 text-cyan-200">
							Info {infoCount}
						</span>
					</div>

					<button
						type="button"
						onClick={handleFixClick}
						disabled={totalIssues === 0 || !hasFixHandler}
						className="inline-flex items-center gap-1.5 rounded-md bg-amber-400 px-3 py-1 text-xs font-semibold text-amber-950 transition hover:bg-amber-300 disabled:cursor-not-allowed disabled:bg-white/10 disabled:text-zinc-500"
						title={
							totalIssues === 0 ? "No issues to fix" : (
								`Review top issue: ${topIssueTitle ?? ""}`
							)
						}
						aria-label={
							totalIssues === 0 ? "No issues to fix" : (
								`Review top issue: ${topIssueTitle ?? "open issue log"}`
							)
						}>
						<Wrench className="h-3 w-3" />
						{totalIssues === 0 ?
							"All clear"
						: topIssueTitle ?
							`Fix: ${truncate(topIssueTitle, 28)}`
						:	"Fix top issue"}
					</button>
				</div>
			</div>
		</section>
	);
}
