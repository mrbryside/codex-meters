use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Shared application state for Tauri commands.
pub struct AppState {
    /// Current usage snapshot (protected by Mutex for interior mutability).
    pub snapshot: Mutex<Option<UsageSnapshot>>,
    /// Application settings.
    pub settings: Mutex<AppSettings>,
}

/// Mirrors the TypeScript UsageSnapshot type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshot {
    pub windows: Vec<UsageWindow>,
    pub status: UsageStatus,
}

/// A single usage window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    pub window: String,
    pub remaining_percent: f32,
    pub reset_at: Option<String>,
}

/// Status of the latest usage data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "camelCase")]
pub enum UsageStatus {
    Fresh { fetched_at: String },
    Stale {
        fetched_at: String,
        failed_at: String,
        message: String,
    },
    Unavailable { message: String },
}

/// Application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub dock_meter_visible: bool,
    pub launch_at_login: bool,
    pub refresh_interval_seconds: u64,
    pub dock_meter_geometry: Option<DockMeterGeometry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DockMeterGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            dock_meter_visible: true,
            launch_at_login: true,
            refresh_interval_seconds: 60,
            dock_meter_geometry: None,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            snapshot: Mutex::new(None),
            settings: Mutex::new(AppSettings::default()),
        }
    }
}
