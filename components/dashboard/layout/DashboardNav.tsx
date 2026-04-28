import {
  AppWindow,
  HardDrive,
  Heart,
  LayoutDashboard,
  MemoryStick,
  Settings,
} from "lucide-react";

const NAV_ITEMS = [
  { label: "Dashboard", icon: LayoutDashboard, active: true },
  { label: "Storage", icon: HardDrive, active: false },
  { label: "Applications", icon: AppWindow, active: false },
  { label: "Memory", icon: MemoryStick, active: false },
  { label: "Health", icon: Heart, active: false },
  { label: "Settings", icon: Settings, active: false },
];

export function DashboardNav(): React.JSX.Element {
  return (
    <aside className="col-span-12 border-r border-white/10 bg-black/20 p-3 md:col-span-3 lg:col-span-2">
      <div className="space-y-2.5">
        {NAV_ITEMS.map((item) => {
          const Icon = item.icon;

          return (
            <button
              key={item.label}
              type="button"
              className={`group flex w-full items-center gap-2.5 rounded-lg px-3 py-2.5 text-left text-sm font-medium transition duration-200 ${
                item.active
                  ? "bg-sky-500/90 text-white shadow-lg shadow-sky-900/40"
                  : "text-zinc-200 hover:bg-white/10"
              }`}
            >
              <Icon className="h-4 w-4" />
              <span>{item.label}</span>
            </button>
          );
        })}
      </div>
    </aside>
  );
}
