# Windows and persisted state

Codex Meters is a menu-bar/accessory app. The main popover is hidden until the menu-bar item is clicked; the Dock meter is a separate always-on-top, transparent, resizable window that can be shown across workspaces without appearing in the Dock.

The two Tauri windows are defined in `src-tauri/tauri.conf.json`:

- `main`: hidden popover window for controls and usage details.
- `dock-meter`: compact Dock-adjacent meter with min/max dimensions, resize support, all-workspace visibility, and skipped taskbar/Dock presence.

`src/main.ts` restores saved Dock size and position through the settings commands. Manual resize and move events must update persisted geometry; reload and relaunch must not silently replace a user-selected size or position. Keep blur handling, click-outside hiding, and Dock toggle state synchronized so toggling the Dock meter does not accidentally close the menu-bar popover.

macOS accessory behavior is part of the product requirement: preserve `LSUIElement` in `src-tauri/Info.plist` and the activation-policy setup in Rust unless the user explicitly asks to change Dock visibility.

Back to [architecture/index.md](index.md)

