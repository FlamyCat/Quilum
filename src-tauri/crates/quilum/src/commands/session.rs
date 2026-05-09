use chrono::{DateTime, Utc};
use std::{sync::Arc, time::Duration};
use tauri::State;
use tokio::{sync::Mutex, task::JoinHandle};

use crate::db::Storage;
use applock::{app_list::AppInfo, start_polling, BlockingSession};

pub struct BlockingState {
    pub session: BlockingSession,
    pub polling_handle: Option<JoinHandle<()>>,
    pub stop_flag: Arc<std::sync::atomic::AtomicBool>,
    _storage: Storage,
}

pub fn blocking_state() -> &'static Arc<Mutex<Option<BlockingState>>> {
    static STATE: std::sync::OnceLock<Arc<Mutex<Option<BlockingState>>>> =
        std::sync::OnceLock::new();
    STATE.get_or_init(|| Arc::new(Mutex::new(None)))
}

async fn stop_blocking(guard: &mut tokio::sync::MutexGuard<'_, Option<BlockingState>>) {
    if let Some(ref mut bs) = **guard {
        bs.stop_flag
            .store(true, std::sync::atomic::Ordering::SeqCst);
        bs.session.stop();
        if let Some(h) = bs.polling_handle.take() {
            let _ = h.abort();
        }
    }
}

fn blocked_apps_to_info(blocked: Vec<quilum_db::blocked_app::BlockedApp>) -> Vec<AppInfo> {
    blocked
        .into_iter()
        .map(|app| AppInfo {
            identifier: app.app_identifier(),
            display_name: app.display_name,
        })
        .collect()
}

async fn end_focus_session_internal() -> Result<(), String> {
    let state = blocking_state();
    let mut guard = state.lock().await;
    stop_blocking(&mut guard).await;

    let storage = guard
        .as_ref()
        .map(|bs| bs._storage.clone())
        .ok_or("No active session")?;

    drop(guard);

    storage.end_session().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_focus_session(storage: State<'_, Storage>, end_time: i64) -> Result<(), String> {
    let end_time = DateTime::from_timestamp(end_time, 0).ok_or("Invalid end time")?;
    let now = Utc::now();

    storage
        .start_session(now.naive_utc(), end_time.naive_utc(), None)
        .await
        .map_err(|e| e.to_string())?;

    let blocked = storage
        .get_blocked_apps()
        .await
        .map_err(|e| e.to_string())?;
    let blocked_info = blocked_apps_to_info(blocked);

    let state = blocking_state();
    let mut guard = state.lock().await;
    stop_blocking(&mut guard).await;

    let session = BlockingSession::new();
    session.start(blocked_info, end_time);

    let blocked_set = session.blocked_apps();
    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let poll_interval = Duration::from_millis(300);

    let handle = start_polling(blocked_set, poll_interval, stop_flag.clone());
    let end_time_for_task = end_time;

    *guard = Some(BlockingState {
        session,
        polling_handle: Some(handle),
        stop_flag,
        _storage: (*storage).clone(),
    });

    drop(guard);

    tauri::async_runtime::spawn(async move {
        let now = Utc::now();
        let sleep_duration = (end_time_for_task - now).to_std().unwrap_or_default();
        tokio::time::sleep(sleep_duration).await;
        let _ = end_focus_session_internal().await;
    });

    Ok(())
}

#[tauri::command]
pub async fn end_focus_session() -> Result<(), String> {
    end_focus_session_internal().await
}

pub fn check_and_restore_session(storage: Storage) {
    tauri::async_runtime::spawn(async move {
        let state = blocking_state();
        let mut guard = state.lock().await;
        stop_blocking(&mut guard).await;
        drop(guard);

        let Ok(Some((task, scheduled_for))) = storage.get_next_scheduled_task().await else {
            return;
        };

        let Ok(blocked) = storage.get_blocked_apps().await else {
            return;
        };

        let blocked_info = blocked_apps_to_info(blocked.clone());
        let now = Utc::now().timestamp();
        let end_timestamp = scheduled_for + task.estimated_duration;

        if scheduled_for <= now && now <= end_timestamp {
            let end = DateTime::from_timestamp(end_timestamp, 0).unwrap_or_default();
            start_blocking(blocked_info, end, storage.clone());
        } else if scheduled_for > now {
            let start_time = DateTime::from_timestamp(scheduled_for, 0).unwrap_or_default();
            let end = DateTime::from_timestamp(end_timestamp, 0).unwrap_or_default();

            let blocked_info_clone = blocked_apps_to_info(blocked);
            let storage_clone = storage.clone();

            tauri::async_runtime::spawn(async move {
                let sleep_duration = (start_time - Utc::now()).to_std().unwrap_or_default();
                tokio::time::sleep(sleep_duration).await;
                start_blocking(blocked_info_clone, end, storage_clone);
            });
        }
    });
}

fn start_blocking(apps: Vec<AppInfo>, end_time: DateTime<Utc>, storage: Storage) {
    tauri::async_runtime::spawn(async move {
        let state = blocking_state();
        let mut guard = state.lock().await;
        stop_blocking(&mut guard).await;

        let bs_session = BlockingSession::new();
        bs_session.start(apps, end_time);

        let blocked_set = bs_session.blocked_apps();
        let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let poll_interval = Duration::from_millis(300);

        let handle = start_polling(blocked_set, poll_interval, stop_flag.clone());
        let end_time_for_task = end_time;

        *guard = Some(BlockingState {
            session: bs_session,
            polling_handle: Some(handle),
            stop_flag,
            _storage: storage,
        });

        drop(guard);

        let _ = tauri::async_runtime::spawn(async move {
            let now = Utc::now();
            let sleep_duration = (end_time_for_task - now).to_std().unwrap_or_default();
            tokio::time::sleep(sleep_duration).await;
            let _ = end_focus_session_internal().await;
        });
    });
}
