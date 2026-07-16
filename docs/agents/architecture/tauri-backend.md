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

Back to [architecture/index.md](index.md)

