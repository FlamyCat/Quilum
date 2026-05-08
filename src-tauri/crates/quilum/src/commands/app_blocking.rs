use std::path::PathBuf;
use tauri::State;

use crate::db::Storage;
use applock::{
    app_list::{AppInfo, get_installed_apps as get_apps},
    model::AppIdentifier,
};

#[derive(serde::Deserialize)]
pub struct AppInfoRaw {
    pub identifier: String,
    pub display_name: String,
}

#[tauri::command]
pub async fn get_installed_apps() -> Result<Vec<AppInfo>, String> {
    Ok(get_apps())
}

#[tauri::command]
pub async fn get_blocked_apps(storage: State<'_, Storage>) -> Result<Vec<AppInfo>, String> {
    let blocked = storage
        .get_blocked_apps()
        .await
        .map_err(|e| e.to_string())?;

    let apps: Vec<AppInfo> = blocked
        .into_iter()
        .map(|app| AppInfo {
            identifier: app.app_identifier(),
            display_name: app.display_name,
        })
        .collect();

    Ok(apps)
}

#[tauri::command]
pub async fn update_blocked_apps(
    storage: State<'_, Storage>,
    apps: Vec<AppInfoRaw>,
) -> Result<(), String> {
    let is_active = storage
        .is_blocking_active()
        .await
        .map_err(|e| e.to_string())?;

    if is_active {
        return Err("Невозможно изменить список приложений во время фокус сессии".to_string());
    }

    storage
        .delete_all_blocked_apps()
        .await
        .map_err(|e| e.to_string())?;

    for app in apps {
        let app_identifier = if app.identifier.contains('/') || app.identifier.contains('\\') {
            AppIdentifier::Path(PathBuf::from(&app.identifier))
        } else {
            AppIdentifier::BundleId(app.identifier.clone())
        };

        storage
            .upsert_blocked_app(app_identifier, &app.display_name)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn is_blocking_active(storage: State<'_, Storage>) -> Result<bool, String> {
    storage
        .is_blocking_active()
        .await
        .map_err(|e| e.to_string())
}
