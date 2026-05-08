use chrono::{DateTime, Utc};
use std::{
    collections::HashSet,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{app_list::AppInfo, model::AppIdentifier};

#[derive(Clone)]
pub struct BlockingSession {
    blocked_apps: Arc<RwLock<HashSet<AppIdentifier>>>,
    active: Arc<AtomicBool>,
    end_time: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl BlockingSession {
    pub fn new() -> Self {
        Self {
            blocked_apps: Arc::new(RwLock::new(HashSet::new())),
            active: Arc::new(AtomicBool::new(false)),
            end_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Start blocking with the given list of apps.
    /// Saves the blocked set so the poller can clone it.
    pub fn start(&self, apps: Vec<AppInfo>, end_time: DateTime<Utc>) {
        let mut blocked = self.blocked_apps.write().unwrap();
        blocked.clear();
        for app in apps {
            blocked.insert(app.identifier.clone());
        }
        *self.end_time.write().unwrap() = Some(end_time);
        self.active.store(true, Ordering::SeqCst);
    }

    /// Stop blocking (unblock all apps, deactivate session).
    pub fn stop(&self) {
        self.active.store(false, Ordering::SeqCst);
        self.blocked_apps.write().unwrap().clear();
        *self.end_time.write().unwrap() = None;
    }

    /// Update the blocklist with a new set of apps.
    /// Only updates if the session is active.
    pub fn update_apps(&self, apps: Vec<AppInfo>) {
        if !self.is_active() {
            return;
        }
        let mut blocked = self.blocked_apps.write().unwrap();
        blocked.clear();
        for app in apps {
            blocked.insert(app.identifier);
        }
    }

    /// Check if blocking is currently active.
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    /// Get the blocked apps set for cloning into blocking layers.
    pub fn blocked_apps(&self) -> Arc<RwLock<HashSet<AppIdentifier>>> {
        self.blocked_apps.clone()
    }

    /// Get the session end time.
    pub fn end_time(&self) -> Option<DateTime<Utc>> {
        *self.end_time.read().unwrap()
    }
}

impl Default for BlockingSession {
    fn default() -> Self {
        Self::new()
    }
}
