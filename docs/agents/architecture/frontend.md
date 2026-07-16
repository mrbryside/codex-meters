# Frontend architecture

The frontend is a small Vite/TypeScript app. `src/main.ts` coordinates application state, refresh scheduling, menu-bar content, popover behavior, and Dock-meter behavior. Keep UI decisions in the UI modules rather than duplicating state in Rust.

| File | Responsibility |
| --- | --- |
| `src/main.ts` | Application orchestration, event handlers, window geometry, settings synchronization, and usage refresh. |
| `src/types.ts` | Shared usage and settings types. |
| `src/ui/state.ts` | Frontend state representation. |
| `src/ui/render.ts` | DOM rendering for bars, percentages, reset times, stale status, and controls. |
| `src/ui/styles.css` | Codex-themed layout, sizing, colors, borders, and responsive meter rows. |
| `src/ui/refresh-controller.ts` | Refresh interval scheduling and refresh-source injection for mock mode. |
| `src/ui/tauri-api.ts` | Frontend wrapper around Tauri commands/events. |

Production frontend output is `dist/`. `bun run build` runs TypeScript checking before Vite emits the bundle. The normal frontend mock flag is `VITE_MOCK_USAGE=true`; it must be paired with the Tauri mock flag when running the full app.

Back to [architecture/index.md](index.md)

