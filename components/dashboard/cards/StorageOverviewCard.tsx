"use client";

import Link from "next/link";
import { ArrowUpRight, CircleHelp, ScanSearch } from "lucide-react";

import { cn } from "@/lib/utils";

import { formatGb } from "@/components/dashboard/shared";
import {
	HoverCard,
	HoverCardContent,
	HoverCardTrigger,
} from "@/components/ui/hover-card";

interface StorageOverviewCardProps {
	totalBytes: number;
	freeBytes: number;
	usedPercent: number;
	className?: string;
	onScan?: () => void;
	isScanning?: boolean;
}

export function StorageOverviewCard({
	totalBytes,
	freeBytes,
	usedPercent,
	className,
	onScan,
	isScanning = false,
}: StorageOverviewCardProps): React.JSX.Element {
	const clamped = Math.max(0, Math.min(100, usedPercent));
	const usedBytes = totalBytes - freeBytes;

	const applicationPercent = Math.max(8, clamped * 0.45);
	const systemPercent = Math.max(6, clamped * 0.28);
	const photosPercent = Math.max(4, clamped * 0.15);
	const otherPercent = Math.max(3, clamped * 0.12);

	return (
		<section
			className={cn(
				"flex h-full flex-col rounded-xl border border-white/15 bg-white/5 p-2",
				className,
			)}>
			<div className="inline-flex items-center gap-1.5">
				<h2 className="text-xs font-semibold">Storage Overview</h2>
				<HoverCard openDelay={120} closeDelay={80}>
					<HoverCardTrigger asChild>
						<button
							type="button"
							className="inline-flex h-5 w-5 items-center justify-center rounded-full border border-white/20 bg-white/5 text-zinc-300 transition hover:bg-white/10 hover:text-zinc-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-300/60"
							aria-label="What this storage overview card shows">
							<CircleHelp className="h-3.5 w-3.5" />
						</button>
					</HoverCardTrigger>
					<HoverCardContent
						align="start"
						className="w-72 border border-white/15 bg-zinc-900/95 text-zinc-100">
						<p className="text-xs font-semibold">Storage Overview</p>
						<p className="mt-1 text-[11px] leading-relaxed text-zinc-300">
							This widget summarises used versus free disk space and the
							breakdown by category. Open the Storage page to see every physical
							disk, mounted volume, and run a reclaimable-space scan.
						</p>
					</HoverCardContent>
				</HoverCard>
			</div>

			<div className="mt-2 flex items-center justify-between text-xs">
				<p className="text-zinc-300">{formatGb(totalBytes)} Total</p>
				<p className="text-zinc-300">{formatGb(freeBytes)} Free</p>
			</div>

			<div className="mt-1.5 flex h-5 overflow-hidden rounded-md bg-white/10">
				<div
					className="bg-sky-500"
					style={{ width: `${applicationPercent}%` }}
				/>
				<div className="bg-violet-500" style={{ width: `${systemPercent}%` }} />
				<div
					className="bg-emerald-500"
					style={{ width: `${photosPercent}%` }}
				/>
				<div className="bg-amber-400" style={{ width: `${otherPercent}%` }} />
			</div>

			<div className="mt-1.5 grid grid-cols-2 gap-x-2 gap-y-1 text-xs text-zinc-300">
				<p className="inline-flex items-center gap-1.5">
					<span className="h-2.5 w-2.5 rounded-full bg-sky-500" />
					Applications
				</p>
				<p className="inline-flex items-center gap-1.5">
					<span className="h-2.5 w-2.5 rounded-full bg-violet-500" />
					System Data
				</p>
				<p className="inline-flex items-center gap-1.5">
					<span className="h-2.5 w-2.5 rounded-full bg-emerald-500" />
					Photos
				</p>
				<p className="inline-flex items-center gap-1.5">
					<span className="h-2.5 w-2.5 rounded-full bg-amber-400" />
					Other
				</p>
			</div>

			<div className="mt-1.5 flex flex-wrap items-center gap-x-3 gap-y-1 border-t border-white/10 pt-1.5 text-[11px] text-zinc-300">
				<span>
					<span className="text-zinc-400">Used:</span> {formatGb(usedBytes)}
				</span>
				<span>
					<span className="text-zinc-400">Free:</span> {formatGb(freeBytes)}
				</span>
				<span
					className={cn(
						"rounded-full border px-1.5 py-0.5 text-[10px] uppercase tracking-wide",
						clamped >= 90 ? "border-red-400/40 bg-red-500/10 text-red-200"
						: clamped >= 75 ?
							"border-amber-400/40 bg-amber-500/10 text-amber-200"
						:	"border-emerald-400/40 bg-emerald-500/10 text-emerald-200",
					)}>
					{clamped.toFixed(1)}% used
				</span>
			</div>

			<div className="mt-auto flex items-center justify-between gap-2 pt-2">
				<Link
					href="/storage"
					className="inline-flex items-center gap-1.5 rounded-md border border-white/15 bg-white/10 px-2.5 py-1 text-xs font-medium text-zinc-100 transition hover:bg-white/15">
					<ArrowUpRight className="h-3 w-3" />
					Open storage page
				</Link>
				{onScan ?
					<button
						type="button"
						onClick={onScan}
						disabled={isScanning}
						className="inline-flex items-center gap-1.5 rounded-md border border-cyan-300/40 bg-cyan-400/15 px-2.5 py-1 text-xs font-semibold text-cyan-100 transition hover:bg-cyan-400/25 disabled:cursor-not-allowed disabled:opacity-60">
						<ScanSearch
							className={cn("h-3 w-3", isScanning && "animate-spin")}
						/>
						{isScanning ? "Scanning…" : "Scan & clean"}
					</button>
				:	<Link
						href="/storage"
						className="inline-flex items-center gap-1.5 rounded-md border border-cyan-300/40 bg-cyan-400/15 px-2.5 py-1 text-xs font-semibold text-cyan-100 transition hover:bg-cyan-400/25">
						<ScanSearch className="h-3 w-3" />
						Scan & clean
					</Link>
				}
			</div>
		</section>
	);
}
