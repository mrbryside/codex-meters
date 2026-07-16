# Installers and curl URLs

There are two supported curl entry points:

```sh
curl -fsSL https://raw.githubusercontent.com/mrbryside/codex-meters/main/install-pkg.sh | sh
curl -fsSL https://raw.githubusercontent.com/mrbryside/codex-meters/main/install.sh | sh
```

`install-pkg.sh` downloads `releases/v0.1.0/Codex Meters.pkg`, asks for administrator permission, installs to `/Applications`, and relies on `scripts/pkg/postinstall` to open the app. The package also preserves the background/accessory app behavior and launch-at-login registration implemented by the app.

`install.sh` downloads the DMG from the same raw asset folder, mounts it, and lets the user choose `/Applications`, `~/Applications`, or a custom destination. It is interactive and does not pretend that a drag-and-drop install can run postinstall scripts.

When a release path or version changes, update both installer defaults and README examples. The raw asset URLs must return HTTP 200 before the command is considered ready.

Back to [release/index.md](index.md)

