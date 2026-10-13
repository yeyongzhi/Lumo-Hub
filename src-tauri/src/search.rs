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

pub fn excerpt(content: &str, query: &str) -> String {
    let chars:Vec<char>=content.chars().collect();
    let needle=query.trim().to_lowercase();
    let index=chars.windows(needle.chars().count().max(1)).position(|part|part.iter().collect::<String>().to_lowercase()==needle).unwrap_or(0);
    let start=index.saturating_sub(30);
    let body:String=chars.iter().skip(start).take(120).collect();
    format!("{}{}{}",if start>0{"…"}else{""},body,if start+120<chars.len(){"…"}else{""})
}

pub fn rank(results: &mut [SearchResult], query: &str) {
    let query=query.trim().to_lowercase();
    results.sort_by_key(|item|if item.title.to_lowercase()==query{0}else if item.title.to_lowercase().starts_with(&query){1}else{2});
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


pub fn find_all(db:&Connection,query:&str)->Result<Vec<SearchResult>,String>{
    let query=query.trim();if query.is_empty(){return Ok(vec![])}
    let escaped=query.replace('\\',"\\\\").replace('%',"\\%").replace('_',"\\_");
    let pattern=format!("%{escaped}%");let prefix=format!("{escaped}%");
    let mut results=find_resources(db,query)?;
    for (table,kind,column) in [("notes","note","content"),("stickies","sticky","content"),("countdowns","countdown","description")] {
        let subtitle=if kind=="countdown"{"target_date || ' · ' || COALESCE(description,'')"}else{"content"};
        let sql=format!("SELECT id,title,{subtitle} FROM {table} WHERE title LIKE ?1 ESCAPE '\\' OR COALESCE({column},'') LIKE ?1 ESCAPE '\\' ORDER BY lower(title)=lower(?2) DESC,lower(title) LIKE lower(?3) ESCAPE '\\' DESC,updated_at DESC LIMIT 12");
        let mut statement=db.prepare(&sql).map_err(|e|e.to_string())?;
        let rows=statement.query_map(rusqlite::params![pattern,query,prefix],|r|Ok(SearchResult{id:r.get(0)?,title:r.get(1)?,subtitle:excerpt(&r.get::<_,String>(2)?,query),kind:kind.into()})).map_err(|e|e.to_string())?;
        results.extend(rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e|e.to_string())?);
    }
    let query=query.to_lowercase();
    results.sort_by_key(|item|if item.title.to_lowercase()==query{0}else if item.title.to_lowercase().starts_with(&query){1}else{2});
    Ok(results)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn unified_search_contains_stickies_and_literal_patterns(){
        let db=crate::db::open(std::path::Path::new(":memory:")).unwrap();
        db.execute("INSERT INTO stickies(id,title,content,created_at,updated_at) VALUES('s','旅行清单','买票','now','now')",[]).unwrap();
        db.execute("INSERT INTO notes(id,title,content,created_at,updated_at) VALUES('n','旅行','100% 完成','now','now')",[]).unwrap();
        let result=find_all(&db,"旅行").unwrap();assert_eq!(result[0].title,"旅行");assert!(result.iter().any(|item|item.kind=="sticky"));
        assert_eq!(find_all(&db,"%").unwrap().len(),1);
        assert!(find_all(&db,"   ").unwrap().is_empty());
    }
}

#[tauri::command]
pub fn search_resources(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    let conn = state.db.lock().map_err(|_| "数据库不可用")?;
    find_resources(&conn, &query)
}
