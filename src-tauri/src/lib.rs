pub mod commands;
pub mod error;
pub mod login_launch;
pub mod provider;
pub mod state;
pub mod settings;
pub mod usage;

pub use commands::{get_app_settings, get_usage_snapshot, refresh_usage, set_dock_meter_geometry, set_dock_meter_visible, set_launch_at_login, set_refresh_interval};
pub use error::AppError;
pub use provider::{CodexUsageProvider, CodexAppServerProvider, ProviderError, ProviderSnapshot, ProviderWindow};
pub use state::{AppSettings, AppState, UsageSnapshot};
pub use usage::UsageService;

use tauri::Emitter;
use tauri::Manager;
use tauri::Listener;
use tauri::{LogicalPosition, LogicalSize, PhysicalPosition};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

fn mock_snapshot() -> UsageSnapshot {
    UsageSnapshot {
        windows: vec![
            crate::state::UsageWindow { window: "5h".into(), remaining_percent: 72.0, reset_at: Some("2026-07-16T19:00:00Z".into()) },
            crate::state::UsageWindow { window: "7d".into(), remaining_percent: 79.0, reset_at: Some("2026-07-22T09:28:00Z".into()) },
        ],
        status: crate::state::UsageStatus::Fresh { fetched_at: chrono::Utc::now().to_rfc3339() },
    }
}

fn meter_icon(windows: &[crate::state::UsageWindow], stale: bool) -> tauri::image::Image<'static> {
    let size = 22u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for (row, window) in windows.iter().take(2).enumerate() {
        let y_start = if windows.len() == 1 { 8 } else { 4 + row as u32 * 10 };
        let fill = ((window.remaining_percent.clamp(0.0, 100.0) / 100.0) * 18.0).round() as u32;
        for y in y_start..(y_start + 5) {
            for x in 2..20 {
                let filled = x - 2 < fill;
                let (r, g, b, alpha) = if stale {
                    (145, 145, 155, 210)
                } else if filled {
                    let t = (x - 2) as f32 / 17.0;
                    ((70.0 + 150.0 * t) as u8, (115.0 - 35.0 * t) as u8, 255, 255)
                } else {
                    (70, 70, 82, 130)
                };
                let i = ((y * size + x) * 4) as usize;
                rgba[i..i + 4].copy_from_slice(&[r, g, b, alpha]);
            }
        }
    }
    tauri::image::Image::new_owned(rgba, size, size)
}

fn menu_bar_title(snapshot: &UsageSnapshot) -> String {
    snapshot.windows.iter().map(|window| {
        format!("{} {}%", window.window, window.remaining_percent.round() as i32)
    }).collect::<Vec<_>>().join("  │  ")
}

/// Tauri 2 application entrypoint.
///
/// Registers `get_usage_snapshot` and `refresh_usage` commands.
#[cfg_attr(not(target_os = "macos"), allow(unused_variables))]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            let _ = app.handle().set_activation_policy(tauri::ActivationPolicy::Accessory);

            let state = state::AppState::new();
            settings::load_into_state(app.handle(), &state);
            let launch_at_login = state.settings.lock().unwrap().launch_at_login;
            if launch_at_login {
                if let Err(error) = login_launch::set_enabled(app.handle(), true) {
                    eprintln!("Could not enable launch at login: {error}");
                }
            }
            app.manage(state);

            if app.state::<state::AppState>().settings.lock().unwrap().dock_meter_visible {
                if let Some(window) = app.get_webview_window("dock-meter") {
                    let settings = app.state::<state::AppState>().settings.lock().unwrap().clone();
                    if let Some(geometry) = settings.dock_meter_geometry {
                        let _ = window.set_size(LogicalSize::new(geometry.width as f64, geometry.height as f64));
                        let _ = window.set_position(tauri::PhysicalPosition::new(geometry.x, geometry.y));
                    } else if let Ok(Some(monitor)) = app.primary_monitor() {
                        let scale = monitor.scale_factor();
                        let size = monitor.size().to_logical::<f64>(scale);
                        let position = monitor.position().to_logical::<f64>(scale);
                        let x = position.x + (size.width - 400.0) / 2.0;
                        let y = position.y + (size.height - 64.0) / 2.0;
                        let _ = window.set_position(LogicalPosition::new(x.max(position.x), y.max(position.y)));
                    }
                    let _ = window.show();
                }
            }

            let service = UsageService::new(CodexAppServerProvider::from_environment());
            app.manage(service.clone());
            let mock_usage = std::env::var("CODEX_MOCK_USAGE").as_deref() == Ok("true");

            let open = MenuItemBuilder::with_id("open", "Open Codex Meters").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&open, &quit]).build()?;
            let tray = TrayIconBuilder::new()
                .icon(meter_icon(&[], false))
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, rect, .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                                return;
                            }
                            let position = rect.position.to_physical::<f64>(1.0);
                            let size = rect.size.to_physical::<f64>(1.0);
                            let x = position.x + (size.width / 2.0) - 140.0;
                            let y = position.y + size.height + 8.0;
                            let _ = window.set_position(PhysicalPosition::new(x.max(8.0), y));
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => { if let Some(window) = app.get_webview_window("main") { let _ = window.show(); let _ = window.set_focus(); } }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            let tray_for_updates = tray.clone();
            let mock_usage_for_tray = mock_usage;
            app.listen("usage-updated", move |event| {
                let snapshot = if mock_usage_for_tray {
                    Some(mock_snapshot())
                } else {
                    serde_json::from_str::<UsageSnapshot>(event.payload()).ok()
                };
                if let Some(snapshot) = snapshot {
                    let stale = !matches!(snapshot.status, crate::state::UsageStatus::Fresh { .. });
                    let title = menu_bar_title(&snapshot);
                    let _ = tray_for_updates.set_title(Some(&title));
                    let _ = tray_for_updates.set_icon(Some(meter_icon(&snapshot.windows, stale)));
                }
            });

            if mock_usage {
                let snapshot = mock_snapshot();
                let title = menu_bar_title(&snapshot);
                let _ = tray.set_title(Some(&title));
                let _ = tray.set_icon(Some(meter_icon(&snapshot.windows, false)));
            }

            // Run an initial refresh in the background.
            let app_handle = app.handle().clone();
            let service = service.clone();
            tauri::async_runtime::spawn(async move {
                if mock_usage {
                    let snapshot = mock_snapshot();
                    let _ = app_handle.emit("usage-updated", &snapshot);
                    return;
                }
                let _ = service.refresh();
                if let Some(snapshot) = service.get_usage_snapshot() {
                    let _ = app_handle.emit("usage-updated", &snapshot);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_usage_snapshot,
            refresh_usage,
            get_app_settings,
            set_dock_meter_visible,
            set_launch_at_login,
            set_refresh_interval,
            set_dock_meter_geometry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
