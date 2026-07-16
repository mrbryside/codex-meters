# Mock data

Use mock mode to verify UI states that are difficult to reproduce from a live subscription, especially simultaneous `5h` and `7d` windows, missing windows, stale values, malformed percentages, and provider failures.

Fixtures currently include:

- `usage-5h-7d.json`
- `usage-7d-only.json`
- `usage-duplicate-windows.json`
- `usage-failed.json`
- `usage-malformed-percent.json`
- `usage-unsupported-window.json`

When adding a fixture, add or update the corresponding Rust test and make sure the frontend handles unavailable windows without inventing a zero value. Mock refresh behavior should not call the live provider or replace mock data with a real response.

Back to [development/index.md](index.md)

