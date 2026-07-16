import { describe, it, expect } from "bun:test";
import {
  normalizeSnapshot,
  type UsageSnapshot,
  type UsageWindow,
  type LimitWindow,
} from "./types";

describe("normalizeSnapshot", () => {
  it("accepts zero windows", () => {
    const snap = normalizeSnapshot([], { kind: "fresh", fetchedAt: "2026-07-16T00:00:00Z" });
    expect(snap.windows).toEqual([]);
    expect(snap.status.kind).toBe("fresh");
  });

  it("accepts one window (5h only)", () => {
    const windows: UsageWindow[] = [
      { window: "5h", remainingPercent: 75, resetAt: "2026-07-16T12:00:00Z" },
    ];
    const snap = normalizeSnapshot(windows, { kind: "fresh", fetchedAt: "2026-07-16T00:00:00Z" });
    expect(snap.windows).toHaveLength(1);
    expect(snap.windows[0]!.window).toBe("5h");
    expect(snap.windows[0]!.remainingPercent).toBe(75);
  });

  it("accepts two unique windows (5h + 7d)", () => {
    const windows: UsageWindow[] = [
      { window: "5h", remainingPercent: 75, resetAt: "2026-07-16T12:00:00Z" },
      { window: "7d", remainingPercent: 40, resetAt: "2026-07-22T00:00:00Z" },
    ];
    const snap = normalizeSnapshot(windows, { kind: "fresh", fetchedAt: "2026-07-16T00:00:00Z" });
    expect(snap.windows).toHaveLength(2);
    const kinds = snap.windows.map((w) => w.window);
    expect(kinds).toContain("5h");
    expect(kinds).toContain("7d");
  });

  it("clamps negative percentages to 0", () => {
    const windows: UsageWindow[] = [
      { window: "5h", remainingPercent: -5, resetAt: null },
    ];
    const snap = normalizeSnapshot(windows, { kind: "fresh", fetchedAt: "2026-07-16T00:00:00Z" });
    expect(snap.windows[0]!.remainingPercent).toBe(0);
  });

  it("clamps percentages >100 to 100", () => {
    const windows: UsageWindow[] = [
      { window: "5h", remainingPercent: 101, resetAt: null },
    ];
    const snap = normalizeSnapshot(windows, { kind: "fresh", fetchedAt: "2026-07-16T00:00:00Z" });
    expect(snap.windows[0]!.remainingPercent).toBe(100);
  });

  it("rejects duplicate windows", () => {
    const windows: UsageWindow[] = [
      { window: "5h", remainingPercent: 50, resetAt: null },
      { window: "5h", remainingPercent: 60, resetAt: null },
    ];
    expect(() => normalizeSnapshot(windows, { kind: "fresh", fetchedAt: "2026-07-16T00:00:00Z" })).toThrow();
  });
});
