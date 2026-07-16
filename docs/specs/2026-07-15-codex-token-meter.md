# Codex Token Meter

## Goal

Build a personal-use macOS Tauri 2 utility that mirrors the current Codex subscription usage limits in two places:

1. A compact status item in the macOS menu bar.
2. An optional two-row floating meter positioned immediately to the right of the bottom Dock, aligned on the Dock's horizontal baseline.

The app must reuse the user's existing local Codex/ChatGPT session, require no API key, and degrade clearly when a usage window is missing or retrieval fails.

## Scope

### In scope

- macOS only, with Tauri 2, Rust commands, and a TypeScript/Vite frontend.
- Personal, standalone `.app`/`.dmg` distribution outside the Mac App Store.
- Menu-bar status item with compact remaining-limit bars for available `5h` and `7d` windows.
- Clickable menu-bar popover containing detailed values, reset times, refresh, Open Codex, settings, and Quit.
- Optional Dock-adjacent floating meter enabled by default.
- Bottom Dock positioning only for the first release.
- Dock-meter positioning beside the Dock on the display where the Dock is active; no duplicate Dock meter on other displays.
- Hiding the Dock meter when the Dock auto-hides; keeping the menu-bar status item available.
- Menu-bar toggle for showing/hiding the Dock meter.
- Both the menu-bar item and Dock meter opening the same detail popover.
- Automatic refresh every 60 seconds, refresh on popover open, and manual refresh.
- Independent optional handling of `5h` and `7d` windows. Missing windows are hidden rather than rendered as disabled rows.
- Last-known values retained in memory when a refresh fails, with a visible failed-status indicator.
- A distinct unavailable state when no successful value has ever been loaded.
- Codex-inspired dark translucent-glass styling with blue/purple accents.
- Automatic status colors: green above 50% remaining, yellow from 20% through 50%, red below 20%, and neutral gray for failed/unavailable status.
- Launch at login enabled by default, no normal Dock app icon, and Quit from the popover.
- Open Codex desktop app when installed, falling back to the Codex web page.
- No historical usage storage and no new credential storage.

### Out of scope

- Windows or Linux support.
- Left- or right-side Dock positioning.
- Mac App Store distribution.
- API billing/token accounting.
- User-entered API keys or alternate providers.
- Accessibility/UI scraping of the Codex interface.
- Usage history, charts, notifications, alerts, or cloud sync.
- Multiple simultaneous Dock meters.

## Chosen approach

### Local Codex session adapter

The Rust backend owns a `CodexUsageProvider` abstraction that reads the existing local Codex/ChatGPT session through the locally available first-party session state and usage source. The frontend never receives credentials and never calls a remote usage endpoint directly.

The provider normalizes the source into independent `5h` and `7d` usage windows. The adapter returns a source-unavailable error when the local session or usage source cannot provide data; the app does not prompt for an API key or fall back to UI automation.

This approach won because it best matches subscription limits, preserves the user's zero-configuration expectation, avoids API billing semantics, and keeps authentication inside the local Rust process. Its known trade-off is dependence on a private or changing Codex usage interface; the provider boundary and fixture-backed tests limit the blast radius of future changes.

## Decisions

### Product surfaces

- The menu-bar status item is always available while the app runs.
- The Dock meter is a separate borderless floating window, not a modification to the Dock.
- The Dock meter is enabled by default and can be toggled from the menu-bar popover.
- The Dock meter contains one row per available window. If only `7d` is returned, the widget shrinks to one row.
- Both entry points open the same popover and share one usage state.

### Usage semantics

- Display remaining percentage, not used percentage.
- `5h` and `7d` are independent optional windows.
- A missing window is omitted from all surfaces.
- A failed refresh preserves the last successful windows and changes the status to failed.
- If no successful snapshot exists, the UI displays unavailable rather than fabricated values.
- Reset timestamps are shown in the detail popover when supplied by the provider.

### Refresh and lifecycle

- Refresh interval: 60 seconds.
- Refresh immediately when the popover opens.
- Manual refresh is available in the popover.
- Launch at login is enabled by default.
- The app uses macOS accessory/application-agent behavior so it does not create a normal Dock application icon.
- Quitting is available from the popover.

### Window positioning

- First release supports the bottom Dock only.
- The Rust window manager detects the active display and bottom Dock geometry, then positions the meter immediately to the right of the Dock with a fixed gap and vertical center alignment.
- Dock geometry is re-evaluated after display changes, Dock preference changes, and meter visibility changes.
- The meter is hidden while macOS reports the Dock as auto-hidden and re-shown when the Dock is visible.
- The meter is hidden in full-screen spaces to avoid obstructing full-screen content.
- The meter remains above normal windows while visible.
- The meter is not draggable in the first release; positioning follows the Dock.

