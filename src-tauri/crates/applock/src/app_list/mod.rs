mod types;

#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

use std::path::PathBuf;

pub use types::AppInfo;

#[cfg(windows)]
pub use windows::get_installed_apps;
#[cfg(target_os = "linux")]
pub use linux::get_installed_apps;
#[cfg(target_os = "macos")]
pub use macos::get_installed_apps;

fn try_to_canonicalize(cmd: &str) -> Option<PathBuf> {
    PathBuf::from(cmd).canonicalize().ok()
}
