use crate::provider::CodexAppServerProvider;
use crate::state::AppState;
use crate::usage::UsageService;
use std::sync::mpsc::{self, SyncSender};
use std::thread;
use std::time::Duration;
use tauri::{Emitter, Manager};

/// Wakes the native usage polling loop when its interval changes.
pub struct RefreshLoopHandle {
    wake_sender: SyncSender<()>,
}

impl RefreshLoopHandle {
    pub fn start(
        app: tauri::AppHandle,
        service: UsageService<CodexAppServerProvider>,
        mock_usage: bool,
    ) -> Self {
        let (wake_sender, wake_receiver) = mpsc::sync_channel(1);

        thread::spawn(move || loop {
            let snapshot = if mock_usage {
                crate::mock_snapshot()
            } else {
                service.refresh().unwrap_or_else(|snapshot| snapshot)
            };
            let _ = app.emit("usage-updated", &snapshot);

            let interval_seconds = app
                .state::<AppState>()
                .settings
                .lock()
                .unwrap()
                .refresh_interval_seconds;

            match wake_receiver.recv_timeout(Duration::from_secs(interval_seconds)) {
                Ok(()) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        });

        Self { wake_sender }
    }

    pub fn wake(&self) {
        let _ = self.wake_sender.try_send(());
    }
}
