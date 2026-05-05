#![allow(dead_code)]

mod db;
mod model;
mod scheduler;

use chrono::NaiveDate;
use quilum_db::{
    event::Event,
    slot::Slot,
    storage::{SlotWithTasks, Storage},
    task::Task,
};
use surrealdb::types::RecordId;
use tauri::{Manager, State};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

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
async fn update_task(storage: State<'_, Storage>, task: Task) -> Result<(), String> {
    storage.update_task(task).await.map_err(|e| e.to_string())
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
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
        ])
        .setup(|app| {
            let storage = tauri::async_runtime::block_on(Storage::new_rocksdb())
                .expect("Failed to initialize database");
            app.manage(storage);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
