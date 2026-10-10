use crate::{backup, AppState};
use chrono::{DateTime, Utc};
use std::{fs, path::PathBuf, time::Duration};
use tauri::{Manager, State};

fn files(root: &std::path::Path) -> Vec<PathBuf> {
    let mut paths = fs::read_dir(root.join("backups"))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("auto-") && n.ends_with(".lumo"))
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

pub fn run(state: &AppState, force: bool) -> Result<Option<String>, String> {
    let _op = state.data_operation.lock().map_err(|_| "数据操作不可用")?;
    if state
        .recovery_error
        .lock()
        .map_err(|_| "状态不可用")?
        .is_some()
    {
        return Err("请先恢复数据目录".into());
    }
    let options = state
        .settings
        .lock()
        .map_err(|_| "设置不可用")?
        .auto_backup
        .clone();
    if !force && !options.enabled {
        return Ok(None);
    }
    let root = state.data_dir.lock().map_err(|_| "路径不可用")?.clone();
    let existing = files(&root);
    let due = existing
        .last()
        .and_then(|p| fs::metadata(p).ok())
        .and_then(|m| m.modified().ok())
        .map(|t| {
            DateTime::<Utc>::from(t) + chrono::Duration::hours(options.interval_hours as i64)
                <= Utc::now()
        })
        .unwrap_or(true);
    if !force && !due {
        return Ok(None);
    }
    let folder = root.join("backups");
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let destination = folder.join(format!(
        "auto-{}-{}.lumo",
        Utc::now().format("%Y%m%d-%H%M%S"),
        uuid::Uuid::new_v4()
    ));
    backup::write_backup(state, &destination)?;
    let paths = files(&root);
    for path in paths
        .iter()
        .take(paths.len().saturating_sub(options.retain.max(1)))
    {
        fs::remove_file(path).map_err(|e| format!("备份成功，但清理旧自动备份失败：{e}"))?;
    }
    *state.backup_error.lock().map_err(|_| "状态不可用")? = None;
    Ok(Some(destination.to_string_lossy().into_owned()))
}

#[tauri::command]
pub async fn run_auto_backup(app: tauri::AppHandle) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || run(&app.state::<AppState>(), true))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn auto_backup_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let root = state.data_dir.lock().map_err(|_| "路径不可用")?.clone();
    let paths = files(&root);
    let last = paths
        .last()
        .and_then(|p| fs::metadata(p).ok())
        .and_then(|m| m.modified().ok())
        .map(|t| DateTime::<Utc>::from(t).to_rfc3339());
    Ok(
        serde_json::json!({"last_backup":last,"count":paths.len(),"directory":root.join("backups").to_string_lossy(),"error":state.backup_error.lock().map_err(|_|"状态不可用")?.clone()}),
    )
}
pub fn start(app: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        if let Err(error) = run(&app.state::<AppState>(), false) {
            if let Ok(mut status) = app.state::<AppState>().backup_error.lock() {
                *status = Some(error);
            }
        }
        std::thread::sleep(Duration::from_secs(60));
    });
}
