use std::path::PathBuf;

use lnk::{Encoding, encoding::UTF_16LE};
use windows_sys::Win32::Globalization;

use crate::{app_list::AppInfo, model::AppIdentifier};

pub fn get_installed_apps() -> Vec<AppInfo> {
    let mut apps = Vec::new();
    let start_menu_paths = super::get_start_menu_paths();

    let fallback_encoding = system_encoding().unwrap_or(&UTF_16LE);

    for start_menu_path in start_menu_paths {
        let lnk_files = super::scan_for_lnk_files(&start_menu_path, 3);

        for lnk_path in lnk_files {
            if let Some(app_info) = parse_lnk_shortcut(&lnk_path, fallback_encoding) {
                apps.push(app_info);
            }
        }
    }

    apps
}

fn parse_lnk_shortcut(lnk_path: &PathBuf, fallback_encoding: Encoding) -> Option<AppInfo> {
    let shell_link = match lnk::ShellLink::open(lnk_path, fallback_encoding) {
        Ok(link) => link,
        Err(_) => return None,
    };

    let target_path = match shell_link.link_target() {
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

fn system_encoding() -> Option<&'static encoding_rs::Encoding> {
    let acp = unsafe { Globalization::GetACP() };
    codepage::to_encoding(acp as u16)
}
