# Codex Meters

## Development modes

Run the Vite dev server and Tauri app in separate terminals:

```sh
bun run dev
bun run tauri dev
```

To preview mock `5h` and `7d` usage, use the matching mock mode in both terminals:

```sh
bun run dev:mock
bun run tauri:mock
```

## Install with curl

For the automatic installer, download the package and install it to `/Applications`:

```bash
curl -fL "https://github.com/mrbryside/codex-meters/releases/download/v0.1.0/Codex%20Meters.pkg" -o /tmp/Codex-Meters.pkg \
  && sudo installer -pkg /tmp/Codex-Meters.pkg -target / \
  && rm /tmp/Codex-Meters.pkg
```

The package installer registers Codex Meters to launch at login and opens it immediately after installation.

For an interactive drag-and-drop style installer, which lets you choose the destination folder:

```bash
curl -fsSL https://raw.githubusercontent.com/mrbryside/codex-meters/main/install.sh \
  | sh -s -- "https://github.com/mrbryside/codex-meters/releases/download/v0.1.0/Codex%20Meters_0.1.0_aarch64.dmg"
```

The release assets are generated with:

```bash
make export
```

This creates `dmg/app/Codex Meters.pkg`, `dmg/app/Codex Meters_0.1.0_aarch64.dmg`, and `dmg/app/Codex Meters.app`.

Personal macOS Tauri app that reads the signed-in Codex desktop app's local app-server rate limits and displays the available `5h` and `7d` remaining percentages.

## Run

```sh
bun install
bun run build
bun run tauri dev
```

The app uses `~/Applications/ChatGPT.app/Contents/Resources/codex` when available. Set `CODEX_CLI_PATH` to override it. It does not ask for an API key or store credentials. If Codex cannot provide a limit, the UI shows an unavailable state; if refresh fails after a successful fetch, cached bars remain visible with a stale warning.

This is an unsigned personal-use build. macOS may require opening it from Privacy & Security after the first launch.