### Visual design

- Dark translucent rounded rectangle consistent with the macOS Dock.
- Codex blue/purple accent for normal bars.
- Status color thresholds are based on remaining percentage: `>50` green, `20..=50` yellow, `<20` red.
- Failed or unavailable status uses gray and a status label/icon rather than a misleading percentage.
- The menu-bar item uses the smallest readable bars and hides unavailable windows.

### Distribution and privacy

- Personal-use unsigned `.dmg` is the first release target.
- No App Store packaging.
- No usage history is persisted.
- No new credentials are persisted.
- Provider failures must not log tokens, cookies, authorization headers, or raw session contents.

## Interfaces and contracts

### Shared TypeScript model

```ts
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
};
```

### Rust provider contract

```rust
pub trait CodexUsageProvider: Send + Sync {
    fn fetch_usage(&self) -> Result<ProviderSnapshot, ProviderError>;
}

pub struct ProviderSnapshot {
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub windows: Vec<ProviderWindow>,
}

pub struct ProviderWindow {
    pub window: LimitWindow,
    pub remaining_percent: f32,
    pub reset_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub enum ProviderError {
    SessionUnavailable,
    UsageSourceUnavailable,
    MalformedUsageData(String),
    Transport(String),
}
```

The provider must clamp accepted percentages to `0.0..=100.0`, reject duplicate windows, ignore unsupported windows, and return an empty successful snapshot when the source explicitly reports no supported windows.

### Tauri commands

```rust
#[tauri::command]
fn get_usage_snapshot(state: tauri::State<AppState>) -> Result<UsageSnapshot, AppError>;

#[tauri::command]
fn refresh_usage(state: tauri::State<AppState>) -> Result<UsageSnapshot, AppError>;

#[tauri::command]
fn get_app_settings(state: tauri::State<AppState>) -> Result<AppSettings, AppError>;

#[tauri::command]
fn set_dock_meter_visible(
    visible: bool,
    state: tauri::State<AppState>,
) -> Result<AppSettings, AppError>;

#[tauri::command]
fn set_launch_at_login(
    enabled: bool,
    state: tauri::State<AppState>,
) -> Result<AppSettings, AppError>;

#[tauri::command]
fn open_codex() -> Result<(), AppError>;
```

`AppError` is serializable and has the shape:

```rust
pub struct AppError {
    pub code: String,
    pub message: String,
}
```

The frontend invokes `get_usage_snapshot` on startup, `refresh_usage` on the 60-second timer, on popover open, and for the manual refresh action. Settings commands persist only the two boolean settings.

### Frontend events and state

The Rust backend emits `usage-updated` with a `UsageSnapshot` after every successful or failed refresh, and `dock-meter-visibility-changed` with `AppSettings` after the Dock visibility changes. The frontend keeps one in-memory `UsageSnapshot` and one persisted `AppSettings` value, rendering both surfaces from those shared values.

### Window contract

The Rust window manager owns a named Dock meter window `dock-meter` and a named detail window `usage-popover`. The menu-bar status item is represented by the app's tray/status item. The Dock meter has no title bar, is transparent, is always-on-top while visible, and is excluded from normal app switching. The detail window is shown adjacent to the clicked surface and hidden when focus leaves it.

## Dependencies and order

1. Establish the empty-repository Tauri 2 shell and test harness.
2. Validate the local Codex session/usage source and implement the provider fixture boundary.
3. Implement normalized usage state, stale/failed handling, and refresh scheduling.
4. Implement persistent settings and launch-at-login behavior.
5. Implement macOS Dock geometry detection, display selection, auto-hide detection, and floating-window positioning.
6. Implement the shared frontend model and status-bar/detail UI.
7. Implement the Dock meter UI and shared popover wiring.
8. Implement Codex-themed styling and status thresholds.
9. Add packaging, personal-use launch instructions, and end-to-end macOS verification.

## Open risks

- The subscription usage source may be private, undocumented, or changed by Codex updates. The provider trait, source adapter, and fixture tests isolate this risk; a source failure results in a truthful unavailable/failed state rather than fabricated usage.
- macOS Dock geometry and auto-hide state are not exposed through one stable cross-version API. Positioning must be verified on supported macOS versions and re-run after display/preference notifications.
- A floating window beside the Dock may be constrained by Spaces, full-screen windows, or Dock animation timing. The initial implementation should debounce geometry updates and hide during full-screen spaces.
- An unsigned personal `.dmg` may require the user to approve the app in macOS security settings.
- Codex session state may be stored in a keychain or file-backed store depending on the local installation. The adapter must support the currently detected local store without copying credentials into app storage.
