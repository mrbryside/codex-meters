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
    // Current snapshot is also set to the fresh result.
    assert!(status_is_fresh(
        &service.get_usage_snapshot().unwrap().status
    ));
}

#[test]
fn refresh_failure_preserves_prior_windows_as_stale() {
    let service = UsageService::new(ToggleProvider::new(true));
    let fresh = service.refresh().unwrap();
    service.provider().set_success(false);

    let stale = service.refresh().unwrap_err();
    assert!(status_is_stale(&stale.status));
    assert_eq!(stale.windows, fresh.windows);
    // Current snapshot reflects the Stale state.
    let current = service.get_usage_snapshot().unwrap();
    assert!(status_is_stale(&current.status));
    assert_eq!(current.windows, fresh.windows);
}

#[test]
fn first_load_failure_returns_unavailable() {
    let service = UsageService::new(FailingProvider);
    let snapshot = service.refresh().unwrap_err();
    assert!(status_is_unavailable(&snapshot.status));
    assert!(snapshot.windows.is_empty());
    // Current snapshot reflects the Unavailable state.
    let current = service.get_usage_snapshot().unwrap();
    assert!(status_is_unavailable(&current.status));
    assert!(current.windows.is_empty());
}

#[test]
fn repeated_failure_before_success_returns_unavailable() {
    // First failure: Unavailable (no prior success).
    let service = UsageService::new(FailingProvider);
    assert!(service.refresh().is_err());
    assert!(status_is_unavailable(
        &service.get_usage_snapshot().unwrap().status
    ));
    // Repeated failures before any success remain Unavailable, not Stale.
    let snapshot = service.refresh().unwrap_err();
    assert!(status_is_unavailable(&snapshot.status));
    assert!(service.get_usage_snapshot().unwrap().windows.is_empty());
}

#[test]
fn success_then_failure_returns_stale_with_preserved_windows() {
    let service = UsageService::new(ToggleProvider::new(true));
    let fresh = service.refresh().unwrap();
    assert!(status_is_fresh(&fresh.status));
    assert_eq!(fresh.windows.len(), 2);

    service.provider().set_success(false);
    let stale = service.refresh().unwrap_err();
    assert!(status_is_stale(&stale.status));
    assert_eq!(stale.windows.len(), 2);
    assert_eq!(stale.windows[0].window, "5h");
    // Current snapshot is Stale, not Fresh.
    let current = service.get_usage_snapshot().unwrap();
    assert!(status_is_stale(&current.status));
    assert_eq!(current.windows[0].window, "5h");
}

#[test]
fn success_then_failure_then_failure_remains_stale() {
    let service = UsageService::new(ToggleProvider::new(true));
    let fresh = service.refresh().unwrap();
    assert!(status_is_fresh(&fresh.status));

    service.provider().set_success(false);
    let stale1 = service.refresh().unwrap_err();
    assert!(status_is_stale(&stale1.status));
    assert_eq!(stale1.windows.len(), 2);

    // Second failure: should still be Stale with preserved windows.
    let stale2 = service.refresh().unwrap_err();
    assert!(status_is_stale(&stale2.status));
    assert_eq!(stale2.windows.len(), 2);
    assert_eq!(stale2.windows[0].window, "5h");
    // Current snapshot remains Stale.
    let current = service.get_usage_snapshot().unwrap();
    assert!(status_is_stale(&current.status));
}

#[test]
fn refresh_after_failure_transitions_to_fresh() {
    let service = UsageService::new(ToggleProvider::new(false));
    assert!(service.refresh().is_err());
    // Current snapshot reflects the Unavailable state.
    assert!(status_is_unavailable(
        &service.get_usage_snapshot().unwrap().status
    ));

    service.provider().set_success(true);
    let snapshot = service.refresh().unwrap();
    assert!(status_is_fresh(&snapshot.status));
    // Current snapshot is now Fresh.
    assert!(status_is_fresh(
        &service.get_usage_snapshot().unwrap().status
    ));
}

#[test]
fn none_before_refresh() {
    let service = UsageService::new(SuccessProvider);
    assert!(service.get_usage_snapshot().is_none());
}

#[test]
fn first_failure_is_unavailable_and_current_equals_returned_error() {
    let service = UsageService::new(FailingProvider);
    let returned = service.refresh().unwrap_err();
    assert!(status_is_unavailable(&returned.status));
    assert!(returned.windows.is_empty());
    let current = service.get_usage_snapshot().unwrap();
    assert!(status_is_unavailable(&current.status));
    assert!(current.windows.is_empty());
    // Current snapshot equals the returned error snapshot.
    assert_eq!(current.status, returned.status);
}

#[test]
fn repeated_failure_is_unavailable_and_current_equals_returned_error() {
    let service = UsageService::new(FailingProvider);
    // First failure
    let first = service.refresh().unwrap_err();
    assert!(status_is_unavailable(&first.status));
    let current1 = service.get_usage_snapshot().unwrap();
    assert_eq!(current1.status, first.status);

    // Second failure — still Unavailable
    let second = service.refresh().unwrap_err();
    assert!(status_is_unavailable(&second.status));
    let current2 = service.get_usage_snapshot().unwrap();
    assert_eq!(current2.status, second.status);
}

