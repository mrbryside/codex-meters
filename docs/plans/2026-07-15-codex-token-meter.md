# Codex Token Meter Implementation Plan

This plan implements `docs/specs/2026-07-15-codex-token-meter.md` as a macOS-only Tauri 2 application with a vanilla TypeScript/Vite frontend and Rust backend.

## Task 1: Establish the Tauri 2 project shell and test boundaries

**Files:** create `package.json`, `tsconfig.json`, `vite.config.ts`, `index.html`, `src/main.ts`, `src/types.ts`, `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/src/error.rs`, `src-tauri/src/state.rs`; test `src/types.test.ts`, `src-tauri/src/error.rs`.

**Depends on:** none

- [x] Write the TypeScript model test asserting `UsageSnapshot` accepts zero, one, and two unique windows and rejects out-of-range percentages through the normalization helper.
- [x] Run the TypeScript test and confirm it fails because the project files and normalization helper do not exist.
- [x] Create the Tauri 2/Vite shell and define `LimitWindow`, `UsageWindow`, `UsageStatus`, `UsageSnapshot`, and `AppSettings` exactly as specified.
- [x] Add Rust `AppError`, `AppState`, and the Tauri application entrypoint with an empty command registration list.
- [x] Run TypeScript and Rust unit tests and confirm they pass.

## Task 2: Validate the local Codex usage source and freeze fixtures

**Files:** create `docs/compatibility/codex-usage-source.md`, `src-tauri/src/provider.rs`, `src-tauri/tests/provider_fixtures.rs`, `src-tauri/fixtures/usage-5h-7d.json`, `src-tauri/fixtures/usage-7d-only.json`, `src-tauri/fixtures/usage-failed.json`.

**Depends on:** Task 1

- [x] Write provider tests for a full `5h`/`7d` response, a `7d`-only response, malformed percentages, duplicate windows, and a missing local session.
- [x] Run the provider tests and confirm they fail because `CodexUsageProvider`, fixture parsing, and error mapping are absent.
- [x] Inspect the local Codex/ChatGPT session state available on the target Mac without copying credentials into the repository; record the concrete source location, access method, and observed response shape in `docs/compatibility/codex-usage-source.md`.
- [x] Implement `CodexUsageProvider`, `ProviderSnapshot`, `ProviderWindow`, and `ProviderError`; isolate the concrete source reader behind the trait and map only `5h` and `7d` into normalized windows.
- [x] Enforce percentage clamping, duplicate rejection, unsupported-window filtering, and redaction-safe error messages.
- [x] Run provider tests and confirm they pass against checked-in non-sensitive fixtures.

## Task 3: Implement snapshot caching, stale status, and refresh commands

**Files:** create `src-tauri/src/usage.rs`, `src-tauri/src/commands.rs`, `src-tauri/tests/usage_state.rs`; modify `src-tauri/src/state.rs`, `src-tauri/src/lib.rs`.

**Depends on:** Task 2

- [x] Write tests asserting successful refresh returns `fresh`, failed refresh preserves the prior windows as `stale`, and first-load failure returns `unavailable`.
- [x] Run the tests and confirm they fail because `UsageService` and the Tauri commands are absent.
- [x] Implement `UsageService::snapshot()` and `UsageService::refresh()` with one in-memory last-successful snapshot and no credential or history persistence.
- [x] Implement `get_usage_snapshot` and `refresh_usage` with serializable `AppError` responses.
- [x] Emit `usage-updated` after every refresh outcome and register both commands in `src-tauri/src/lib.rs`.
- [x] Run Rust tests and confirm the state and command-layer tests pass.

## Task 4: Add persisted settings and launch-at-login control

**Files:** create `src-tauri/src/settings.rs`, `src-tauri/src/login_launch.rs`, `src-tauri/tests/settings.rs`; modify `src-tauri/src/state.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`.

**Depends on:** Task 1

- [ ] Write tests asserting first launch defaults to `dockMeterVisible: true` and `launchAtLogin: true`, and that both booleans round-trip through the settings store.
- [ ] Run the tests and confirm they fail because the settings store and launch-at-login adapter are absent.
- [ ] Implement settings persistence for only the two booleans, using an app-local preferences file with restrictive permissions and no usage/auth data.
- [ ] Implement `set_dock_meter_visible`, `set_launch_at_login`, and `get_app_settings`; emit `dock-meter-visibility-changed` after updates.
- [ ] Implement the macOS launch-at-login adapter and default the app to accessory/application-agent behavior.
- [ ] Run Rust settings tests and confirm they pass.

## Task 5: Implement macOS Dock geometry and floating-window management

**Files:** create `src-tauri/src/macos_windows.rs`, `src-tauri/tests/macos_windows.rs`; modify `src-tauri/src/lib.rs`, `src-tauri/src/settings.rs`.

**Depends on:** Task 4

- [ ] Write geometry tests for bottom Dock placement, active-display selection, fixed gap to the right of the Dock, vertical centering, and one-row/two-row meter heights.
- [ ] Run the tests and confirm they fail because the geometry calculator and window manager are absent.
- [ ] Implement `DockGeometry`, `DisplayGeometry`, and `DockMeterPosition` with deterministic pure calculations before connecting to macOS APIs.
- [ ] Implement the named `dock-meter` borderless transparent window with always-on-top behavior, no normal app-switcher entry, and visibility controlled by `AppSettings`.
- [ ] Add debounced re-positioning on display changes, Dock preference changes, auto-hide changes, and usage-row count changes.
- [ ] Hide the Dock meter during auto-hide and full-screen spaces; restore it when the Dock is visible and the setting is enabled.
- [ ] Run Rust geometry tests and perform a manual macOS smoke test with the Dock at the bottom on one and two displays.

## Task 6: Build the shared frontend state and refresh controller

