"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
	AppWindow,
	HardDrive,
	Heart,
	LayoutDashboard,
	MemoryStick,
	Settings,
} from "lucide-react";

interface NavItem {
	label: string;
	icon: React.ComponentType<{ className?: string }>;
	href?: string;
}

const NAV_ITEMS: NavItem[] = [
	{ label: "Dashboard", icon: LayoutDashboard, href: "/" },
	{ label: "Storage", icon: HardDrive, href: "/storage" },
	{ label: "Applications", icon: AppWindow },
	{ label: "Memory", icon: MemoryStick },
	{ label: "Health", icon: Heart, href: "/health" },
	{ label: "Settings", icon: Settings },
];

export function DashboardNav(): React.JSX.Element {
	const pathname = usePathname();

	return (
		<aside className="col-span-12 border-r border-white/10 bg-black/20 p-3 md:col-span-3 lg:col-span-2">
			<div className="space-y-2.5">
				{NAV_ITEMS.map((item) => {
					const Icon = item.icon;
					const isActive = item.href ? pathname === item.href : false;
					const className = `group flex w-full items-center gap-2.5 rounded-lg px-3 py-2.5 text-left text-sm font-medium transition duration-200 ${
						isActive ?
							"bg-sky-500/90 text-white shadow-lg shadow-sky-900/40"
						:	"text-zinc-200 hover:bg-white/10"
					}`;

					if (item.href) {
						return (
							<Link key={item.label} href={item.href} className={className}>
								<Icon className="h-4 w-4" />
								<span>{item.label}</span>
							</Link>
						);
					}

					return (
						<button key={item.label} type="button" className={className}>
							<Icon className="h-4 w-4" />
							<span>{item.label}</span>
						</button>
					);
				})}
			</div>
		</aside>
	);
}
