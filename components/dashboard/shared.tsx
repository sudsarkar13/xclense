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

export const formatGb = (bytes: number): string => `${(bytes / 1024 ** 3).toFixed(1)} GB`;

export const processBarClass = (index: number): string => {
  const colors = [
    "from-sky-500 to-blue-400",
    "from-orange-500 to-red-400",
    "from-emerald-500 to-green-400",
    "from-violet-500 to-fuchsia-400",
  ];

  return colors[index % colors.length];
};
