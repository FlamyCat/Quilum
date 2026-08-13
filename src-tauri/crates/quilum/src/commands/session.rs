use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use applock::{AppBlocker, app_list::AppInfo};
use chrono::{DateTime, Utc};
use tauri::State;
use tauri_plugin_notification::NotificationExt;

use crate::db::Storage;

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
    blocker: State<'_, AppBlocker>,
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
    let on_kill = build_on_kill(app_handle.clone(), end_time);

    blocker
        .start(blocked_info, on_kill)
        .map_err(|e| e.to_string())?;

    let blocker_for_task = blocker.inner().clone();
    let end_time_for_task = end_time;
    tauri::async_runtime::spawn(async move {
        let now = Utc::now();
        let sleep_duration = (end_time_for_task - now).to_std().unwrap_or_default();
        tokio::time::sleep(sleep_duration).await;
        let _ = blocker_for_task.stop();
        let _ = end_focus_session_internal(app_handle).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn end_focus_session(
    blocker: State<'_, AppBlocker>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    blocker.stop().map_err(|e| e.to_string())?;
    end_focus_session_internal(app_handle)
        .await
        .map_err(|e| e.to_string())
}

pub fn check_and_restore_session(
    blocker: AppBlocker,
    storage: Storage,
    app_handle: tauri::AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        let Ok(Some(task)) = storage.get_next_scheduled_task().await else {
            let _ = blocker.stop();
            return;
        };
        let Ok(blocked) = storage.get_blocked_apps().await else {
            return;
        };
        let blocked_info = blocked_apps_to_info(blocked);

        let now = Utc::now();
        let scheduled_for = task
            .scheduled_for()
            .expect("The task is expected to be scheduled at this point");
        let end = scheduled_for + task.estimated_duration;
        let task_name = task.name().to_string();
        let task_duration = task.estimated_duration;

        if scheduled_for <= now && now <= end {
            start_blocking(
                blocked_info,
                end,
                blocker,
                app_handle,
                task_name,
                task_duration,
            );
        } else if scheduled_for > now {
            let _ = blocker.stop();
            let blocked_info_clone = blocked_info;
            let blocker_clone = blocker.clone();
            let start_time = scheduled_for;

            tauri::async_runtime::spawn(async move {
                let sleep_duration = (start_time - Utc::now()).to_std().unwrap_or_default();
                tokio::time::sleep(sleep_duration).await;
                start_blocking(
                    blocked_info_clone,
                    end,
                    blocker_clone,
                    app_handle,
                    task_name,
                    task_duration,
                );
            });
        } else {
            let _ = blocker.stop();
        }
    });
}

fn start_blocking(
    apps: Vec<AppInfo>,
    end_time: DateTime<Utc>,
    blocker: AppBlocker,
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

    let on_kill = build_on_kill(app_handle.clone(), end_time);
    if let Err(err) = blocker.start(apps, on_kill) {
        eprintln!("Failed to start blocking: {err}");
        return;
    }

    let blocker_for_task = blocker.clone();
    let end_time_for_task = end_time;
    tauri::async_runtime::spawn(async move {
        let now = Utc::now();
        let sleep_duration = (end_time_for_task - now).to_std().unwrap_or_default();
        tokio::time::sleep(sleep_duration).await;
        let _ = blocker_for_task.stop();
        let _ = end_focus_session_internal(app_handle).await;
    });
}
