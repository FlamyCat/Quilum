//! Cooperative application blocking.
//!
//! [`AppBlocker`] is the public handle to a worker thread that periodically
//! scans running processes and kills any that are currently blocked. All
//! interaction with that thread happens through a [`mpsc`] channel:
//!
//! - [`AppBlocker::start`] hands the worker a new blocklist (loaded by the
//!   caller, not here) plus the reason blocking was started;
//! - [`AppBlocker::stop`] asks the worker to stop blocking.
//!
//! The worker publishes [`Event`]s about what it did on the notification
//! channel supplied at construction ([`AppBlocker::spawn`]): a session start
//! ([`Event::Started`]) carrying the start reason, one [`Event::AppsKilled`]
//! per scan that killed processes, and a session end ([`Event::Stopped`]).
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

use quilum_db::model::task::Task;

use crate::{app_list::AppInfo, process_polling::ProcessPoller};

/// How often the worker scans running processes while blocking is active.
const POLL_INTERVAL: Duration = Duration::from_millis(300);

/// Reason for which application blocking was started.
///
/// [`AppBlocker`] is generic over this type: it never inspects a
/// [`BlockReason`], it only forwards it to the notification service as part of
/// [`Event::Started`]. The default variant is the one used by this project,
/// but the genericity lets other applications that consume the crate define
/// their own domain-specific reasons.
#[derive(Clone, Debug)]
pub enum BlockReason {
    /// The user started working on a task.
    Task(Task),
}

/// An event produced by the blocker and published on the notification channel.
///
/// [`AppBlocker::spawn`] is given the sending end of the channel; the worker
/// thread emits events strictly in the order they happen: [`Event::Started`],
/// then zero or more [`Event::AppsKilled`], then [`Event::Stopped`].
#[derive(Clone, Debug)]
pub enum Event<R> {
    /// Blocking started; carries the reason it was started for.
    Started(R),
    /// At least one blocked process was killed; the payload is the number of
    /// processes killed in a single scan.
    AppsKilled(usize),
    /// Blocking stopped.
    Stopped,
}

/// A command sent from an [`AppBlocker`] handle to the worker thread.
enum Command<R> {
    /// Begin (or replace) blocking with `apps`, reporting the given reason.
    Start { apps: Vec<AppInfo>, reason: R },
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
/// passes it to [`Self::start`]. Both `Sender`s (see struct fields) are `Send + Sync`,
/// so this can be shared freely, e.g. managed as Tauri state.
#[derive(Clone)]
pub struct AppBlocker<R = BlockReason> {
    worker_tx: Sender<Command<R>>,
    // Kept so the handle owns the notification channel as well; the worker
    // emits on its own clone, but this one keeps the channel alive for the
    // lifetime of the managed handle.
    #[allow(dead_code)]
    event_tx: Sender<Event<R>>,
}

impl<R> AppBlocker<R>
where
    R: Send + 'static,
{
    /// Spawn the worker thread and return a handle to it.
    ///
    /// `event_tx` is the sending end of the channel the worker publishes
    /// [`Event`]s on; the receiving end belongs to the notification service.
    /// The thread starts idle and waits for commands. Exactly one thread is
    /// created regardless of how many clones of this handle exist; it stays
    /// alive until every sender is dropped.
    pub fn spawn(event_tx: Sender<Event<R>>) -> Self {
        let (worker_tx, worker_rx) = mpsc::channel();

        let worker_event_tx = event_tx.clone();
        thread::Builder::new()
            .name("app-blocker".to_string())
            .spawn(move || BlockerWorker::new(worker_event_tx).run(worker_rx))
            .expect("failed to spawn the blocking worker thread");

        Self {
            worker_tx,
            event_tx,
        }
    }

    /// Start blocking the given apps.
    ///
    /// `apps` replaces any previously configured blocklist. `reason` is not
    /// interpreted here — it is forwarded to the notification service as part
    /// of [`Event::Started`].
    pub fn start(&self, apps: Vec<AppInfo>, reason: R) -> Result<(), BlockerError> {
        self.send(Command::Start { apps, reason })
    }

    /// Stop blocking all apps.
    ///
    /// The worker stays alive and waits for the next [`Self::start`].
    pub fn stop(&self) -> Result<(), BlockerError> {
        self.send(Command::Stop)
    }

    fn send(&self, command: Command<R>) -> Result<(), BlockerError> {
        self.worker_tx.send(command).map_err(|_| BlockerError)
    }
}

/// State owned by the blocking worker thread.
///
/// Every field is private to the thread and only ever accessed sequentially
/// from within [`BlockerWorker::run`], so no locks or atomics are required.
struct BlockerWorker<R> {
    poller: ProcessPoller,
    blocked_apps: HashSet<PathBuf>,
    event_tx: Sender<Event<R>>,
    active: bool,
}

impl<R> BlockerWorker<R> {
    fn new(event_tx: Sender<Event<R>>) -> Self {
        Self {
            poller: ProcessPoller::new(),
            blocked_apps: HashSet::new(),
            event_tx,
            active: false,
        }
    }

    /// The worker's event loop.
    ///
    /// Waits for a command for at most [`POLL_INTERVAL`]; every timeout is one
    /// poll cycle. Exits as soon as the channel is closed by all senders.
    fn run(mut self, rx: Receiver<Command<R>>) {
        loop {
            match rx.recv_timeout(POLL_INTERVAL) {
                Ok(command) => match command {
                    Command::Start { apps, reason } => self.handle_start(apps, reason),
                    Command::Stop => self.handle_stop(),
                },
                Err(RecvTimeoutError::Timeout) => self.poll_once(),
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    }

    fn handle_start(&mut self, apps: Vec<AppInfo>, reason: R) {
        self.blocked_apps = apps.into_iter().map(|app| app.identifier).collect();
        self.active = true;
        let _ = self.event_tx.send(Event::Started(reason));
    }

    fn handle_stop(&mut self) {
        self.blocked_apps.clear();
        let was_active = self.active;
        self.active = false;
        if was_active {
            let _ = self.event_tx.send(Event::Stopped);
        }
    }

    /// Scan the running processes once and kill any that are blocked.
    fn poll_once(&mut self) {
        if !self.active {
            return;
        }

        let killed = self.poller.scan_and_kill(&self.blocked_apps);

        if killed > 0 {
            let _ = self.event_tx.send(Event::AppsKilled(killed));
        }
    }
}
