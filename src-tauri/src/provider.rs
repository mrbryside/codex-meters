use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Supported usage windows.
const SUPPORTED_WINDOWS: &[&str] = &["5h", "7d"];

/// A single usage window from the provider.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderWindow {
    /// Window identifier (e.g. "5h", "7d").
    pub window: String,
    /// Remaining percentage, clamped to [0.0, 100.0].
    pub remaining_percent: f32,
    /// ISO-8601 reset timestamp, or None if unavailable.
    pub reset_at: Option<String>,
}

/// A parsed usage snapshot from the provider.
#[derive(Debug, Clone, Serialize)]
pub struct ProviderSnapshot {
    /// Filtered, clamped, deduplicated windows.
    pub windows: Vec<ProviderWindow>,
}

/// Errors that can occur when fetching or parsing usage data.
#[derive(Debug, Clone, Serialize)]
pub enum ProviderError {
    /// The local session source could not be read or is unavailable.
    #[serde(rename = "source_unavailable")]
    SourceUnavailable {
        source: String,
        detail: String,
    },
    /// The JSON response could not be parsed or has invalid structure.
    #[serde(rename = "parse_error")]
    ParseError {
        source: String,
        detail: String,
    },
    /// Duplicate window identifiers were found in the response.
    #[serde(rename = "duplicate_window")]
    DuplicateWindow {
        window: String,
    },
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::SourceUnavailable { source, detail } => {
                write!(f, "Source unavailable [{}]: {}", source, detail)
            }
            ProviderError::ParseError { source, detail } => {
                write!(f, "Parse error [{}]: {}", source, detail)
            }
            ProviderError::DuplicateWindow { window } => {
                write!(f, "Duplicate window: {}", window)
            }
        }
    }
}

impl std::error::Error for ProviderError {}

/// Raw JSON structure from the Codex usage source.
#[derive(Deserialize)]
struct RawResponse {
    windows: Option<Vec<RawWindow>>,
    error: Option<String>,
    #[allow(dead_code)]
    message: Option<String>,
}

#[derive(Deserialize)]
struct RawWindow {
    #[serde(rename = "window")]
    window: String,
    #[serde(rename = "remainingPercent")]
    remaining_percent: Option<f32>,
    #[serde(rename = "resetAt")]
    reset_at: Option<String>,
}

/// Parse a raw JSON string into a `ProviderSnapshot`.
///
/// This is the core normalizer: it validates structure, rejects duplicates,
/// filters unsupported windows, and clamps percentages to [0.0, 100.0].
pub fn parse_provider_response(raw: &str) -> Result<ProviderSnapshot, ProviderError> {
    let response: RawResponse = serde_json::from_str(raw).map_err(|e| {
        ProviderError::ParseError {
            source: "json".to_string(),
            detail: e.to_string(),
        }
    })?;

    // If the response contains an error field, treat it as unavailable.
    if let Some(ref err) = response.error {
        return Err(ProviderError::SourceUnavailable {
            source: "codex_response".to_string(),
            detail: err.clone(),
        });
    }

    let raw_windows = response.windows.unwrap_or_default();

    // Check for duplicates before processing.
    let mut seen = HashSet::new();
    for rw in &raw_windows {
        if !seen.insert(&rw.window) {
            return Err(ProviderError::DuplicateWindow {
                window: rw.window.clone(),
            });
        }
    }

    // Filter to supported windows, clamp percentages, and build windows.
    let mut windows = Vec::new();
    for rw in raw_windows {
        // Skip unsupported windows.
        if !SUPPORTED_WINDOWS.contains(&rw.window.as_str()) {
            continue;
        }

        let remaining = rw.remaining_percent.unwrap_or(0.0).clamp(0.0, 100.0);

        windows.push(ProviderWindow {
            window: rw.window,
            remaining_percent: remaining,
            reset_at: rw.reset_at,
        });
    }

    Ok(ProviderSnapshot { windows })
}

/// The Codex usage provider trait.
///
/// Implementations read from a concrete source (file, API, etc.) and
/// return a `ProviderSnapshot` or a `ProviderError`.
pub trait CodexUsageProvider {
    /// Fetch the raw usage data from the source.
    fn fetch(&self) -> Result<String, ProviderError>;

    /// Fetch and parse in one step.
    fn fetch_and_parse(&self) -> Result<ProviderSnapshot, ProviderError> {
        let raw = self.fetch()?;
        parse_provider_response(&raw)
    }
}

