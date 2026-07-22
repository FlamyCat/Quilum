#![allow(dead_code)]

mod commands;
mod db;
mod model;
mod scheduler;

use quilum_db::storage::Storage;
use surrealdb::types::RecordId;
use tauri::{Manager, State};

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
        .schedule_task(&slot_id, &task_id, scheduled_for)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::timetable::today_timetable,
            commands::timetable::week_timetable,
            commands::events::create_event,
            commands::events::read_event,
            commands::events::update_event,
            commands::events::delete_event,
            commands::slots::create_slot,
            commands::slots::read_slot,
            commands::slots::update_slot,
            commands::slots::delete_slot,
            commands::tasks::create_task,
            commands::tasks::read_task,
            commands::tasks::update_task,
            commands::tasks::delete_task,
            relate_task_to_slot,
            commands::tasklists::get_all_task_lists,
            commands::tasklists::create_task_list,
            commands::tasklists::update_task_list,
            commands::tasklists::delete_task_list,
            relate_task_to_list,
            commands::scheduler::run_scheduler,
            commands::app_blocking::get_installed_apps,
            commands::app_blocking::get_blocked_apps,
            commands::app_blocking::update_blocked_apps,
            commands::app_blocking::is_blocking_active,
            commands::session::start_focus_session,
            commands::session::end_focus_session,
        ])
        .setup(|app| {
            let storage = tauri::async_runtime::block_on(Storage::new_surrealkv())
                .expect("Failed to initialize database");
            app.manage(storage.clone());

            commands::session::check_and_restore_session(storage, app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
