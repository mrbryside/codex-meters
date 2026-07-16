import { api } from "./tauri-api";
import type { Store } from "./state";
import type { UsageSnapshot } from "../types";
type RefreshResult = { snapshot: UsageSnapshot; error: boolean };
export function createRefreshController(store: Store, render: () => void, refreshSource: () => Promise<RefreshResult> = api.refresh) {
  const refresh = async () => { try { const result = await refreshSource(); store.snapshot = result.snapshot; } finally { store.loading = false; render(); } };
  let timer = window.setInterval(refresh, store.settings.refreshIntervalSeconds * 1000);
  return { refresh, setInterval: (seconds: number) => { window.clearInterval(timer); timer = window.setInterval(refresh, seconds * 1000); }, dispose: () => window.clearInterval(timer) };
}
