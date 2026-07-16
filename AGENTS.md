# AGENTS.md — Codex Meters

macOS-first Tauri menu-bar and Dock meter for displaying Codex `5h` and `7d` usage limits.

`Last documented commit: 98c892820e975d39ad435b28fb711dcebbd383ce`

## Project structure

| Path | Purpose |
| --- | --- |
| `src/` | Vite/TypeScript frontend, UI state, rendering, refresh behavior, and Tauri API calls. |
| `src-tauri/src/` | Rust/Tauri runtime, Codex provider integration, tray menu, windows, settings, and login-at-startup behavior. |
| `src-tauri/fixtures/` | Provider and usage fixtures used by tests and mock scenarios. |
| `src-tauri/tests/` | Rust usage and provider behavior tests. |
| `src-tauri/tauri.conf.json` | Tauri windows, bundle metadata, macOS behavior, and app identity. |
| `scripts/` | Package postinstall and macOS smoke-test scripts. |
| `install.sh` | Interactive DMG installer with a selectable destination folder. |
| `install-pkg.sh` | Automatic package installer for `/Applications`. |
| `Makefile` | Reproducible frontend build, native bundle, package, and release-asset workflow. |
| `releases/v0.1.0/` | Raw-downloadable `.pkg` and Apple Silicon `.dmg` assets used by curl installers. |
| `docs/` | Product specifications, compatibility notes, and implementation plans. |
| `docs/agents/` | Agent-focused project context; use its indexes before opening detail files. |

Only open the sections below when they are relevant to the current task.

| If you want to know... | Go to |
| --- | --- |
| Runtime architecture and data flow | [docs/agents/architecture/index.md](docs/agents/architecture/index.md) |
| Development, mock mode, and tests | [docs/agents/development/index.md](docs/agents/development/index.md) |
| Build, release, and installer workflow | [docs/agents/release/index.md](docs/agents/release/index.md) |
| Rules for safe changes and documentation updates | [docs/agents/maintenance/index.md](docs/agents/maintenance/index.md) |

## Maintenance note

When adding new context:

1. Put detail in the relevant `docs/agents/{category}/` subfile.
2. If the category does not exist, create the folder and an `index.md`.
3. Link the new subfile from the category `index.md`.
4. If it is a new top-level category, add a row to the table above and to `docs/agents/README.md`.
5. Never paste long details directly into `AGENTS.md`.
6. Any new document under `docs/agents/` must start with a short “when to read this” description, use an index table when it covers multiple subtopics, and keep long details in linked subfiles.

