use tauri::State;
use quilum_db::{
    event::Event,
    Storage
};
use surrealdb::types::RecordId;

#[tauri::command]
pub async fn create_event(
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
pub async fn read_event(
    storage: State<'_, Storage>,
    id_table: String,
    id_key: String,
) -> Result<Event, String> {
    let id = RecordId::new(id_table.as_str(), id_key.as_str());
    storage.read_event(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_event(storage: State<'_, Storage>, event: Event) -> Result<(), String> {
    storage.update_event(event).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_event(
    storage: State<'_, Storage>,
    id_table: String,
    id_key: String,
) -> Result<(), String> {
    let id = RecordId::new(id_table.as_str(), id_key.as_str());
    storage.delete_event(&id).await.map_err(|e| e.to_string())
}