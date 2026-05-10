use crate::app_list::AppInfo;
use crate::model::AppIdentifier;
use std::path::PathBuf;

pub fn get_installed_apps() -> Vec<AppInfo> {
    let mut apps = Vec::new();
    let start_menu_paths = super::get_start_menu_paths();

    for start_menu_path in start_menu_paths {
        let lnk_files = super::scan_for_lnk_files(&start_menu_path, 3);

        for lnk_path in lnk_files {
            if let Some(app_info) = parse_lnk_shortcut(&lnk_path) {
                apps.push(app_info);
            }
        }
    }

    apps
}

fn parse_lnk_shortcut(lnk_path: &PathBuf) -> Option<AppInfo> {
    let lnk_content = match std::fs::read(lnk_path) {
        Ok(content) => content,
        Err(_) => return None,
    };

    let shell_link = match lnk::ShellLink::open(&lnk_content) {
        Ok(link) => link,
        Err(_) => return None,
    };

    let target_path = match shell_link.target_path() {
        Some(path) => PathBuf::from(path),
        None => return None,
    };

    if target_path.extension().is_none_or(|ext| ext != "exe") {
        return None;
    }

    let display_name = lnk_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    Some(AppInfo::new(AppIdentifier::Path(target_path), display_name))
}
