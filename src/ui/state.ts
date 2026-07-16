import type { AppSettings, UsageSnapshot } from "../types";

export type Store = { snapshot: UsageSnapshot | null; settings: AppSettings; loading: boolean };
export const defaultSettings: AppSettings = { dockMeterVisible: true, launchAtLogin: true, refreshIntervalSeconds: 60, dockMeterGeometry: null };
export function createStore(): Store { return { snapshot: null, settings: defaultSettings, loading: true }; }
