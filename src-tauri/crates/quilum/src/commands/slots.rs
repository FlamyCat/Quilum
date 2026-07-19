use quilum_db::{storage::model::slot::Slot, Storage};
use surrealdb::types::RecordId;
use tauri::State;

#[tauri::command]
pub async fn create_slot(
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
pub async fn read_slot(
    storage: State<'_, Storage>,
    id_table: String,
    id_key: String,
) -> Result<Slot, String> {
    let id = RecordId::new(id_table.as_str(), id_key.as_str());
    storage.read_slot(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_slot(storage: State<'_, Storage>, slot: Slot) -> Result<(), String> {
    storage.update_slot(slot).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_slot(
    storage: State<'_, Storage>,
    id_table: String,
    id_key: String,
) -> Result<(), String> {
    let id = RecordId::new(id_table.as_str(), id_key.as_str());
    storage.delete_slot(&id).await.map_err(|e| e.to_string())
}
