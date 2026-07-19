# Tauri backend and Codex provider

The Rust side reads the signed-in Codex desktop app's local app-server rate-limit data and exposes normalized usage to the frontend. It does not require an API key and must not add one.

| File | Responsibility |
| --- | --- |
| `src-tauri/src/provider.rs` | Locate and query the Codex provider, parse responses, and handle provider failures. |
| `src-tauri/src/usage.rs` | Normalize supported windows, percentages, reset timestamps, unavailable values, and duplicate windows. |
| `src-tauri/src/commands.rs` | Tauri commands used by the frontend for usage, settings, geometry, and controls. |
| `src-tauri/src/state.rs` | Shared runtime state and event coordination. |
| `src-tauri/src/settings.rs` | Persist refresh interval, Dock mode, launch behavior, and Dock geometry. |
| `src-tauri/src/login_launch.rs` | Register launch-at-login using macOS launch services/fallback behavior. |
| `src-tauri/src/lib.rs` | Tauri setup, tray/menu-bar meter, windows, activation policy, and event wiring. |

The mock backend is enabled with `CODEX_MOCK_USAGE=true`. Provider fixtures under `src-tauri/fixtures/` are the source of truth for supported, failed, malformed, duplicate, and 5h/7d-only scenarios.

`CodexAppServerProvider` resolves the executable in this order: `CODEX_CLI_PATH`, the system and user ChatGPT application bundles, known Homebrew/CLI locations, then PATH. The resolver validates files and has isolated precedence tests; keep GUI-launch environments in mind because Finder does not inherit a terminal-only PATH.

`UsageService` keeps the current UI snapshot separate from the last genuinely successful snapshot. Failures before any success are `Unavailable`; failures after success are `Stale` while preserving the last successful windows. Do not hold the cache lock while calling the provider.

The tray starts with a visible neutral glyph even when no usage windows are available. Only the main popover creates the frontend refresh controller; the Dock meter consumes usage events without starting another timer.

Back to [architecture/index.md](index.md)
