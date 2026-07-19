use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persisted application settings stored in the app-local preferences directory.
///
/// Only two booleans are persisted — no usage data or auth tokens.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PersistedSettings {
    pub dock_meter_visible: bool,
    pub launch_at_login: bool,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_seconds: u64,
    #[serde(default)]
    pub dock_meter_geometry: Option<crate::state::DockMeterGeometry>,
}

fn default_refresh_interval() -> u64 {
    60
}

impl Default for PersistedSettings {
    fn default() -> Self {
        Self {
            dock_meter_visible: true,
            launch_at_login: true,
            refresh_interval_seconds: 60,
            dock_meter_geometry: None,
        }
    }
}

/// Returns the path to the settings JSON file inside the app's preferences directory.
fn settings_path(_app_handle: &tauri::AppHandle) -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
        .join("Library/Application Support/Codex Token Meter/settings.json")
}

/// Load persisted settings from disk, falling back to defaults.
pub fn load_settings(app_handle: &tauri::AppHandle) -> PersistedSettings {
    let path = settings_path(app_handle);
    if let Ok(contents) = fs::read_to_string(&path) {
        if let Ok(settings) = serde_json::from_str::<PersistedSettings>(&contents) {
            return settings;
        }
    }
    PersistedSettings::default()
}

/// Persist settings to disk.
pub fn save_settings(
    app_handle: &tauri::AppHandle,
    settings: &PersistedSettings,
) -> Result<(), String> {
    let path = settings_path(app_handle);
    // Ensure parent directory exists.
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create settings directory: {e}"))?;
    }
    let contents = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;
    fs::write(&path, contents).map_err(|e| format!("Failed to write settings file: {e}"))
}

pub fn load_into_state(app_handle: &tauri::AppHandle, state: &crate::state::AppState) {
    *state.settings.lock().unwrap() = load_settings(app_handle).into();
}

impl From<PersistedSettings> for crate::state::AppSettings {
    fn from(value: PersistedSettings) -> Self {
        Self {
            dock_meter_visible: value.dock_meter_visible,
            launch_at_login: value.launch_at_login,
            refresh_interval_seconds: value.refresh_interval_seconds,
            dock_meter_geometry: value.dock_meter_geometry.clone(),
        }
    }
}

impl From<&crate::state::AppSettings> for PersistedSettings {
    fn from(value: &crate::state::AppSettings) -> Self {
        Self {
            dock_meter_visible: value.dock_meter_visible,
            launch_at_login: value.launch_at_login,
            refresh_interval_seconds: value.refresh_interval_seconds,
            dock_meter_geometry: value.dock_meter_geometry.clone(),
        }
    }
}
