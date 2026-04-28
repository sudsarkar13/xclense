import { invoke } from "@tauri-apps/api/core";

export interface PingResponse {
  service: string;
  status: string;
  version: string;
}

/**
 * Determines whether the app is running inside a Tauri runtime.
 */
export const isTauriRuntime = (): boolean => {
  if (typeof window === "undefined") {
    return false;
  }

  return "__TAURI_INTERNALS__" in window;
};

/**
 * Calls the Rust backend ping command.
 * Returns null when not running inside Tauri, allowing web-only UI work.
 */
export const pingBackend = async (): Promise<PingResponse | null> => {
  if (!isTauriRuntime()) {
    return null;
  }

  return invoke<PingResponse>("ping_backend");
};
