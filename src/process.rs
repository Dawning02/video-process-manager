use crate::config::{process_name_matches, AppEntry};
use std::{process::Command, thread, time::Duration};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub app_name: String,
    pub process_name: String,
    pub pid: u32,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
}

pub fn find_matching_processes(apps: &[AppEntry]) -> Vec<ProcessInfo> {
    let mut system = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    );
    thread::sleep(Duration::from_millis(220));
    system.refresh_processes();

    let mut matches = system
        .processes()
        .iter()
        .filter_map(|(pid, process)| {
            let process_name = process.name();
            let app = apps.iter().find(|app| {
                app.process_names
                    .iter()
                    .any(|configured| process_name_matches(configured, process_name))
            })?;

            Some(ProcessInfo {
                app_name: app.app_name.clone(),
                process_name: process_name.to_string(),
                pid: pid.as_u32(),
                cpu_usage: process.cpu_usage(),
                memory_bytes: process.memory(),
            })
        })
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| {
        left.app_name
            .cmp(&right.app_name)
            .then(left.process_name.cmp(&right.process_name))
            .then(left.pid.cmp(&right.pid))
    });
    matches
}

pub fn close_processes(targets: &[ProcessInfo]) -> Vec<String> {
    let system = System::new_all();

    let mut failures = Vec::new();
    for target in targets {
        let mut closed = false;
        for (pid, process) in system.processes() {
            if pid.as_u32() == target.pid
                && process_name_matches(&target.process_name, process.name())
            {
                closed = process.kill() || force_close_process(target.pid);
                break;
            }
        }

        if !closed {
            failures.push(target.process_name.clone());
        }
    }

    failures
}

#[cfg(target_os = "windows")]
fn force_close_process(pid: u32) -> bool {
    Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
fn force_close_process(_pid: u32) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::normalize_process_name;

    #[test]
    fn matches_configured_process_name_keys() {
        let apps = vec![AppEntry {
            app_name: "示例".to_string(),
            process_names: vec!["Example".to_string(), "Helper.exe".to_string()],
            preset: false,
        }];
        let keys = apps
            .iter()
            .flat_map(|app| app.process_names.iter().map(|name| normalize_process_name(name)))
            .collect::<Vec<_>>();

        assert_eq!(keys, vec!["example", "helper.exe"]);
    }
}
