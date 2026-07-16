use crate::provider::CodexAppServerProvider;
use crate::usage::UsageService;
use crate::AppError;
use serde::Serialize;
use tauri::Emitter;
use tauri::Manager;
use tauri::{LogicalPosition, LogicalSize, PhysicalPosition};

/// Serializable response for get_usage_snapshot.
#[derive(Debug, Serialize)]
pub struct SnapshotResponse {
    pub snapshot: Option<crate::UsageSnapshot>,
    pub error: Option<AppError>,
}

/// Serializable response for refresh_usage.
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub snapshot: crate::UsageSnapshot,
    pub error: bool,
}

/// Get the current usage snapshot without refreshing.
///
/// Returns the cached snapshot if available, or `None` if no refresh has
/// been attempted yet.
#[tauri::command]
pub async fn get_usage_snapshot(
    _app: tauri::AppHandle,
    service: tauri::State<'_, UsageService<CodexAppServerProvider>>,
) -> Result<SnapshotResponse, AppError> {
    Ok(SnapshotResponse {
        snapshot: service.get_usage_snapshot(),
        error: service.get_usage_snapshot().is_none().then(|| {
            AppError::usage_unavailable("No snapshot available yet")
        }),
    })
}

/// Refresh usage data from the Codex provider.
///
/// Emits `usage-updated` with the resulting snapshot regardless of outcome.
#[tauri::command]
pub async fn refresh_usage(
    app: tauri::AppHandle,
    service: tauri::State<'_, UsageService<CodexAppServerProvider>>,
) -> Result<RefreshResponse, AppError> {
    match service.refresh() {
        Ok(snapshot) => {
            let _ = app.emit("usage-updated", &snapshot);
            Ok(RefreshResponse {
                snapshot,
                error: false,
            })
        }
        Err(snapshot) => {
            let _ = app.emit("usage-updated", &snapshot);
            Ok(RefreshResponse {
                snapshot,
                error: true,
            })
        }
    }
}

#[tauri::command]
pub fn get_app_settings(app: tauri::AppHandle, state: tauri::State<'_, crate::state::AppState>) -> crate::state::AppSettings {
    let _ = app;
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_dock_meter_visible(app: tauri::AppHandle, state: tauri::State<'_, crate::state::AppState>, visible: bool) -> Result<crate::state::AppSettings, AppError> {
    let mut settings = state.settings.lock().unwrap();
    settings.dock_meter_visible = visible;
    let persisted = crate::settings::PersistedSettings::from(&*settings);
    crate::settings::save_settings(&app, &persisted).map_err(AppError::settings_error)?;
    if let Some(window) = app.get_webview_window("dock-meter") {
        if visible {
            if let Some(geometry) = settings.dock_meter_geometry.as_ref() {
                let _ = window.set_size(LogicalSize::new(geometry.width as f64, geometry.height as f64));
                let _ = window.set_position(PhysicalPosition::new(geometry.x, geometry.y));
            } else if let Ok(Some(monitor)) = app.primary_monitor() {
                let scale = monitor.scale_factor();
                let size = monitor.size().to_logical::<f64>(scale);
                let position = monitor.position().to_logical::<f64>(scale);
                let x = position.x + (size.width - 400.0) / 2.0;
                let y = position.y + (size.height - 64.0) / 2.0;
                let _ = window.set_position(LogicalPosition::new(x.max(position.x), y.max(position.y)));
            }
            let _ = window.show();
        } else { let _ = window.hide(); }
    }
    let _ = app.emit("dock-meter-visibility-changed", &*settings);
    Ok(settings.clone())
}

#[tauri::command]
pub fn set_dock_meter_geometry(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::state::AppState>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<crate::state::AppSettings, AppError> {
    if !(160..=520).contains(&width) || !(48..=180).contains(&height) {
        return Err(AppError::settings_error("Dock meter geometry is outside the supported range"));
    }
    let mut settings = state.settings.lock().unwrap();
    settings.dock_meter_geometry = Some(crate::state::DockMeterGeometry { x, y, width, height });
    let persisted = crate::settings::PersistedSettings::from(&*settings);
    crate::settings::save_settings(&app, &persisted).map_err(AppError::settings_error)?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn set_launch_at_login(app: tauri::AppHandle, state: tauri::State<'_, crate::state::AppState>, enabled: bool) -> Result<crate::state::AppSettings, AppError> {
    crate::login_launch::set_enabled(&app, enabled).map_err(AppError::settings_error)?;
    let mut settings = state.settings.lock().unwrap();
    settings.launch_at_login = enabled;
    let persisted = crate::settings::PersistedSettings::from(&*settings);
    crate::settings::save_settings(&app, &persisted).map_err(AppError::settings_error)?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn set_refresh_interval(app: tauri::AppHandle, state: tauri::State<'_, crate::state::AppState>, seconds: u64) -> Result<crate::state::AppSettings, AppError> {
    if !matches!(seconds, 10 | 30 | 60) { return Err(AppError::settings_error("Refresh interval must be 10, 30, or 60 seconds")); }
    let mut settings = state.settings.lock().unwrap();
    settings.refresh_interval_seconds = seconds;
    let persisted = crate::settings::PersistedSettings::from(&*settings);
    crate::settings::save_settings(&app, &persisted).map_err(AppError::settings_error)?;
    Ok(settings.clone())
}
