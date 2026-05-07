use std::fs;
use std::path::{Path, PathBuf};

use crate::app_list::types::AppInfo;
use crate::model::AppIdentifier;

#[cfg(target_os = "macos")]
pub fn get_installed_apps() -> Vec<AppInfo> {
    let mut apps = Vec::new();
    let app_dirs = get_application_dirs();

    for dir in app_dirs {
        if Path::new(&dir).exists() {
            scan_directory(&dir, &mut apps);
        }
    }

    apps
}

#[cfg(target_os = "macos")]
fn get_application_dirs() -> Vec<String> {
    let mut dirs = Vec::new();

    dirs.push("/Applications".to_string());
    dirs.push("/System/Applications".to_string());

    if let Some(home_dir) = dirs::home_dir() {
        dirs.push(home_dir.join("Applications").to_string_lossy().to_string());
    }

    dirs
}

#[cfg(target_os = "macos")]
fn scan_directory(dir: &str, apps: &mut Vec<AppInfo>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name.ends_with(".app") {
                if let Some(app_info) = parse_app_bundle(&path) {
                    apps.push(app_info);
                }
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn parse_app_bundle(app_path: &Path) -> Option<AppInfo> {
    let info_plist_path = app_path.join("Contents/Info.plist");

    if let Some(app_info) = parse_info_plist(&info_plist_path, app_path) {
        return Some(app_info);
    }

    let dir_name = app_path.file_name()?.to_string_lossy().to_string();
    let display_name = dir_name.strip_suffix(".app").unwrap_or(&dir_name).to_string();

    Some(AppInfo::new(
        AppIdentifier::BundleId(display_name.clone()),
        display_name,
    ))
}

#[cfg(target_os = "macos")]
fn parse_info_plist(info_plist_path: &Path, app_path: &Path) -> Option<AppInfo> {
    let plist_data = fs::read(info_plist_path).ok()?;
    let plist_value: plist::Value = plist::from_bytes(&plist_data).ok()?;

    let bundle_id = plist_value
        .as_dictionary()?
        .get("CFBundleIdentifier")?
        .as_string()?
        .to_string();

    let display_name = plist_value
        .as_dictionary()?
        .get("CFBundleDisplayName")
        .or_else(|| {
            plist_value
                .as_dictionary()
                .and_then(|d| d.get("CFBundleName"))
        })
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            let dir_name = app_path.file_name().unwrap().to_string_lossy().to_string();
            dir_name.strip_suffix(".app").unwrap_or(&dir_name).to_string()
        });

    Some(AppInfo::new(
        AppIdentifier::BundleId(bundle_id),
        display_name,
    ))
}
