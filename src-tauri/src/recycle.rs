use crate::{db, AppState};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;
use tauri::State;

fn table(kind: &str) -> Result<&'static str, String> {
    match kind {
        "note" => Ok("notes"),
        "sticky" => Ok("stickies"),
        "countdown" => Ok("countdowns"),
        _ => Err("不支持的回收站类型".into()),
    }
}

pub fn trash(conn: &mut Connection, kind: &str, id: &str) -> Result<(), String> {
    let table = table(kind)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let json = if kind == "countdown" {
        "json_object('id',id,'title',title,'description',description,'target_date',target_date,'repeat_type',repeat_type,'created_at',created_at,'updated_at',updated_at)"
    } else {
        "json_object('id',id,'title',title,'content',content,'created_at',created_at,'updated_at',updated_at)"
    };
    let (title, payload): (String, String) = tx
        .query_row(
            &format!("SELECT title,{json} FROM {table} WHERE id=?1"),
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| "内容已不存在")?;
    tx.execute(
        "INSERT INTO recycle_bin(id,kind,title,payload,deleted_at) VALUES (?1,?2,?3,?4,?5)",
        params![db::id(), kind, title, payload, db::timestamp()],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(&format!("DELETE FROM {table} WHERE id=?1"), [id])
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

pub fn trash_note(conn: &mut Connection, root: &std::path::Path, id: &str) -> Result<(), String> {
    let source = crate::note_files::find_path(root, id)?;
    let tags = crate::note_files::list(root)?
        .into_iter()
        .find(|(note, _)| note.id == id)
        .map(|(_, tags)| tags)
        .unwrap_or_default();
    let trash_dir = root.join("notes/.recycle");
    std::fs::create_dir_all(&trash_dir).map_err(|e| e.to_string())?;
    let staged = trash_dir.join(format!("{id}.md"));
    if staged.exists() {
        return Err("回收站中已存在同 ID 笔记文件，已停止删除。".into());
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let result = (|| {
        let (title, payload): (String, String) = tx
            .query_row(
                "SELECT title,json_object('id',id,'title',title,'content',content,'created_at',created_at,'updated_at',updated_at) FROM notes WHERE id=?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| "内容已不存在")?;
        let mut payload: serde_json::Value = serde_json::from_str(&payload).map_err(|e| e.to_string())?;
        payload["tags"] = serde_json::to_value(tags).map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO recycle_bin(id,kind,title,payload,deleted_at) VALUES (?1,'note',?2,?3,?4)",
            params![db::id(), title, serde_json::to_string(&payload).map_err(|e| e.to_string())?, db::timestamp()],
        )
        .map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM notes WHERE id=?1", [id]).map_err(|e| e.to_string())?;
        std::fs::rename(&source, &staged).map_err(|e| format!("无法暂存笔记文件：{e}"))?;
        tx.commit().map_err(|e| e.to_string())
    })();
    if let Err(error) = result {
        if staged.exists() {
            std::fs::rename(&staged, &source)
                .map_err(|restore| format!("{error}；笔记仍保存在 {}，但自动恢复失败：{restore}", staged.display()))?;
        }
        return Err(error);
    }
    Ok(())
}

#[tauri::command]
pub fn list_trash(state: State<'_, AppState>) -> Result<Vec<Value>, String> {
    let conn = state.db.lock().map_err(|_| "数据库不可用")?;
    let mut s = conn
        .prepare("SELECT id,kind,title,deleted_at FROM recycle_bin ORDER BY deleted_at DESC")
        .map_err(|e| e.to_string())?;
    let rows=s.query_map([],|r|Ok(serde_json::json!({"id":r.get::<_,String>(0)?,"kind":r.get::<_,String>(1)?,"title":r.get::<_,String>(2)?,"deleted_at":r.get::<_,String>(3)?}))).map_err(|e|e.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_trash(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let _op = state.data_operation.lock().map_err(|_| "数据操作不可用")?;
    let mut conn = state.db.lock().map_err(|_| "数据库不可用")?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let (kind, payload): (String, String) = tx
        .query_row(
            "SELECT kind,payload FROM recycle_bin WHERE id=?1",
            [&id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| "回收站内容不存在")?;
    let table = table(&kind)?;
    let root = state.data_dir.lock().map_err(|_| "数据目录不可用")?.clone();
    let mut staged_note: Option<(std::path::PathBuf, std::path::PathBuf, bool)> = None;
    if kind == "note" {
        let note: crate::models::Note = serde_json::from_str(&payload).map_err(|e| e.to_string())?;
        if crate::note_files::list(&root)?.iter().any(|(n, _)| n.id == note.id) {
            return Err("同 ID 笔记已存在；原笔记仍在回收站，请先整理重复文件。".into());
        }
        let value: serde_json::Value = serde_json::from_str(&payload).map_err(|e| e.to_string())?;
        let tags = serde_json::from_value(value.get("tags").cloned().unwrap_or(serde_json::json!([]))).map_err(|e| e.to_string())?;
        let staged = root.join("notes/.recycle").join(format!("{}.md", note.id));
        let active = root.join("notes").join(format!("{}.md", note.id));
        if active.exists() {
            return Err("目标笔记文件已存在；原内容仍在回收站，请先整理重复文件。".into());
        }
        if staged.exists() {
            std::fs::rename(&staged, &active).map_err(|e| format!("无法恢复笔记文件：{e}"))?;
            staged_note = Some((staged, active, true));
        } else {
            crate::note_files::write(&root, &note, tags)?;
            staged_note = Some((staged, active, false));
        }
    }
    let result = (|| {
        if kind == "countdown" {
            tx.execute(&format!("INSERT INTO {table}(id,title,description,target_date,repeat_type,created_at,updated_at) SELECT json_extract(?1,'$.id'),json_extract(?1,'$.title'),json_extract(?1,'$.description'),json_extract(?1,'$.target_date'),json_extract(?1,'$.repeat_type'),json_extract(?1,'$.created_at'),json_extract(?1,'$.updated_at')"), [&payload])
        } else {
            tx.execute(&format!("INSERT INTO {table}(id,title,content,created_at,updated_at) SELECT json_extract(?1,'$.id'),json_extract(?1,'$.title'),json_extract(?1,'$.content'),json_extract(?1,'$.created_at'),json_extract(?1,'$.updated_at')"), [&payload])
        }.map_err(|e| format!("恢复失败，原内容仍在回收站：{e}"))?;
        tx.execute("DELETE FROM recycle_bin WHERE id=?1", [&id]).map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    })();
    if let Err(error) = result {
        if let Some((staged, active, came_from_staging)) = staged_note {
            if came_from_staging {
                std::fs::rename(&active, &staged).map_err(|restore| format!("{error}；笔记仍保存在 {}，但自动回滚失败：{restore}", active.display()))?;
            } else {
                std::fs::remove_file(&active).map_err(|restore| format!("{error}；无法清理未完成的恢复文件 {}：{restore}", active.display()))?;
            }
        }
        return Err(error);
    }
    Ok(())
}

#[tauri::command]
pub fn purge_trash(state: State<'_, AppState>, id: Option<String>) -> Result<(), String> {
    let _op = state.data_operation.lock().map_err(|_| "数据操作不可用")?;
    let conn = state.db.lock().map_err(|_| "数据库不可用")?;
    if let Some(id) = id {
        let note_id: Option<String> = conn.query_row("SELECT json_extract(payload,'$.id') FROM recycle_bin WHERE id=?1 AND kind='note'", [&id], |row| row.get(0)).optional().map_err(|e| e.to_string())?;
        if let Some(note_id) = note_id {
            let staged = state.data_dir.lock().map_err(|_| "数据目录不可用")?.join("notes/.recycle").join(format!("{note_id}.md"));
            if staged.exists() { std::fs::remove_file(staged).map_err(|e| e.to_string())?; }
        }
        conn.execute("DELETE FROM recycle_bin WHERE id=?1", [id])
    } else {
        let note_ids = {
            let mut statement = conn.prepare("SELECT json_extract(payload,'$.id') FROM recycle_bin WHERE kind='note'").map_err(|e| e.to_string())?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0)).map_err(|e| e.to_string())?;
            let ids = rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())?;
            ids
        };
        let trash_dir = state.data_dir.lock().map_err(|_| "数据目录不可用")?.join("notes/.recycle");
        for note_id in note_ids {
            let staged = trash_dir.join(format!("{note_id}.md"));
            if staged.exists() { std::fs::remove_file(staged).map_err(|e| e.to_string())?; }
        }
        conn.execute("DELETE FROM recycle_bin", [])
    }
    .map_err(|e| e.to_string())?;
    Ok(())
}
