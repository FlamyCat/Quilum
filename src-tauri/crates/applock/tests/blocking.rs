//! Integration tests for the app blocker.
//!
//! These tests spawn a dummy process (an infinite sleep loop) and verify that
//! [`AppBlocker`] kills it when the dummy is included in the blocked
//! list. A SIGKILLed, unreaped dummy lingers in the process table as a zombie,
//! so the tests assert on the process *status* rather than mere presence.

use std::{path::PathBuf, process::Command, thread, time::Duration};

use applock::{AppBlocker, app_list::AppInfo};
use sysinfo::{System, UpdateKind};

fn build_dummy() -> PathBuf {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let crates_dir = manifest.parent().unwrap();
    let src_tauri_dir = crates_dir.parent().unwrap();
    let dummy_bin = src_tauri_dir
        .join("target")
        .join("debug")
        .join("applock-test-dummy");

    if !dummy_bin.exists() {
        let status = Command::new("cargo")
            .args(["build", "--bin", "applock-test-dummy"])
            .current_dir(src_tauri_dir)
            .status()
            .expect("Failed to build applock-test-dummy");
        assert!(status.success(), "Failed to build applock-test-dummy");
    }

    dummy_bin
}

fn spawn_dummy() -> sysinfo::Pid {
    let dummy_path = build_dummy();
    let child = Command::new(&dummy_path)
        .spawn()
        .expect("Failed to spawn applock-test-dummy");
    thread::sleep(Duration::from_millis(500));
    sysinfo::Pid::from_u32(child.id())
}

fn get_dummy_pids() -> Vec<(sysinfo::Pid, PathBuf)> {
    let mut sys = System::new_with_specifics(
        sysinfo::RefreshKind::nothing().with_processes(
            sysinfo::ProcessRefreshKind::nothing()
                .with_cmd(UpdateKind::Always)
                .with_exe(UpdateKind::Always),
        ),
    );
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    sys.processes()
        .iter()
        .filter(|(_, process)| {
            let name = process.name().to_string_lossy().to_lowercase();
            name.contains("applock-test-du") && !name.contains("test_dummy")
        })
        .map(|(pid, process)| {
            (
                *pid,
                process.exe().map(|p| p.to_path_buf()).unwrap_or_default(),
            )
        })
        .collect()
}

/// Number of dummy processes still alive, i.e. not a zombie or a dead process.
fn count_running_dummies() -> usize {
    let mut sys = System::new_with_specifics(
        sysinfo::RefreshKind::nothing().with_processes(
            sysinfo::ProcessRefreshKind::nothing()
                .with_cmd(UpdateKind::Always)
                .with_exe(UpdateKind::Always),
        ),
    );
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    sys.processes()
        .values()
        .filter(|process| {
            let name = process.name().to_string_lossy().to_lowercase();
            name.contains("applock-test-du")
                && !name.contains("test_dummy")
                && !matches!(
                    process.status(),
                    sysinfo::ProcessStatus::Zombie | sysinfo::ProcessStatus::Dead
                )
        })
        .count()
}

fn cleanup_dummies() {
    for (pid, _) in get_dummy_pids() {
        let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
    }
    thread::sleep(Duration::from_millis(100));
}

/// Give the blocker enough time to run a couple of poll cycles.
fn wait_for_kill() {
    thread::sleep(Duration::from_millis(1000));
}

#[test]
#[cfg(target_os = "linux")]
fn test_dummy_gets_killed_by_path() {
    let _pid = spawn_dummy();
    let dummies_before = get_dummy_pids();
    assert!(!dummies_before.is_empty(), "Dummy should be running");
    let (_dummy_pid, exe_path) = dummies_before.into_iter().next().unwrap();

    let blocker = AppBlocker::spawn();
    blocker
        .start(vec![AppInfo::new(exe_path, "test".to_string())], None)
        .unwrap();

    wait_for_kill();

    assert_eq!(
        count_running_dummies(),
        0,
        "blocker should have killed the matched process"
    );

    blocker.stop().unwrap();
    cleanup_dummies();
}

#[test]
#[cfg(target_os = "linux")]
fn test_dummy_gets_killed_by_name() {
    let _pid = spawn_dummy();
    let dummies_before = get_dummy_pids();
    assert!(!dummies_before.is_empty(), "Dummy should be running");

    let blocker = AppBlocker::spawn();
    blocker
        .start(
            vec![AppInfo::new(
                PathBuf::from("applock-test-dummy"),
                "test".to_string(),
            )],
            None,
        )
        .unwrap();

    wait_for_kill();

    assert_eq!(
        count_running_dummies(),
        0,
        "blocker should have killed the matched process"
    );

    blocker.stop().unwrap();
    cleanup_dummies();
}
