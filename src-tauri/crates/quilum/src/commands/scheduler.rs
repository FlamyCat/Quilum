use std::collections::HashSet;
use tauri::State;
use quilum_db::Storage;
use crate::SchedulerResult;

#[tauri::command]
pub async fn run_scheduler(
    storage: State<'_, Storage>,
    app_handle: tauri::AppHandle,
) -> Result<SchedulerResult, String> {
    use crate::scheduler::Scheduler;
    use chrono::Utc;
    use surrealdb::types::{RecordId, RecordIdKey};
    use crate::commands::session::check_and_restore_session;

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

    let task_ids: HashSet<RecordId> = tasks.iter().map(|t| t.id().clone()).collect();
    storage
        .unschedule_tasks(task_ids)
        .await
        .map_err(|e| e.to_string())?;

    let now = Utc::now();

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