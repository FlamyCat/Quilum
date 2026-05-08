use std::{
    collections::HashSet, process::Command, sync::{Arc, RwLock}, thread, time::Duration
};
use sysinfo::{System, UpdateKind};

fn build_dummy() -> std::path::PathBuf {
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
            .current_dir(&src_tauri_dir)
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

fn get_dummy_pids() -> Vec<(sysinfo::Pid, std::path::PathBuf)> {
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

fn cleanup_dummies() {
    for (pid, _) in get_dummy_pids() {
        let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
    }
    thread::sleep(Duration::from_millis(100));
}

#[test]
#[cfg(target_os = "linux")]
fn test_dummy_gets_killed_by_path() {
    use applock::model::AppIdentifier;
    use applock::process_polling::ProcessPoller;

    let _pid = spawn_dummy();
    let dummies_before = get_dummy_pids();
    assert!(!dummies_before.is_empty(), "Dummy should be running");
    let (_dummy_pid, exe_path) = dummies_before.into_iter().next().unwrap();

    let blocked = AppIdentifier::Path(exe_path.clone());
    let blocked_set: Arc<RwLock<HashSet<AppIdentifier>>> =
        Arc::new(RwLock::new(HashSet::from([blocked])));

    let poller = ProcessPoller::new(blocked_set.clone());
    let killed = poller.scan_and_kill();

    assert!(
        killed > 0,
        "scan_and_kill should call kill() on matched processes"
    );

    cleanup_dummies();
}

#[test]
#[cfg(target_os = "linux")]
fn test_dummy_gets_killed_by_name() {
    use applock::model::AppIdentifier;
    use applock::process_polling::ProcessPoller;

    let _pid = spawn_dummy();
    let dummies_before = get_dummy_pids();
    assert!(!dummies_before.is_empty(), "Dummy should be running");

    let blocked = AppIdentifier::Path(std::path::PathBuf::from("applock-test-dummy"));
    let blocked_set: Arc<RwLock<HashSet<AppIdentifier>>> =
        Arc::new(RwLock::new(HashSet::from([blocked])));

    let poller = ProcessPoller::new(blocked_set.clone());
    let killed = poller.scan_and_kill();

    assert!(
        killed > 0,
        "scan_and_kill should call kill() on matched processes (got {})",
        killed
    );

    cleanup_dummies();
}
