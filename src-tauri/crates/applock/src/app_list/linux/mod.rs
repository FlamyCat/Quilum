use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use regex::Regex;

use desktop_edit::Desktop;

use crate::{app_list::types::AppInfo, model::AppIdentifier};

pub fn get_installed_apps() -> Vec<AppInfo> {
    let mut apps = Vec::new();
    let app_dirs = get_application_dirs();

    for dir in app_dirs {
        if Path::new(&dir).exists() {
            scan_directory(&dir, &mut apps);
        }
    }

    // Deduplicate by identifier (keep first occurrence)
    let mut seen = HashSet::new();
    apps.retain(|app| seen.insert(app.identifier.clone()));

    apps
}

fn get_application_dirs() -> Vec<String> {
    let mut dirs = Vec::new();

    // $XDG_DATA_HOME (default: ~/.local/share)
    let xdg_data_home = env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
        dirs::home_dir()
            .map(|h| h.join(".local/share"))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    });
    dirs.push(format!("{}/applications", xdg_data_home));

    // $XDG_DATA_DIRS (default: /usr/local/share:/usr/share)
    let xdg_data_dirs =
        env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());

    for dir in xdg_data_dirs.split(':') {
        if !dir.is_empty() {
            dirs.push(format!("{}/applications", dir));
        }
    }

    // NixOS path
    dirs.push("/run/current-system/sw/share/applications".to_string());

    dirs
}

fn scan_directory(dir: &str, apps: &mut Vec<AppInfo>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path.extension().is_some_and(|ext| ext == "desktop")
            && let Some(app_info) = parse_desktop_file(&path)
        {
            apps.push(app_info);
        }
    }
}

fn parse_desktop_file(path: &Path) -> Option<AppInfo> {
    let content = fs::read_to_string(path).ok()?;

    let desktop: Desktop = FromStr::from_str(&content).ok()?;

    // Get the "Desktop Entry" group
    let group = desktop.get_group("Desktop Entry")?;

    // Filter by Type
    let type_ = group.get("Type")?;
    if type_ != "Application" {
        return None;
    }

    // Skip if NoDisplay is true
    if let Some(no_display) = group.get("NoDisplay")
        && no_display == "true"
    {
        return None;
    }

    // Skip if Hidden is true
    if let Some(hidden) = group.get("Hidden")
        && hidden == "true"
    {
        return None;
    }

    // Get Name (try localized version first, fallback to non-localized)
    let name = group.get_locale("Name", "").or_else(|| group.get("Name"))?;

    // Get Exec command
    let exec = group.get("Exec")?;

    let binary_path = extract_binary_from_exec(&exec)?;

    Some(AppInfo::new(AppIdentifier::Path(binary_path), name))
}

fn extract_binary_from_exec(exec_cmd: &str) -> Option<PathBuf> {
    let tokens = tokenize_exec(exec_cmd)?;
    if tokens.is_empty() {
        return None;
    }

    let first_token = tokens.get(0)?;
    let exe_name = Path::new(first_token)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    if exe_name == "env" {
        if tokens.len() >= 2 {
            let second_token = tokens.get(1)?;
            if second_token.contains('=')
                && let Some(value) = second_token.split('=').nth(1)
                && !value.is_empty()
            {
                let resolved = try_to_resolve_path(value);
                if resolved.is_some() {
                    return resolved;
                }
            }
        }
        return None;
    }

    match exe_name {
        "flatpak" => extract_flatpak_binary(&tokens),
        "java" | "javaw" => extract_java_binary(&tokens),
        "python" | "python2" | "python3" => extract_python_binary(&tokens),
        "node" => extract_script_binary(&tokens, &["node"]),
        "perl" => extract_script_binary(&tokens, &["perl"]),
        "ruby" => extract_script_binary(&tokens, &["ruby"]),
        "sh" | "bash" | "zsh" => extract_shell_binary(&tokens),
        _ => {
            if first_token.contains('/') {
                Some(PathBuf::from(first_token))
            } else {
                try_to_resolve_path(first_token)
            }
        }
    }
}

