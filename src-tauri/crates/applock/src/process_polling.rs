use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, UpdateKind};
use tokio::task::JoinHandle;

fn get_exe_path(process: &sysinfo::Process) -> PathBuf {
    process
        .exe()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(process.name()))
}

pub struct ProcessPoller {
    sys: Mutex<System>,
    blocked: Arc<RwLock<HashSet<PathBuf>>>,
}

impl ProcessPoller {
    pub fn new(blocked: Arc<RwLock<HashSet<PathBuf>>>) -> Self {
        let sys = System::new_with_specifics(
            RefreshKind::nothing().with_processes(
                ProcessRefreshKind::nothing()
                    .without_tasks()
                    .with_cmd(UpdateKind::Always)
                    .with_exe(UpdateKind::Always),
            ),
        );
        Self {
            sys: Mutex::new(sys),
            blocked,
        }
    }

    /// Scan running processes and kill any that are blocked.
    /// Returns the number of processes killed.
    pub fn scan_and_kill(&self) -> usize {
        let blocked = self.blocked.read().unwrap();
        let mut sys = self.sys.lock().unwrap();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let mut killed = 0;
        for process in sys.processes().values() {
            let exe_path = get_exe_path(process);

            for blocked_path in blocked.iter() {
                let Some(blocked_file_name) = blocked_path.file_name() else {
                    continue;
                };

                let Some(process_file_name) = exe_path.file_name() else {
                    continue;
                };

                if process_file_name == blocked_file_name {
                    let _ = process.kill();
                    killed += 1;
                    break;
                }
            }
        }

        killed
    }
}

pub type OnKill = Arc<dyn Fn(usize) + Send + Sync>;

pub fn start_polling(
    blocked: Arc<RwLock<HashSet<PathBuf>>>,
    poll_interval: Duration,
    stop_flag: Arc<std::sync::atomic::AtomicBool>,
    on_kill: Option<OnKill>,
) -> JoinHandle<()> {
    let poller = ProcessPoller::new(blocked.clone());

    tokio::spawn(async move {
        poller.scan_and_kill();
        loop {
            tokio::select! {
                _ = tokio::time::sleep(poll_interval) => {
                    if stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }
                    let killed = poller.scan_and_kill();
                    if killed > 0 && let Some(on_kill) = &on_kill {
                        on_kill(killed);
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    if stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }
                }
            }
        }
    })
}
