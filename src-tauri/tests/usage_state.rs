use codex_token_meter_lib::provider::CodexUsageProvider;
use codex_token_meter_lib::state::UsageStatus;
use codex_token_meter_lib::usage::UsageService;
use codex_token_meter_lib::ProviderError;

fn status_is_fresh(status: &UsageStatus) -> bool {
    matches!(status, UsageStatus::Fresh { .. })
}

fn status_is_stale(status: &UsageStatus) -> bool {
    matches!(status, UsageStatus::Stale { .. })
}

fn status_is_unavailable(status: &UsageStatus) -> bool {
    matches!(status, UsageStatus::Unavailable { .. })
}

#[test]
fn refresh_success_returns_fresh_snapshot() {
    let service = UsageService::new(SuccessProvider);
    let snapshot = service.refresh().expect("fixture provider should succeed");
    assert!(status_is_fresh(&snapshot.status));
    assert_eq!(snapshot.windows.len(), 2);
    assert_eq!(snapshot.windows[0].window, "5h");
}

#[test]
fn refresh_failure_preserves_prior_windows_as_stale() {
    let service = UsageService::new(ToggleProvider::new(true));
    let fresh = service.refresh().unwrap();
    service.provider().set_success(false);

    let stale = service.refresh().unwrap_err();
    assert!(status_is_stale(&stale.status));
    assert_eq!(stale.windows, fresh.windows);
}

#[test]
fn first_load_failure_returns_unavailable() {
    let service = UsageService::new(FailingProvider);
    let snapshot = service.refresh().unwrap_err();
    assert!(status_is_unavailable(&snapshot.status));
    assert!(snapshot.windows.is_empty());
}

#[test]
fn get_usage_snapshot_returns_current_state_without_refreshing() {
    let service = UsageService::new(FailingProvider);
    assert!(service.get_usage_snapshot().is_none());
    let _ = service.refresh();
    let snapshot = service.get_usage_snapshot().expect("failure is cached");
    assert!(status_is_unavailable(&snapshot.status));
}

#[test]
fn refresh_after_failure_transitions_to_fresh() {
    let service = UsageService::new(ToggleProvider::new(false));
    assert!(service.refresh().is_err());
    assert!(status_is_unavailable(&service.get_usage_snapshot().unwrap().status));

    service.provider().set_success(true);
    let snapshot = service.refresh().unwrap();
    assert!(status_is_fresh(&snapshot.status));
}

#[test]
fn repeated_failure_after_first_load_is_stale() {
    let service = UsageService::new(FailingProvider);
    assert!(service.refresh().is_err());
    let snapshot = service.refresh().unwrap_err();
    assert!(status_is_stale(&snapshot.status));
}

struct SuccessProvider;

impl CodexUsageProvider for SuccessProvider {
    fn fetch(&self) -> Result<String, ProviderError> {
        Ok(fixture_json())
    }
}

struct FailingProvider;

impl CodexUsageProvider for FailingProvider {
    fn fetch(&self) -> Result<String, ProviderError> {
        Err(ProviderError::SourceUnavailable {
            source: "test".to_string(),
            detail: "always fails".to_string(),
        })
    }
}

struct ToggleProvider {
    success: std::sync::Mutex<bool>,
}

impl ToggleProvider {
    fn new(success: bool) -> Self {
        Self { success: std::sync::Mutex::new(success) }
    }

    fn set_success(&self, success: bool) {
        *self.success.lock().unwrap() = success;
    }
}

impl CodexUsageProvider for ToggleProvider {
    fn fetch(&self) -> Result<String, ProviderError> {
        if *self.success.lock().unwrap() {
            Ok(fixture_json())
        } else {
            Err(ProviderError::SourceUnavailable {
                source: "test_toggle".to_string(),
                detail: "toggle says no".to_string(),
            })
        }
    }
}

fn fixture_json() -> String {
    r#"{"windows":[{"window":"5h","remainingPercent":75.0,"resetAt":"2026-07-16T12:00:00Z"},{"window":"7d","remainingPercent":60.0,"resetAt":"2026-07-23T00:00:00Z"}]}"#.to_string()
}