#[test]
fn success_then_failure_is_stale_with_preserved_windows_and_fetched_at() {
    let service = UsageService::new(ToggleProvider::new(true));
    let fresh = service.refresh().unwrap();
    assert!(status_is_fresh(&fresh.status));

    // Capture fetched_at from the fresh snapshot
    let fetched_at = match &fresh.status {
        UsageStatus::Fresh { fetched_at } => fetched_at.clone(),
        _ => unreachable!(),
    };

    service.provider().set_success(false);
    let stale = service.refresh().unwrap_err();
    assert!(status_is_stale(&stale.status));
    assert_eq!(stale.windows.len(), 2);

    // Verify fetched_at is preserved in Stale status
    let stale_fetched_at = match &stale.status {
        UsageStatus::Stale { fetched_at, .. } => fetched_at.clone(),
        _ => unreachable!(),
    };
    assert_eq!(stale_fetched_at, fetched_at);

    // Current snapshot equals the returned stale snapshot
    let current = service.get_usage_snapshot().unwrap();
    assert_eq!(current.status, stale.status);
}

#[test]
fn success_then_failure_then_failure_is_stale_with_preserved_data() {
    let service = UsageService::new(ToggleProvider::new(true));
    let fresh = service.refresh().unwrap();
    let fetched_at = match &fresh.status {
        UsageStatus::Fresh { fetched_at } => fetched_at.clone(),
        _ => unreachable!(),
    };

    service.provider().set_success(false);
    let stale1 = service.refresh().unwrap_err();
    assert!(status_is_stale(&stale1.status));
    assert_eq!(stale1.windows.len(), 2);

    // Second failure — still Stale, same fetched_at, same windows
    let stale2 = service.refresh().unwrap_err();
    assert!(status_is_stale(&stale2.status));
    assert_eq!(stale2.windows.len(), 2);
    assert_eq!(stale2.windows[0].window, "5h");

    let stale2_fetched_at = match &stale2.status {
        UsageStatus::Stale { fetched_at, .. } => fetched_at.clone(),
        _ => unreachable!(),
    };
    assert_eq!(stale2_fetched_at, fetched_at);

    // Current snapshot equals the latest stale
    let current = service.get_usage_snapshot().unwrap();
    assert_eq!(current.status, stale2.status);
}

#[test]
fn failure_then_success_then_failure_uses_only_the_success() {
    let service = UsageService::new(ToggleProvider::new(false));
    // Initial failure — Unavailable
    assert!(status_is_unavailable(
        &service.refresh().unwrap_err().status
    ));

    // Success — now we have a Fresh snapshot
    service.provider().set_success(true);
    let fresh = service.refresh().unwrap();
    assert!(status_is_fresh(&fresh.status));
    let fresh_fetched_at = match &fresh.status {
        UsageStatus::Fresh { fetched_at } => fetched_at.clone(),
        _ => unreachable!(),
    };
    let fresh_windows = fresh.windows.clone();

    // Failure after success — should be Stale using the success data
    service.provider().set_success(false);
    let stale = service.refresh().unwrap_err();
    assert!(status_is_stale(&stale.status));
    assert_eq!(stale.windows, fresh_windows);
    let stale_fetched_at = match &stale.status {
        UsageStatus::Stale { fetched_at, .. } => fetched_at.clone(),
        _ => unreachable!(),
    };
    assert_eq!(stale_fetched_at, fresh_fetched_at);
}

#[test]
fn success_a_then_success_b_then_failure_falls_back_to_b() {
    let mut service = UsageService::new(VariantProvider::new("A"));

    // Success A — distinct data
    let snap_a = service.refresh().unwrap();
    assert!(status_is_fresh(&snap_a.status));
    assert_eq!(snap_a.windows[0].remaining_percent, 10.0); // A's percentage

    // Success B — distinct data
    service.provider_mut().set_variant("B");
    let snap_b = service.refresh().unwrap();
    assert!(status_is_fresh(&snap_b.status));
    assert_eq!(snap_b.windows[0].remaining_percent, 90.0); // B's percentage
    let windows_b = snap_b.windows.clone();

    // Failure — should fall back to B (the most recent success), not A
    service.provider().set_variant("fail");
    let stale = service.refresh().unwrap_err();
    assert!(status_is_stale(&stale.status));
    assert_eq!(stale.windows, windows_b); // must preserve B, not A
    assert_eq!(stale.windows[0].remaining_percent, 90.0);

    // Verify current snapshot equals the stale
    let current = service.get_usage_snapshot().unwrap();
    assert_eq!(current.status, stale.status);
    assert_eq!(current.windows, windows_b);
}

/// Provider that returns different fixture data based on a variant name.
struct VariantProvider {
    variant: std::sync::Mutex<String>,
}

impl VariantProvider {
    fn new(variant: &str) -> Self {
        Self {
            variant: std::sync::Mutex::new(variant.to_string()),
        }
    }

    fn set_variant(&self, variant: &str) {
        *self.variant.lock().unwrap() = variant.to_string();
    }
}

impl CodexUsageProvider for VariantProvider {
    fn fetch(&self) -> Result<String, ProviderError> {
        let v = self.variant.lock().unwrap().clone();
        match v.as_str() {
            "A" => Ok(r#"{"windows":[{"window":"5h","remainingPercent":10.0,"resetAt":"2026-07-16T12:00:00Z"},{"window":"7d","remainingPercent":20.0,"resetAt":"2026-07-23T00:00:00Z"}]}"#.to_string()),
            "B" => Ok(r#"{"windows":[{"window":"5h","remainingPercent":90.0,"resetAt":"2026-07-16T12:00:00Z"},{"window":"7d","remainingPercent":80.0,"resetAt":"2026-07-23T00:00:00Z"}]}"#.to_string()),
            _ => Err(ProviderError::SourceUnavailable {
                source: "test_variant".to_string(),
                detail: "variant says no".to_string(),
            }),
        }
    }
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
        Self {
            success: std::sync::Mutex::new(success),
        }
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
