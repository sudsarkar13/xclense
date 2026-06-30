"use client";

import { useMemo, useState } from "react";
import {
	CheckCircle2,
	ChevronRight,
	Info,
	Loader2,
	Sparkles,
	Trash2,
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

	const categoryById = useMemo(() => {
		const map = new Map<string, StorageCategory>();
		for (const category of scan?.categories ?? []) {
			map.set(category.id, category);
		}
		return map;
	}, [scan?.categories]);

	const visibleItems = useMemo(() => {
		const items = scan?.items ?? [];
		if (filterCategory === "all") return items;
		return items.filter((item) => item.categoryId === filterCategory);
	}, [scan?.items, filterCategory]);

	const selectedItems = useMemo(
		() => visibleItems.filter((item) => selectedIds.has(item.id)),
		[visibleItems, selectedIds],
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
		visibleItems.length > 0 && selectedItems.length === visibleItems.length;
	const hasResults = Boolean(result && result.results.length > 0);

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
				for (const item of visibleItems) {
					next.delete(item.id);
				}
				return next;
			}
			const next = new Set(previous);
			for (const item of visibleItems) {
				next.add(item.id);
			}
			return next;
		});
	};

	const handleClose = (next: boolean): void => {
		if (!next) {
			setSelectedIds(new Set());
			setAcknowledged(false);
			setFilterCategory("all");
		}
		onOpenChange(next);
	};

	const handleRun = (): void => {
		if (selectedItems.length === 0) return;
		const request: CleanupRequest = {
			itemIds: Array.from(selectedIds),
			acknowledgedRisk: acknowledged,
			reason: "User acknowledged selected items will be moved to Trash.",
		};
		onRun(request);
	};

	const canRun = selectedItems.length > 0 && acknowledged && !hasResults;

	return (
		<Dialog open={open} onOpenChange={handleClose}>
			<DialogContent
				showCloseButton
				className="!max-w-none w-[1000px] max-w-[92vw] max-h-[80vh] overflow-y-auto border-white/15 bg-zinc-900/95 text-zinc-100 data-[state=open]:!translate-x-[calc(-50%+8.3333vw)] data-[state=closed]:!translate-x-[calc(-50%+8.3333vw)]">
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
					</div>
				</DialogHeader>

				{errorMessage ?
					<div className="rounded-md border border-red-400/40 bg-red-500/10 p-2.5 text-xs text-red-200">
						{errorMessage}
					</div>
				:	null}

				{isScanning ?
					<div className="flex items-center gap-2 rounded-md border border-white/15 bg-white/5 p-3 text-xs text-zinc-300">
						<Loader2 className="h-3.5 w-3.5 animate-spin" />
						Scanning for reclaimable space…
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
								className="rounded-md border border-white/15 bg-white/5 px-2.5 py-1 text-[11px] font-medium text-zinc-200 transition hover:bg-white/10">
								{allSelected ? "Clear selection" : "Select all visible"}
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
								const done = result?.status === "succeeded";
								return (
									<div
										key={item.id}
										className={cn(
											"flex items-center gap-3 border-b border-white/5 p-2.5 text-xs last:border-b-0",
											done ? "bg-emerald-500/5" : "hover:bg-white/5",
										)}>
										<input
											type="checkbox"
											checked={selectedIds.has(item.id)}
											onChange={() => toggleItem(item.id)}
											disabled={hasResults}
											className="h-3.5 w-3.5 rounded border-white/20 bg-white/5"
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
												:	null}
											</div>
											<p className="mt-1 truncate font-mono text-[11px] text-zinc-200">
												{item.path}
											</p>
											<p className="text-[11px] text-zinc-400">
												{item.recommendation}
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
										Per-item size is measured with <code>du -sk</code>; items
										below 5 MB are skipped from Downloads/Trash.
									</span>
								</li>
								<li className="flex items-start gap-1.5">
									<ChevronRight className="mt-0.5 h-3 w-3 shrink-0 text-zinc-400" />
									<span>
										Cleaned items are moved to Finder&apos;s Trash so they can
										be recovered.
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
									I understand selected items will be moved to Trash and can be
									restored from there.
								</span>
							</label>
						</div>
					</div>
				}

				<DialogFooter className="mt-2 flex flex-wrap items-center justify-between gap-2 rounded-lg border border-white/10 bg-[#0b1021]/85 p-2 shadow-inner shadow-black/35 sm:justify-between">
					<p className="px-1 text-xs font-medium leading-relaxed text-cyan-100">
						Reclaimable items are grouped; pick what you want to send to Trash.
					</p>
					<div className="flex flex-wrap items-center gap-2">
						<Button
							variant="ghost"
							size="sm"
							onClick={() => {
								setSelectedIds(new Set());
								setAcknowledged(false);
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
	);
}
