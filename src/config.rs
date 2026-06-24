use std::{env, fs, path::PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AppEntry {
    pub app_name: String,
    pub process_names: Vec<String>,
    #[serde(default)]
    pub preset: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct StoredConfig {
    custom_apps: Vec<AppEntry>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PresetConfig {
    preset_apps: Vec<AppEntry>,
}

pub fn load_preset_apps() -> anyhow::Result<Vec<AppEntry>> {
    let content = fs::read_to_string(resolve_presets_path()?)?;
    let config: PresetConfig = toml::from_str(&content)?;
    Ok(config
        .preset_apps
        .into_iter()
        .map(|mut app| {
            app.preset = true;
            app
        })
        .collect())
}

pub fn load_preset_apps_or_builtin() -> (Vec<AppEntry>, Option<String>) {
    match load_preset_apps() {
        Ok(apps) if !apps.is_empty() => (apps, None),
        Ok(_) => (
            builtin_preset_apps(),
            Some("预设配置为空，已使用内置预设".to_string()),
        ),
        Err(error) => (
            builtin_preset_apps(),
            Some(format!("加载预设配置失败，已使用内置预设：{error}")),
        ),
    }
}

fn builtin_preset_apps() -> Vec<AppEntry> {
    vec![
        preset("腾讯视频", &["QQLive.exe"]),
        preset("爱奇艺", &["QyClient.exe"]),
        preset("芒果TV", &["MangoTV.exe", "mgtv.exe"]),
        preset("优酷", &["YoukuDesktop.exe"]),
        preset("哔哩哔哩", &["bilibili.exe"]),
        preset("抖音", &["douyin.exe"]),
    ]
}

pub fn load_custom_apps() -> anyhow::Result<Vec<AppEntry>> {
    let path = resolve_config_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)?;
    let config: StoredConfig = toml::from_str(&content)?;
    Ok(config
        .custom_apps
        .into_iter()
        .map(|mut app| {
            app.preset = false;
            app
        })
        .collect())
}

pub fn save_custom_apps(apps: &[AppEntry]) -> anyhow::Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config = StoredConfig {
        custom_apps: apps
            .iter()
            .filter(|app| !app.preset)
            .cloned()
            .collect(),
    };
    let temp_path = path.with_extension("toml.tmp");
    fs::write(&temp_path, toml::to_string_pretty(&config)?)?;
    fs::rename(temp_path, path)?;
    Ok(())
}

pub fn validate_custom_app(
    app_name: &str,
    process_names_input: &str,
    existing_apps: &[AppEntry],
) -> Result<AppEntry, String> {
    let app_name = app_name.trim();
    let process_names = parse_process_names(process_names_input);

    if app_name.is_empty() || process_names.is_empty() {
        return Err("应用名称和进程名不能为空".to_string());
    }

    let mut new_keys = Vec::new();
    for process_name in &process_names {
        let key = comparable_process_name(process_name);
        if new_keys.contains(&key) {
            return Err("该进程名已存在，请勿重复添加".to_string());
        }
        new_keys.push(key);
    }

    let is_duplicate = existing_apps.iter().any(|app| {
        app.process_names.iter().any(|process_name| {
            let existing_key = comparable_process_name(process_name);
            new_keys.contains(&existing_key)
        })
    });

    if is_duplicate {
        return Err("该进程名已存在，请勿重复添加".to_string());
    }

    Ok(AppEntry {
        app_name: app_name.to_string(),
        process_names,
        preset: false,
    })
}

pub fn parse_process_names(input: &str) -> Vec<String> {
    input
        .split(|ch| ch == ',' || ch == '，' || ch == '\n' || ch == '\r')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn process_names_text(names: &[String]) -> String {
    names.join(" / ")
}

pub fn normalize_process_name(name: &str) -> String {
    name.trim().to_lowercase()
}

pub fn process_name_matches(configured: &str, running: &str) -> bool {
    comparable_process_name(configured) == comparable_process_name(running)
}

fn comparable_process_name(name: &str) -> String {
    let normalized = normalize_process_name(name);
    if let Some(stripped) = normalized.strip_suffix(".exe") {
        stripped.to_string()
    } else {
        normalized
    }
}

fn preset(app_name: &str, process_names: &[&str]) -> AppEntry {
    AppEntry {
        app_name: app_name.to_string(),
        process_names: process_names.iter().map(|name| name.to_string()).collect(),
        preset: true,
    }
}

pub fn config_path() -> anyhow::Result<PathBuf> {
    Ok(exe_dir()?.join("config.toml"))
}

pub fn local_config_path() -> PathBuf {
    PathBuf::from("config.toml")
}

pub fn user_config_path() -> anyhow::Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "video-tools", "VideoProcessManager")
        .ok_or_else(|| anyhow::anyhow!("无法定位用户配置目录"))?;
    Ok(dirs.config_dir().join("config.toml"))
}

