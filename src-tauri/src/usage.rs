use crate::provider::CodexUsageProvider;
use crate::state::{UsageSnapshot, UsageStatus, UsageWindow};
use std::sync::{Arc, Mutex};

/// In-memory service that wraps a `CodexUsageProvider` and manages
/// the last-successful snapshot. It never persists credentials or
/// usage history.
pub struct UsageService<P: CodexUsageProvider> {
    /// The underlying provider.
    provider: P,
    /// The last-successful snapshot, if any.
    last_success: Arc<Mutex<Option<UsageSnapshot>>>,
    /// Timestamp of the last successful fetch (ISO-8601).
    fetched_at: Arc<Mutex<String>>,
}

impl<P: CodexUsageProvider + Clone> Clone for UsageService<P> {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            last_success: Arc::clone(&self.last_success),
            fetched_at: Arc::clone(&self.fetched_at),
        }
    }
}

impl<P: CodexUsageProvider> UsageService<P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            last_success: Arc::new(Mutex::new(None)),
            fetched_at: Arc::new(Mutex::new(String::new())),
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
    /// Returns `None` if no snapshot has ever been cached.
    pub fn get_usage_snapshot(&self) -> Option<UsageSnapshot> {
        self.last_success.lock().unwrap().clone()
    }

    /// Refreshes usage data from the provider.
    ///
    /// - On success: returns `Ok(UsageSnapshot)` with `Fresh` status.
    /// - On failure with a prior cached snapshot: returns `Err(UsageSnapshot)`
    ///   with `Stale` status, preserving the old windows.
    /// - On failure with no prior snapshot: returns `Err(UsageSnapshot)`
    ///   with `Unavailable` status.
    pub fn refresh(&self) -> Result<UsageSnapshot, UsageSnapshot> {
        let now = chrono::Utc::now().to_rfc3339();

        match self.provider.fetch_and_parse() {
            Ok(provider_snapshot) => {
                let windows: Vec<UsageWindow> = provider_snapshot
                    .windows
                    .into_iter()
                    .map(|pw| UsageWindow {
                        window: pw.window,
                        remaining_percent: pw.remaining_percent,
                        reset_at: pw.reset_at,
                    })
                    .collect();

                let snapshot = UsageSnapshot {
                    windows,
                    status: UsageStatus::Fresh {
                        fetched_at: now.clone(),
                    },
                };

                *self.fetched_at.lock().unwrap() = now;
                *self.last_success.lock().unwrap() = Some(snapshot.clone());
                Ok(snapshot)
            }
            Err(_) => {
                let mut last = self.last_success.lock().unwrap();
                if let Some(ref cached) = *last {
                    // Preserve cached windows but mark stale.
                    let stale_status = UsageStatus::Stale {
                        fetched_at: (*self.fetched_at.lock().unwrap()).clone(),
                        failed_at: now,
                        message: "Usage refresh failed; showing cached data".to_string(),
                    };
                    let stale_snapshot = UsageSnapshot {
                        windows: cached.windows.clone(),
                        status: stale_status,
                    };
                    // Update the cached snapshot to the stale version so
                    // subsequent refreshes also see stale (not unavailable).
                    *last = Some(stale_snapshot.clone());
                    Err(stale_snapshot)
                } else {
                    // No prior data — first load failed.
                    let unavailable = UsageSnapshot {
                        windows: Vec::new(),
                        status: UsageStatus::Unavailable {
                            message: "Usage data is currently unavailable".to_string(),
                        },
                    };
                    // Store the unavailable snapshot so get_usage_snapshot
                    // returns it, and subsequent failures see stale.
                    *last = Some(unavailable.clone());
                    Err(unavailable)
                }
            }
        }
    }
}
