"use client";

import { useEffect } from "react";

/**
 * Tracks whether the app is doing work that must not be interrupted.
 *
 * The updater installs by replacing the app bundle and relaunching, which kills
 * any in-flight operation. A storage cleanup moves files to Trash one at a time,
 * so being killed midway leaves the user unsure what was actually moved. Any
 * component running such work registers a reason here, and the updater waits
 * until the set is empty before it relaunches.
 */
const activeReasons = new Set<string>();
const listeners = new Set<(busy: boolean) => void>();

const notify = (): void => {
	const busy = activeReasons.size > 0;
	for (const listener of listeners) {
		listener(busy);
	}
};

export const markBusy = (reason: string): void => {
	activeReasons.add(reason);
	notify();
};

export const clearBusy = (reason: string): void => {
	activeReasons.delete(reason);
	notify();
};

export const isAppBusy = (): boolean => activeReasons.size > 0;

/** Human-readable list of what is currently blocking, for status messages. */
export const busyReasons = (): string[] => Array.from(activeReasons);

export const subscribeBusy = (
	listener: (busy: boolean) => void,
): (() => void) => {
	listeners.add(listener);
	return () => {
		listeners.delete(listener);
	};
};

/**
 * Registers `reason` as blocking for as long as `active` is true. The cleanup
 * function clears it on unmount so a crashed or closed overlay cannot leave the
 * updater blocked forever.
 */
export const useBusyWhile = (active: boolean, reason: string): void => {
	useEffect(() => {
		if (!active) {
			clearBusy(reason);
			return;
		}
		markBusy(reason);
		return () => {
			clearBusy(reason);
		};
	}, [active, reason]);
};
