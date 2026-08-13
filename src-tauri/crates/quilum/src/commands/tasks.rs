use applock::AppBlocker;
use chrono::{DateTime, TimeDelta, Utc};
use quilum_db::{
    Storage,
    model::task::{Priority, Task},
};
use surrealdb::types::RecordId;
use tauri::State;

use crate::commands::session::check_and_restore_session;

#[tauri::command]
pub async fn create_task(
    storage: State<'_, Storage>,
    name: String,
    description: String,
    priority: String,
    estimated_duration: i64,
    deadline: DateTime<Utc>,
) -> Result<Task, String> {
    let priority = match priority.as_str() {
        "Low" => Priority::Low,
        "Medium" => Priority::Medium,
        "High" => Priority::High,
        _ => return Err("Invalid priority".to_string()),
    };
    let estimated_duration = TimeDelta::seconds(estimated_duration);
    storage
        .create_task(name, description, priority, estimated_duration, deadline)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_task(
    storage: State<'_, Storage>,
    id_table: String,
    id_key: String,
) -> Result<Task, String> {
    let id = RecordId::new(id_table.as_str(), id_key.as_str());
    storage.read_task(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_task(
    blocker: State<'_, AppBlocker>,
    storage: State<'_, Storage>,
    app_handle: tauri::AppHandle,
    task: Task,
) -> Result<(), String> {
    let result = storage.update_task(task).await.map_err(|e| e.to_string());
    check_and_restore_session(
        blocker.inner().clone(),
        storage.inner().clone(),
        app_handle.clone(),
    );
    result
}

#[tauri::command]
pub async fn delete_task(
    storage: State<'_, Storage>,
    id_table: String,
    id_key: String,
) -> Result<(), String> {
    let id = RecordId::new(id_table.as_str(), id_key.as_str());
    storage.delete_task(&id).await.map_err(|e| e.to_string())
}
