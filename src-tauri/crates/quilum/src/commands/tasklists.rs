use applock::AppBlocker;
use quilum_db::{Storage, TaskListWithTasks, tasklist::TaskList};
use surrealdb::types::RecordId;
use tauri::State;

use crate::commands::session::check_and_restore_session;

#[tauri::command]
pub async fn create_task_list(
    storage: State<'_, Storage>,
    title: String,
) -> Result<TaskList, String> {
    storage
        .create_task_list(title)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_task_list(
    blocker: State<'_, AppBlocker>,
    storage: State<'_, Storage>,
    app_handle: tauri::AppHandle,
    task_list: TaskList,
) -> Result<(), String> {
    let result = storage
        .update_task_list(task_list)
        .await
        .map_err(|e| e.to_string());
    check_and_restore_session(
        blocker.inner().clone(),
        storage.inner().clone(),
        app_handle.clone(),
    );
    result
}

#[tauri::command]
pub async fn delete_task_list(
    storage: State<'_, Storage>,
    id_table: String,
    id_key: String,
) -> Result<(), String> {
    let id = RecordId::new(id_table.as_str(), id_key.as_str());
    storage
        .delete_task_list(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_task_lists(
    storage: State<'_, Storage>,
) -> Result<Vec<TaskListWithTasks>, String> {
    storage
        .get_all_task_lists_with_tasks()
        .await
        .map_err(|e| e.to_string())
}
