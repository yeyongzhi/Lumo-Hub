use crate::{db, AppState};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn reveal_resource(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "数据库不可用")?;
    let (kind, target): (String, String) = conn
        .query_row("SELECT kind,target FROM resources WHERE id=?1", [id], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .map_err(|e| e.to_string())?;
    if kind == "web" {
        return Err("网页没有本地文件夹".into());
    }
    if !std::path::Path::new(&target).exists() {
        return Err("路径已失效，请重新定位".into());
    }
    tauri_plugin_opener::reveal_item_in_dir(target).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn batch_resources(
    state: State<'_, AppState>,
    ids: Vec<String>,
    action: String,
    group_id: Option<String>,
) -> Result<(), String> {
    if ids.is_empty() || ids.len() > 10000 {
        return Err("请选择有效的启动项".into());
    }
    if !["move", "favorite", "unfavorite", "delete"].contains(&action.as_str()) {
        return Err("批量操作无效".into());
    }
    let _op = state.data_operation.lock().map_err(|_| "数据操作不可用")?;
    let mut conn = state.db.lock().map_err(|_| "数据库不可用")?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut removed_icons = Vec::<String>::new();
    for id in ids {
        if action == "delete" {
            let icon: Option<String> = tx
                .query_row("SELECT icon_path FROM resources WHERE id=?1", [&id], |r| {
                    r.get(0)
                })
                .map_err(|_| "启动项已不存在，所有操作已撤销。")?;
            if let Some(icon) = icon {
                removed_icons.push(icon);
            }
        }
        let count = match action.as_str() {
            "move" => tx.execute(
                "UPDATE resources SET group_id=?1,updated_at=?2 WHERE id=?3",
                params![group_id, db::timestamp(), id],
            ),
            "delete" => tx.execute("DELETE FROM resources WHERE id=?1", [id]),
            _ => tx.execute(
                "UPDATE resources SET is_favorite=?1,updated_at=?2 WHERE id=?3",
                params![i64::from(action == "favorite"), db::timestamp(), id],
            ),
        }
        .map_err(|e| e.to_string())?;
        if count == 0 {
            return Err("部分启动项已不存在，请刷新后重试。所有操作已撤销。".into());
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    drop(conn);
    let root = state.data_dir.lock().map_err(|_| "路径不可用")?.clone();
    for icon in removed_icons {
        let path = std::path::Path::new(&icon);
        if path.parent() == Some(std::path::Path::new("icons"))
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
        {
            let _ = std::fs::remove_file(root.join(path));
        }
    }
    Ok(())
}
