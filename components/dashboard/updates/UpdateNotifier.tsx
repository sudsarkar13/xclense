"use client";

import { useEffect, useState } from "react";
import {
	CheckCircle2,
	Download,
	Loader2,
	RefreshCcw,
	XIcon,
} from "lucide-react";

import { cn } from "@/lib/utils";
import { formatBytes } from "@/components/dashboard/shared";
import {
	getUpdaterState,
	startUpdateScheduler,
	subscribeUpdater,
	type UpdaterState,
} from "@/lib/updater";

/** Phases with nothing worth showing the user. */
const HIDDEN_PHASES: UpdaterState["phase"][] = ["idle", "checking"];

export function UpdateNotifier(): React.JSX.Element | null {
	const [state, setState] = useState<UpdaterState>(getUpdaterState);
	// Dismissal is keyed to the update it applies to rather than a boolean, so a
	// later update re-opens the panel without needing to reset the flag.
	const [dismissedKey, setDismissedKey] = useState<string | null>(null);

	useEffect(() => subscribeUpdater(setState), []);
	useEffect(() => startUpdateScheduler(), []);

	const cycleKey = state.version ?? state.phase;

	// Transient end states clear themselves.
	useEffect(() => {
		if (state.phase !== "up-to-date" && state.phase !== "failed") return;
		const key = state.version ?? state.phase;
		const timer = window.setTimeout(() => setDismissedKey(key), 6000);
		return () => window.clearTimeout(timer);
	}, [state.phase, state.version]);

	if (dismissedKey === cycleKey || HIDDEN_PHASES.includes(state.phase)) {
		return null;
	}

	const isWorking =
		state.phase === "downloading" ||
		state.phase === "installing" ||
		state.phase === "relaunching";
	const isBlocked = state.phase === "waiting-for-idle";
	const isFailed = state.phase === "failed";
	// The updater cannot be cancelled mid-install without leaving a partial
	// bundle, so dismissal is only offered once it is safe.
	const canDismiss = !isWorking;

	return (
		<div
			className={cn(
				"fixed right-6 bottom-6 z-[90] w-[380px] max-w-[calc(100vw-3rem)] rounded-2xl border p-4 shadow-2xl shadow-black/50 ring-1 ring-white/10 backdrop-blur-md",
				isFailed ? "border-red-400/30 bg-[#2a0f16]/95 text-red-50"
				: isBlocked ? "border-amber-300/25 bg-[#241a07]/95 text-amber-50"
				: "border-cyan-300/25 bg-[#071426]/95 text-cyan-50",
			)}>
			<div className="flex items-start justify-between gap-3">
				<div className="flex items-center gap-3">
					<div
						className={cn(
							"flex h-9 w-9 items-center justify-center rounded-full border",
							isFailed ? "border-red-400/30 bg-red-400/10"
							: isBlocked ? "border-amber-300/30 bg-amber-400/10"
							: "border-cyan-300/30 bg-cyan-400/10",
						)}>
						{state.phase === "downloading" ?
							<Download className="h-4 w-4 animate-pulse text-cyan-200" />
						: state.phase === "installing" || state.phase === "relaunching" ?
							<Loader2 className="h-4 w-4 animate-spin text-cyan-200" />
						: state.phase === "waiting-for-idle" ?
							<RefreshCcw className="h-4 w-4 text-amber-200" />
						: state.phase === "failed" ?
							<XIcon className="h-4 w-4 text-red-200" />
						:	<CheckCircle2 className="h-4 w-4 text-emerald-300" />}
					</div>
					<div>
						<h3 className="text-sm font-semibold">
							{state.phase === "downloading" ?
								`Downloading v${state.version}`
							: state.phase === "installing" ?
								`Installing v${state.version}`
							: state.phase === "waiting-for-idle" ?
								"Update ready"
							: state.phase === "relaunching" ?
								"Restarting"
							: state.phase === "failed" ?
								"Update failed"
							:	"Xclense is up to date"}
						</h3>
						<p className="mt-0.5 text-[11px] opacity-75">
							{state.message || "Updates install automatically."}
						</p>
					</div>
				</div>
				{canDismiss ?
					<button
						type="button"
						onClick={() => setDismissedKey(cycleKey)}
						className="rounded-full p-1 opacity-70 transition hover:bg-white/10 hover:opacity-100"
						aria-label="Dismiss update notice">
						<XIcon className="h-4 w-4" />
					</button>
				:	null}
			</div>

			{state.phase === "downloading" ?
				<>
					<div className="mt-3 h-2 overflow-hidden rounded-full bg-cyan-950/80">
						<div
							className={cn(
								"h-full rounded-full bg-gradient-to-r from-cyan-300 via-sky-400 to-emerald-300",
								state.percent === null ?
									"xc-progress-sheen w-1/3"
								:	"transition-all duration-300",
							)}
							style={
								state.percent === null ?
									undefined
								:	{ width: `${state.percent}%` }
							}
						/>
					</div>
					<div className="mt-2 flex items-center justify-between text-[11px] opacity-80">
						<span className="tabular-nums">
							{formatBytes(state.downloadedBytes)}
							{state.totalBytes ? ` / ${formatBytes(state.totalBytes)}` : ""}
						</span>
						<span className="tabular-nums">
							{state.percent === null ? "" : `${state.percent}%`}
						</span>
					</div>
				</>
			:	null}

			{isFailed && state.error ?
				<p className="mt-2 max-h-20 overflow-hidden rounded-lg border border-white/10 bg-black/25 p-2 font-mono text-[10px] leading-relaxed">
					{state.error}
				</p>
			:	null}
		</div>
	);
}
