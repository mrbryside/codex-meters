# Testing and verification

Run the smallest relevant checks first, then the release check when packaging behavior changes:

```sh
bun run build
bun test
cargo test --manifest-path src-tauri/Cargo.toml
sh -n install.sh install-pkg.sh scripts/pkg/postinstall scripts/verify-pkg.sh
make export
scripts/verify-pkg.sh "dmg/app/Codex Meters.pkg"
```

`make export` is the important integration check for release changes: it rebuilds the frontend, compiles Tauri, creates the app and DMG, creates the package, and refreshes the raw-downloadable assets. Do not claim a release is verified if this command fails.

The package verifier checks the flat package without installing it: the payload must be rooted at `Applications/Codex Meters.app`, the package install location must be `/`, the package must contain `postinstall`, and relocation metadata must be absent. Use `cmp` to confirm the exported package and the copied release package are identical.

Back to [development/index.md](index.md)
