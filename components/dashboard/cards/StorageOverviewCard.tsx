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
	const safeTotalBytes = Math.max(0, totalBytes);
	const safeFreeBytes = Math.min(Math.max(0, freeBytes), safeTotalBytes);
	const usedBytes = Math.max(0, safeTotalBytes - safeFreeBytes);
	const measuredUsedPercent =
		safeTotalBytes > 0 ? (usedBytes / safeTotalBytes) * 100 : usedPercent;
	const clamped = Math.max(0, Math.min(100, measuredUsedPercent));
	const categoryRatios = [
		{
			label: "Applications",
			ratio: 0.24,
			colorClass: "bg-sky-500",
			textClass: "text-sky-200",
			description:
				"Estimated application footprint within currently used storage.",
		},
		{
			label: "Photos",
			ratio: 0.12,
			colorClass: "bg-emerald-500",
			textClass: "text-emerald-200",
			description: "Estimated photo libraries and image collections.",
		},
		{
			label: "Videos",
			ratio: 0.08,
			colorClass: "bg-rose-500",
			textClass: "text-rose-200",
			description:
				"Estimated videos, movies, screen recordings, and media files.",
		},
		{
			label: "Downloads",
			ratio: 0.1,
			colorClass: "bg-amber-400",
			textClass: "text-amber-200",
			description: "Estimated files and installers kept in Downloads.",
		},
		{
			label: "System Data",
			ratio: 0.22,
			colorClass: "bg-violet-500",
			textClass: "text-violet-200",
			description: "Estimated macOS system data, caches, and support files.",
		},
	];
	const usedCategoryBytes = categoryRatios.map((category) => ({
		...category,
		bytes: Math.round(usedBytes * category.ratio),
	}));
	const categorizedBytes = usedCategoryBytes.reduce(
		(sum, segment) => sum + segment.bytes,
		0,
	);
	const segments = [
		...usedCategoryBytes,
		{
			label: "Other",
			bytes: Math.max(0, usedBytes - categorizedBytes),
			colorClass: "bg-cyan-300",
			textClass: "text-cyan-200",
			description:
				"Estimated remaining occupied storage that does not fit the visible groups.",
		},
		{
			label: "Free space",
			bytes: safeFreeBytes,
			colorClass: "bg-zinc-800/80",
			textClass: "text-zinc-200",
			description:
				"Available disk space. This is intentionally shown as a muted area rather than a bright category color.",
		},
	].map((segment) => ({
		...segment,
		percent:
			safeTotalBytes > 0 ?
				Math.max(0, Math.min(100, (segment.bytes / safeTotalBytes) * 100))
			:	0,
	}));

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
							This widget uses measured total, used, and free bytes from the
							startup volume. The colored category split is an infographic
							estimate, while the Used, Free, and percent badge stay calculated
							from real disk bytes.
						</p>
					</HoverCardContent>
				</HoverCard>
			</div>

			<div className="mt-2 flex items-center justify-between text-xs">
				<p className="text-zinc-300">{formatGb(safeTotalBytes)} Total</p>
				<p className="text-zinc-300">{formatGb(safeFreeBytes)} Free</p>
			</div>

			<div className="mt-1.5 flex h-6 overflow-hidden rounded-md bg-white/10 ring-1 ring-white/10">
				{segments.map((segment) => (
					<HoverCard key={segment.label} openDelay={100} closeDelay={80}>
						<HoverCardTrigger asChild>
							<div
								className={cn(
									"h-full transition-opacity hover:opacity-85 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-300/70",
									segment.colorClass,
								)}
								style={{ width: `${segment.percent}%` }}
								tabIndex={0}
								role="button"
								aria-label={`${segment.label}: ${formatGb(segment.bytes)}, ${segment.percent.toFixed(1)} percent`}
							/>
						</HoverCardTrigger>
						<HoverCardContent className="w-64 border border-white/15 bg-zinc-900/95 text-zinc-100">
							<p className={cn("text-xs font-semibold", segment.textClass)}>
								{segment.label}
							</p>
							<p className="mt-1 text-[11px] leading-relaxed text-zinc-300">
								{segment.description}
							</p>
							<p className="mt-2 text-xs text-zinc-100">
								{formatGb(segment.bytes)} · {segment.percent.toFixed(1)}%
							</p>
						</HoverCardContent>
					</HoverCard>
				))}
			</div>

			<div className="mt-1.5 grid grid-cols-2 gap-x-2 gap-y-1 text-xs text-zinc-300">
				{segments.map((segment) => (
					<HoverCard key={segment.label} openDelay={100} closeDelay={80}>
						<HoverCardTrigger asChild>
							<p className="inline-flex cursor-help items-center gap-1.5 rounded px-1 py-0.5 hover:bg-white/5">
								<span
									className={cn("h-2.5 w-2.5 rounded-full", segment.colorClass)}
								/>
								<span>{segment.label}</span>
								<span className="text-zinc-500">
									{segment.percent.toFixed(1)}%
								</span>
							</p>
						</HoverCardTrigger>
						<HoverCardContent className="w-64 border border-white/15 bg-zinc-900/95 text-zinc-100">
							<p className={cn("text-xs font-semibold", segment.textClass)}>
								{segment.label}
							</p>
							<p className="mt-1 text-[11px] leading-relaxed text-zinc-300">
								{segment.description}
							</p>
							<p className="mt-2 text-xs text-zinc-100">
								{formatGb(segment.bytes)} · {segment.percent.toFixed(1)}%
							</p>
						</HoverCardContent>
					</HoverCard>
				))}
			</div>

			<div className="mt-1.5 flex flex-wrap items-center gap-x-3 gap-y-1 border-t border-white/10 pt-1.5 text-[11px] text-zinc-300">
				<span>
					<span className="text-zinc-400">Used:</span> {formatGb(usedBytes)}
				</span>
				<span>
					<span className="text-zinc-400">Free:</span> {formatGb(safeFreeBytes)}
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
