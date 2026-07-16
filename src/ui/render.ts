import type { UsageSnapshot } from "../types";
export function renderBars(snapshot: UsageSnapshot | null): string {
  if (!snapshot) return `<div class="empty">Waiting for Codex usage…</div>`;
  const rows = snapshot.windows.map(w => `<div class="usage-group"><div class="row"><span>${w.window}</span><div class="track"><i style="width:${w.remainingPercent}%"></i></div><b>${Math.round(w.remainingPercent)}%</b></div><small class="reset">${resetLabel(w.resetAt)}</small></div>`).join("");
  const failed = snapshot.status.kind !== "fresh" ? `<div class="status" role="status"><span class="status-icon">⚠</span><span class="status-text">${snapshot.status.kind === "stale" ? "Showing cached values" : "Usage unavailable"}</span></div>` : "";
  return rows + failed;
}

export function renderDockBars(snapshot: UsageSnapshot | null): string {
  if (!snapshot) return `<div class="dock-empty">Waiting…</div>`;
  return `<div class="dock-layout">${snapshot.windows.map((window, index) => `<section class="dock-window"><div class="dock-row"><span>${window.window}</span><div class="track"><i style="width:${window.remainingPercent}%"></i></div><b>${Math.round(window.remainingPercent)}%</b></div><small class="reset">${resetLabel(window.resetAt)}</small></section>${index < snapshot.windows.length - 1 ? `<div class="dock-divider" role="separator"></div>` : ""}`).join("")}</div>`;
}

function resetLabel(resetAt: string | null): string {
  if (!resetAt) return "reset time unavailable";
  const remaining = new Date(resetAt).getTime() - Date.now();
  if (remaining <= 0) return "resetting soon";
  const formatted = new Intl.DateTimeFormat(undefined, {
    day: "2-digit", month: "short", hour: "2-digit", minute: "2-digit",
  }).format(new Date(resetAt));
  return `resets ${formatted}`;
}
