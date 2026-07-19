use codex_token_meter_lib::provider::{
    parse_provider_response, ProviderError, ProviderSnapshot, ProviderWindow,
};

/// Load a fixture JSON file from the fixtures directory.
fn load_fixture(name: &str) -> String {
    let fixtures_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/fixtures/{}", fixtures_dir, name);
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("Fixture file not found: {}", path))
}

// ── Full 5h + 7d response ──────────────────────────────────────────

#[test]
fn parse_full_5h_7d_response() {
    let json = load_fixture("usage-5h-7d.json");
    let result = parse_provider_response(&json);
    assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
    let snapshot = result.unwrap();
    assert_eq!(snapshot.windows.len(), 2);

    // Verify 5h window
    let window_5h = snapshot
        .windows
        .iter()
        .find(|w| w.window == "5h")
        .expect("5h window missing");
    assert!((window_5h.remaining_percent - 72.5).abs() < f32::EPSILON);
    assert_eq!(window_5h.reset_at, Some("2026-07-16T08:00:00Z".to_string()));

    // Verify 7d window
    let window_7d = snapshot
        .windows
        .iter()
        .find(|w| w.window == "7d")
        .expect("7d window missing");
    assert!((window_7d.remaining_percent - 45.0).abs() < f32::EPSILON);
    assert_eq!(window_7d.reset_at, Some("2026-07-22T00:00:00Z".to_string()));
}

// ── 7d-only response ──────────────────────────────────────────────

#[test]
fn parse_7d_only_response() {
    let json = load_fixture("usage-7d-only.json");
    let result = parse_provider_response(&json);
    assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
    let snapshot = result.unwrap();
    assert_eq!(snapshot.windows.len(), 1);

    let window = snapshot.windows.first().unwrap();
    assert_eq!(window.window, "7d");
    assert!((window.remaining_percent - 18.3).abs() < f32::EPSILON);
}

// ── Malformed percentages (out of range) ──────────────────────────

#[test]
fn clamp_out_of_range_percentages() {
    let json = load_fixture("usage-malformed-percent.json");
    let result = parse_provider_response(&json);
    assert!(
        result.is_ok(),
        "Expected Ok with clamping, got Err: {:?}",
        result
    );
    let snapshot = result.unwrap();
    assert_eq!(snapshot.windows.len(), 2);

    let window_5h = snapshot
        .windows
        .iter()
        .find(|w| w.window == "5h")
        .expect("5h window missing");
    // 120.0 should be clamped to 100.0
    assert!((window_5h.remaining_percent - 100.0).abs() < f32::EPSILON);

    let window_7d = snapshot
        .windows
        .iter()
        .find(|w| w.window == "7d")
        .expect("7d window missing");
    // -5.0 should be clamped to 0.0
    assert!((window_7d.remaining_percent - 0.0).abs() < f32::EPSILON);
}

// ── Duplicate windows ─────────────────────────────────────────────

#[test]
fn reject_duplicate_windows() {
    let json = load_fixture("usage-duplicate-windows.json");
    let result = parse_provider_response(&json);
    assert!(
        result.is_err(),
        "Expected Err for duplicate windows, got Ok: {:?}",
        result
    );
    match result {
        Err(ProviderError::DuplicateWindow { window }) => {
            assert_eq!(window, "5h");
        }
        other => panic!("Expected DuplicateWindow error, got: {:?}", other),
    }
}

// ── Unsupported window filtering ──────────────────────────────────

#[test]
fn filter_unsupported_windows() {
    let json = load_fixture("usage-unsupported-window.json");
    let result = parse_provider_response(&json);
    assert!(
        result.is_ok(),
        "Expected Ok (unsupported filtered), got Err: {:?}",
        result
    );
    let snapshot = result.unwrap();
    assert_eq!(snapshot.windows.len(), 1);
    assert_eq!(snapshot.windows[0].window, "7d");
}

// ── Failed / error response ───────────────────────────────────────

#[test]
fn handle_failed_response() {
    let json = load_fixture("usage-failed.json");
    let result = parse_provider_response(&json);
    assert!(
        result.is_err(),
        "Expected Err for failed response, got Ok: {:?}",
        result
    );
    match result {
        Err(ProviderError::SourceUnavailable { .. }) => {}
        other => panic!("Expected SourceUnavailable error, got: {:?}", other),
    }
}

// ── Missing fixture file (fixture loader panics by design) ────────

#[test]
#[should_panic(expected = "nonexistent-fixture.json")]
fn handle_missing_fixture_file() {
    let _ = load_fixture("nonexistent-fixture.json");
}

// ── Empty windows array ───────────────────────────────────────────

#[test]
fn handle_empty_windows_array() {
    let json = r#"{"windows": []}"#.to_string();
    let result = parse_provider_response(&json);
    assert!(
        result.is_ok(),
        "Expected Ok for empty windows, got Err: {:?}",
        result
    );
    let snapshot = result.unwrap();
    assert_eq!(snapshot.windows.len(), 0);
}

// ── ProviderError serialization ───────────────────────────────────

#[test]
fn provider_error_serializes_without_secrets() {
    let err = ProviderError::SourceUnavailable {
        source: "test_source".to_string(),
        detail: "test detail".to_string(),
    };
    let json = serde_json::to_string(&err).unwrap();
    // Ensure no sensitive data leaks (tokens, cookies, etc.)
    assert!(!json.contains("token"));
    assert!(!json.contains("cookie"));
    assert!(!json.contains("secret"));
    assert!(!json.contains("authorization"));
}

// ── ProviderSnapshot fields ───────────────────────────────────────

#[test]
fn provider_snapshot_has_expected_fields() {
    let snapshot = ProviderSnapshot {
        windows: vec![ProviderWindow {
            window: "5h".to_string(),
            remaining_percent: 50.0,
            reset_at: Some("2026-07-16T08:00:00Z".to_string()),
        }],
    };
    assert_eq!(snapshot.windows.len(), 1);
    assert_eq!(snapshot.windows[0].window, "5h");
    assert_eq!(snapshot.windows[0].remaining_percent, 50.0);
}

// ── Clamping boundary values ──────────────────────────────────────

#[test]
fn clamp_boundary_values() {
    let json = r#"{
        "windows": [
            {"window": "5h", "remainingPercent": 0.0, "resetAt": "2026-07-16T08:00:00Z"},
            {"window": "7d", "remainingPercent": 100.0, "resetAt": "2026-07-22T00:00:00Z"},
            {"window": "5h", "remainingPercent": 50.5, "resetAt": "2026-07-16T09:00:00Z"}
        ]
    }"#;
    let result = parse_provider_response(json);
    // 5h appears twice — should be rejected as duplicate
    assert!(
        result.is_err(),
        "Expected duplicate error for boundary test"
    );
}

// ── Error message redaction ───────────────────────────────────────

#[test]
fn error_messages_do_not_contain_secrets() {
    let err = ProviderError::ParseError {
        source: "test".to_string(),
        detail: "invalid json".to_string(),
    };
    let json = serde_json::to_string(&err).unwrap();
    assert!(!json.contains("token"));
    assert!(!json.contains("cookie"));
    assert!(!json.contains("secret"));
    assert!(!json.contains("authorization"));
    assert!(!json.contains("session"));
}
