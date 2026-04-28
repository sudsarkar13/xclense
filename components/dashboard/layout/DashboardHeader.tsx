interface DashboardHeaderProps {
  isLoading: boolean;
  onRefresh: () => void;
}

export function DashboardHeader({ isLoading, onRefresh }: DashboardHeaderProps): React.JSX.Element {
  return (
    <div className="mb-3 flex items-center justify-between">
      <p className="text-xs text-zinc-300">
        {isLoading ? "Loading diagnostics..." : "Live diagnostics from Rust/Tauri services"}
      </p>
      <button
        type="button"
        onClick={onRefresh}
        className="rounded-md border border-white/20 bg-white/10 px-3 py-1.5 text-xs font-medium transition hover:bg-white/20"
      >
        Refresh
      </button>
    </div>
  );
}
