# Codex Usage Source

## Status

**Validated and wired through an adapter.** The installed Codex CLI exposes the local app-server protocol used by the desktop app. The provider launches that CLI, requests rate limits, and never handles the underlying authentication material.

## What We Know

### Concrete source

- CLI path: `$CODEX_CLI_PATH` when set, otherwise `~/Applications/ChatGPT.app/Contents/Resources/codex`, with `codex` as the final executable fallback.
- Transport: `codex app-server --stdio`.
- Handshake: JSON-RPC `initialize`, `initialized`, then `account/rateLimits/read` with `params: null`.
- Response: `rateLimitsByLimitId.codex` when present, otherwise `rateLimits`.
- Window mapping: `windowDurationMins = 300` maps to `5h`; `windowDurationMins = 10080` maps to `7d`; `remainingPercent = 100 - usedPercent`.
- Reset timestamps are Unix seconds and are normalized to ISO-8601.
- The child process has a 10-second timeout; stdout/stderr are not logged and credentials remain inside Codex.

### Normalized target data shape

The Codex usage endpoint returns JSON with the following structure:

```json
{
  "windows": [
    {
      "window": "5h",
      "remainingPercent": 72.5,
      "resetAt": "2026-07-16T08:00:00Z"
    },
    {
      "window": "7d",
      "remainingPercent": 45.0,
      "resetAt": "2026-07-22T00:00:00Z"
    }
  ]
}
```

Or an error response:

```json
{
  "error": "session_expired",
  "message": "Local session data could not be read"
}
```

### Supported Windows

| Window | Description |
|--------|-------------|
| `5h`   | Last 5 hours of usage |
| `7d`   | Last 7 days of usage |

Unsupported windows (e.g., `30d`) are silently filtered out.

## Security Boundary

- **Do not** read, print, copy, or persist auth tokens, cookies, authorization headers, raw session contents, or secrets.
- **Do not** attempt UI automation.
- Only inspect non-sensitive file names/metadata needed to identify the source.
- Error messages must not contain any sensitive data (enforced by tests).

## Implementation Notes

The `CodexUsageProvider` trait in `src-tauri/src/provider.rs` is designed to be implemented for any source type:

- `FileBackedProvider` — reads from a local file path.
- `SourceUnavailableAdapter` — safe fallback when the CLI cannot be launched.
- `CodexAppServerProvider` — launches the installed Codex CLI app-server and parses `account/rateLimits/read`.

## Verification

The protocol was validated locally with the installed Codex CLI and the generated app-server schema. Production tests use a captured non-sensitive JSON-RPC response fixture; live requests remain opt-in to avoid logging or persisting account data.