**Files:** create `src/ui/state.ts`, `src/ui/tauri-api.ts`, `src/ui/refresh-controller.ts`, `src/ui/state.test.ts`, `src/ui/refresh-controller.test.ts`; modify `src/main.ts`, `src/types.ts`.

**Depends on:** Tasks 3 and 4

- [ ] Write frontend tests for startup load, immediate refresh on popover open, 60-second refresh scheduling, manual refresh, stale snapshots, and hidden unavailable windows.
- [ ] Run the tests and confirm they fail because the Tauri API adapter and shared state store are absent.
- [ ] Implement typed wrappers for `get_usage_snapshot`, `refresh_usage`, `get_app_settings`, `set_dock_meter_visible`, `set_launch_at_login`, and `open_codex`.
- [ ] Implement one shared store containing `UsageSnapshot` and `AppSettings`; subscribe to `usage-updated` and `dock-meter-visibility-changed`.
- [ ] Implement a refresh controller that runs once on startup, on popover open, every 60 seconds, and on manual refresh, with timer cleanup on window teardown.
- [ ] Run frontend tests and confirm they pass.

## Task 7: Implement menu-bar status item and detail popover

**Files:** create `src/ui/menu-bar.ts`, `src/ui/popover.ts`, `src/ui/popover.test.ts`, `src/ui/dom.ts`.

**Depends on:** Task 6

- [ ] Write DOM tests asserting the menu-bar item renders only available windows and the popover renders reset times, fresh/stale/unavailable status, refresh, Open Codex, Dock toggle, launch-at-login toggle, and Quit.
- [ ] Run the DOM tests and confirm they fail because the menu-bar and popover renderers are absent.
- [ ] Implement compact menu-bar bars for available `5h` and `7d` windows, hiding missing windows and preserving the failed indicator when stale.
- [ ] Implement the shared `usage-popover` window content and wire both settings toggles to typed Tauri commands.
- [ ] Make `open_codex` prefer the installed desktop app and fall back to the Codex web page; expose Quit through the Tauri process API.
- [ ] Run frontend DOM tests and verify the popover manually on macOS.

## Task 8: Implement the Dock meter and shared interactions

**Files:** create `src/ui/dock-meter.ts`, `src/ui/dock-meter.test.ts`, `src-tauri/src/dock_meter.rs`; modify `src/main.ts`, `src-tauri/src/lib.rs`.

**Depends on:** Tasks 5 and 7

- [ ] Write tests asserting the Dock meter renders two rows for `5h`/`7d`, one row for `7d` only, no row for missing windows, and stale status without clearing cached bars.
- [ ] Run the tests and confirm they fail because the Dock meter renderer and click wiring are absent.
- [ ] Implement the two-row Dock meter with the same shared state and the same click action as the menu-bar item.
- [ ] Connect the Dock meter row count to the Rust window-height calculation and reposition it after usage updates.
- [ ] Ensure the menu-bar toggle immediately hides/shows the Dock meter and that the setting survives restart.
- [ ] Run frontend tests and perform a manual interaction test from both the menu-bar item and Dock meter.

## Task 9: Apply Codex theme, status colors, and failure states

**Files:** create `src/ui/styles.css`, `src/ui/status-colors.ts`, `src/ui/status-colors.test.ts`; modify `src/main.ts`, `src/ui/menu-bar.ts`, `src/ui/popover.ts`, `src/ui/dock-meter.ts`.

**Depends on:** Tasks 7 and 8

- [ ] Write tests for green `>50`, yellow `20..=50`, red `<20`, and gray stale/unavailable states.
- [ ] Run the tests and confirm they fail because status color classification and theme styles are absent.
- [ ] Implement the threshold classifier and apply accessible color plus text/icon status indicators.
- [ ] Add dark translucent-glass surfaces, rounded corners, Codex blue/purple accents, compact typography, hover/focus states, and reduced-motion-safe transitions.
- [ ] Verify the one-row and two-row layouts do not change the Dock meter's horizontal attachment or clip reset text in the popover.
- [ ] Run frontend tests and visually inspect the menu-bar item, popover, and Dock meter in light and dark macOS appearances.

## Task 10: Package and verify the personal macOS release

**Files:** create `README.md`, `src-tauri/Info.plist`, `scripts/smoke-test-macos.sh`; modify `src-tauri/tauri.conf.json`, `package.json`.

**Depends on:** Tasks 1 through 9

- [ ] Write the smoke-test checklist for first launch, existing-session success, `7d`-only data, failed refresh with cached values, first-load unavailable, Dock auto-hide, two displays, menu-bar toggle, launch at login, Open Codex fallback, and Quit.
- [ ] Run the smoke-test script before packaging and confirm it fails until the built app and required checks exist.
- [ ] Configure unsigned personal-use `.app` and `.dmg` bundling, accessory application behavior, and the `Codex Token Meter` app metadata.
- [ ] Add README instructions covering installation, Gatekeeper approval for an unsigned personal app, launch-at-login behavior, and the failed/unavailable indicators.
- [ ] Build the `.dmg`, run every smoke-test case on macOS, and record any platform-specific findings in the README.
- [ ] Run the final TypeScript, Rust, and packaging checks and confirm they pass.

## Self-check

- Every product decision in the design is represented: menu-bar bars, Dock toggle, two-row meter, hidden missing windows, stale failed status, 60-second refresh, bottom Dock, active display, auto-hide, Codex theme, launch at login, personal `.dmg`, and no API-key setup.
- The provider boundary is explicit and testable before the concrete local Codex source is wired in.
- Tasks have one primary responsibility, exact file boundaries, dependency markings, failing-test-first steps, minimal implementation steps, and verification steps.
- No implementation code, configuration, build output, or generated project files were created during planning.
