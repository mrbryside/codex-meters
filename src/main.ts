import "./ui/styles.css";
import { api } from "./ui/tauri-api";
import { createStore } from "./ui/state";
import { createRefreshController } from "./ui/refresh-controller";
import { renderBars, renderDockBars } from "./ui/render";
import type { DockMeterGeometry, UsageSnapshot } from "./types";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";

const store = createStore();
const currentWindow = getCurrentWindow();
const isDockMeter = currentWindow.label === "dock-meter";
const mockUsage = import.meta.env.VITE_MOCK_USAGE === "true";
const mockSnapshot: UsageSnapshot = {
  windows: [
    { window: "5h", remainingPercent: 72, resetAt: "2026-07-16T19:00:00Z" },
    { window: "7d", remainingPercent: 79, resetAt: "2026-07-22T09:28:00Z" },
  ],
  status: { kind: "fresh", fetchedAt: new Date().toISOString() },
};
const root = document.querySelector<HTMLDivElement>("#app")!;

let settingsLoaded = !isDockMeter;
let dockSize: { width: number; height: number } | null = null;
let appliedDockSize: { width: number; height: number } | null = null;
let dockSizeRows = -1;
let dockSizeUserResized = false;
let dockGeometry: DockMeterGeometry | null = null;
let geometrySaveTimer: number | undefined;
let suppressBlurUntil = 0;

const scheduleDockGeometrySave = () => {
  if (!dockGeometry) return;
  if (geometrySaveTimer !== undefined) window.clearTimeout(geometrySaveTimer);
  geometrySaveTimer = window.setTimeout(() => {
    if (dockGeometry) void api.setDockGeometry(dockGeometry).catch((error) => console.error("Could not save Dock meter geometry", error));
  }, 250);
};

if (!isDockMeter) {
  window.addEventListener("blur", () => {
    if (Date.now() < suppressBlurUntil) return;
    window.setTimeout(() => {
      if (Date.now() >= suppressBlurUntil) void currentWindow.hide();
    }, 80);
  });
} else {
  void Promise.all([currentWindow.outerPosition(), currentWindow.outerSize()]).then(([position, size]) => {
    const scale = window.devicePixelRatio || 1;
    dockGeometry = { x: position.x, y: position.y, width: Math.round(size.width / scale), height: Math.round(size.height / scale) };
  });
  void currentWindow.onMoved(({ payload }) => {
    dockGeometry = { ...(dockGeometry ?? { x: payload.x, y: payload.y, width: 400, height: 64 }), x: payload.x, y: payload.y };
    scheduleDockGeometrySave();
  });
  void currentWindow.onResized(({ payload }) => {
    const scale = window.devicePixelRatio || 1;
    dockGeometry = { ...(dockGeometry ?? { x: 0, y: 0, width: 400, height: 64 }), width: Math.round(payload.width / scale), height: Math.round(payload.height / scale) };
    scheduleDockGeometrySave();
  });
}

