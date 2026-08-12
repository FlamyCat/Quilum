use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use applock::{BlockingSession, app_list::AppInfo, start_polling};
use chrono::{DateTime, Utc};
use tauri::State;
use tauri_plugin_notification::NotificationExt;
use tokio::{sync::Mutex, task::JoinHandle};

use crate::db::Storage;

pub struct BlockingState {
    pub session: BlockingSession,
    pub polling_handle: Option<JoinHandle<()>>,
    pub stop_flag: Arc<std::sync::atomic::AtomicBool>,
    _storage: Storage,
    _app_handle: tauri::AppHandle,
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

async fn end_focus_session_internal(
    app_handle: tauri::AppHandle,
) -> tauri_plugin_notification::Result<()> {
    app_handle
        .notification()
        .builder()
        .title("Период концентрации")
        .body("Период концентрации окончен. Приложения разблокированы.")
        .show()
}

fn build_on_kill(
    app_handle: tauri::AppHandle,
    end_time: DateTime<Utc>,
) -> Option<Arc<dyn Fn(usize) + Send + Sync>> {
    let last_notify_at = Arc::new(AtomicU64::new(0));
    let last_notify_clone = last_notify_at.clone();

    Some(Arc::new(move |killed: usize| {
        if killed == 0 {
            return;
        }

        let now = Utc::now().timestamp() as u64;
        let prev = last_notify_clone.load(Ordering::SeqCst);
        if now.saturating_sub(prev) < 5 {
            return;
        }
        last_notify_clone.store(now, Ordering::SeqCst);

        let remaining = ((end_time - Utc::now()).num_seconds().max(0) as f64 / 60.0).ceil() as i64;
        let _ = app_handle
            .notification()
            .builder()
            .title("Сосредоточьтесь на работе!")
            .body(format!(
                "Активен период концентрации. Осталось {} мин.",
                remaining
            ))
            .show();
    }))
}

#[tauri::command]
pub async fn start_focus_session(
    storage: State<'_, Storage>,
    app_handle: tauri::AppHandle,
    end_time: i64,
) -> Result<(), String> {
    let end_time = DateTime::from_timestamp(end_time, 0).ok_or("Invalid end time")?;

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

    let on_kill = build_on_kill(app_handle.clone(), end_time);
    let handle = start_polling(blocked_set, poll_interval, stop_flag.clone(), on_kill);
    let end_time_for_task = end_time;

    *guard = Some(BlockingState {
        session,
        polling_handle: Some(handle),
        stop_flag,
        _storage: (*storage).clone(),
        _app_handle: app_handle.clone(),
    });

    drop(guard);

    tauri::async_runtime::spawn(async move {
        let now = Utc::now();
        let sleep_duration = (end_time_for_task - now).to_std().unwrap_or_default();
        tokio::time::sleep(sleep_duration).await;
        let _ = end_focus_session_internal(app_handle).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn end_focus_session(app_handle: tauri::AppHandle) -> Result<(), String> {
    end_focus_session_internal(app_handle)
        .await
        .map_err(|e| e.to_string())
}

pub fn check_and_restore_session(storage: Storage, app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state = blocking_state();
        let mut guard = state.lock().await;
        stop_blocking(&mut guard).await;
        drop(guard);

        let Ok(Some(task)) = storage.get_next_scheduled_task().await else {
            return;
        };

        let Ok(blocked) = storage.get_blocked_apps().await else {
            return;
        };

        let blocked_info = blocked_apps_to_info(blocked.clone());
        let now = Utc::now();
        let scheduled_for = task
            .scheduled_for()
            .expect("The task is expected to be scheduled at this point");
        let end_timestamp = scheduled_for + task.estimated_duration;
        let task_name = task.name().to_string();
        let task_duration = task.estimated_duration;

        if scheduled_for <= now && now <= end_timestamp {
            let end = end_timestamp;
            start_blocking(
                blocked_info,
                end,
                storage.clone(),
                app_handle,
                task_name,
                task_duration,
            );
        } else if scheduled_for > now {
            let start_time = scheduled_for;

            let blocked_info_clone = blocked_apps_to_info(blocked);
            let storage_clone = storage.clone();
            let app_handle_clone = app_handle.clone();
            let task_name_clone = task_name;
            let task_duration_clone = task_duration;

            tauri::async_runtime::spawn(async move {
                let sleep_duration = (start_time - Utc::now()).to_std().unwrap_or_default();
                tokio::time::sleep(sleep_duration).await;
                start_blocking(
                    blocked_info_clone,
                    end_timestamp,
                    storage_clone,
                    app_handle_clone,
                    task_name_clone,
                    task_duration_clone,
                );
            });
        }
    });
}

fn start_blocking(
    apps: Vec<AppInfo>,
    end_time: DateTime<Utc>,
    storage: Storage,
    app_handle: tauri::AppHandle,
    task_name: String,
    task_duration: Duration,
) {
    let duration_minutes = task_duration.as_secs() / 60;
    let notification_body = format!(
        "Начался период концентрации: \"{}\" ({} мин.). Отвлекающие приложения заблокированы!",
        task_name, duration_minutes
    );
    let _ = app_handle
        .notification()
        .builder()
        .title("Период концентрации")
        .body(&notification_body)
        .show();

    tauri::async_runtime::spawn(async move {
        let state = blocking_state();
        let mut guard = state.lock().await;
        stop_blocking(&mut guard).await;

        let bs_session = BlockingSession::new();
        bs_session.start(apps, end_time);

        let blocked_set = bs_session.blocked_apps();
        let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let poll_interval = Duration::from_millis(300);

        let on_kill = build_on_kill(app_handle.clone(), end_time);
        let handle = start_polling(blocked_set, poll_interval, stop_flag.clone(), on_kill);
        let end_time_for_task = end_time;
        let app_handle_clone = app_handle.clone();

        *guard = Some(BlockingState {
            session: bs_session,
            polling_handle: Some(handle),
            stop_flag,
            _storage: storage,
            _app_handle: app_handle,
        });

        drop(guard);

        let _ = tauri::async_runtime::spawn(async move {
            let now = Utc::now();
            let sleep_duration = (end_time_for_task - now).to_std().unwrap_or_default();
            tokio::time::sleep(sleep_duration).await;
            let _ = end_focus_session_internal(app_handle_clone).await;
        });
    });
}
