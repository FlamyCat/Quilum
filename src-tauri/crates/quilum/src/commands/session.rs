use applock::{AppBlocker, app_list::AppInfo, blocker::BlockReason};
use chrono::{DateTime, Utc};
use quilum_db::model::task::Task;
use tauri::State;

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

#[tauri::command]
pub async fn start_focus_session(
    blocker: State<'_, AppBlocker>,
    storage: State<'_, Storage>,
    end_time: i64,
) -> Result<(), String> {
    let end_time = DateTime::from_timestamp(end_time, 0).ok_or("Invalid end time")?;

    let blocked = storage
        .get_blocked_apps()
        .await
        .map_err(|e| e.to_string())?;
    let blocked_info = blocked_apps_to_info(blocked);

    let task = storage
        .get_next_scheduled_task()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("No scheduled task for the focus session")?;

    blocker
        .start(blocked_info, BlockReason::Task(task))
        .map_err(|e| e.to_string())?;

    let blocker_for_task = blocker.inner().clone();
    tauri::async_runtime::spawn(async move {
        let now = Utc::now();
        let sleep_duration = (end_time - now).to_std().unwrap_or_default();
        tokio::time::sleep(sleep_duration).await;
        let _ = blocker_for_task.stop();
    });

    Ok(())
}

#[tauri::command]
pub async fn end_focus_session(blocker: State<'_, AppBlocker>) -> Result<(), String> {
    blocker.stop().map_err(|e| e.to_string())
}

pub fn check_and_restore_session(blocker: AppBlocker, storage: Storage) {
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

        if scheduled_for <= now && now <= end {
            start_blocking(blocked_info, blocker, task);
        } else if scheduled_for > now {
            let _ = blocker.stop();
            let blocked_info_clone = blocked_info;
            let blocker_clone = blocker.clone();
            let start_time = scheduled_for;

            tauri::async_runtime::spawn(async move {
                let sleep_duration = (start_time - Utc::now()).to_std().unwrap_or_default();
                tokio::time::sleep(sleep_duration).await;
                start_blocking(blocked_info_clone, blocker_clone, task);
            });
        } else {
            let _ = blocker.stop();
        }
    });
}

fn start_blocking(apps: Vec<AppInfo>, blocker: AppBlocker, task: Task) {
    let end_time = task
        .scheduled_for()
        .expect("The task is expected to be scheduled at this point")
        + task.estimated_duration;

    if let Err(err) = blocker.start(apps, BlockReason::Task(task)) {
        eprintln!("Failed to start blocking: {err}");
        return;
    }

    let blocker_for_task = blocker.clone();
    tauri::async_runtime::spawn(async move {
        let now = Utc::now();
        let sleep_duration = (end_time - now).to_std().unwrap_or_default();
        tokio::time::sleep(sleep_duration).await;
        let _ = blocker_for_task.stop();
    });
}
