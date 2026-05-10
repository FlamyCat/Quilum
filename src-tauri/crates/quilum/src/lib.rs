#![allow(dead_code)]

mod commands;
mod db;
mod model;
mod scheduler;

use crate::commands::session::check_and_restore_session;
use chrono::NaiveDate;
use quilum_db::{
    event::Event,
    slot::Slot,
    storage::{SlotWithTasks, Storage, TaskListWithTasks},
    task::Task,
    tasklist::TaskList,
};
use surrealdb::types::RecordId;
use tauri::{Manager, State};

#[tauri::command]
async fn today_timetable(
    storage: State<'_, Storage>,
    today: String,
) -> Result<(Vec<Event>, Vec<(Task, i64)>), String> {
    let today = NaiveDate::parse_from_str(&today, "%Y-%m-%d").map_err(|e| e.to_string())?;
    storage
        .get_today_timetable(today)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn week_timetable(
    storage: State<'_, Storage>,
    week_start: String,
) -> Result<(Vec<Event>, Vec<SlotWithTasks>), String> {
    let week_start =
        NaiveDate::parse_from_str(&week_start, "%Y-%m-%d").map_err(|e| e.to_string())?;
    storage
        .get_week_timetable(week_start)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_event(
    storage: State<'_, Storage>,
    name: String,
    description: String,
    starts_at: i64,
    ends_at: i64,
) -> Result<Event, String> {
    use chrono::NaiveDateTime;
    let starts_at = NaiveDateTime::from_timestamp(starts_at, 0);
    let ends_at = NaiveDateTime::from_timestamp(ends_at, 0);
    storage
        .create_event(name, description, starts_at, ends_at)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_event(
    storage: State<'_, Storage>,
    id_table: String,
    id_key: String,
) -> Result<Event, String> {
    let id = RecordId::new(id_table.as_str(), id_key.as_str());
    storage.read_event(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_event(storage: State<'_, Storage>, event: Event) -> Result<(), String> {
    storage.update_event(event).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_event(
    storage: State<'_, Storage>,
    id_table: String,
    id_key: String,
) -> Result<(), String> {
    let id = RecordId::new(id_table.as_str(), id_key.as_str());
    storage.delete_event(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_slot(
    storage: State<'_, Storage>,
    starts_at: i64,
    ends_at: i64,
) -> Result<Slot, String> {
    use chrono::NaiveDateTime;
    let starts_at = NaiveDateTime::from_timestamp(starts_at, 0);
    let ends_at = NaiveDateTime::from_timestamp(ends_at, 0);
    storage
        .create_slot(starts_at, ends_at)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_slot(
    storage: State<'_, Storage>,
    id_table: String,
    id_key: String,
) -> Result<Slot, String> {
    let id = RecordId::new(id_table.as_str(), id_key.as_str());
    storage.read_slot(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_slot(storage: State<'_, Storage>, slot: Slot) -> Result<(), String> {
    storage.update_slot(slot).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_slot(
    storage: State<'_, Storage>,
    id_table: String,
    id_key: String,
) -> Result<(), String> {
    let id = RecordId::new(id_table.as_str(), id_key.as_str());
    storage.delete_slot(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_task(
    storage: State<'_, Storage>,
    name: String,
    description: String,
    priority: String,
    estimated_duration: i64,
    deadline: i64,
) -> Result<Task, String> {
    use chrono::NaiveDateTime;
    use quilum_db::task::Priority;
    let priority = match priority.as_str() {
        "Low" => Priority::Low,
        "Medium" => Priority::Medium,
        "High" => Priority::High,
        _ => return Err("Invalid priority".to_string()),
    };
    let deadline = NaiveDateTime::from_timestamp(deadline, 0);
    let estimated_duration = chrono::TimeDelta::seconds(estimated_duration);
    storage
        .create_task(name, description, priority, estimated_duration, deadline)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_task(
    storage: State<'_, Storage>,
    id_table: String,
    id_key: String,
) -> Result<Task, String> {
    let id = RecordId::new(id_table.as_str(), id_key.as_str());
    storage.read_task(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_task(
    storage: State<'_, Storage>,
    app_handle: tauri::AppHandle,
    task: Task,
) -> Result<(), String> {
    let result = storage.update_task(task).await.map_err(|e| e.to_string());
    check_and_restore_session(storage.inner().clone(), app_handle.clone());
    result
}

#[tauri::command]
async fn delete_task(
    storage: State<'_, Storage>,
    id_table: String,
    id_key: String,
) -> Result<(), String> {
    let id = RecordId::new(id_table.as_str(), id_key.as_str());
    storage.delete_task(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn relate_task_to_slot(
    storage: State<'_, Storage>,
    slot_id_table: String,
    slot_id_key: String,
    task_id_table: String,
    task_id_key: String,
    scheduled_for: i64,
) -> Result<(), String> {
    use chrono::NaiveDateTime;
    let slot_id = RecordId::new(slot_id_table.as_str(), slot_id_key.as_str());
    let task_id = RecordId::new(task_id_table.as_str(), task_id_key.as_str());
    let scheduled_for = NaiveDateTime::from_timestamp(scheduled_for, 0);
    storage
        .relate_task_to_slot(&slot_id, &task_id, scheduled_for)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_all_task_lists(storage: State<'_, Storage>) -> Result<Vec<TaskListWithTasks>, String> {
    storage
        .get_all_task_lists_with_tasks()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_task_list(storage: State<'_, Storage>, title: String) -> Result<TaskList, String> {
    storage
        .create_task_list(title)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_task_list(
    storage: State<'_, Storage>,
    app_handle: tauri::AppHandle,
    task_list: TaskList,
) -> Result<(), String> {
    let result = storage
        .update_task_list(task_list)
        .await
        .map_err(|e| e.to_string());
    check_and_restore_session(storage.inner().clone(), app_handle.clone());
    result
}

#[tauri::command]
async fn delete_task_list(
    storage: State<'_, Storage>,
    id_table: String,
    id_key: String,
) -> Result<(), String> {
    let id = RecordId::new(id_table.as_str(), id_key.as_str());
    storage
        .delete_tasks_in_list(&id)
        .await
        .map_err(|e| e.to_string())?;
    storage
        .delete_task_list(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn relate_task_to_list(
    storage: State<'_, Storage>,
    task_id_table: String,
    task_id_key: String,
    list_id_table: String,
    list_id_key: String,
) -> Result<(), String> {
    eprintln!("Relating task to list...");
    let task_id = RecordId::new(task_id_table.as_str(), task_id_key.as_str());
    let list_id = RecordId::new(list_id_table.as_str(), list_id_key.as_str());
    storage
        .relate_task_to_list(&task_id, &list_id)
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct SchedulerResult {
    scheduled: usize,
    discarded: Vec<String>,
}

#[tauri::command]
async fn run_scheduler(
    storage: State<'_, Storage>,
    app_handle: tauri::AppHandle,
) -> Result<SchedulerResult, String> {
    use crate::scheduler::Scheduler;
    use chrono::Utc;
    use surrealdb::types::RecordIdKey;

    let tasks = storage
        .get_uncompleted_tasks()
        .await
        .map_err(|e| e.to_string())?;

    if tasks.is_empty() {
        return Ok(SchedulerResult {
            scheduled: 0,
            discarded: vec![],
        });
    }

    let slots = storage
        .get_future_slots()
        .await
        .map_err(|e| e.to_string())?;

    if slots.is_empty() {
        return Ok(SchedulerResult {
            scheduled: 0,
            discarded: tasks
                .iter()
                .map(|t| {
                    let key = match &t.id().key {
                        RecordIdKey::String(s) => s.clone(),
                        RecordIdKey::Number(n) => n.to_string(),
                        RecordIdKey::Uuid(u) => u.to_string(),
                        RecordIdKey::Array(a) => format!("{:?}", a),
                        RecordIdKey::Object(o) => format!("{:?}", o),
                        RecordIdKey::Range(r) => format!("{:?}", r),
                    };
                    format!("{}:{}", t.id().table, key)
                })
                .collect(),
        });
    }

    let task_ids: Vec<RecordId> = tasks.iter().map(|t| t.id().clone()).collect();
    storage
        .delete_task_slot_relations(&task_ids)
        .await
        .map_err(|e| e.to_string())?;

    let now = Utc::now().naive_utc();

    let scheduler = Scheduler::new(&tasks, &slots, now, &storage);
    let plan = scheduler
        .schedule_and_commit()
        .await
        .map_err(|e| e.to_string())?;

    check_and_restore_session(storage.inner().clone(), app_handle.clone());

    let scheduled_count = plan.tasks().len();
    let discarded_ids: Vec<String> = plan
        .discarded_tasks()
        .iter()
        .map(|id| {
            let key = match &id.key {
                RecordIdKey::String(s) => s.clone(),
                RecordIdKey::Number(n) => n.to_string(),
                RecordIdKey::Uuid(u) => u.to_string(),
                RecordIdKey::Array(a) => format!("{:?}", a),
                RecordIdKey::Object(o) => format!("{:?}", o),
                RecordIdKey::Range(r) => format!("{:?}", r),
            };
            format!("{}:{}", id.table, key)
        })
        .collect();

    Ok(SchedulerResult {
        scheduled: scheduled_count,
        discarded: discarded_ids,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            today_timetable,
            week_timetable,
            create_event,
            read_event,
            update_event,
            delete_event,
            create_slot,
            read_slot,
            update_slot,
            delete_slot,
            create_task,
            read_task,
            update_task,
            delete_task,
            relate_task_to_slot,
            get_all_task_lists,
            create_task_list,
            update_task_list,
            delete_task_list,
            relate_task_to_list,
            run_scheduler,
            commands::app_blocking::get_installed_apps,
            commands::app_blocking::get_blocked_apps,
            commands::app_blocking::update_blocked_apps,
            commands::app_blocking::is_blocking_active,
            commands::session::start_focus_session,
            commands::session::end_focus_session,
        ])
        .setup(|app| {
            let storage = tauri::async_runtime::block_on(Storage::new_rocksdb())
                .expect("Failed to initialize database");
            app.manage(storage.clone());

            commands::session::check_and_restore_session(storage, app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
