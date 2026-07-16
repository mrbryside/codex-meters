# Release and update checklist

Use this sequence for a release or any change that affects installers:

1. Update the version consistently in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`/`Cargo.lock` when required, `Makefile`, installer defaults, README examples, and the `releases/v{version}/` directory.
2. Run `bun run build`, `bun test`, Rust tests, and shell syntax checks.
3. Run `make export` so the app, DMG, package, and raw assets are generated from current source.
4. Check artifact names, file sizes, architecture, and package postinstall behavior.
5. Verify the raw script and binary URLs with `curl -fsSL -o /dev/null -w '%{http_code} %{size_download}\n' ...`.
6. Review `git status`, `git diff --cached`, and ensure generated files outside `releases/` are not accidentally staged.
7. Commit with a release-focused message and push to `origin main`.
8. If switching from raw repository assets to GitHub Releases, update both scripts and documentation together; do not leave curl URLs pointing at an unpublished release.

The current workflow keeps small personal-use installer binaries in Git so raw URLs work without a separate Release API step. If the project grows, move binaries to GitHub Releases or another artifact store and keep the installer URLs stable through a documented redirect/release policy.

Back to [release/index.md](index.md)

