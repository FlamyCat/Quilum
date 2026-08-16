//! Notification service.
//!
//! [`NotificationService`] consumes the [`Event`]s emitted by the app blocker
//! over a channel and turns them into desktop notifications. It is created and
//! started in the Tauri setup; ownership of the receiver and of all session
//! state lives entirely in the single background thread running
//! [`NotificationService::run`], so no synchronization is required.

use std::{
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use applock::blocker::{BlockReason, Event};
use chrono::{DateTime, Utc};
use tauri_plugin_notification::NotificationExt;

/// How often the "keep focusing" reminder may fire after apps were killed.
const MIN_KILL_NOTIFY_INTERVAL: Duration = Duration::from_secs(5);

/// Consumes blocker events and shows desktop notifications for them.
pub struct NotificationService {
    handle: tauri::AppHandle,
    event_rx: Receiver<Event<BlockReason>>,
    session_end: Option<DateTime<Utc>>,
    last_kill_notify_at: Option<Instant>,
}

impl NotificationService {
    pub fn new(handle: tauri::AppHandle, event_rx: Receiver<Event<BlockReason>>) -> Self {
        Self {
            handle,
            event_rx,
            session_end: None,
            last_kill_notify_at: None,
        }
    }

    /// Run the service until the event channel is closed (which only happens
    /// when every `AppBlocker` sender is dropped, i.e. on shutdown).
    pub fn run(mut self) {
        while let Ok(event) = self.event_rx.recv() {
            self.handle_event(event);
        }
    }

    fn handle_event(&mut self, event: Event<BlockReason>) {
        match event {
            Event::Started(reason) => self.on_started(reason),
            Event::AppsKilled(killed) => self.on_apps_killed(killed),
            Event::Stopped => self.on_stopped(),
        }
    }

    fn on_started(&mut self, reason: BlockReason) {
        let BlockReason::Task(task) = reason;

        self.session_end = task
            .scheduled_for()
            .map(|start| start + task.estimated_duration);
        self.last_kill_notify_at = None;

        let duration_minutes = task.estimated_duration().as_secs() / 60;
        let body = format!(
            "Начался период концентрации: \"{}\" ({} мин.). Отвлекающие приложения заблокированы!",
            task.name(),
            duration_minutes
        );
        let _ = self
            .handle
            .notification()
            .builder()
            .title("Период концентрации")
            .body(body)
            .show();
    }

    fn on_apps_killed(&mut self, killed: usize) {
        if killed == 0 {
            return;
        }

        let now = Instant::now();
        if self
            .last_kill_notify_at
            .is_some_and(|prev| now.duration_since(prev) < MIN_KILL_NOTIFY_INTERVAL)
        {
            return;
        }
        self.last_kill_notify_at = Some(now);

        let body = match self.session_end {
            Some(end) => {
                let remaining =
                    ((end - Utc::now()).num_seconds().max(0) as f64 / 60.0).ceil() as i64;
                format!("Активен период концентрации. Осталось {} мин.", remaining)
            }
            None => "Активен период концентрации.".to_string(),
        };
        let _ = self
            .handle
            .notification()
            .builder()
            .title("Сосредоточьтесь на работе!")
            .body(body)
            .show();
    }

    fn on_stopped(&mut self) {
        self.session_end = None;
        self.last_kill_notify_at = None;

        let _ = self
            .handle
            .notification()
            .builder()
            .title("Период концентрации")
            .body("Период концентрации окончен. Приложения разблокированы.")
            .show();
    }
}
