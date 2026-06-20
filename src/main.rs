#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use slint::{ModelRc, SharedString, VecModel};

mod config;
mod process;

slint::include_modules!();

use config::{
    config_path, load_custom_apps, load_preset_apps_or_builtin, process_names_text, save_custom_apps,
    validate_custom_app, AppEntry,
};
use process::{close_processes, find_matching_processes, ProcessInfo};

#[derive(Default)]
struct AppState {
    apps: Vec<AppEntry>,
    process_rows: Vec<ProcessStateRow>,
    expanded_apps: BTreeSet<String>,
}

#[derive(Clone, Debug)]
struct ProcessStateRow {
    process: ProcessInfo,
    selected: bool,
}

fn main() -> anyhow::Result<()> {
    let ui = MainWindow::new()?;
    let (apps, load_error) = load_apps();
    let state = Rc::new(RefCell::new(AppState {
        apps,
        process_rows: Vec::new(),
        expanded_apps: BTreeSet::new(),
    }));

    refresh_app_rows(&ui, &state.borrow().apps);
    if let Some(message) = load_error {
        ui.set_status_text(message.into());
    }

    {
        let ui_weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_search_processes(move || {
            if let Some(ui) = ui_weak.upgrade() {
                search_and_refresh(&ui, &state);
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_toggle_process_selection(move |index, selected| {
            if let Some(ui) = ui_weak.upgrade() {
                toggle_process_selection(&ui, &state, index as usize, selected);
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_toggle_process_group(move |index| {
            if let Some(ui) = ui_weak.upgrade() {
                toggle_process_group(&ui, &state, index as usize);
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_close_selected(move || {
            if let Some(ui) = ui_weak.upgrade() {
                close_selected_and_refresh(&ui, &state);
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        ui.on_show_custom_page(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_custom_page_visible(true);
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        ui.on_show_main_page(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_custom_page_visible(false);
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_select_all_processes(move || {
            if let Some(ui) = ui_weak.upgrade() {
                select_all_processes(&ui, &state);
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_invert_process_selection(move || {
            if let Some(ui) = ui_weak.upgrade() {
                invert_process_selection(&ui, &state);
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_add_custom_app(move |app_name, process_name| {
            if let Some(ui) = ui_weak.upgrade() {
                add_custom_app(&ui, &state, app_name.as_str(), process_name.as_str());
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_delete_custom_app(move |index| {
            if let Some(ui) = ui_weak.upgrade() {
                delete_custom_app(&ui, &state, index as usize);
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_reload_config(move || {
            if let Some(ui) = ui_weak.upgrade() {
                reload_config(&ui, &state);
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        ui.on_open_config_file(move || {
            if let Some(ui) = ui_weak.upgrade() {
                open_config_file(&ui);
            }
        });
    }

    ui.run()?;
    Ok(())
}

fn load_apps() -> (Vec<AppEntry>, Option<String>) {
    let (mut apps, preset_load_message) = load_preset_apps_or_builtin();
    let load_error = match load_custom_apps() {
        Ok(mut custom_apps) => {
            apps.append(&mut custom_apps);
            preset_load_message
        }
        Err(error) => {
            let custom_error = format!("加载自定义应用失败：{error}");
            Some(match preset_load_message {
                Some(message) => format!("{message}；{custom_error}"),
                None => custom_error,
            })
        }
    };
    (apps, load_error)
}

fn search_and_refresh(ui: &MainWindow, state: &Rc<RefCell<AppState>>) {
    let apps = state.borrow().apps.clone();
    let process_rows = find_matching_processes(&apps)
        .into_iter()
        .map(|process| ProcessStateRow {
            process,
            selected: false,
        })
        .collect::<Vec<_>>();

    {
        let mut state = state.borrow_mut();
        state.process_rows = process_rows;
        set_process_rows(ui, &state);
    }

    let borrowed = state.borrow();
    let process_count = borrowed.process_rows.len();
    if process_count == 0 {
        ui.set_status_text("未检测到目标应用运行".into());
    } else {
        let app_count = borrowed
            .process_rows
            .iter()
            .map(|row| row.process.app_name.as_str())
            .collect::<BTreeSet<_>>()
            .len();
        ui.set_status_text(format!("检测到 {app_count} 个应用，{process_count} 个目标进程").into());
    }
}

fn close_selected_and_refresh(ui: &MainWindow, state: &Rc<RefCell<AppState>>) {
    let selected = state
        .borrow()
        .process_rows
        .iter()
        .filter(|row| row.selected)
        .map(|row| row.process.clone())
        .collect::<Vec<_>>();

    if selected.is_empty() {
        ui.set_status_text("请至少选择一个进程".into());
        return;
    }

    let failures = close_processes(&selected);
    search_and_refresh(ui, state);

    if failures.is_empty() {
        ui.set_status_text("关闭完成，已刷新搜索结果".into());
    } else {
        ui.set_status_text(format!("关闭失败：进程名{}", failures.join("、")).into());
    }
}

fn select_all_processes(ui: &MainWindow, state: &Rc<RefCell<AppState>>) {
    let mut state = state.borrow_mut();
    if state.process_rows.is_empty() {
        ui.set_status_text("暂无可选择进程".into());
        return;
    }

    for row in &mut state.process_rows {
        row.selected = true;
    }
    set_process_rows(ui, &state);
    ui.set_status_text("已全选当前搜索结果".into());
}

fn invert_process_selection(ui: &MainWindow, state: &Rc<RefCell<AppState>>) {
    let mut state = state.borrow_mut();
    if state.process_rows.is_empty() {
        ui.set_status_text("暂无可选择进程".into());
        return;
    }

    for row in &mut state.process_rows {
        row.selected = !row.selected;
    }
    set_process_rows(ui, &state);
    ui.set_status_text("已反选当前搜索结果".into());
}

fn toggle_process_selection(
    ui: &MainWindow,
    state: &Rc<RefCell<AppState>>,
    display_index: usize,
    selected: bool,
) {
    let mut state = state.borrow_mut();
    if apply_display_selection(&mut state, display_index, selected) {
        set_process_rows(ui, &state);
    }
}

fn toggle_process_group(ui: &MainWindow, state: &Rc<RefCell<AppState>>, display_index: usize) {
    let mut state = state.borrow_mut();
    let Some(display_row) = build_process_display_rows(&state).get(display_index).cloned() else {
        return;
    };

    if !display_row.is_group {
        return;
    }

    let app_name = display_row.app_name.to_string();
    if !state.expanded_apps.insert(app_name.clone()) {
        state.expanded_apps.remove(&app_name);
    }
    set_process_rows(ui, &state);
}

fn add_custom_app(
    ui: &MainWindow,
    state: &Rc<RefCell<AppState>>,
    app_name: &str,
    process_names: &str,
) {
    let new_app = {
        let state = state.borrow();
        validate_custom_app(app_name, process_names, &state.apps)
    };

    match new_app {
        Ok(app) => {
            let mut next_apps = state.borrow().apps.clone();
            next_apps.push(app);

            if let Err(error) = save_custom_apps(&next_apps) {
                ui.set_status_text(format!("保存自定义应用失败：{error}").into());
                return;
            }

            let mut state = state.borrow_mut();
            state.apps = next_apps;
            refresh_app_rows(ui, &state.apps);
            ui.set_custom_app_name(SharedString::new());
            ui.set_custom_process_names(SharedString::new());
            ui.set_status_text("自定义应用已添加".into());
        }
        Err(message) => ui.set_status_text(message.into()),
    }
}

fn delete_custom_app(ui: &MainWindow, state: &Rc<RefCell<AppState>>, index: usize) {
    let mut state = state.borrow_mut();
    let Some(app) = state.apps.get(index) else {
        return;
    };

    if app.preset {
        ui.set_status_text("预设应用不可删除".into());
        return;
    }

    let mut next_apps = state.apps.clone();
    next_apps.remove(index);

    if let Err(error) = save_custom_apps(&next_apps) {
        ui.set_status_text(format!("保存自定义应用失败：{error}").into());
        return;
    }

    state.apps = next_apps;
    refresh_app_rows(ui, &state.apps);
    ui.set_status_text("自定义应用已删除".into());
}

fn reload_config(ui: &MainWindow, state: &Rc<RefCell<AppState>>) {
    let (apps, load_error) = load_apps();
    if let Some(message) = load_error {
        ui.set_status_text(message.into());
        return;
    }

    let mut state = state.borrow_mut();
    state.apps = apps;
    state.process_rows.clear();
    state.expanded_apps.clear();
    refresh_app_rows(ui, &state.apps);
    set_process_rows(ui, &state);
    ui.set_status_text("配置已重新加载".into());
}

fn open_config_file(ui: &MainWindow) {
    match ensure_config_file().and_then(open_path) {
        Ok(()) => ui.set_status_text("已打开配置文件".into()),
        Err(error) => ui.set_status_text(format!("打开配置文件失败：{error}").into()),
    }
}

fn ensure_config_file() -> anyhow::Result<std::path::PathBuf> {
    let path = config_path()?;
    if !path.exists() {
        save_custom_apps(&[])?;
    }
    Ok(path)
}

fn open_path(path: std::path::PathBuf) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", path.to_string_lossy().as_ref()])
            .spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()?;
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    }

    Ok(())
}

fn refresh_app_rows(ui: &MainWindow, apps: &[AppEntry]) {
    let rows = apps
        .iter()
        .map(|app| AppRow {
            app_name: app.app_name.clone().into(),
            process_names_text: process_names_text(&app.process_names).into(),
            preset: app.preset,
        })
        .collect::<Vec<_>>();
    ui.set_app_rows(ModelRc::new(VecModel::from(rows)));
}

fn set_process_rows(ui: &MainWindow, state: &AppState) {
    ui.set_process_rows(ModelRc::new(VecModel::from(build_process_display_rows(
        state,
    ))));
}

fn build_process_display_rows(state: &AppState) -> Vec<ProcessRow> {
    let mut groups: BTreeMap<&str, Vec<(usize, &ProcessStateRow)>> = BTreeMap::new();
    for (index, row) in state.process_rows.iter().enumerate() {
        groups
            .entry(row.process.app_name.as_str())
            .or_default()
            .push((index, row));
    }

    let mut display_rows = Vec::new();
    for (app_name, rows) in groups {
        let child_count = rows.len();
        let selected_count = rows.iter().filter(|(_, row)| row.selected).count();
        let expanded = state.expanded_apps.contains(app_name);
        let selected = child_count > 0 && selected_count == child_count;
        let cpu_usage = rows
            .iter()
            .map(|(_, row)| row.process.cpu_usage)
            .sum::<f32>();
        let memory_bytes = rows
            .iter()
            .map(|(_, row)| row.process.memory_bytes)
            .sum::<u64>();
        let summary_text = if selected_count == 0 {
            format!("{child_count} 个进程")
        } else {
            format!("{child_count} 个进程，已选 {selected_count}")
        };
        let name_text = if child_count > 1 {
            format!("{app_name}（{child_count}）")
        } else {
            app_name.to_string()
        };

        display_rows.push(ProcessRow {
            app_name: app_name.into(),
            process_name: summary_text.clone().into(),
            pid_text: SharedString::new(),
            summary_text: summary_text.clone().into(),
            name_text: name_text.into(),
            row_type_text: "应用".into(),
            cpu_text: format_cpu_usage(cpu_usage).into(),
            memory_text: format_memory(memory_bytes).into(),
            status_text: summary_text.into(),
            expand_text: if expanded { "v" } else { ">" }.into(),
            pid: -1,
            source_index: -1,
            selected,
            is_group: true,
            expanded,
        });

        if expanded {
            for (source_index, row) in rows {
                display_rows.push(ProcessRow {
                    app_name: row.process.app_name.clone().into(),
                    process_name: row.process.process_name.clone().into(),
                    pid_text: row.process.pid.to_string().into(),
                    summary_text: SharedString::new(),
                    name_text: row.process.process_name.clone().into(),
                    row_type_text: "子进程".into(),
                    cpu_text: format_cpu_usage(row.process.cpu_usage).into(),
                    memory_text: format_memory(row.process.memory_bytes).into(),
                    status_text: if row.selected { "已选择" } else { "" }.into(),
                    expand_text: SharedString::new(),
                    pid: row.process.pid as i32,
                    source_index: source_index as i32,
                    selected: row.selected,
                    is_group: false,
                    expanded: false,
                });
            }
        }
    }

    display_rows
}

fn format_cpu_usage(cpu_usage: f32) -> String {
    format!("{:.1}%", cpu_usage.max(0.0))
}

fn format_memory(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes = bytes as f64;
    if bytes >= GB {
        format!("{:.1} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes / KB)
    } else {
        format!("{} B", bytes as u64)
    }
}

fn apply_display_selection(state: &mut AppState, display_index: usize, selected: bool) -> bool {
    let Some(display_row) = build_process_display_rows(state).get(display_index).cloned() else {
        return false;
    };

    if display_row.is_group {
        let app_name = display_row.app_name.to_string();
        for row in &mut state.process_rows {
            if row.process.app_name == app_name {
                row.selected = selected;
            }
        }
        return true;
    }

    if display_row.source_index < 0 {
        return false;
    }

    let Some(row) = state
        .process_rows
        .get_mut(display_row.source_index as usize)
    else {
        return false;
    };
    row.selected = selected;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with_processes(processes: Vec<(&str, &str, u32)>) -> AppState {
        AppState {
            apps: Vec::new(),
            process_rows: processes
                .into_iter()
                .map(|(app_name, process_name, pid)| ProcessStateRow {
                    process: ProcessInfo {
                        app_name: app_name.to_string(),
                        process_name: process_name.to_string(),
                        pid,
                        cpu_usage: 0.0,
                        memory_bytes: 0,
                    },
                    selected: false,
                })
                .collect(),
            expanded_apps: BTreeSet::new(),
        }
    }

    #[test]
    fn groups_process_rows_by_app_name() {
        let state = state_with_processes(vec![
            ("应用A", "main.exe", 1),
            ("应用A", "helper.exe", 2),
            ("应用B", "main.exe", 3),
        ]);

        let rows = build_process_display_rows(&state);

        assert_eq!(rows.len(), 2);
        assert!(rows[0].is_group);
        assert_eq!(rows[0].app_name.as_str(), "应用A");
        assert_eq!(rows[0].name_text.as_str(), "应用A（2）");
        assert_eq!(rows[0].summary_text.as_str(), "2 个进程");
        assert_eq!(rows[1].app_name.as_str(), "应用B");
        assert_eq!(rows[1].name_text.as_str(), "应用B");
    }

    #[test]
    fn expanded_group_includes_child_process_rows() {
        let mut state = state_with_processes(vec![
            ("应用A", "main.exe", 1),
            ("应用A", "helper.exe", 2),
        ]);
        state.expanded_apps.insert("应用A".to_string());

        let rows = build_process_display_rows(&state);

        assert_eq!(rows.len(), 3);
        assert!(rows[0].is_group);
        assert!(!rows[1].is_group);
        assert_eq!(rows[1].process_name.as_str(), "main.exe");
        assert_eq!(rows[2].process_name.as_str(), "helper.exe");
    }

    #[test]
    fn group_summary_reports_selected_count() {
        let mut state = state_with_processes(vec![
            ("应用A", "main.exe", 1),
            ("应用A", "helper.exe", 2),
        ]);
        state.process_rows[0].selected = true;

        let rows = build_process_display_rows(&state);

        assert_eq!(rows[0].summary_text.as_str(), "2 个进程，已选 1");
        assert!(!rows[0].selected);
    }

    #[test]
    fn group_rows_sum_cpu_and_memory() {
        let mut state = state_with_processes(vec![
            ("应用A", "main.exe", 1),
            ("应用A", "helper.exe", 2),
        ]);
        state.process_rows[0].process.cpu_usage = 1.25;
        state.process_rows[0].process.memory_bytes = 1024 * 1024;
        state.process_rows[1].process.cpu_usage = 2.25;
        state.process_rows[1].process.memory_bytes = 2 * 1024 * 1024;

        let rows = build_process_display_rows(&state);

        assert_eq!(rows[0].cpu_text.as_str(), "3.5%");
        assert_eq!(rows[0].memory_text.as_str(), "3.0 MB");
    }

    #[test]
    fn formats_memory_units() {
        assert_eq!(format_memory(512), "512 B");
        assert_eq!(format_memory(1024), "1.0 KB");
        assert_eq!(format_memory(1024 * 1024), "1.0 MB");
        assert_eq!(format_memory(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn selecting_group_selects_all_child_processes() {
        let mut state = state_with_processes(vec![
            ("应用A", "main.exe", 1),
            ("应用A", "helper.exe", 2),
            ("应用B", "main.exe", 3),
        ]);

        assert!(apply_display_selection(&mut state, 0, true));

        assert!(state.process_rows[0].selected);
        assert!(state.process_rows[1].selected);
        assert!(!state.process_rows[2].selected);
    }

    #[test]
    fn selecting_child_process_updates_only_that_process() {
        let mut state = state_with_processes(vec![
            ("应用A", "main.exe", 1),
            ("应用A", "helper.exe", 2),
        ]);
        state.expanded_apps.insert("应用A".to_string());

        assert!(apply_display_selection(&mut state, 2, true));

        assert!(!state.process_rows[0].selected);
        assert!(state.process_rows[1].selected);
    }
}
