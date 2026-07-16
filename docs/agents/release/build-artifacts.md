# Build artifacts

`make export` is the canonical one-shot release command. It always runs the latest frontend build first, then creates the native bundles from that same build:

1. `bun run build`
2. `bun run tauri build --bundles app dmg`
3. Copy `Codex Meters.app` and the DMG into `dmg/app/`.
4. Create `dmg/app/Codex Meters.pkg` with `pkgbuild` and `scripts/pkg`.
5. Copy the `.pkg` and Apple Silicon `.dmg` into `releases/v0.1.0/` for raw GitHub downloads.

Individual targets exist for `make dmg`, `make app`, and `make pkg`; they share the `bundle` target and still rebuild the frontend. `make clean` only removes local `dmg/app/` output.

The current project is Apple Silicon only. Keep the Tauri version, product name (`Codex Meters`), bundle identifier (`com.codex.tokenmeter`), package identifier, and artifact filenames aligned.

Back to [release/index.md](index.md)

