mod types;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

use std::path::PathBuf;

#[cfg(target_os = "linux")]
pub use linux::get_installed_apps;
#[cfg(target_os = "macos")]
pub use macos::get_installed_apps;
pub use types::AppInfo;
#[cfg(windows)]
pub use windows::get_installed_apps;

fn try_to_canonicalize(cmd: &str) -> Option<PathBuf> {
    PathBuf::from(cmd).canonicalize().ok()
}