fn resolve_config_path() -> anyhow::Result<PathBuf> {
    let exe_path = config_path()?;
    if exe_path.exists() {
        return Ok(exe_path);
    }

    let working_path = local_config_path();
    if working_path.exists() {
        return Ok(working_path);
    }

    let user_path = user_config_path()?;
    if user_path.exists() {
        return Ok(user_path);
    }

    Ok(exe_path)
}

pub fn presets_path() -> PathBuf {
    PathBuf::from("presets.toml")
}

pub fn exe_presets_path() -> anyhow::Result<PathBuf> {
    Ok(exe_dir()?.join("presets.toml"))
}

pub fn user_presets_path() -> anyhow::Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "video-tools", "VideoProcessManager")
        .ok_or_else(|| anyhow::anyhow!("无法定位用户配置目录"))?;
    Ok(dirs.config_dir().join("presets.toml"))
}

fn resolve_presets_path() -> anyhow::Result<PathBuf> {
    let user_path = user_presets_path()?;
    if user_path.exists() {
        return Ok(user_path);
    }

    let exe_path = exe_presets_path()?;
    if exe_path.exists() {
        return Ok(exe_path);
    }

    let working_path = presets_path();
    if working_path.exists() {
        return Ok(working_path);
    }

    Ok(user_path)
}

fn exe_dir() -> anyhow::Result<PathBuf> {
    let exe = env::current_exe()?;
    let parent = exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("无法定位程序所在目录"))?;
    Ok(parent.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_file_name(path: &std::path::Path) -> Option<&str> {
        path.file_name().and_then(|name| name.to_str())
    }

    #[test]
    fn rejects_empty_custom_app_fields() {
        let result = validate_custom_app("", "example.exe", &[]);
        assert_eq!(result.unwrap_err(), "应用名称和进程名不能为空");
    }

    #[test]
    fn rejects_duplicate_process_names_case_insensitively() {
        let existing = vec![AppEntry {
            app_name: "示例".to_string(),
            process_names: vec!["Example.exe".to_string()],
            preset: false,
        }];

        let result = validate_custom_app("另一个示例", "example", &existing);
        assert_eq!(result.unwrap_err(), "该进程名已存在，请勿重复添加");
    }

    #[test]
    fn normalizes_process_name_with_exe_suffix() {
        assert_eq!(normalize_process_name(" DouYin "), "douyin");
        assert_eq!(normalize_process_name("DouYin.EXE"), "douyin.exe");
    }

    #[test]
    fn matches_windows_exe_suffix_compatibly() {
        assert!(process_name_matches("DouYin", "douyin.exe"));
        assert!(process_name_matches("DouYin.exe", "douyin"));
    }

    #[test]
    fn parses_multiple_process_names() {
        let names = parse_process_names(" app.exe, helper.exe，player.exe\nworker.exe ");
        assert_eq!(names, vec!["app.exe", "helper.exe", "player.exe", "worker.exe"]);
    }

    #[test]
    fn rejects_duplicate_names_inside_one_custom_app() {
        let result = validate_custom_app("示例", "Example.exe, example", &[]);
        assert_eq!(result.unwrap_err(), "该进程名已存在，请勿重复添加");
    }

    #[test]
    fn stores_only_custom_apps() {
        let apps = vec![
            preset("预设", &["preset.exe"]),
            AppEntry {
                app_name: "自定义".to_string(),
                process_names: vec!["custom.exe".to_string()],
                preset: false,
            },
        ];
        let config = StoredConfig {
            custom_apps: apps.iter().filter(|app| !app.preset).cloned().collect(),
        };

        assert_eq!(config.custom_apps.len(), 1);
        assert_eq!(config.custom_apps[0].app_name, "自定义");
    }

    #[test]
    fn loads_preset_apps_from_toml() {
        let config: PresetConfig = toml::from_str(
            r#"
            [[preset_apps]]
            app_name = "示例"
            process_names = ["example.exe", "helper.exe"]
            "#,
        )
        .unwrap();

        assert_eq!(config.preset_apps.len(), 1);
        assert_eq!(config.preset_apps[0].app_name, "示例");
        assert_eq!(
            config.preset_apps[0].process_names,
            vec!["example.exe", "helper.exe"]
        );
    }

    #[test]
    fn config_path_defaults_to_local_file() {
        assert_eq!(
            path_file_name(&config_path().unwrap()),
            Some("config.toml")
        );
    }

    #[test]
    fn local_config_path_uses_packaged_directory() {
        assert_eq!(local_config_path(), PathBuf::from("config.toml"));
    }

    #[test]
    fn exe_presets_path_uses_presets_file_name() {
        assert_eq!(
            path_file_name(&exe_presets_path().unwrap()),
            Some("presets.toml")
        );
    }
}