/// A concrete implementation that reads from a file path.
///
/// This is the default adapter for local session file reading.
pub struct FileBackedProvider {
    source_path: String,
    source_name: String,
}

impl FileBackedProvider {
    pub fn new(source_path: impl Into<String>, source_name: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
            source_name: source_name.into(),
        }
    }
}

impl CodexUsageProvider for FileBackedProvider {
    fn fetch(&self) -> Result<String, ProviderError> {
        std::fs::read_to_string(&self.source_path).map_err(|e| {
            ProviderError::SourceUnavailable {
                source: self.source_name.clone(),
                detail: e.to_string(),
            }
        })
    }
}

/// A source-unavailable adapter used when the concrete local Codex
/// source cannot be identified or accessed.
///
/// This adapter always returns `SourceUnavailable` with a documented
/// explanation, so that the rest of the system can handle the missing
/// data gracefully.
pub struct SourceUnavailableAdapter;

impl CodexUsageProvider for SourceUnavailableAdapter {
    fn fetch(&self) -> Result<String, ProviderError> {
        Err(ProviderError::SourceUnavailable {
            source: "local_codex_session".to_string(),
            detail: "Local Codex session source could not be identified or accessed. See docs/compatibility/codex-usage-source.md for details.".to_string(),
        })
    }
}

/// Reads subscription limits through the installed Codex app-server protocol.
/// The Codex process owns authentication; this app only sends protocol requests
/// and receives the normalized rate-limit response.
#[derive(Clone)]
pub struct CodexAppServerProvider {
    codex_path: PathBuf,
    timeout: Duration,
}

impl CodexAppServerProvider {
    pub fn new(codex_path: impl Into<PathBuf>, timeout: Duration) -> Self {
        Self {
            codex_path: codex_path.into(),
            timeout,
        }
    }

    pub fn from_environment() -> Self {
        let path = std::env::var_os("CODEX_CLI_PATH")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| {
                    PathBuf::from(home)
                        .join("Applications/ChatGPT.app/Contents/Resources/codex")
                })
            })
            .unwrap_or_else(|| PathBuf::from("codex"));
        Self::new(path, Duration::from_secs(10))
    }

    fn read_protocol_output(&self) -> Result<String, ProviderError> {
        let mut child = Command::new(&self.codex_path)
            .args(["app-server", "--stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| ProviderError::SourceUnavailable {
                source: "codex_app_server".to_string(),
                detail: "Codex app-server could not be started".to_string(),
            })?;

        let requests = concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"clientInfo\":{\"name\":\"codex-token-meter\",\"title\":\"Codex Meters\",\"version\":\"0.1.0\"},\"capabilities\":{}}}\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"initialized\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"account/rateLimits/read\",\"params\":null}\n"
        );
        let mut stdin = child.stdin.take().ok_or_else(|| ProviderError::SourceUnavailable {
            source: "codex_app_server".to_string(),
            detail: "Codex app-server stdin could not be opened".to_string(),
        })?;
        {
            stdin.write_all(requests.as_bytes()).map_err(|_| ProviderError::SourceUnavailable {
                source: "codex_app_server".to_string(),
                detail: "Codex app-server request could not be sent".to_string(),
            })?;
        }

        let stdout = child.stdout.take().ok_or_else(|| ProviderError::SourceUnavailable {
            source: "codex_app_server".to_string(),
            detail: "Codex app-server stdout could not be opened".to_string(),
        })?;
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().flatten() {
                let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
                if value.get("id") == Some(&serde_json::json!(2)) {
                    let _ = sender.send(line);
                    break;
                }
            }
        });

        let response = match receiver.recv_timeout(self.timeout) {
            Ok(response) => response,
            Err(_) => {
                drop(stdin);
                let _ = child.kill();
                let _ = child.wait();
                return Err(ProviderError::SourceUnavailable {
                    source: "codex_app_server".to_string(),
                    detail: "Codex app-server request timed out".to_string(),
                });
            }
        };
        drop(stdin);
        let _ = child.kill();
        let _ = child.wait();
        Ok(response)
    }
}

impl Default for CodexAppServerProvider {
    fn default() -> Self {
        Self::from_environment()
    }
}

impl CodexUsageProvider for CodexAppServerProvider {
    fn fetch(&self) -> Result<String, ProviderError> {
        let output = self.read_protocol_output()?;
        let snapshot = parse_app_server_response(&output)?;
        serde_json::to_string(&serde_json::json!({ "windows": snapshot.windows })).map_err(|_| {
            ProviderError::ParseError {
                source: "codex_app_server".to_string(),
                detail: "Normalized usage response could not be encoded".to_string(),
            }
        })
    }
}

