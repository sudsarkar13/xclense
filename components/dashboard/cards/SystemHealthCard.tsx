import { formatGb, statusColorClass, statusTone } from "@/components/dashboard/shared";

interface SystemHealthCardProps {
  score: number;
  totalIssues: number;
  memoryUsedBytes: number;
  memoryFreeBytes: number;
}

export function SystemHealthCard({
  score,
  totalIssues,
  memoryUsedBytes,
  memoryFreeBytes,
}: SystemHealthCardProps): React.JSX.Element {
  return (
    <section className="rounded-xl border border-white/15 bg-white/5 p-4">
      <h2 className="text-xl font-semibold">Overall System Health</h2>
      <div className="mt-4 flex items-center gap-4">
        <div className="relative h-20 w-20 rounded-full border-[6px] border-cyan-400/80">
          <div className="absolute inset-0 grid place-items-center text-2xl font-bold">{score}%</div>
        </div>
        <div>
          <p className={`text-2xl font-semibold ${statusColorClass(score)}`}>{statusTone(score)}</p>
          <p className="mt-1 text-sm text-zinc-300">Mac stability appears normal in this cycle.</p>
        </div>
      </div>

      <div className="mt-4 border-t border-white/10 pt-3 text-sm text-zinc-300">
        <p>{totalIssues > 0 ? `${totalIssues} issue(s) need attention.` : "No urgent issues found."}</p>
        <p className="mt-1 text-xs text-zinc-400">
          Memory snapshot: {formatGb(memoryUsedBytes)} used / {formatGb(memoryFreeBytes)} free
        </p>
      </div>
    </section>
  );
}
