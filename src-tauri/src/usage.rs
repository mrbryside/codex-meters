use crate::provider::CodexUsageProvider;
use crate::state::{UsageSnapshot, UsageStatus, UsageWindow};
use std::sync::{Arc, Mutex};

/// Shared cache: holds the last successful (Fresh) snapshot and the current snapshot.
#[derive(Default)]
struct UsageCache {
    /// The most recent snapshot returned by `get_usage_snapshot()`.
    current_snapshot: Option<UsageSnapshot>,
    /// The last genuinely Fresh snapshot — used as fallback data for Stale state.
    last_successful_snapshot: Option<UsageSnapshot>,
}

/// In-memory service that wraps a `CodexUsageProvider` and manages
/// the last-successful snapshot. It never persists credentials or
/// usage history.
pub struct UsageService<P: CodexUsageProvider> {
    /// The underlying provider.
    provider: P,
    /// Shared cache for current and last-successful snapshots.
    cache: Arc<Mutex<UsageCache>>,
}

impl<P: CodexUsageProvider + Clone> Clone for UsageService<P> {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            cache: Arc::clone(&self.cache),
        }
    }
}

impl<P: CodexUsageProvider> UsageService<P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            cache: Arc::new(Mutex::new(UsageCache::default())),
        }
    }

    /// Returns a reference to the provider (for testing).
    pub fn provider(&self) -> &P {
        &self.provider
    }

    /// Returns a mutable reference to the provider (for testing).
    pub fn provider_mut(&mut self) -> &mut P {
        &mut self.provider
    }

    /// Returns the current snapshot without refreshing.
    /// Returns `None` only if no refresh has ever been attempted.
    pub fn get_usage_snapshot(&self) -> Option<UsageSnapshot> {
        self.cache.lock().unwrap().current_snapshot.clone()
    }

    /// Refreshes usage data from the provider.
    ///
    /// - On success: returns `Ok(UsageSnapshot)` with `Fresh` status.
    /// - On failure with a prior Fresh snapshot: returns `Err(UsageSnapshot)`
    ///   with `Stale` status, preserving the old windows and fetched_at.
    /// - On failure with no prior Fresh snapshot: returns `Err(UsageSnapshot)`
    ///   with `Unavailable` status.
    ///
    /// Every resulting state (Fresh, Stale, Unavailable) is stored as the
    /// current snapshot so `get_usage_snapshot()` always returns the latest.
    pub fn refresh(&self) -> Result<UsageSnapshot, UsageSnapshot> {
        let now = chrono::Utc::now().to_rfc3339();

        match self.provider.fetch_and_parse() {
            Ok(raw) => {
                let windows: Vec<UsageWindow> = raw
                    .windows
                    .into_iter()
                    .map(|pw| UsageWindow {
                        window: pw.window,
                        remaining_percent: pw.remaining_percent,
                        reset_at: pw.reset_at,
                    })
                    .collect();

                let fresh = UsageSnapshot {
                    windows,
                    status: UsageStatus::Fresh {
                        fetched_at: now.clone(),
                    },
                };

                let mut cache = self.cache.lock().unwrap();
                cache.last_successful_snapshot = Some(fresh.clone());
                cache.current_snapshot = Some(fresh.clone());
                Ok(fresh)
            }
            Err(_) => {
                let mut cache = self.cache.lock().unwrap();
                let failure = if let Some(success) = cache.last_successful_snapshot.as_ref() {
                    let fetched_at = match &success.status {
                        UsageStatus::Fresh { fetched_at } => fetched_at.clone(),
                        _ => unreachable!(),
                    };
                    UsageSnapshot {
                        windows: success.windows.clone(),
                        status: UsageStatus::Stale {
                            fetched_at,
                            failed_at: now,
                            message: "Usage refresh failed; showing cached data".into(),
                        },
                    }
                } else {
                    UsageSnapshot {
                        windows: vec![],
                        status: UsageStatus::Unavailable {
                            message: "Usage data is currently unavailable".into(),
                        },
                    }
                };
                cache.current_snapshot = Some(failure.clone());
                Err(failure)
            }
        }
    }
}