fn tokenize_exec(exec_cmd: &str) -> Option<Vec<String>> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    let mut chars = exec_cmd.chars().peekable();

    while let Some(c) = chars.next() {
        if in_double_quote {
            if c == '"' {
                in_double_quote = false;
            } else if c == '\\' {
                if let Some(&next) = chars.peek() {
                    chars.next();
                    match next {
                        '"' | '\\' | '$' => current.push(next),
                        _ => {
                            current.push('\\');
                            current.push(next);
                        }
                    }
                }
            } else {
                current.push(c);
            }
        } else if in_single_quote {
            if c == '\'' {
                in_single_quote = false;
            } else {
                current.push(c);
            }
        } else {
            match c {
                '"' => in_double_quote = true,
                '\'' => in_single_quote = true,
                ' ' | '\t' => {
                    if !current.is_empty() {
                        result.push(std::mem::take(&mut current));
                    }
                }
                _ => current.push(c),
            }
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    if result.is_empty() {
        return None;
    }

    // Бранные слова здесь ни при чем. Это все допустимые коды подстановки в
    // значении ключа Exec файла Desktop (в том числе устаревшие).
    // Подробнее - см. спецификацию
    // (https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html)
    // Дата обращения - 7 мая 2026.
    let field_code_regex = Regex::new(r"%[fFuUcCkVdDnNvVm]").ok()?;
    for token in &mut result {
        *token = field_code_regex.replace_all(token, "").to_string();
        *token = token.replace("%%", "%");
    }

    result.retain(|t| !t.is_empty());

    (!result.is_empty()).then_some(result)
}

fn extract_flatpak_binary(tokens: &[String]) -> Option<PathBuf> {
    if tokens.len() < 2 || tokens[1] != "run" {
        return None;
    }

    let mut i = 2;
    let mut explicit_command: Option<&str> = None;

    while i < tokens.len() {
        let token = &tokens[i];
        if token.starts_with('-') {
            if token == "--command" && i + 1 < tokens.len() {
                explicit_command = Some(&tokens[i + 1]);
                i += 2;
            } else if token.starts_with("--command=") {
                explicit_command = token.strip_prefix("--command=");
                i += 1;
            } else {
                i += 1;
            }
        } else {
            break;
        }
    }

    let app_id = tokens.get(i)?;

    let command = match explicit_command {
        Some(cmd) => cmd.to_string(),
        None => {
            let metadata_path = format!("/var/lib/flatpak/app/{}/current/active/metadata", app_id);
            let metadata = fs::read_to_string(&metadata_path).ok()?;

            let cmd_regex = Regex::new(r"(?m)^command=(.*?)$").ok()?;
            let caps = cmd_regex.captures(&metadata)?;
            caps.get(1)?.as_str().to_string()
        }
    };

    let binary_path = format!(
        "/var/lib/flatpak/app/{}/current/active/files/bin/{}",
        app_id, command
    );

    let path = PathBuf::from(binary_path);
    path.exists().then_some(path)
}

fn extract_java_binary(tokens: &[String]) -> Option<PathBuf> {
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i] == "-jar" && i + 1 < tokens.len() {
            let jar_path = &tokens[i + 1];
            if jar_path.starts_with('/') {
                return Some(PathBuf::from(jar_path));
            } else {
                return try_to_resolve_path(jar_path);
            }
        }
        i += 1;
    }

    None
}

fn extract_python_binary(tokens: &[String]) -> Option<PathBuf> {
    let mut i = 1;
    while i < tokens.len() {
        let token = &tokens[i];
        if token.starts_with('-') {
            if token == "-m" || token == "-c" {
                return None;
            }
            i += 1;
        } else {
            let script_path = PathBuf::from(token);

            return script_path
                .is_absolute()
                .then_some(script_path)
                .or_else(|| try_to_resolve_path(token));
        }
    }
    None
}

fn extract_script_binary(tokens: &[String], _interpreter: &[&str]) -> Option<PathBuf> {
    if tokens.len() < 2 {
        return None;
    }

    let script_token = &tokens[1];
    let canonical_path = super::try_to_canonicalize(script_token);
    canonical_path.or(try_to_resolve_path(script_token))
}

fn extract_shell_binary(tokens: &[String]) -> Option<PathBuf> {
    if tokens.len() < 2 {
        return None;
    }

    let second_token = &tokens[1];
    if second_token == "-c" {
        return None;
    }

    if second_token.starts_with('/') {
        Some(PathBuf::from(second_token))
    } else {
        try_to_resolve_path(second_token)
    }
}

fn try_to_resolve_path(cmd: &str) -> Option<PathBuf> {
    let canonical_path = super::try_to_canonicalize(cmd);
    if canonical_path.is_some() {
        return canonical_path;
    }

    let path_env = env::var_os("PATH")?;
    for dir in path_env.to_str()?.split(':') {
        let candidate = PathBuf::from(dir).join(cmd);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}
