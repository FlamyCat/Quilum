//! Process scanning and killing for the application blocker.
//!
//! [`ProcessPoller`] owns a [`System`] and can kill any running
//! process whose file name matches one of a given blocked list. It is consumed
//! by exactly one worker thread (see [`crate::blocker`]), so it needs no
//! synchronization.

use std::{collections::HashSet, ffi::OsStr, path::PathBuf};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, UpdateKind};

/// Polls the running processes and kills any that match a blocked list.
pub(crate) struct ProcessPoller {
    system: System,
}

impl ProcessPoller {
    pub(crate) fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::nothing().with_processes(
                ProcessRefreshKind::nothing()
                    .without_tasks()
                    .with_cmd(UpdateKind::Always)
                    .with_exe(UpdateKind::Always),
            ),
        );
        Self { system }
    }

    /// Scan the running processes and kill any whose file name matches one of
    /// the blocked apps' file names. Returns the number of processes killed.
    pub(crate) fn scan_and_kill(&mut self, blocked: &HashSet<PathBuf>) -> usize {
        let blocked_names: HashSet<&OsStr> =
            blocked.iter().filter_map(|path| path.file_name()).collect();

        self.system.refresh_processes(ProcessesToUpdate::All, true);

        self.system
            .processes()
            .values()
            .filter(|process| {
                process_name(process).is_some_and(|name| blocked_names.contains(name))
            })
            .map(|process| process.kill() as usize)
            .sum()
    }
}

/// The process's executable file name, falling back to the process name when
/// the executable path cannot be resolved.
fn process_name(process: &sysinfo::Process) -> Option<&OsStr> {
    process
        .exe()
        .and_then(|path| path.file_name())
        .or_else(|| Some(process.name()))
}
