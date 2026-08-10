"use client";

import Link from "next/link";
import { useCallback, useEffect, useMemo, useState } from "react";
import {
	ArrowLeft,
	HardDrive,
	RefreshCw,
	Sparkles,
	AlertTriangle,
} from "lucide-react";

import { DashboardHeader } from "@/components/dashboard/layout/DashboardHeader";
import { DashboardNav } from "@/components/dashboard/layout/DashboardNav";
import { useBusyWhile } from "@/lib/app-busy";
import { StorageCleanupOverlay } from "@/components/dashboard/storage/StorageCleanupOverlay";
import { cn } from "@/lib/utils";
import {
	formatBytes,
	formatGb,
	formatTimestamp,
	severityBadgeClass,
} from "@/components/dashboard/shared";
import {
	type CleanupRequest,
	type CleanupResult,
	type PhysicalDisk,
	type StorageDetail,
	type StorageScanResult,
	type VolumeInfo,
	cleanupStorageItems,
	getStorageDetail,
	isTauriRuntime,
	scanStorageForCleanup,
} from "@/lib/tauri-client";
import {
	HoverCard,
	HoverCardContent,
	HoverCardTrigger,
} from "@/components/ui/hover-card";

const diskKindBadgeClass = (kind: string): string => {
	const lower = kind.toLowerCase();
	if (
		lower.includes("nvme") ||
		lower.includes("internal ssd") ||
		lower.includes("ssd")
	) {
		return "border-emerald-400/35 bg-emerald-500/10 text-emerald-200";
	}
	if (lower.includes("fusion")) {
		return "border-violet-400/35 bg-violet-500/10 text-violet-200";
	}
	if (lower.includes("hdd") || lower.includes("rotational")) {
		return "border-amber-400/35 bg-amber-500/10 text-amber-200";
	}
	return "border-zinc-400/35 bg-zinc-500/10 text-zinc-200";
};

const volumeBarColor = (usedPercent: number): string => {
	if (usedPercent >= 90) return "bg-red-400";
	if (usedPercent >= 75) return "bg-amber-400";
	if (usedPercent >= 50) return "bg-sky-400";
	return "bg-emerald-400";
};

