use chrono::{DateTime, Utc};
use std::{sync::Arc, time::Duration};
use tauri::State;
use tokio::{sync::Mutex, task::JoinHandle};

use crate::db::Storage;
use applock::{BlockingSession, app_list::AppInfo, start_polling};

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

pub fn check_and_restore_session(storage: &Storage) {
    let rt = tokio::runtime::Handle::current();
    rt.block_on(async {
        if let Ok(Some(session)) = storage.get_active_session().await
            && let Some(end) = DateTime::from_timestamp(session.end_time, 0)
            && end > Utc::now()
            && let Ok(blocked) = storage.get_blocked_apps().await
        {
            let blocked_info = blocked_apps_to_info(blocked);

            let state = blocking_state();
            let mut guard = state.lock().await;
            stop_blocking(&mut guard).await;

            let bs_session = BlockingSession::new();
            bs_session.start(blocked_info, end);

            let blocked_set = bs_session.blocked_apps();
            let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let poll_interval = Duration::from_millis(300);

            let handle = start_polling(blocked_set, poll_interval, stop_flag.clone());
            let end_time_for_task = end;

            *guard = Some(BlockingState {
                session: bs_session,
                polling_handle: Some(handle),
                stop_flag,
                _storage: storage.clone(),
            });

            drop(guard);

            let _ = tauri::async_runtime::spawn(async move {
                let now = Utc::now();
                let sleep_duration = (end_time_for_task - now).to_std().unwrap_or_default();
                tokio::time::sleep(sleep_duration).await;
                let _ = end_focus_session_internal().await;
            });
        }
    });
}
