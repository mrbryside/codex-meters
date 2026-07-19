# Build artifacts

`make export` is the canonical one-shot release command. It always runs the latest frontend build first, then creates the native bundles from that same build:

1. `bun run build`
2. `bun run tauri build --bundles app dmg`
3. Copy `Codex Meters.app` and the DMG into `dmg/app/`.
4. Stage the app at `Applications/Codex Meters.app`, then create `dmg/app/Codex Meters.pkg` with `pkgbuild`, `scripts/pkg-components.plist`, and `scripts/pkg`. The component metadata marks the bundle non-relocatable so PackageKit cannot move it to a development copy.
5. Copy the `.pkg` and Apple Silicon `.dmg` into `releases/v0.1.0/` for raw GitHub downloads.

`scripts/verify-pkg.sh` runs before the package is copied to `releases/`. It validates the payload path, `/` install location, embedded postinstall script, and absence of `<relocate>` metadata. Keep the staging root and package recipe shared by `make export` and `make pkg`.

Individual targets exist for `make dmg`, `make app`, and `make pkg`; they share the `bundle` target and still rebuild the frontend. `make clean` only removes local `dmg/app/` output.

The current project is Apple Silicon only. Keep the Tauri version, product name (`Codex Meters`), bundle identifier (`com.codex.tokenmeter`), package identifier, and artifact filenames aligned.

Back to [release/index.md](index.md)