const render = () => {
  const dockOn = store.settings.dockMeterVisible;
  const rows = store.snapshot?.windows.length ?? 0;

  if (isDockMeter) {
    if (!settingsLoaded) return;
    const hasSavedGeometry = store.settings.dockMeterGeometry !== null;
    if (!dockSize || (!hasSavedGeometry && !dockSizeUserResized && dockSizeRows !== rows)) {
      const saved = store.settings.dockMeterGeometry;
      dockSize = saved ? { width: saved.width, height: saved.height } : { width: rows > 1 ? 400 : 210, height: 64 };
      dockSizeRows = rows;
    }
    if (!appliedDockSize || appliedDockSize.width !== dockSize.width || appliedDockSize.height !== dockSize.height) {
      appliedDockSize = { ...dockSize };
      void currentWindow.setSize(new LogicalSize(dockSize.width, dockSize.height));
    }
    root.innerHTML = `<main class="dock-card" id="dock-drag" data-tauri-drag-region="true">${renderDockBars(store.snapshot)}<span class="resize-handle" id="resize-handle" title="Drag to resize" aria-label="Drag to resize"></span></main>`;
    root.querySelector<HTMLElement>("#dock-drag")?.addEventListener("mousedown", async (event: MouseEvent) => {
      if (event.button !== 0) return;
      event.preventDefault();
      try { await currentWindow.startDragging(); } catch (error) { console.error("Dock meter drag failed", error); }
    });
    root.querySelector<HTMLElement>("#resize-handle")?.addEventListener("mousedown", (event: MouseEvent) => {
      if (event.button !== 0 || !dockSize) return;
      event.preventDefault();
      event.stopPropagation();
      const originX = event.clientX;
      const originY = event.clientY;
      const startWidth = dockSize.width;
      const startHeight = dockSize.height;
      const move = (moveEvent: MouseEvent) => {
        if (!dockSize) return;
        dockSizeUserResized = true;
        dockSize.width = Math.min(520, Math.max(180, startWidth + moveEvent.clientX - originX));
        dockSize.height = Math.min(180, Math.max(48, startHeight + moveEvent.clientY - originY));
        appliedDockSize = { ...dockSize };
        void currentWindow.setSize(new LogicalSize(dockSize.width, dockSize.height));
      };
      const stop = () => { window.removeEventListener("mousemove", move); window.removeEventListener("mouseup", stop); scheduleDockGeometrySave(); };
      window.addEventListener("mousemove", move);
      window.addEventListener("mouseup", stop, { once: true });
    });
    return;
  }

  const hasStatus = store.snapshot?.status.kind !== "fresh";
  void currentWindow.setSize(new LogicalSize(280, (rows > 1 ? 156 : 124) + (hasStatus ? 18 : 0)));
  root.innerHTML = `<main class="card"><div class="title"><span>Codex limits</span><button class="icon-button" id="refresh" aria-label="Refresh usage" title="Refresh usage">↻</button></div><div class="divider" role="separator"></div>${renderBars(store.snapshot)}<div class="controls"><button class="icon-button ${dockOn ? "active" : ""}" id="dock" aria-label="Toggle Dock meter" aria-pressed="${dockOn}" title="Dock meter: ${dockOn ? "On" : "Off"}">▥</button><select id="interval" aria-label="Refresh interval" title="Refresh interval"><option value="10" ${store.settings.refreshIntervalSeconds === 10 ? "selected" : ""}>10s</option><option value="30" ${store.settings.refreshIntervalSeconds === 30 ? "selected" : ""}>30s</option><option value="60" ${store.settings.refreshIntervalSeconds === 60 ? "selected" : ""}>60s</option></select></div></main>`;
  root.querySelector("#refresh")?.addEventListener("click", () => controller.refresh());
  root.querySelector("#dock")?.addEventListener("click", async () => {
    suppressBlurUntil = Date.now() + 500;
    try { store.settings = await api.setDock(!store.settings.dockMeterVisible); render(); } catch (error) { console.error("Could not toggle Dock meter", error); }
  });
  root.querySelector<HTMLSelectElement>("#interval")?.addEventListener("change", async (event) => {
    const seconds = Number((event.target as HTMLSelectElement).value) as 10 | 30 | 60;
    try { store.settings = await api.setRefreshInterval(seconds); controller.setInterval(seconds); render(); } catch (error) { console.error("Could not update refresh interval", error); }
  });
};

// Only the main popover window owns refresh scheduling and its interval timer.
// The dock-meter window subscribes to "usage-updated" events and renders,
// but must not start its own refresh timer. createRefreshController is only
// called for the main window so its setInterval never runs on dock.
const controller: { refresh: () => void; setInterval: (s: number) => void } = isDockMeter
  ? { refresh: () => {}, setInterval: (_s: number) => {} }
  : createRefreshController(store, render, mockUsage ? async () => ({ snapshot: mockSnapshot, error: false }) : api.refresh);

api.snapshot().then((response) => { store.snapshot = mockUsage ? mockSnapshot : response.snapshot; store.loading = false; render(); }).catch(() => { store.snapshot = mockUsage ? mockSnapshot : null; store.loading = false; render(); });
api.settings().then((settings) => { store.settings = settings; settingsLoaded = true; render(); }).catch(() => { settingsLoaded = true; render(); });
api.onUsage((snapshot: UsageSnapshot) => { store.snapshot = mockUsage ? mockSnapshot : snapshot; render(); });
if (!isDockMeter) {
  controller.refresh();
}
