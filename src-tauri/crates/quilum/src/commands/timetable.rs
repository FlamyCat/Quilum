use chrono::NaiveDate;
use quilum_db::{
    storage::model::{
        event::Event,
        task::{Scheduled, Task},
    }, SlotWithTasks,
    Storage,
};
use tauri::State;

#[tauri::command]
pub async fn today_timetable(
    storage: State<'_, Storage>,
    today: String,
) -> Result<(Vec<Event>, Vec<Task<Scheduled>>), String> {
    let today = NaiveDate::parse_from_str(&today, "%Y-%m-%d").map_err(|e| e.to_string())?;
    storage
        .get_today_timetable(today)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn week_timetable(
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
