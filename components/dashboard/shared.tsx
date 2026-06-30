import { type SeverityLevel } from "@/components/dashboard/types";

export const statusTone = (score: number): string => {
	if (score >= 80) return "Healthy";
	if (score >= 60) return "Moderate";
	return "Critical";
};

export const statusColorClass = (score: number): string => {
	if (score >= 80) return "text-emerald-300";
	if (score >= 60) return "text-amber-300";
	return "text-red-300";
};

export const severityDotClass = (severity: SeverityLevel): string => {
	if (severity === "critical") return "bg-red-400";
	if (severity === "warning") return "bg-amber-400";
	return "bg-emerald-400";
};

export const severityBadgeClass = (severity: SeverityLevel): string => {
	if (severity === "critical")
		return "border-red-400/35 bg-red-500/10 text-red-200";
	if (severity === "warning")
		return "border-amber-400/35 bg-amber-500/10 text-amber-200";
	return "border-cyan-400/35 bg-cyan-500/10 text-cyan-200";
};

export const formatGb = (bytes: number): string =>
	`${(bytes / 1024 ** 3).toFixed(1)} GB`;

export const processBarClass = (index: number): string => {
	const colors = [
		"from-sky-500 to-blue-400",
		"from-orange-500 to-red-400",
		"from-emerald-500 to-green-400",
		"from-violet-500 to-fuchsia-400",
	];

	return colors[index % colors.length];
};

/**
 * Penalty applied when memory pressure rises above 60%.
 * 0 below 60%, linear 0..19 from 60..79%, 20..25 (capped) above 80%.
 */
export const pressurePenalty = (pressurePercent: number): number => {
	const clamped = Math.max(0, Math.min(100, pressurePercent));
	if (clamped < 60) return 0;
	if (clamped < 80) return clamped - 60;
	return Math.min(25, 20 + (clamped - 80) / 2);
};

/**
 * Penalty applied when the 1m load average exceeds ~70% of available cores.
 * Treats the machine as a single core when `logicalCoreCount <= 1` for safety.
 */
export const loadPenalty = (
	loadAverage1m: number,
	logicalCoreCount: number,
): number => {
	if (logicalCoreCount <= 0) return 0;
	const effective = loadAverage1m / logicalCoreCount;
	if (effective < 0.7) return 0;
	if (effective <= 1.4) return ((effective - 0.7) / 0.7) * 10;
	const overshoot = effective - 1.4;
	return Math.min(15, 10 + overshoot * 2.5);
};

const clamp = (value: number, min: number, max: number): number =>
	Math.max(min, Math.min(max, value));

export type HealthToneLabel = "Healthy" | "Watch" | "Action needed";

/**
 * Compute a transparent health score breakdown from raw inputs.
 * Each penalty component is exposed so the UI can render the math on hover.
 */
export const computeHealthScore = (input: {
	pressurePercent: number;
	loadAverage1m: number;
	logicalCoreCount: number;
	criticalCount: number;
	warningCount: number;
}): {
	base: number;
	criticalPenalty: number;
	warningPenalty: number;
	pressurePenalty: number;
	loadPenalty: number;
	final: number;
	tone: HealthToneLabel;
} => {
	const criticalPenalty = Math.min(input.criticalCount, 4) * 25;
	const warningPenalty = Math.min(input.warningCount, 6) * 10;
	const pressure = pressurePenalty(input.pressurePercent);
	const load = loadPenalty(input.loadAverage1m, input.logicalCoreCount);

	const totalPenalty = criticalPenalty + warningPenalty + pressure + load;
	const finalScore = clamp(Math.round(100 - totalPenalty), 0, 100);

	const tone: HealthToneLabel =
		finalScore >= 80 ? "Healthy"
		: finalScore >= 60 ? "Watch"
		: "Action needed";

	return {
		base: 100,
		criticalPenalty,
		warningPenalty,
		pressurePenalty: Math.round(pressure),
		loadPenalty: Math.round(load),
		final: finalScore,
		tone,
	};
};

export const healthToneClass = (tone: HealthToneLabel): string => {
	if (tone === "Healthy") return "text-emerald-300";
	if (tone === "Watch") return "text-amber-300";
	return "text-red-300";
};
