use crate::{db, AppState};
use rusqlite::{params, Connection};
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
    let table = table(&kind)?; if kind=="note" {let note:crate::models::Note=serde_json::from_str(&payload).map_err(|e|e.to_string())?;let root=state.data_dir.lock().map_err(|_|"数据目录不可用")?.clone();if crate::note_files::list(&root)?.iter().any(|(n,_)|n.id==note.id){return Err("同 ID 笔记已存在".into())}let value:serde_json::Value=serde_json::from_str(&payload).map_err(|e|e.to_string())?;let tags=serde_json::from_value(value.get("tags").cloned().unwrap_or(serde_json::json!([]))).map_err(|e|e.to_string())?;crate::note_files::write(&root,&note,tags)?;}
    if kind=="countdown" {
        tx.execute(&format!("INSERT INTO {table}(id,title,description,target_date,repeat_type,created_at,updated_at) SELECT json_extract(?1,'$.id'),json_extract(?1,'$.title'),json_extract(?1,'$.description'),json_extract(?1,'$.target_date'),json_extract(?1,'$.repeat_type'),json_extract(?1,'$.created_at'),json_extract(?1,'$.updated_at')"),[&payload])
    } else {
        tx.execute(&format!("INSERT INTO {table}(id,title,content,created_at,updated_at) SELECT json_extract(?1,'$.id'),json_extract(?1,'$.title'),json_extract(?1,'$.content'),json_extract(?1,'$.created_at'),json_extract(?1,'$.updated_at')"),[&payload])
    }.map_err(|e| format!("恢复失败，原内容仍在回收站：{e}"))?;
    tx.execute("DELETE FROM recycle_bin WHERE id=?1", [id])
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn purge_trash(state: State<'_, AppState>, id: Option<String>) -> Result<(), String> {
    let _op = state.data_operation.lock().map_err(|_| "数据操作不可用")?;
    let conn = state.db.lock().map_err(|_| "数据库不可用")?;
    if let Some(id) = id {
        conn.execute("DELETE FROM recycle_bin WHERE id=?1", [id])
    } else {
        conn.execute("DELETE FROM recycle_bin", [])
    }
    .map_err(|e| e.to_string())?;
    Ok(())
}
