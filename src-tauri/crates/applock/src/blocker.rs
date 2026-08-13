//! Cooperative application blocking.
//!
//! [`AppBlocker`] is the public handle to a worker thread that periodically
//! scans running processes and kills any that are currently blocked. All
//! interaction with that thread happens through a [`mpsc`] channel:
//!
//! - [`AppBlocker::start`] hands the worker a new blocklist (loaded by the
//!   caller, not here) plus an optional kill callback;
//! - [`AppBlocker::stop`] asks the worker to stop blocking.
//!
//! The worker thread is spawned once (see [`AppBlocker::spawn`]) and then
//! stays alive: after a [`Command::Stop`] it simply goes idle until the next
//! `start`. Cancellation is cooperative — the worker waits for commands with
//! [`Receiver::recv_timeout`], so every command is picked up within at most
//! one poll interval, and each timeout results in a poll cycle.

use std::{
    collections::HashSet,
    path::PathBuf,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread,
    time::Duration,
};

use crate::{
    app_list::AppInfo,
    process_polling::{OnKill, ProcessPoller},
};

/// How often the worker scans running processes while blocking is active.
const POLL_INTERVAL: Duration = Duration::from_millis(300);

/// A command sent from an [`AppBlocker`] handle to the worker thread.
enum Command {
    /// Begin (or replace) blocking with `apps`, invoking `on_kill` on kills.
    Start {
        apps: Vec<AppInfo>,
        on_kill: Option<OnKill>,
    },
    /// Stop blocking; the worker remains alive waiting for the next `Start`.
    Stop,
}

/// Error returned when a command cannot be delivered to the worker thread,
/// normally because every sender has been dropped and the thread exited.
#[derive(Debug, thiserror::Error)]
#[error("blocking worker is not responding")]
pub struct BlockerError;

/// Public handle to the blocking worker thread.
///
/// Cloning is cheap and all clones address the same worker. This type never
/// loads the list of apps to block — the caller is responsible for that and
/// passes it to [`Self::start`]. `Sender` is `Send + Sync`, so this can be
/// shared freely, e.g. managed as Tauri state.
#[derive(Clone)]
pub struct AppBlocker {
    tx: Sender<Command>,
}

impl AppBlocker {
    /// Spawn the worker thread and return a handle to it.
    ///
    /// The thread starts idle and waits for commands. Exactly one thread is
    /// created regardless of how many clones of this handle exist; it stays
    /// alive until every sender is dropped.
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel();

        thread::Builder::new()
            .name("app-blocker".to_string())
            .spawn(move || BlockerWorker::new().run(rx))
            .expect("failed to spawn the blocking worker thread");

        Self { tx }
    }

    /// Start blocking the given apps.
    ///
    /// `apps` replaces any previously configured blocklist. `on_kill`, when
    /// provided, is invoked from the worker thread whenever at least one
    /// matching process is killed.
    pub fn start(&self, apps: Vec<AppInfo>, on_kill: Option<OnKill>) -> Result<(), BlockerError> {
        self.send(Command::Start { apps, on_kill })
    }

    /// Stop blocking all apps.
    ///
    /// The worker stays alive and waits for the next [`Self::start`].
    pub fn stop(&self) -> Result<(), BlockerError> {
        self.send(Command::Stop)
    }

    fn send(&self, command: Command) -> Result<(), BlockerError> {
        self.tx.send(command).map_err(|_| BlockerError)
    }
}

/// State owned by the blocking worker thread.
///
/// Every field is private to the thread and only ever accessed sequentially
/// from within [`BlockerWorker::run`], so no locks or atomics are required.
struct BlockerWorker {
    poller: ProcessPoller,
    blocked_apps: HashSet<PathBuf>,
    on_kill: Option<OnKill>,
    active: bool,
}

impl BlockerWorker {
    fn new() -> Self {
        Self {
            poller: ProcessPoller::new(),
            blocked_apps: HashSet::new(),
            on_kill: None,
            active: false,
        }
    }

    /// The worker's event loop.
    ///
    /// Waits for a command for at most [`POLL_INTERVAL`]; every timeout is one
    /// poll cycle. Exits as soon as the channel is closed by all senders.
    fn run(mut self, rx: Receiver<Command>) {
        loop {
            match rx.recv_timeout(POLL_INTERVAL) {
                Ok(command) => match command {
                    Command::Start { apps, on_kill } => self.handle_start(apps, on_kill),
                    Command::Stop => self.handle_stop(),
                },
                Err(RecvTimeoutError::Timeout) => self.poll_once(),
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    }

    fn handle_start(&mut self, apps: Vec<AppInfo>, on_kill: Option<OnKill>) {
        self.blocked_apps = apps.into_iter().map(|app| app.identifier).collect();
        self.on_kill = on_kill;
        self.active = true;
    }

    fn handle_stop(&mut self) {
        self.blocked_apps.clear();
        self.on_kill = None;
        self.active = false;
    }

    /// Scan the running processes once and kill any that are blocked.
    fn poll_once(&mut self) {
        if !self.active {
            return;
        }

        let killed = self.poller.scan_and_kill(&self.blocked_apps);

        if killed > 0
            && let Some(on_kill) = &self.on_kill
        {
            on_kill(killed);
        }
    }
}
