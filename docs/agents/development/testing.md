# Testing and verification

Run the smallest relevant checks first, then the release check when packaging behavior changes:

```sh
bun run build
bun test
cargo test --manifest-path src-tauri/Cargo.toml
sh -n install.sh install-pkg.sh scripts/pkg/postinstall
make export
```

`make export` is the important integration check for release changes: it rebuilds the frontend, compiles Tauri, creates the app and DMG, creates the package, and refreshes the raw-downloadable assets. Do not claim a release is verified if this command fails.

Back to [development/index.md](index.md)

