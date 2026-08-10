"use client";

import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

import { isTauriRuntime } from "@/lib/tauri-client";
import { isAppBusy, busyReasons, subscribeBusy } from "@/lib/app-busy";

export type UpdaterPhase =
	| "idle"
	| "checking"
	| "downloading"
	| "installing"
	| "waiting-for-idle"
	| "relaunching"
	| "up-to-date"
	| "failed";

export interface UpdaterState {
	phase: UpdaterPhase;
	/** Version being installed, once an update has been found. */
	version: string | null;
	notes: string | null;
	downloadedBytes: number;
	/** Total size in bytes, when the server reported a content length. */
	totalBytes: number | null;
	percent: number | null;
	message: string;
	error: string | null;
}

const INITIAL_STATE: UpdaterState = {
	phase: "idle",
	version: null,
	notes: null,
	downloadedBytes: 0,
	totalBytes: null,
	percent: null,
	message: "",
	error: null,
};

let state: UpdaterState = INITIAL_STATE;
const listeners = new Set<(next: UpdaterState) => void>();

/** Guards against overlapping checks from the launch timer and the interval. */
let inFlight = false;
/** Set once an update is staged, so we never install the same update twice. */
let installedAwaitingRelaunch = false;

const setState = (patch: Partial<UpdaterState>): void => {
	state = { ...state, ...patch };
	for (const listener of listeners) {
		listener(state);
	}
};

export const getUpdaterState = (): UpdaterState => state;

export const subscribeUpdater = (
	listener: (next: UpdaterState) => void,
): (() => void) => {
	listeners.add(listener);
	listener(state);
	return () => {
		listeners.delete(listener);
	};
};

/**
 * Relaunches as soon as the app is idle. If work is in flight the relaunch is
 * deferred and retried when the busy set empties, so a cleanup is never cut off
 * midway.
 */
const relaunchWhenIdle = async (): Promise<void> => {
	if (!isAppBusy()) {
		setState({ phase: "relaunching", message: "Restarting Xclense…" });
		await relaunch();
		return;
	}

	setState({
		phase: "waiting-for-idle",
		message: `Update installed. Restarting once ${busyReasons()[0] ?? "current work"} finishes.`,
	});

	const unsubscribe = subscribeBusy((busy) => {
		if (busy) return;
		unsubscribe();
		setState({ phase: "relaunching", message: "Restarting Xclense…" });
		void relaunch();
	});
};

/**
 * Checks for an update and, when one exists, downloads and installs it, then
 * relaunches once the app is idle.
 *
 * `silent` suppresses the "you are up to date" end state so scheduled background
 * checks do not flash a notice at the user; a manual check shows it.
 */
export const runUpdateCycle = async (
	options: { silent?: boolean } = {},
): Promise<void> => {
	const { silent = true } = options;

	if (!isTauriRuntime() || inFlight) return;
	if (installedAwaitingRelaunch) {
		// An update is already staged; just retry the relaunch.
		await relaunchWhenIdle();
		return;
	}

	inFlight = true;
	setState({
		phase: "checking",
		error: null,
		message: "Checking for updates…",
	});

	let update: Update | null = null;
	try {
		update = await check();
	} catch (error: unknown) {
		const detail = error instanceof Error ? error.message : String(error);
		// A failed check is expected offline and must never disrupt the app.
		setState({
			phase: silent ? "idle" : "failed",
			error: detail,
			message: silent ? "" : `Could not check for updates: ${detail}`,
		});
		inFlight = false;
		return;
	}

	if (!update) {
		setState({
			phase: silent ? "idle" : "up-to-date",
			message: silent ? "" : "Xclense is up to date.",
		});
		inFlight = false;
		return;
	}

	setState({
		phase: "downloading",
		version: update.version,
		notes: update.body ?? null,
		downloadedBytes: 0,
		totalBytes: null,
		percent: 0,
		message: `Downloading v${update.version}…`,
	});

	try {
		let downloaded = 0;
		let total: number | null = null;

		await update.downloadAndInstall((event) => {
			if (event.event === "Started") {
				total = event.data.contentLength ?? null;
				setState({ totalBytes: total, downloadedBytes: 0, percent: 0 });
				return;
			}
			if (event.event === "Progress") {
				downloaded += event.data.chunkLength;
				setState({
					downloadedBytes: downloaded,
					percent:
						total && total > 0 ?
							Math.min(100, Math.round((downloaded / total) * 100))
						:	null,
				});
				return;
			}
			if (event.event === "Finished") {
				setState({
					phase: "installing",
					percent: 100,
					message: `Installing v${update?.version ?? ""}…`,
				});
			}
		});

		installedAwaitingRelaunch = true;
		await relaunchWhenIdle();
	} catch (error: unknown) {
		const detail = error instanceof Error ? error.message : String(error);
		setState({
			phase: "failed",
			error: detail,
			message: `Update failed: ${detail}`,
		});
	} finally {
		inFlight = false;
	}
};

/** Delay before the first check so startup diagnostics are not competing for I/O. */
export const LAUNCH_CHECK_DELAY_MS = 5_000;
export const PERIODIC_CHECK_INTERVAL_MS = 6 * 60 * 60 * 1000;

/**
 * Starts the launch and periodic checks. Returns a cleanup function that stops
 * all scheduled work.
 */
export const startUpdateScheduler = (): (() => void) => {
	if (!isTauriRuntime()) return () => undefined;

	const launchTimer = window.setTimeout(() => {
		void runUpdateCycle({ silent: true });
	}, LAUNCH_CHECK_DELAY_MS);

	const interval = window.setInterval(() => {
		void runUpdateCycle({ silent: true });
	}, PERIODIC_CHECK_INTERVAL_MS);

	return () => {
		window.clearTimeout(launchTimer);
		window.clearInterval(interval);
	};
};
