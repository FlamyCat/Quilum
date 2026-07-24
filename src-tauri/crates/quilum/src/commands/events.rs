use chrono::{DateTime, Utc};
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
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
) -> Result<Event, String> {
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