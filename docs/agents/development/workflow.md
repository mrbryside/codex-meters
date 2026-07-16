# Development workflow

Install dependencies once, then run Vite and Tauri in separate terminals:

```sh
bun install
bun run dev
bun run tauri dev
```

For mock usage data:

```sh
bun run dev:mock
bun run tauri:mock
```

The app is macOS-first. Keep the local Codex CLI/provider path behavior intact; `CODEX_CLI_PATH` can override the detected Codex executable when needed. Do not commit `node_modules/`, `dist/`, `src-tauri/target/`, or local release output under `dmg/app/`.

Back to [development/index.md](index.md)