export default function StoragePage(): React.JSX.Element {
	const [detail, setDetail] = useState<StorageDetail | null>(null);
	const [scan, setScan] = useState<StorageScanResult | null>(null);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);
	const [isLoadingDetail, setIsLoadingDetail] = useState<boolean>(false);
	const [isScanning, setIsScanning] = useState<boolean>(false);
	const [overlayOpen, setOverlayOpen] = useState<boolean>(false);
	const [cleanupResult, setCleanupResult] = useState<CleanupResult | null>(
		null,
	);
	const [cleanupError, setCleanupError] = useState<string | null>(null);
	const [isExecuting, setIsExecuting] = useState<boolean>(false);

	// Hold off an update relaunch while files are being moved to Trash, so a
	// cleanup is never killed part-way through.
	useBusyWhile(isExecuting, "a storage cleanup");
	useBusyWhile(isScanning, "a storage scan");

	const loadDetail = useCallback(async (): Promise<void> => {
		if (!isTauriRuntime()) {
			setErrorMessage(
				"Tauri runtime is not available. Open this page in the Xclense desktop app.",
			);
			return;
		}
		try {
			setIsLoadingDetail(true);
			setErrorMessage(null);
			const result = await getStorageDetail();
			setDetail(result);
		} catch (error) {
			const message =
				error instanceof Error ? error.message : "Unknown storage load error";
			setErrorMessage(`Failed to load storage details: ${message}`);
		} finally {
			setIsLoadingDetail(false);
		}
	}, []);

	const runScan = useCallback(async (): Promise<void> => {
		if (!isTauriRuntime()) {
			return;
		}
		try {
			setIsScanning(true);
			setErrorMessage(null);
			const result = await scanStorageForCleanup();
			setScan(result);
		} catch (error) {
			const message =
				error instanceof Error ? error.message : "Unknown scan error";
			setErrorMessage(`Failed to scan storage: ${message}`);
		} finally {
			setIsScanning(false);
		}
	}, []);

	useEffect(() => {
		const timer = window.setTimeout(() => {
			void loadDetail();
		}, 0);
		return () => window.clearTimeout(timer);
	}, [loadDetail]);

	const handleOpenOverlay = useCallback((): void => {
		setCleanupResult(null);
		setCleanupError(null);
		setOverlayOpen(true);
		if (!scan) {
			void runScan();
		}
	}, [scan, runScan]);

	const handleRunCleanup = useCallback(
		async (request: CleanupRequest): Promise<void> => {
			try {
				setIsExecuting(true);
				setCleanupError(null);
				const result = await cleanupStorageItems(request);
				setCleanupResult(result);
				await loadDetail();
			} catch (error) {
				const message =
					error instanceof Error ? error.message : "Unknown cleanup error";
				setCleanupError(message);
			} finally {
				setIsExecuting(false);
			}
		},
		[loadDetail],
	);

	const handleRescan = useCallback((): void => {
		setCleanupResult(null);
		void runScan();
	}, [runScan]);

	const totalPhysicalBytes = useMemo(
		() =>
			(detail?.physicalDisks ?? []).reduce(
				(sum, disk) => sum + disk.sizeBytes,
				0,
			),
		[detail?.physicalDisks],
	);

	const volumesByFilesystem = useMemo(() => {
		const map = new Map<string, VolumeInfo[]>();
		for (const volume of detail?.volumes ?? []) {
			const list = map.get(volume.filesystem) ?? [];
			list.push(volume);
			map.set(volume.filesystem, list);
		}
		return Array.from(map.entries());
	}, [detail?.volumes]);

	const reclaimableCount = scan?.items.length ?? 0;
	const reclaimableBytes = scan?.totalRecoverableBytes ?? 0;

	return (
		<div className="h-screen w-screen overflow-hidden bg-[radial-gradient(circle_at_top_right,#3347ad_0%,#11152f_40%,#0a0f24_100%)] font-sans text-zinc-100">
			<div className="grid h-full min-h-0 w-full grid-cols-12 border border-white/15 bg-[#0d1226]/90 shadow-2xl">
				<DashboardNav />

				<main className="col-span-12 min-h-0 overflow-auto p-3 md:col-span-9 lg:col-span-10 md:p-4">
					<DashboardHeader />

					{errorMessage ?
						<div className="mb-3 rounded-lg border border-red-400/40 bg-red-500/10 p-3 text-sm text-red-200">
							{errorMessage}
						</div>
					:	null}

					<section className="mb-3 flex flex-wrap items-center gap-3 rounded-xl border border-white/15 bg-white/5 p-3">
						<Link
							href="/"
							className="inline-flex items-center gap-1.5 rounded-md border border-white/15 bg-white/10 px-2.5 py-1 text-xs font-medium text-zinc-200 transition hover:bg-white/15">
							<ArrowLeft className="h-3.5 w-3.5" />
							Back to dashboard
						</Link>
						<div className="flex flex-col">
							<div className="inline-flex items-center gap-2">
								<HardDrive className="h-4 w-4 text-cyan-300" />
								<h1 className="text-sm font-semibold">Storage</h1>
							</div>
							{detail ?
								<p className="text-[11px] text-zinc-400">
									{detail.macModel} · {detail.architecture} · macOS{" "}
									{detail.macosVersion}
								</p>
							:	<p className="text-[11px] text-zinc-400">
									Loading device details…
								</p>
							}
						</div>

						<div className="ml-auto flex flex-wrap items-center gap-2">
							<button
								type="button"
								onClick={() => void loadDetail()}
								disabled={isLoadingDetail}
								className="inline-flex items-center gap-1.5 rounded-md border border-white/15 bg-white/10 px-2.5 py-1 text-xs font-medium text-zinc-200 transition hover:bg-white/15 disabled:cursor-not-allowed disabled:opacity-60">
								<RefreshCw
									className={cn(
										"h-3.5 w-3.5",
										isLoadingDetail && "animate-spin",
									)}
								/>
								{isLoadingDetail ? "Refreshing…" : "Refresh"}
							</button>
							<button
								type="button"
								onClick={handleOpenOverlay}
								disabled={isScanning}
								className="inline-flex items-center gap-1.5 rounded-md border border-cyan-300/40 bg-cyan-400/15 px-3 py-1 text-xs font-semibold text-cyan-100 transition hover:bg-cyan-400/25 disabled:cursor-not-allowed disabled:opacity-60">
								<Sparkles className="h-3.5 w-3.5" />
								{isScanning ?
									"Scanning…"
								: scan ?
									"Open scan & clean"
								:	"Scan for cleanup"}
							</button>
						</div>
					</section>

					<section className="mb-3 grid gap-3 md:grid-cols-3">
						<div className="rounded-xl border border-white/15 bg-white/5 p-3">
							<p className="text-[11px] uppercase tracking-wide text-zinc-400">
								Total capacity
							</p>
							<p className="mt-1 text-lg font-semibold tabular-nums">
								{formatGb(totalPhysicalBytes)}
							</p>
							<p className="text-[11px] text-zinc-400">
								Across {detail?.physicalDisks.length ?? 0} physical disk
								{(detail?.physicalDisks.length ?? 0) === 1 ? "" : "s"}
							</p>
						</div>
						<div className="rounded-xl border border-white/15 bg-white/5 p-3">
							<p className="text-[11px] uppercase tracking-wide text-zinc-400">
								Free space
							</p>
							<p className="mt-1 text-lg font-semibold tabular-nums text-emerald-300">
								{formatGb(detail?.summary.freeBytes ?? 0)}
							</p>
							<p className="text-[11px] text-zinc-400">
								Used {formatGb(detail?.summary.usedBytes ?? 0)} of{" "}
								{formatGb(detail?.summary.totalBytes ?? 0)}
							</p>
						</div>
						<div className="rounded-xl border border-white/15 bg-white/5 p-3">
							<p className="text-[11px] uppercase tracking-wide text-zinc-400">
								Reclaimable
							</p>
							<p className="mt-1 text-lg font-semibold tabular-nums text-amber-300">
								{scan ? formatBytes(reclaimableBytes) : "—"}
							</p>
							<p className="text-[11px] text-zinc-400">
								{scan ?
									`${reclaimableCount} item${reclaimableCount === 1 ? "" : "s"} found across ${scan.categories.length} categories`
								:	"Run a scan to identify safe-to-clear items"}
							</p>
						</div>
					</section>

					<section className="mb-3 rounded-xl border border-white/15 bg-white/5 p-3">
						<div className="mb-2 flex items-center justify-between">
							<h2 className="text-xs font-semibold">Physical disks</h2>
							<HoverCard openDelay={120} closeDelay={80}>
								<HoverCardTrigger asChild>
									<button
										type="button"
										className="inline-flex h-5 w-5 items-center justify-center rounded-full border border-white/20 bg-white/5 text-zinc-300 transition hover:bg-white/10 hover:text-zinc-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-300/60"
										aria-label="What physical disks shows">
										<AlertTriangle className="h-3.5 w-3.5" />
									</button>
								</HoverCardTrigger>
								<HoverCardContent
									align="start"
									className="w-72 border border-white/15 bg-zinc-900/95 text-zinc-100">
									<p className="text-xs font-semibold">Physical disks</p>
									<p className="mt-1 text-[11px] leading-relaxed text-zinc-300">
										Lists every physical disk reported by <code>diskutil</code>.
										The total capacity here excludes APFS volume overlap and is
										the authoritative number for &quot;how big is this
										Mac&quot;.
									</p>
								</HoverCardContent>
							</HoverCard>
						</div>

						{(detail?.physicalDisks ?? []).length === 0 ?
							<p className="text-xs text-zinc-300">
								No physical disks reported.
							</p>
						:	<div className="overflow-x-auto">
								<table className="w-full text-xs">
									<thead>
										<tr className="text-left text-[11px] uppercase tracking-wide text-zinc-400">
											<th className="px-2 py-1">Device</th>
											<th className="px-2 py-1">Type</th>
											<th className="px-2 py-1 text-right">Capacity</th>
											<th className="px-2 py-1">Removable</th>
										</tr>
									</thead>
									<tbody>
										{(detail?.physicalDisks ?? []).map((disk: PhysicalDisk) => (
											<tr
												key={disk.device}
												className="border-t border-white/10 hover:bg-white/5">
												<td className="px-2 py-1.5 font-mono text-zinc-100">
													{disk.device}
												</td>
												<td className="px-2 py-1.5">
													<span
														className={cn(
															"inline-flex items-center rounded-full border px-2 py-0.5 text-[11px]",
															diskKindBadgeClass(disk.kind),
														)}>
														{disk.kind}
													</span>
												</td>
												<td className="px-2 py-1.5 text-right tabular-nums">
													{disk.sizeBytes > 0 ?
														formatGb(disk.sizeBytes)
													:	"Unknown"}
												</td>
												<td className="px-2 py-1.5 text-zinc-300">
													{disk.removable ?
														"Yes"
													: disk.internal ?
														"No (internal)"
													:	"Unknown"}
												</td>
											</tr>
										))}
									</tbody>
								</table>
							</div>
						}
					</section>

					<section className="rounded-xl border border-white/15 bg-white/5 p-3">
						<div className="mb-2 flex items-center justify-between">
							<h2 className="text-xs font-semibold">Mounted volumes</h2>
							<p className="text-[11px] text-zinc-400">
								Source: <code>df -k</code>
							</p>
						</div>

						{volumesByFilesystem.length === 0 ?
							<p className="text-xs text-zinc-300">
								No mounted volumes reported.
							</p>
						:	<div className="space-y-3">
								{volumesByFilesystem.map(([filesystem, entries]) => (
									<div
										key={filesystem}
										className="rounded-md border border-white/10 bg-white/5 p-2.5">
										<div className="mb-1.5 flex items-center justify-between">
											<span className="font-mono text-[11px] text-zinc-200">
												{filesystem}
											</span>
											<span className="text-[11px] text-zinc-400">
												{entries.length} volume{entries.length === 1 ? "" : "s"}
											</span>
										</div>
										<div className="space-y-1.5">
											{entries.map((volume) => (
												<div
													key={`${filesystem}-${volume.mountPoint}`}
													className="rounded border border-white/10 p-2">
													<div className="flex flex-wrap items-center gap-2 text-xs">
														<span className="font-medium text-zinc-100">
															{volume.mountPoint}
														</span>
														<span
															className={cn(
																"rounded-full border px-1.5 py-0.5 text-[10px]",
																volume.usedPercent >= 90 ?
																	severityBadgeClass("critical")
																: volume.usedPercent >= 75 ?
																	severityBadgeClass("warning")
																:	severityBadgeClass("info"),
															)}>
															{volume.usedPercent.toFixed(1)}% used
														</span>
													</div>
													<div className="mt-1.5 flex h-2 overflow-hidden rounded bg-white/10">
														<div
															className={cn(
																"h-full transition-all",
																volumeBarColor(volume.usedPercent),
															)}
															style={{
																width: `${Math.min(100, volume.usedPercent)}%`,
															}}
														/>
													</div>
													<div className="mt-1 flex flex-wrap items-center justify-between text-[11px] text-zinc-300">
														<span>Used: {formatBytes(volume.usedBytes)}</span>
														<span>Free: {formatBytes(volume.freeBytes)}</span>
														<span>Total: {formatBytes(volume.totalBytes)}</span>
													</div>
												</div>
											))}
										</div>
									</div>
								))}
							</div>
						}
					</section>

					{detail ?
						<p className="mt-3 text-[11px] text-zinc-400">
							Last refreshed {formatTimestamp(detail.scannedAtEpochMs)}
						</p>
					:	null}
				</main>
			</div>

			<StorageCleanupOverlay
				open={overlayOpen}
				onOpenChange={setOverlayOpen}
				scan={
					scan ?
						{
							items: scan.items,
							categories: scan.categories,
							totalRecoverableBytes: scan.totalRecoverableBytes,
						}
					:	null
				}
				isScanning={isScanning}
				isExecuting={isExecuting}
				result={cleanupResult}
				errorMessage={cleanupError}
				onRun={(request) => void handleRunCleanup(request)}
				onRescan={handleRescan}
			/>
		</div>
	);
}
