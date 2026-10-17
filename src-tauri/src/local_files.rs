use crate::{note_files, AppState};
use tauri::{Emitter, State};
#[tauri::command]
pub fn read_local(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<serde_json::Value>, String> {
    if state
        .recovery_error
        .lock()
        .map_err(|_| "状态不可用")?
        .is_some()
    {
        return Err("请先恢复数据目录".into());
    }
    if !key.starts_with("lumo-") || key.len() > 200 {
        return Err("无效存储键".into());
    }
    if !key.starts_with("lumo-feed-") {
        return Ok(state
            .settings
            .lock()
            .map_err(|_| "设置不可用")?
            .client
            .get(&key)
            .cloned());
    }
    let root = state.data_dir.lock().map_err(|_| "目录不可用")?;
    let p = root.join("cache").join(format!(
        "{}.json",
        key.bytes().map(|b| format!("{b:02x}")).collect::<String>()
    ));
    if !p.exists() {
        return Ok(None);
    }
    Ok(Some(
        serde_json::from_slice(&std::fs::read(p).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?,
    ))
}
#[tauri::command]
pub fn write_local(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    if state
        .recovery_error
        .lock()
        .map_err(|_| "状态不可用")?
        .is_some()
    {
        return Err("请先恢复数据目录".into());
    }
    if !key.starts_with("lumo-") || key.len() > 200 {
        return Err("无效存储键".into());
    }
    let _op = state.data_operation.lock().map_err(|_| "操作不可用")?;
    let root = state.data_dir.lock().map_err(|_| "目录不可用")?.clone();
    if key.starts_with("lumo-feed-") {
        let p = root.join("cache").join(format!(
            "{}.json",
            key.bytes().map(|b| format!("{b:02x}")).collect::<String>()
        ));
        return note_files::atomic(&p, &serde_json::to_vec(&value).map_err(|e| e.to_string())?);
    }
    let mut settings = state.settings.lock().map_err(|_| "设置不可用")?;
    let previous = settings.clone();
    let pinned_group_changed = key.starts_with("lumo-launcher-");
    settings.client.insert(key, value);
    if let Err(e) = crate::storage::save_settings(&root, &settings) {
        *settings = previous;
        return Err(e);
    }
    drop(settings);
    if pinned_group_changed { let _ = app.emit("lumo-settings-changed", ()); }
    Ok(())
}
