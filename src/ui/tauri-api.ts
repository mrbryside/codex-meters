import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppSettings, UsageSnapshot } from "../types";

export const api = {
  snapshot: () => invoke<{ snapshot: UsageSnapshot | null }>("get_usage_snapshot"),
  refresh: () => invoke<{ snapshot: UsageSnapshot; error: boolean }>("refresh_usage"),
  settings: () => invoke<AppSettings>("get_app_settings"),
  setDock: (visible: boolean) => invoke<AppSettings>("set_dock_meter_visible", { visible }),
  setLogin: (enabled: boolean) => invoke<AppSettings>("set_launch_at_login", { enabled }),
  setRefreshInterval: (seconds: number) => invoke<AppSettings>("set_refresh_interval", { seconds }),
  setDockGeometry: (geometry: { x: number; y: number; width: number; height: number }) => invoke<AppSettings>("set_dock_meter_geometry", geometry),
  onUsage: (handler: (snapshot: UsageSnapshot) => void) => listen<UsageSnapshot>("usage-updated", (e) => handler(e.payload)),
};
