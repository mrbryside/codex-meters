// Codex Token Meter — shared TypeScript model

export type LimitWindow = "5h" | "7d";

export type UsageWindow = {
  window: LimitWindow;
  remainingPercent: number; // inclusive range 0..100
  resetAt: string | null; // ISO-8601 timestamp when known
};

export type UsageStatus =
  | { kind: "fresh"; fetchedAt: string }
  | { kind: "stale"; fetchedAt: string; failedAt: string; message: string }
  | { kind: "unavailable"; message: string };

export type UsageSnapshot = {
  windows: UsageWindow[]; // zero, one, or two entries; at most one per window
  status: UsageStatus;
};

export type AppSettings = {
  dockMeterVisible: boolean; // default true
  launchAtLogin: boolean; // default true
  refreshIntervalSeconds: 10 | 30 | 60;
  dockMeterGeometry: DockMeterGeometry | null;
};

export type DockMeterGeometry = {
  x: number;
  y: number;
  width: number;
  height: number;
};

/**
 * Normalize raw usage windows into a UsageSnapshot.
 *
 * Validates:
 * - Percentages are in the inclusive range 0..100 (clamps out-of-range values).
 * - No duplicate LimitWindow values.
 *
 * Throws on duplicate windows; clamps percentages instead of throwing.
 */
export function normalizeSnapshot(
  rawWindows: UsageWindow[],
  status: UsageStatus,
): UsageSnapshot {
  // Clamp percentages to [0, 100]
  const clamped = rawWindows.map((w) => ({
    ...w,
    remainingPercent: Math.max(0, Math.min(100, w.remainingPercent)),
  }));

  // Reject duplicates
  const seen = new Set<LimitWindow>();
  for (const w of clamped) {
    if (seen.has(w.window)) {
      throw new Error(`Duplicate window: ${w.window}`);
    }
    seen.add(w.window);
  }

  return { windows: clamped, status };
}
