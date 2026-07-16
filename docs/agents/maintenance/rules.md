# Project rules

These are operational rules for future changes:

- Keep the app macOS-first and Apple Silicon compatible unless the user explicitly expands the target platform.
- Preserve menu-bar/accessory behavior: the app should not appear as a normal Dock app, and the Dock meter should remain a separate optional window.
- Treat live Codex usage values as unavailable when the provider does not expose a window. Do not convert missing `5h` or `7d` data into a fake `0%` bar.
- Keep mock mode isolated from live provider calls. Add a fixture and test when introducing a new usage or failure state.
- Persist user-selected refresh interval, Dock visibility, Dock geometry, and other settings across reloads and relaunches.
- Do not add API keys, tokens, or credentials to source, fixtures, logs, release assets, or documentation. Use environment variables only for local paths and mock flags.
- Run focused tests before packaging. Any change to release, installer, startup, window behavior, or versioning must include `make export` verification.
- Keep `README.md`, installer defaults, `Makefile`, release asset paths, and version strings synchronized.
- Do not manually edit generated Tauri icons or bundle output when a source/config change and regeneration is appropriate.
- Preserve executable permissions on `install.sh`, `install-pkg.sh`, `scripts/pkg/postinstall`, and smoke-test scripts.

If a rule changes, update this file, the affected detailed architecture/release document, and the relevant README section in the same change.

Back to [maintenance/index.md](index.md)

