# Installers and curl URLs

There are two supported curl entry points:

```sh
curl -fsSL https://raw.githubusercontent.com/mrbryside/codex-meters/main/install-pkg.sh | sh
curl -fsSL https://raw.githubusercontent.com/mrbryside/codex-meters/main/install.sh | sh
```

`install-pkg.sh` downloads `releases/v0.1.0/Codex Meters.pkg`, validates its payload and metadata without requiring a repository checkout, asks for administrator permission, and installs to `/Applications`. It verifies the real bundle, executable, and bundle identifier after installation; a stale receipt is retried once, and the script never reports success when `/Applications/Codex Meters.app` is missing. `scripts/pkg/postinstall` opens the app as the console user. The package also preserves the background/accessory app behavior and launch-at-login registration implemented by the app.

`install.sh` downloads the DMG from the same raw asset folder, mounts it, and lets the user choose `/Applications`, `~/Applications`, or a custom destination before copying the app with `ditto`. It is interactive, requires a TTY, and does not run package postinstall scripts.

When a release path or version changes, update both installer defaults and README examples. The raw asset URLs must return HTTP 200 before the command is considered ready.

Back to [release/index.md](index.md)
