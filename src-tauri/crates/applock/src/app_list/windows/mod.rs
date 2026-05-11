use crate::app_list::AppInfo;
use std::path::PathBuf;

mod app_list;

pub use app_list::get_installed_apps;

fn get_start_menu_paths() -> Vec<PathBuf> {
    use windows::Win32::UI::Shell::{
        FOLDERID_CommonStartMenu, FOLDERID_StartMenu, SHGetKnownFolderPath,
    };
    use windows::core::GUID;

    let mut paths = Vec::new();

    let folder_ids = [
        FOLDERID_StartMenu,
        FOLDERID_CommonStartMenu,
    ];

    for &folder_id in &folder_ids {
        unsafe {
            let folder_path_result = SHGetKnownFolderPath(
                &folder_id,
                windows::Win32::UI::Shell::KNOWN_FOLDER_FLAG(0),
                None,
            );

            if let Ok(path_ptr) = folder_path_result {
                let wide_str = path_ptr.as_wide();
                if !wide_str.is_empty() {
                    let path = String::from_utf16_lossy(wide_str);
                    let mut path_buf = PathBuf::from(path);
                    path_buf.push("Programs");
                    paths.push(path_buf);
                }
                windows::Win32::System::Com::CoTaskMemFree(Some(
                    path_ptr.as_ptr() as *const std::ffi::c_void
                ));
            }
        }
    }

    paths
}

fn scan_for_lnk_files(dir: &PathBuf, max_depth: usize) -> Vec<PathBuf> {
    let mut results = Vec::new();

    if max_depth == 0 {
        return results;
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                results.extend(scan_for_lnk_files(&path, max_depth - 1));
            } else if path.extension().is_some_and(|ext| ext == "lnk") {
                results.push(path);
            }
        }
    }

    results
}