#[derive(Debug, Deserialize)]
struct AppServerEnvelope {
    id: Option<serde_json::Value>,
    result: Option<RateLimitResult>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RateLimitResult {
    #[serde(rename = "rateLimits")]
    rate_limits: Option<RateLimitSnapshot>,
    #[serde(rename = "rateLimitsByLimitId")]
    rate_limits_by_limit_id: Option<HashMap<String, RateLimitSnapshot>>,
}

#[derive(Debug, Deserialize)]
struct RateLimitSnapshot {
    primary: Option<RateLimitWindow>,
    secondary: Option<RateLimitWindow>,
}

#[derive(Debug, Deserialize)]
struct RateLimitWindow {
    #[serde(rename = "usedPercent")]
    used_percent: i32,
    #[serde(rename = "resetsAt")]
    resets_at: Option<i64>,
    #[serde(rename = "windowDurationMins")]
    window_duration_mins: Option<i64>,
}

pub fn parse_app_server_response(raw: &str) -> Result<ProviderSnapshot, ProviderError> {
    for line in raw.lines() {
        let envelope: AppServerEnvelope = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if envelope.id != Some(serde_json::json!(2)) {
            continue;
        }
        if let Some(error) = envelope.error {
            let detail = error
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("Codex app-server returned an error");
            return Err(ProviderError::SourceUnavailable {
                source: "codex_app_server".to_string(),
                detail: detail.to_string(),
            });
        }
        let result = envelope.result.ok_or_else(|| ProviderError::ParseError {
            source: "codex_app_server".to_string(),
            detail: "Rate-limit response did not contain a result".to_string(),
        })?;
        let snapshot = result
            .rate_limits_by_limit_id
            .as_ref()
            .and_then(|limits| limits.get("codex"))
            .or(result.rate_limits.as_ref())
            .ok_or_else(|| ProviderError::SourceUnavailable {
                source: "codex_app_server".to_string(),
                detail: "Codex did not provide rate-limit data".to_string(),
            })?;

        let mut windows = Vec::new();
        for window in [snapshot.primary.as_ref(), snapshot.secondary.as_ref()].into_iter().flatten() {
            let Some(name) = window_name(window.window_duration_mins) else { continue };
            if windows.iter().any(|existing: &ProviderWindow| existing.window == name) {
                return Err(ProviderError::DuplicateWindow { window: name.to_string() });
            }
            let used = window.used_percent.clamp(0, 100) as f32;
            let reset_at = window
                .resets_at
                .and_then(|seconds| chrono::DateTime::from_timestamp(seconds, 0))
                .map(|timestamp| timestamp.to_rfc3339());
            windows.push(ProviderWindow {
                window: name.to_string(),
                remaining_percent: 100.0 - used,
                reset_at,
            });
        }
        return Ok(ProviderSnapshot { windows });
    }
    Err(ProviderError::ParseError {
        source: "codex_app_server".to_string(),
        detail: "No rate-limit response was received".to_string(),
    })
}

fn window_name(duration_minutes: Option<i64>) -> Option<&'static str> {
    match duration_minutes {
        Some(300) => Some("5h"),
        Some(10080) => Some("7d"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_response() {
        let json = r#"{
            "windows": [
                {"window": "5h", "remainingPercent": 72.5, "resetAt": "2026-07-16T08:00:00Z"},
                {"window": "7d", "remainingPercent": 45.0, "resetAt": "2026-07-22T00:00:00Z"}
            ]
        }"#;
        let result = parse_provider_response(json);
        assert!(result.is_ok());
        let snapshot = result.unwrap();
        assert_eq!(snapshot.windows.len(), 2);
    }

    #[test]
    fn parse_error_field_returns_unavailable() {
        let json = r#"{"error": "session_expired"}"#;
        let result = parse_provider_response(json);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProviderError::SourceUnavailable { .. }));
    }

    #[test]
    fn parse_duplicate_windows() {
        let json = r#"{
            "windows": [
                {"window": "5h", "remainingPercent": 50.0, "resetAt": "2026-07-16T08:00:00Z"},
                {"window": "5h", "remainingPercent": 60.0, "resetAt": "2026-07-16T09:00:00Z"}
            ]
        }"#;
        let result = parse_provider_response(json);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProviderError::DuplicateWindow { .. }));
    }

    #[test]
    fn filter_unsupported_windows() {
        let json = r#"{
            "windows": [
                {"window": "30d", "remainingPercent": 25.0, "resetAt": "2026-07-30T00:00:00Z"},
                {"window": "7d", "remainingPercent": 80.0, "resetAt": "2026-07-22T00:00:00Z"}
            ]
        }"#;
        let result = parse_provider_response(json);
        assert!(result.is_ok());
        let snapshot = result.unwrap();
        assert_eq!(snapshot.windows.len(), 1);
        assert_eq!(snapshot.windows[0].window, "7d");
    }

    #[test]
    fn clamp_percentages() {
        let json = r#"{
            "windows": [
                {"window": "5h", "remainingPercent": 150.0, "resetAt": "2026-07-16T08:00:00Z"},
                {"window": "7d", "remainingPercent": -10.0, "resetAt": "2026-07-22T00:00:00Z"}
            ]
        }"#;
        let result = parse_provider_response(json);
        assert!(result.is_ok());
        let snapshot = result.unwrap();
        assert!((snapshot.windows[0].remaining_percent - 100.0).abs() < f32::EPSILON);
        assert!((snapshot.windows[1].remaining_percent - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn empty_windows_array() {
        let json = r#"{"windows": []}"#;
        let result = parse_provider_response(json);
        assert!(result.is_ok());
        let snapshot = result.unwrap();
        assert_eq!(snapshot.windows.len(), 0);
    }

    #[test]
    fn missing_windows_field_treated_as_empty() {
        let json = r#"{}"#;
        let result = parse_provider_response(json);
        assert!(result.is_ok());
        let snapshot = result.unwrap();
        assert_eq!(snapshot.windows.len(), 0);
    }

    #[test]
    fn source_unavailable_adapter() {
        let provider = SourceUnavailableAdapter;
        let result = provider.fetch();
        assert!(result.is_err());
        match result.unwrap_err() {
            ProviderError::SourceUnavailable { source, detail } => {
                assert_eq!(source, "local_codex_session");
                assert!(!detail.contains("token"));
                assert!(!detail.contains("cookie"));
                assert!(!detail.contains("secret"));
            }
            other => panic!("Expected SourceUnavailable, got: {:?}", other),
        }
    }

    #[test]
    fn file_backed_provider_reads_file() {
        let provider = FileBackedProvider::new(
            "/Users/s103450/personal/token-meter/src-tauri/fixtures/usage-5h-7d.json",
            "test_fixture",
        );
        let result = provider.fetch();
        assert!(result.is_ok());
        let json = result.unwrap();
        let snapshot = parse_provider_response(&json).unwrap();
        assert_eq!(snapshot.windows.len(), 2);
    }

    #[test]
    fn file_backed_provider_missing_file() {
        let provider = FileBackedProvider::new(
            "/nonexistent/path.json",
            "missing_fixture",
        );
        let result = provider.fetch();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProviderError::SourceUnavailable { .. }));
    }

    #[test]
    fn error_messages_no_secrets() {
        let err = ProviderError::SourceUnavailable {
            source: "test".to_string(),
            detail: "session expired".to_string(),
        };
        let msg = format!("{}", err);
        assert!(!msg.to_lowercase().contains("token"));
        assert!(!msg.to_lowercase().contains("cookie"));
        assert!(!msg.to_lowercase().contains("secret"));
        assert!(!msg.to_lowercase().contains("authorization"));
    }

    #[test]
    fn parse_app_server_response_maps_codex_windows() {
        let raw = r#"{"method":"remoteControl/status/changed","params":{"status":"disabled"}}
{"id":2,"result":{"rateLimits":{"primary":{"usedPercent":28,"resetsAt":1784188800,"windowDurationMins":300},"secondary":{"usedPercent":55,"resetsAt":1784678400,"windowDurationMins":10080}},"rateLimitsByLimitId":null}}"#;
        let snapshot = parse_app_server_response(raw).expect("app-server response should parse");
        assert_eq!(snapshot.windows.len(), 2);
        assert_eq!(snapshot.windows[0].window, "5h");
        assert!((snapshot.windows[0].remaining_percent - 72.0).abs() < f32::EPSILON);
        assert_eq!(snapshot.windows[1].window, "7d");
        assert!((snapshot.windows[1].remaining_percent - 45.0).abs() < f32::EPSILON);
    }
}
