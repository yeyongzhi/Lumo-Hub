use crate::{models::SearchResult, AppState};
use pinyin::ToPinyin;
use rusqlite::Connection;
use tauri::State;

pub fn matches(name: &str, aliases: &str, target: &str, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }
    let text = format!("{name} {aliases}").to_lowercase();
    let mut initials = String::new();
    let mut full = String::new();
    for ch in text.chars() {
        if let Some(py) = ch.to_pinyin() {
            let plain = py.plain();
            initials.push(plain.chars().next().unwrap());
            full.push_str(plain);
        } else if ch.is_alphanumeric() {
            initials.push(ch);
            full.push(ch);
        }
    }
    text.contains(&query)
        || target.to_lowercase().contains(&query)
        || initials.contains(&query)
        || full.contains(&query)
}

pub fn find_resources(db: &Connection, query: &str) -> Result<Vec<SearchResult>, String> {
    let mut statement = db.prepare("SELECT id,name,target,aliases,is_favorite,usage_count,last_opened_at FROM resources ORDER BY is_favorite DESC,usage_count DESC,last_opened_at DESC,name COLLATE NOCASE").map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    for row in rows {
        let (id, title, target, aliases) = row.map_err(|e| e.to_string())?;
        if matches(&title, &aliases, &target, query) {
            results.push(SearchResult {
                id,
                title,
                subtitle: target,
                kind: "resource".into(),
            });
        }
    }
    Ok(results)
}

#[tauri::command]
pub fn search_resources(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    let conn = state.db.lock().map_err(|_| "数据库不可用")?;
    find_resources(&conn, &query)
}
