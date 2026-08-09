use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use winreg::{HKCU, HKLM, RegKey};

use crate::{app_list::AppInfo, model::AppIdentifier};

pub fn get_installed_apps() -> Vec<AppInfo> {
    collect_from_registry()
}

fn collect_from_registry() -> Vec<AppInfo> {
    let mut seen = HashSet::new();

    uninstall_roots()
        .into_iter()
        .flat_map(|root| {
            let names: Vec<_> = root.enum_keys().flatten().collect();
            names
                .into_iter()
                .filter_map(move |name| root.open_subkey(name).ok())
        })
        .filter_map(|entry| parse_or_discard_entry(&entry))
        .filter(|info| seen.insert(info.identifier.clone()))
        .collect()
}

fn uninstall_roots() -> Vec<RegKey> {
    [HKLM, HKCU]
        .into_iter()
        .flat_map(|root| {
            [
                r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
                r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
            ]
                .into_iter()
                .map(move |sub| root.open_subkey(sub))
        })
        .flatten()
        .collect()
}

fn parse_or_discard_entry(entry: &RegKey) -> Option<AppInfo> {
    if is_system_component(entry) {
        return None;
    }

    let display_name = read_display_name(entry)?;
    let executable = read_executable_path(entry)?;

    Some(AppInfo::new(AppIdentifier::Path(executable), display_name))
}

fn is_system_component(entry: &RegKey) -> bool {
    entry
        .get_value::<u32, _>("SystemComponent")
        .is_ok_and(|value| value == 1)
}

fn read_display_name(entry: &RegKey) -> Option<String> {
    entry
        .get_value::<String, _>("DisplayName")
        .ok()
        .filter(|name| !name.is_empty())
}

fn read_executable_path(entry: &RegKey) -> Option<PathBuf> {
    let icon = entry.get_value::<String, _>("DisplayIcon").ok()?;
    let path = Path::new(strip_icon_index(&icon));
    is_exe(path).then_some(path.to_path_buf())
}

fn strip_icon_index(icon: &str) -> &str {
    icon.split_once(',').map_or(icon, |(path, _)| path)
}

fn is_exe(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
}
