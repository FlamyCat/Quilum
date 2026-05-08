use std::{
    collections::HashSet,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, UpdateKind};
use tokio::task::JoinHandle;

use crate::model::AppIdentifier;

#[cfg(any(windows, target_os = "linux"))]
fn get_exe_path(process: &sysinfo::Process) -> std::path::PathBuf {
    process
        .exe()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from(process.name()))
}

#[cfg(any(windows, target_os = "linux"))]
pub struct ProcessPoller {
    sys: Mutex<System>,
    blocked: Arc<RwLock<HashSet<AppIdentifier>>>,
}

#[cfg(any(windows, target_os = "linux"))]
impl ProcessPoller {
    pub fn new(blocked: Arc<RwLock<HashSet<AppIdentifier>>>) -> Self {
        let sys = System::new_with_specifics(
            RefreshKind::nothing().with_processes(
                ProcessRefreshKind::nothing()
                    .without_tasks()
                    .with_cmd(UpdateKind::Always)
                    .with_exe(UpdateKind::Always)
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
        for (_pid, process) in sys.processes() {
            let exe_path = get_exe_path(process);
            let exe_name = process.name().to_string_lossy().to_lowercase();

            for blocked_app in blocked.iter() {
                let matches = match blocked_app {
                    AppIdentifier::Path(blocked_path) => {
                        let blocked_str = blocked_path.to_string_lossy().to_lowercase();
                        let exe_str = exe_path.to_string_lossy().to_lowercase();
                        exe_str.ends_with(&blocked_str)
                            || exe_str.contains(&blocked_str)
                            || exe_name == blocked_str
                    }
                    AppIdentifier::BundleId(_) => false,
                };

                if matches {
                    let _ = process.kill();
                    killed += 1;
                    break;
                }
            }
        }

        killed
    }
}

#[cfg(any(windows, target_os = "linux"))]
pub fn start_polling(
    blocked: Arc<RwLock<HashSet<AppIdentifier>>>,
    poll_interval: Duration,
    stop_flag: Arc<std::sync::atomic::AtomicBool>,
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
                    poller.scan_and_kill();
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
