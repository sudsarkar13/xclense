"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
	CheckCircle2,
	ChevronRight,
	EyeOff,
	Info,
	Loader2,
	Lock,
	Radar,
	RefreshCcw,
	ShieldAlert,
	ShieldCheck,
	Sparkles,
	Trash2,
	XIcon,
} from "lucide-react";

import { cn } from "@/lib/utils";
import {
	formatBytes,
	formatGb,
	formatTimestamp,
} from "@/components/dashboard/shared";
import {
	type CleanupRequest,
	type CleanupResult,
	type StorageCategory,
	type StorageScanItem,
	type StorageScanProgressEvent,
	type PermissionStatus,
	checkFullDiskAccess,
	openFullDiskAccessSettings,
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

interface StorageCleanupOverlayProps {
	open: boolean;
	onOpenChange: (next: boolean) => void;
	scan: {
		items: StorageScanItem[];
		categories: StorageCategory[];
		totalRecoverableBytes: number;
	} | null;
	isScanning: boolean;
	isExecuting: boolean;
	errorMessage?: string | null;
	result: CleanupResult | null;
	onRun: (request: CleanupRequest) => void;
	onRescan: () => void;
}

interface CleanupProgressEvent {
	auditId: string;
	phase: "started" | "item_started" | "item_completed" | "completed";
	current: number;
	total: number;
	itemId?: string | null;
	path?: string | null;
	status?: "running" | "succeeded" | "failed" | "unknown" | null;
	message: string;
	reclaimedBytes: number;
}

const riskBadgeClass = (risk: StorageScanItem["riskLevel"]): string => {
	if (risk === "low")
		return "border-emerald-400/35 bg-emerald-500/10 text-emerald-200";
	if (risk === "medium")
		return "border-amber-400/35 bg-amber-500/10 text-amber-200";
	return "border-red-400/35 bg-red-500/10 text-red-200";
};

const resultBadgeClass = (
	status: CleanupResult["results"][number]["status"],
): string => {
	if (status === "succeeded")
		return "border-emerald-400/35 bg-emerald-500/10 text-emerald-200";
	if (status === "failed")
		return "border-red-400/35 bg-red-500/10 text-red-200";
	if (status === "skipped")
		return "border-amber-400/35 bg-amber-500/10 text-amber-200";
	return "border-white/20 bg-white/5 text-zinc-300";
};

const resultLabel = (
	status: CleanupResult["results"][number]["status"],
): string => {
	switch (status) {
		case "succeeded":
			return "Cleaned";
		case "failed":
			return "Failed";
		case "skipped":
			return "Skipped";
		default:
			return "Unknown";
	}
};

const liveStatusBadgeClass = (
	status: CleanupProgressEvent["status"],
): string => {
	if (status === "running")
		return "border-cyan-400/35 bg-cyan-500/10 text-cyan-200";
	if (status === "succeeded")
		return "border-emerald-400/35 bg-emerald-500/10 text-emerald-200";
	if (status === "failed")
		return "border-red-400/35 bg-red-500/10 text-red-200";
	return "border-white/20 bg-white/5 text-zinc-300";
};

const liveStatusLabel = (status: CleanupProgressEvent["status"]): string => {
	if (status === "running") return "Cleaning";
	if (status === "succeeded") return "Cleaned";
	if (status === "failed") return "Failed";
	return "Queued";
};

const safetyBadgeClass = (score: number): string => {
	if (score >= 70)
		return "border-emerald-400/35 bg-emerald-500/10 text-emerald-200";
	if (score >= 40) return "border-amber-400/35 bg-amber-500/10 text-amber-200";
	return "border-red-400/35 bg-red-500/10 text-red-200";
};

const safetyBarClass = (score: number): string => {
	if (score >= 70) return "bg-emerald-400";
	if (score >= 40) return "bg-amber-400";
	return "bg-red-400";
};

const shortenPath = (path: string, maxLength = 72): string => {
	if (path.length <= maxLength) return path;
	return `…${path.slice(path.length - maxLength + 1)}`;
};

const colorClass = (color: string): string => {
	switch (color) {
		case "sky":
			return "bg-sky-500";
		case "violet":
			return "bg-violet-500";
		case "emerald":
			return "bg-emerald-500";
		case "amber":
			return "bg-amber-400";
		case "rose":
			return "bg-rose-400";
		case "fuchsia":
			return "bg-fuchsia-500";
		default:
			return "bg-zinc-400";
	}
};

export function StorageCleanupOverlay({
	open,
	onOpenChange,
	scan,
	isScanning,
	isExecuting,
	errorMessage,
	result,
	onRun,
	onRescan,
}: StorageCleanupOverlayProps): React.JSX.Element {
	const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
	const [acknowledged, setAcknowledged] = useState<boolean>(false);
	const [filterCategory, setFilterCategory] = useState<string>("all");
	const [cleanupProgress, setCleanupProgress] =
		useState<CleanupProgressEvent | null>(null);
	const [liveStatusById, setLiveStatusById] = useState<
		Map<string, CleanupProgressEvent["status"]>
	>(new Map());
	const [showProgressWindow, setShowProgressWindow] = useState<boolean>(false);
	const [hiddenOnly, setHiddenOnly] = useState<boolean>(false);
	const [scanProgress, setScanProgress] =
		useState<StorageScanProgressEvent | null>(null);
	const [recentFinds, setRecentFinds] = useState<string[]>([]);
	const [permission, setPermission] = useState<PermissionStatus | null>(null);

	// Queried here rather than passed in, so granting access mid-session is
	// picked up without the parent page having to re-scan first.
	const refreshPermission = useCallback((): void => {
		void checkFullDiskAccess()
			.then(setPermission)
			.catch(() => setPermission(null));
	}, []);

	useEffect(() => {
		if (!open) return;
		refreshPermission();
	}, [open, isScanning, refreshPermission]);

	useEffect(() => {
		if (!open) return;

		let unlisten: UnlistenFn | undefined;
		void listen<StorageScanProgressEvent>("storage-scan-progress", (event) => {
			const progress = event.payload;
			setScanProgress(progress);
			if (progress.phase === "started") {
				setRecentFinds([]);
			}
			if (progress.phase === "item_found" && progress.currentPath) {
				const found = progress.currentPath;
				setRecentFinds((previous) =>
					[found, ...previous.filter((entry) => entry !== found)].slice(0, 3),
				);
			}
		}).then((dispose) => {
			unlisten = dispose;
		});

		return () => {
			unlisten?.();
		};
	}, [open]);

	useEffect(() => {
		if (!open && !isExecuting && !showProgressWindow) return;

		let unlisten: UnlistenFn | undefined;
		void listen<CleanupProgressEvent>("storage-cleanup-progress", (event) => {
			const progress = event.payload;
			setCleanupProgress(progress);
			if (progress.itemId) {
				setLiveStatusById((previous) => {
					const next = new Map(previous);
					next.set(progress.itemId ?? "", progress.status ?? "unknown");
					return next;
				});
			}
		}).then((dispose) => {
			unlisten = dispose;
		});

		return () => {
			unlisten?.();
		};
	}, [isExecuting, open, showProgressWindow]);

	const resultProgress = useMemo<CleanupProgressEvent | null>(() => {
		if (!result) return null;
		return {
			auditId: result.auditId,
			phase: "completed",
			current: result.results.length,
			total: result.results.length,
			status: result.allSucceeded ? "succeeded" : "failed",
			message:
				result.allSucceeded ? "Cleanup completed." : (
					"Cleanup completed with issues."
				),
			reclaimedBytes: result.totalReclaimedBytes,
		};
	}, [result]);

	const resultLiveStatusById = useMemo(() => {
		return new Map(
			(result?.results ?? []).map((entry) => [
				entry.itemId,
				entry.status === "skipped" ? "unknown" : entry.status,
			]),
		);
	}, [result?.results]);

	const categoryById = useMemo(() => {
		const map = new Map<string, StorageCategory>();
		for (const category of scan?.categories ?? []) {
			map.set(category.id, category);
		}
		return map;
	}, [scan?.categories]);

	const visibleItems = useMemo(() => {
		let items = scan?.items ?? [];
		if (filterCategory !== "all") {
			items = items.filter((item) => item.categoryId === filterCategory);
		}
		if (hiddenOnly) {
			items = items.filter((item) => item.hidden);
		}
		return items;
	}, [scan?.items, filterCategory, hiddenOnly]);

	const selectableVisibleItems = useMemo(
		() =>
			visibleItems.filter(
				(item) => item.riskLevel !== "high" && !item.protected,
			),
		[visibleItems],
	);

	const hiddenCount = useMemo(
		() => (scan?.items ?? []).filter((item) => item.hidden).length,
		[scan?.items],
	);

	const protectedCount = useMemo(
		() => (scan?.items ?? []).filter((item) => item.protected).length,
		[scan?.items],
	);

	const scanPercent = useMemo(() => {
		if (!scanProgress || scanProgress.totalStages === 0) return 0;
		if (scanProgress.phase === "completed") return 100;
		const raw = Math.round(
			(scanProgress.completedStages / scanProgress.totalStages) * 100,
		);
		return Math.min(99, Math.max(2, raw));
	}, [scanProgress]);

	const selectedItems = useMemo(
		() => selectableVisibleItems.filter((item) => selectedIds.has(item.id)),
		[selectableVisibleItems, selectedIds],
	);
	const selectedBytes = useMemo(
		() => selectedItems.reduce((sum, item) => sum + item.sizeBytes, 0),
		[selectedItems],
	);

	const resultsById = useMemo(() => {
		const map = new Map<string, CleanupResult["results"][number]>();
		for (const entry of result?.results ?? []) {
			map.set(entry.itemId, entry);
		}
		return map;
	}, [result]);

	const allSelected =
		selectableVisibleItems.length > 0 &&
		selectedItems.length === selectableVisibleItems.length;
	const hasResults = Boolean(result && result.results.length > 0);
	const activeCleanupProgress = resultProgress ?? cleanupProgress;
	const activeLiveStatusById =
		hasResults ? resultLiveStatusById : liveStatusById;
	const cleanupPercent =
		activeCleanupProgress?.total ?
			Math.min(
				100,
				Math.round(
					(activeCleanupProgress.current / activeCleanupProgress.total) * 100,
				),
			)
		:	0;

	const toggleItem = (id: string): void => {
		setSelectedIds((previous) => {
			const next = new Set(previous);
			if (next.has(id)) {
				next.delete(id);
			} else {
				next.add(id);
			}
			return next;
		});
	};

	const toggleAll = (): void => {
		setSelectedIds((previous) => {
			if (allSelected) {
				const next = new Set(previous);
				for (const item of selectableVisibleItems) {
					next.delete(item.id);
				}
				return next;
			}
			const next = new Set(previous);
			for (const item of selectableVisibleItems) {
				next.add(item.id);
			}
			return next;
		});
	};

	const handleClose = (next: boolean): void => {
		if (!next && !isExecuting) {
			setSelectedIds(new Set());
			setAcknowledged(false);
			setFilterCategory("all");
			setCleanupProgress(null);
			setLiveStatusById(new Map());
			setShowProgressWindow(false);
			setScanProgress(null);
			setRecentFinds([]);
			setHiddenOnly(false);
		}
		onOpenChange(next);
	};

	const handleRun = (): void => {
		if (selectedItems.length === 0) return;
		setShowProgressWindow(true);
		setCleanupProgress({
			auditId: "pending",
			phase: "started",
			current: 0,
			total: selectedItems.length,
			message: `Preparing to move ${selectedItems.length} selected item(s) to Trash…`,
			reclaimedBytes: 0,
		});
		setLiveStatusById(
			new Map(selectedItems.map((item) => [item.id, "unknown" as const])),
		);
		const request: CleanupRequest = {
			itemIds: Array.from(selectedIds),
			acknowledgedRisk: acknowledged,
			reason: "User acknowledged selected items will be moved to Trash.",
		};
		onRun(request);
	};

	const canRun = selectedItems.length > 0 && acknowledged && !hasResults;

	useEffect(() => {
		if (
			isExecuting ||
			!showProgressWindow ||
			activeCleanupProgress?.phase !== "completed"
		) {
			return;
		}

		const timer = window.setTimeout(() => {
			setShowProgressWindow(false);
		}, 4500);

		return () => window.clearTimeout(timer);
	}, [activeCleanupProgress?.phase, isExecuting, showProgressWindow]);

	return (
		<>
			<Dialog open={open} onOpenChange={handleClose}>
				<DialogContent
					showCloseButton
					className="!max-w-none w-[1000px] max-w-[92vw] max-h-[80vh] overflow-y-auto border-white/15 bg-zinc-900/95 text-zinc-100 shadow-2xl shadow-cyan-950/20 duration-350 ease-[cubic-bezier(0.16,1,0.3,1)] data-[state=open]:!translate-x-[calc(-50%+8.3333vw)] data-[state=closed]:!translate-x-[calc(-50%+8.3333vw)] data-[state=open]:slide-in-from-bottom-2 data-[state=closed]:slide-out-to-bottom-2">
					<DialogHeader className="space-y-1.5">
						<div className="flex items-center gap-2">
							<Sparkles className="h-4 w-4 text-cyan-300" />
							<DialogTitle className="text-base">
								Scan &amp; clean storage
							</DialogTitle>
						</div>
						<DialogDescription className="text-xs text-zinc-300">
							Review reclaimable files grouped by category. Items are sent to
							Trash so you can restore them if needed.
						</DialogDescription>
						<div className="mt-1 flex flex-wrap items-center gap-3 text-[11px] text-zinc-300">
							<span>
								<span className="text-zinc-400">Items found:</span>{" "}
								<span className="font-semibold">{scan?.items.length ?? 0}</span>
							</span>
							<span>
								<span className="text-zinc-400">Reclaimable:</span>{" "}
								<span className="font-semibold">
									{formatBytes(scan?.totalRecoverableBytes ?? 0)}
								</span>
							</span>
							<span>
								<span className="text-zinc-400">Selected:</span>{" "}
								<span className="font-semibold">
									{selectedItems.length} · {formatBytes(selectedBytes)}
								</span>
							</span>
							<span>
								<span className="text-zinc-400">Hidden:</span>{" "}
								<span className="font-semibold">{hiddenCount}</span>
							</span>
							<span>
								<span className="text-zinc-400">Protected:</span>{" "}
								<span className="font-semibold">{protectedCount}</span>
							</span>
						</div>
					</DialogHeader>

					{errorMessage ?
						<div className="rounded-md border border-red-400/40 bg-red-500/10 p-2.5 text-xs text-red-200">
							{errorMessage}
						</div>
					:	null}

					{permission && !permission.fullDiskAccess ?
						<div className="rounded-md border border-amber-400/35 bg-amber-500/10 p-2.5 text-xs text-amber-100">
							<div className="flex items-start gap-2">
								<ShieldAlert className="mt-0.5 h-3.5 w-3.5 shrink-0 text-amber-300" />
								<div className="min-w-0 flex-1">
									<p className="font-semibold">
										Grant Full Disk Access once to scan everything
									</p>
									<p className="mt-0.5 text-[11px] text-amber-100/80">
										Xclense is skipping{" "}
										{permission.protectedLocationCount.toLocaleString()} app
										locations, plus Downloads, Desktop, Documents, Movies and
										Pictures. macOS guards each of those separately and asks
										every time, so Xclense skips them rather than interrupting
										your scan with prompts. Full Disk Access replaces all of
										them with a single approval — nothing is asked again.
									</p>
									<div className="mt-2 flex flex-wrap items-center gap-2">
										<button
											type="button"
											onClick={() => void openFullDiskAccessSettings()}
											className="inline-flex items-center gap-1.5 rounded-md border border-amber-300/40 bg-amber-400/15 px-2.5 py-1 text-[11px] font-semibold text-amber-50 transition hover:bg-amber-400/25">
											<ShieldCheck className="h-3 w-3" />
											Open Privacy Settings
										</button>
										<button
											type="button"
											onClick={() => void refreshPermission()}
											className="rounded-md border border-white/15 bg-white/5 px-2.5 py-1 text-[11px] font-medium text-amber-50 transition hover:bg-white/10">
											I&apos;ve granted it — re-check
										</button>
									</div>
									<p className="mt-1.5 text-[10px] text-amber-100/60">
										Add Xclense under Privacy &amp; Security ➔ Full Disk Access,
										then re-check. macOS may ask you to quit and reopen Xclense.
									</p>
								</div>
							</div>
						</div>
					:	null}

					{isScanning ?
						<div className="xc-scan-panel relative overflow-hidden rounded-lg border border-cyan-300/25 bg-cyan-950/35 p-3 text-xs text-cyan-100 shadow-inner shadow-black/25">
							<div className="flex items-start gap-3">
								<span className="relative mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center">
									<span className="absolute inset-0 animate-ping rounded-full border border-cyan-300/40" />
									<span className="absolute inset-1 rounded-full border border-cyan-300/25" />
									<span className="flex h-9 w-9 items-center justify-center rounded-full border border-cyan-300/35 bg-cyan-400/10">
										<Radar className="xc-spin-slow h-4 w-4 text-cyan-200" />
									</span>
								</span>
								<div className="min-w-0 flex-1">
									<div className="flex flex-wrap items-baseline justify-between gap-2">
										<span className="font-semibold">
											Deep scanning storage locations…
										</span>
										<span className="tabular-nums text-[11px] text-cyan-100/80">
											Stage {Math.max(1, scanProgress?.completedStages ?? 1)} of{" "}
											{scanProgress?.totalStages ?? 14} · {scanPercent}%
										</span>
									</div>
									<p className="mt-0.5 truncate text-[11px] text-cyan-100/80">
										{scanProgress?.categoryLabel ?
											`Checking ${scanProgress.categoryLabel}`
										:	"Checking caches, hidden dot-folders, developer artifacts, package manager data, node_modules, build caches, and large files."
										}
									</p>
								</div>
							</div>

							<div className="mt-3 h-1.5 overflow-hidden rounded-full bg-cyan-950/80">
								<div
									className="xc-progress-sheen h-full rounded-full bg-gradient-to-r from-cyan-400 via-sky-300 to-emerald-300 transition-all duration-500 ease-out"
									style={{ width: `${Math.max(scanPercent, 4)}%` }}
								/>
							</div>

							<div className="xc-scan-window relative mt-2.5 overflow-hidden rounded-md border border-cyan-300/15 bg-black/30 px-2 py-1.5">
								<p className="truncate font-mono text-[11px] leading-relaxed text-cyan-100/90">
									{scanProgress?.currentPath ?
										shortenPath(scanProgress.currentPath, 90)
									:	"Preparing scan…"}
								</p>
							</div>

							<div className="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 text-[11px] text-cyan-100/80">
								<span>
									<span className="text-cyan-100/55">Locations read:</span>{" "}
									<span className="font-semibold tabular-nums">
										{(scanProgress?.scannedPaths ?? 0).toLocaleString()}
									</span>
								</span>
								<span>
									<span className="text-cyan-100/55">Items found:</span>{" "}
									<span className="font-semibold tabular-nums">
										{scanProgress?.itemsFound ?? 0}
									</span>
								</span>
								<span>
									<span className="text-cyan-100/55">Reclaimable so far:</span>{" "}
									<span className="font-semibold tabular-nums">
										{formatBytes(scanProgress?.reclaimableBytes ?? 0)}
									</span>
								</span>
							</div>

							{recentFinds.length > 0 ?
								<ul className="mt-2 space-y-0.5">
									{recentFinds.map((path, index) => (
										<li
											key={path}
											className={cn(
												"flex items-center gap-1.5 truncate font-mono text-[10px]",
												index === 0 ? "text-emerald-200" : "text-cyan-100/45",
											)}>
											<CheckCircle2 className="h-3 w-3 shrink-0" />
											<span className="truncate">{shortenPath(path, 84)}</span>
										</li>
									))}
								</ul>
							:	null}
						</div>
					: !scan ?
						<p className="text-xs text-zinc-300">
							No scan has been run yet. Click Re-scan to start.
						</p>
					: scan.items.length === 0 ?
						<p className="text-xs text-zinc-300">
							Nothing obviously reclaimable right now — your disk looks healthy.
						</p>
					:	<div className="space-y-3">
							<div className="flex flex-wrap items-center gap-2">
								<button
									type="button"
									onClick={toggleAll}
									disabled={selectableVisibleItems.length === 0}
									className="rounded-md border border-white/15 bg-white/5 px-2.5 py-1 text-[11px] font-medium text-zinc-200 transition hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-45">
									{allSelected ? "Clear selection" : "Select all safe visible"}
								</button>
								<button
									type="button"
									onClick={() => setHiddenOnly((previous) => !previous)}
									className={cn(
										"inline-flex items-center gap-1.5 rounded-md border px-2.5 py-1 text-[11px] font-medium transition",
										hiddenOnly ?
											"border-cyan-300/40 bg-cyan-500/15 text-cyan-100"
										:	"border-white/15 bg-white/5 text-zinc-200 hover:bg-white/10",
									)}>
									<EyeOff className="h-3 w-3" />
									Hidden only ({hiddenCount})
								</button>
								<div className="flex flex-wrap items-center gap-1.5">
									<button
										type="button"
										onClick={() => setFilterCategory("all")}
										className={cn(
											"rounded-full border px-2 py-0.5 text-[11px] transition",
											filterCategory === "all" ?
												"border-cyan-300/40 bg-cyan-500/15 text-cyan-100"
											:	"border-white/15 bg-white/5 text-zinc-300 hover:bg-white/10",
										)}>
										All categories
									</button>
									{scan.categories.map((category) => {
										const isActive = filterCategory === category.id;
										return (
											<button
												key={category.id}
												type="button"
												onClick={() => setFilterCategory(category.id)}
												className={cn(
													"inline-flex items-center gap-1.5 rounded-full border px-2 py-0.5 text-[11px] transition",
													isActive ?
														"border-cyan-300/40 bg-cyan-500/15 text-cyan-100"
													:	"border-white/15 bg-white/5 text-zinc-300 hover:bg-white/10",
												)}>
												<span
													className={cn(
														"h-1.5 w-1.5 rounded-full",
														colorClass(category.color),
													)}
												/>
												{category.label}
											</button>
										);
									})}
								</div>
							</div>

							<div className="max-h-[42vh] overflow-y-auto rounded-md border border-white/10 bg-white/5">
								{visibleItems.map((item) => {
									const category = categoryById.get(item.categoryId);
									const result = resultsById.get(item.id);
									const liveStatus = activeLiveStatusById.get(item.id);
									const done =
										result?.status === "succeeded" ||
										liveStatus === "succeeded";
									const running = liveStatus === "running";
									const isProtectedHighRisk =
										item.riskLevel === "high" || item.protected;
									return (
										<div
											key={item.id}
											className={cn(
												"flex items-center gap-3 border-b border-white/5 p-2.5 text-xs transition-colors last:border-b-0",
												done ? "bg-emerald-500/5"
												: running ? "bg-cyan-500/10"
												: "hover:bg-white/5",
											)}>
											<input
												type="checkbox"
												checked={
													!isProtectedHighRisk && selectedIds.has(item.id)
												}
												onChange={() => toggleItem(item.id)}
												disabled={hasResults || isProtectedHighRisk}
												className="h-3.5 w-3.5 rounded border-white/20 bg-white/5 disabled:cursor-not-allowed disabled:opacity-40"
												title={
													isProtectedHighRisk ? item.recommendation : undefined
												}
												aria-label={`Select ${item.path}`}
											/>
											<div className="min-w-0 flex-1">
												<div className="flex flex-wrap items-center gap-1.5">
													{category ?
														<span
															className={cn(
																"inline-flex items-center gap-1 rounded-full border px-1.5 py-0.5 text-[10px]",
																"border-white/15 bg-white/5 text-zinc-200",
															)}>
															<span
																className={cn(
																	"h-1.5 w-1.5 rounded-full",
																	colorClass(category.color),
																)}
															/>
															{category.label}
														</span>
													:	null}
													<span
														className={cn(
															"rounded-full border px-1.5 py-0.5 text-[10px]",
															riskBadgeClass(item.riskLevel),
														)}>
														{item.riskLevel} risk
													</span>
													<span
														className={cn(
															"inline-flex items-center gap-1 rounded-full border px-1.5 py-0.5 text-[10px] tabular-nums",
															safetyBadgeClass(item.safetyScore),
														)}
														title="Safe-to-remove score: higher means the data is regenerated or non-essential.">
														<span className="inline-block h-1 w-6 overflow-hidden rounded-full bg-black/40">
															<span
																className={cn(
																	"block h-full rounded-full",
																	safetyBarClass(item.safetyScore),
																)}
																style={{ width: `${item.safetyScore}%` }}
															/>
														</span>
														safe {item.safetyScore}
													</span>
													{item.hidden ?
														<span className="inline-flex items-center gap-1 rounded-full border border-cyan-300/30 bg-cyan-500/10 px-1.5 py-0.5 text-[10px] text-cyan-100">
															<EyeOff className="h-2.5 w-2.5" />
															hidden
														</span>
													:	null}
													{item.protected ?
														<span className="inline-flex items-center gap-1 rounded-full border border-red-400/35 bg-red-500/10 px-1.5 py-0.5 text-[10px] text-red-200">
															<Lock className="h-2.5 w-2.5" />
															protected
														</span>
													:	null}
													{item.regenerates ?
														<span className="inline-flex items-center gap-1 rounded-full border border-emerald-400/30 bg-emerald-500/10 px-1.5 py-0.5 text-[10px] text-emerald-200">
															<RefreshCcw className="h-2.5 w-2.5" />
															rebuilds itself
														</span>
													:	null}
													<span className="text-[11px] text-zinc-400">
														{formatTimestamp(item.modifiedEpochMs)}
													</span>
													{result ?
														<span
															className={cn(
																"rounded-full border px-1.5 py-0.5 text-[10px]",
																resultBadgeClass(result.status),
															)}>
															{result.status === "succeeded" ?
																<CheckCircle2 className="mr-0.5 inline h-3 w-3" />
															: result.status === "failed" ?
																<Info className="mr-0.5 inline h-3 w-3" />
															:	null}
															{resultLabel(result.status)}
														</span>
													: liveStatus ?
														<span
															className={cn(
																"rounded-full border px-1.5 py-0.5 text-[10px]",
																liveStatusBadgeClass(liveStatus),
															)}>
															{liveStatus === "running" ?
																<Loader2 className="mr-0.5 inline h-3 w-3 animate-spin" />
															: liveStatus === "succeeded" ?
																<CheckCircle2 className="mr-0.5 inline h-3 w-3" />
															:	null}
															{liveStatusLabel(liveStatus)}
														</span>
													:	null}
												</div>
												<p className="mt-1 truncate text-[11px] font-semibold text-zinc-100">
													{item.label}
													<span className="ml-1.5 font-normal text-zinc-400">
														{item.owner} · {item.entryKind}
														{item.identified ? "" : " · unidentified"}
													</span>
												</p>
												<p className="truncate font-mono text-[11px] text-zinc-200">
													{item.path}
												</p>
												<p className="text-[11px] text-zinc-400">
													{item.recommendation}
												</p>
												<p className="text-[11px] text-amber-200/70">
													<span className="text-zinc-400">If removed:</span>{" "}
													{item.impactIfRemoved}
												</p>
												{result ?
													<p className="mt-0.5 text-[11px] text-zinc-300">
														<span className="text-zinc-400">Result:</span>{" "}
														{result.message}{" "}
														{result.reclaimedBytes > 0 ?
															<span className="text-emerald-300">
																(reclaimed {formatBytes(result.reclaimedBytes)})
															</span>
														:	null}
													</p>
												:	null}
											</div>
											<div className="shrink-0 text-right">
												<p className="text-sm font-semibold tabular-nums">
													{formatBytes(item.sizeBytes)}
												</p>
												<p className="text-[10px] uppercase tracking-wide text-zinc-400">
													{formatGb(item.sizeBytes)}
												</p>
											</div>
										</div>
									);
								})}
							</div>

							<details className="rounded-md border border-white/10 bg-white/5 p-3 text-[11px] text-zinc-300">
								<summary className="cursor-pointer text-cyan-300 hover:text-cyan-200">
									How is reclaimable space calculated?
								</summary>
								<ol className="mt-1.5 space-y-1">
									<li className="flex items-start gap-1.5">
										<ChevronRight className="mt-0.5 h-3 w-3 shrink-0 text-zinc-400" />
										<span>
											Categories map to known safe-to-clear locations (caches,
											logs, Downloads, Trash, etc.).
										</span>
									</li>
									<li className="flex items-start gap-1.5">
										<ChevronRight className="mt-0.5 h-3 w-3 shrink-0 text-zinc-400" />
										<span>
											Every dot-file and dot-folder in your home root is listed
											— at any size — plus hidden folders under ~/Library and
											hidden build caches inside projects.
										</span>
									</li>
									<li className="flex items-start gap-1.5">
										<ChevronRight className="mt-0.5 h-3 w-3 shrink-0 text-zinc-400" />
										<span>
											Each entry is matched against a catalogue of known tools
											to show its owner and what breaks if it is missing.
											Unmatched hidden files stay marked <em>unidentified</em>.
										</span>
									</li>
									<li className="flex items-start gap-1.5">
										<ChevronRight className="mt-0.5 h-3 w-3 shrink-0 text-zinc-400" />
										<span>
											The safe score combines risk level, whether the tool
											rebuilds the data, and how recently it was modified.
										</span>
									</li>
									<li className="flex items-start gap-1.5">
										<ChevronRight className="mt-0.5 h-3 w-3 shrink-0 text-zinc-400" />
										<span>
											Per-item size is measured with <code>du -sk</code>; nested
											paths are never counted twice in the total.
										</span>
									</li>
									<li className="flex items-start gap-1.5">
										<ChevronRight className="mt-0.5 h-3 w-3 shrink-0 text-zinc-400" />
										<span>
											Cleaned items are moved to Finder&apos;s Trash so they can
											be recovered.
										</span>
									</li>
									<li className="flex items-start gap-1.5">
										<ChevronRight className="mt-0.5 h-3 w-3 shrink-0 text-zinc-400" />
										<span>
											Credential and configuration paths (.ssh, .gnupg, .aws,
											.kube, .config, .local, shell rc files) and Cline
											directories are protected — visible, but never selectable.
										</span>
									</li>
								</ol>
							</details>

							<div className="rounded-md border border-white/10 bg-white/5 p-3">
								<label className="inline-flex items-center gap-2 text-[11px] text-zinc-300">
									<input
										type="checkbox"
										checked={acknowledged}
										onChange={(event) => setAcknowledged(event.target.checked)}
										className="h-3.5 w-3.5 rounded border-white/20 bg-white/5"
									/>
									<span>
										I understand selected items will be moved to Trash and can
										be restored from there.
									</span>
								</label>
							</div>
						</div>
					}

					<DialogFooter className="mt-2 flex flex-wrap items-center justify-between gap-2 rounded-lg border border-white/10 bg-[#0b1021]/85 p-2 shadow-inner shadow-black/35 sm:justify-between">
						<p className="px-1 text-xs font-medium leading-relaxed text-cyan-100">
							Reclaimable items are grouped; pick what you want to send to
							Trash.
						</p>
						<div className="flex flex-wrap items-center gap-2">
							<Button
								variant="ghost"
								size="sm"
								onClick={() => {
									setSelectedIds(new Set());
									setAcknowledged(false);
									setCleanupProgress(null);
									setLiveStatusById(new Map());
									setScanProgress(null);
									setRecentFinds([]);
									onRescan();
								}}
								disabled={isExecuting || isScanning}>
								<Loader2
									className={cn(
										"mr-1 h-3.5 w-3.5",
										isScanning ? "animate-spin" : "hidden",
									)}
								/>
								Re-scan
							</Button>
							<Button
								size="sm"
								onClick={handleRun}
								disabled={!canRun || isExecuting}>
								{isExecuting ?
									<Loader2 className="mr-1 h-3.5 w-3.5 animate-spin" />
								:	<Trash2 className="mr-1 h-3.5 w-3.5" />}
								Clean {selectedItems.length || ""} item
								{selectedItems.length === 1 ? "" : "s"}
							</Button>
						</div>
					</DialogFooter>
				</DialogContent>
			</Dialog>

			{showProgressWindow && activeCleanupProgress ?
				<div className="fixed right-6 bottom-6 z-[80] w-[420px] max-w-[calc(100vw-3rem)] rounded-2xl border border-cyan-300/25 bg-[#071426]/95 p-5 text-cyan-50 shadow-2xl shadow-black/50 ring-1 ring-white/10 backdrop-blur-md">
					<div className="flex items-start justify-between gap-4">
						<div className="flex items-center gap-3">
							<div className="flex h-10 w-10 items-center justify-center rounded-full border border-cyan-300/30 bg-cyan-400/10">
								{isExecuting ?
									<Loader2 className="h-5 w-5 animate-spin text-cyan-200" />
								:	<CheckCircle2 className="h-5 w-5 text-emerald-300" />}
							</div>
							<div>
								<h3 className="text-sm font-semibold">
									{isExecuting ?
										"Cleaning selected items"
									:	"Cleanup completed"}
								</h3>
								<p className="mt-0.5 text-[11px] text-cyan-100/75">
									Items are moved to Finder Trash, not permanently deleted.
								</p>
							</div>
						</div>
						<button
							type="button"
							onClick={() => setShowProgressWindow(false)}
							disabled={isExecuting}
							className="rounded-full p-1 text-cyan-100/70 transition hover:bg-white/10 hover:text-cyan-50 disabled:cursor-not-allowed disabled:opacity-35"
							aria-label="Close cleanup progress window">
							<XIcon className="h-4 w-4" />
						</button>
					</div>
					<div className="mt-4 flex items-center justify-between gap-3 text-xs">
						<span className="font-semibold tabular-nums text-cyan-100">
							{activeCleanupProgress.current}/{activeCleanupProgress.total} ·{" "}
							{cleanupPercent}%
						</span>
						<span className="text-cyan-100/70">
							{isExecuting ? "Running" : "Done"}
						</span>
					</div>
					<div className="mt-2 h-2.5 overflow-hidden rounded-full bg-cyan-950/80">
						<div
							className="h-full rounded-full bg-gradient-to-r from-cyan-300 via-sky-400 to-emerald-300 transition-all duration-300"
							style={{ width: `${cleanupPercent}%` }}
						/>
					</div>
					<p className="mt-3 text-xs leading-relaxed text-cyan-100/90">
						{activeCleanupProgress.message}
					</p>
					{activeCleanupProgress.path ?
						<p className="mt-2 max-h-16 overflow-hidden rounded-lg border border-white/10 bg-black/25 p-2 font-mono text-[11px] leading-relaxed text-zinc-200">
							{activeCleanupProgress.path}
						</p>
					:	null}
					<div className="mt-4 flex flex-wrap items-center justify-between gap-2 text-[11px] text-cyan-100/80">
						<span>
							Reclaimed so far:{" "}
							{formatBytes(activeCleanupProgress.reclaimedBytes)}
						</span>
						{isExecuting ?
							<span>You can keep using Xclense while cleanup runs.</span>
						:	<span>This window will close automatically.</span>}
					</div>
				</div>
			:	null}
		</>
	);
}
