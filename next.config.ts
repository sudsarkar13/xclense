import type { NextConfig } from "next";

const nextConfig: NextConfig = {
	// Tauri bundles a static frontend: `next build` must emit a servable
	// HTML/CSS/JS tree in `out/`, which tauri.conf.json points at via
	// `frontendDist`. Every route in this app is client-rendered, so nothing
	// here depends on a Node server at runtime.
	output: "export",
};

export default nextConfig;
