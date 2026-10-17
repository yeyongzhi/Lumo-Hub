use crate::models::{AppSettings, Note};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
#[derive(Serialize, Deserialize)]
struct Header {
    id: String,
    title: String,
    created_at: String,
    updated_at: String,
    #[serde(default)]
    tags: Vec<String>,
}
pub fn atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    fs::create_dir_all(path.parent().ok_or("无效路径")?).map_err(|e| e.to_string())?;
    let tmp = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    let old = path.with_extension("previous");
    if old.exists() {
        return Err(format!("存在待恢复文件：{}", old.display()));
    }
    if path.exists() {
        fs::rename(path, &old).map_err(|e| e.to_string())?;
    }
    if let Err(e) = fs::rename(&tmp, path) {
        if old.exists() {
            let _ = fs::rename(&old, path);
        }
        return Err(e.to_string());
    }
    if old.exists() {
        fs::remove_file(old).map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn decode(path: &Path) -> Result<(Note, Vec<String>), String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let text = text.strip_prefix("---\n").ok_or("笔记缺少元数据头部")?;
    let (head, body) = text.split_once("\n---\n").ok_or("笔记头部格式无效")?;
    // Standard YAML front matter is readable by other Markdown tools.
    let h: Header =
        serde_yaml_ng::from_str(head).map_err(|e| format!("{}: {e}", path.display()))?;
    uuid::Uuid::parse_str(&h.id).map_err(|_| "笔记 ID 无效")?;
    Ok((
        Note {
            id: h.id,
            title: h.title,
            content: body.to_string(),
            created_at: h.created_at,
            updated_at: h.updated_at,
        },
        h.tags,
    ))
}
pub fn inspect(root: &Path) -> Result<(Vec<(Note, Vec<String>)>, Vec<String>), String> {
    let dir = root.join("notes");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut items = vec![];
    let mut issues = vec![];
    let mut ids = std::collections::HashSet::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_file()
            && entry.path().extension().is_some_and(|s| s == "md")
        {
            let path = entry.path();
            let item = match decode(&path) {
                Ok(item) => item,
                Err(error) => {
                    issues.push(format!("{}：{}", path.display(), error));
                    continue;
                }
            };
            if !ids.insert(item.0.id.clone()) {
                issues.push(format!("{}：笔记 ID 与其他文件重复", path.display()));
                continue;
            }
            items.push(item);
        }
    }
    items.sort_by(|a, b| b.0.updated_at.cmp(&a.0.updated_at));
    Ok((items, issues))
}
pub fn list(root: &Path) -> Result<Vec<(Note, Vec<String>)>, String> {
    Ok(inspect(root)?.0)
}
pub fn write(root: &Path, n: &Note, tags: Vec<String>) -> Result<(), String> {
    uuid::Uuid::parse_str(&n.id).map_err(|_| "笔记 ID 无效")?;
    let h = Header {
        id: n.id.clone(),
        title: n.title.clone(),
        created_at: n.created_at.clone(),
        updated_at: n.updated_at.clone(),
        tags,
    };
    let text = format!(
        "---\n{}\n---\n{}",
        serde_yaml_ng::to_string(&h).map_err(|e| e.to_string())?,
        n.content
    );
    let existing = list(root)?.into_iter().find(|(item, _)| item.id == n.id);
    // Filenames remain stable even when a title changes.
    let path = if existing.is_some() {
        find_path(root, &n.id)?
    } else {
        root.join("notes").join(format!("{}.md", n.id))
    };
    atomic(&path, text.as_bytes())
}
pub fn find_path(root: &Path, id: &str) -> Result<std::path::PathBuf, String> {
    for entry in fs::read_dir(root.join("notes")).map_err(|e| e.to_string())? {
        let p = entry.map_err(|e| e.to_string())?.path();
        if p.extension().is_some_and(|s| s == "md") {
            if decode(&p).is_ok_and(|(note, _)| note.id == id) {
                return Ok(p);
            }
        }
    }
    Err("找不到笔记文件".into())
}
pub fn sync(root: &Path, conn: &mut Connection) -> Result<(), String> {
    let (items, issues) = inspect(root)?;
    // Keep the last indexed snapshot until every Markdown file can be parsed. This
    // prevents one damaged file from making other notes disappear from the index.
    if !issues.is_empty() {
        return Ok(());
    }
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM notes", [])
        .map_err(|e| e.to_string())?;
    for (n, _) in items {
        tx.execute(
            "INSERT INTO notes(id,title,content,created_at,updated_at) VALUES(?1,?2,?3,?4,?5)",
            params![n.id, n.title, n.content, n.created_at, n.updated_at],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())
}
pub fn migrate(root: &Path, conn: &mut Connection, s: &AppSettings) -> Result<(), String> {
    reconcile_trash(root, conn)?;
    for name in ["setting.json", "app.json"] {
        let p = root.join(name);
        if p.exists() {
            serde_json::from_slice::<AppSettings>(&fs::read(&p).map_err(|e| e.to_string())?)
                .map_err(|e| format!("设置文件损坏，已保留原文件：{e}"))?;
            break;
        }
    }
    if root.join("notes/.migrated").exists() {
        return sync(root, conn);
    }
    let snapshot = root.join("backups/pre-markdown-migration.db");
    if !snapshot.exists() {
        fs::create_dir_all(root.join("backups")).map_err(|e| e.to_string())?;
        let mut dest = Connection::open(&snapshot).map_err(|e| e.to_string())?;
        rusqlite::backup::Backup::new(conn, &mut dest)
            .map_err(|e| e.to_string())?
            .run_to_completion(64, std::time::Duration::from_millis(10), None)
            .map_err(|e| e.to_string())?;
    }
    let notes = {
        let mut st = conn
            .prepare("SELECT id,title,content,created_at,updated_at FROM notes")
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| {
                Ok(Note {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    content: r.get(2)?,
                    created_at: r.get(3)?,
                    updated_at: r.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?
    };
    for note in notes {
        if !list(root)?.iter().any(|(n, _)| n.id == note.id) {
            write(
                root,
                &note,
                s.product
                    .note_tags
                    .get(&note.id)
                    .cloned()
                    .unwrap_or_default(),
            )?;
        }
    }
    sync(root, conn)?;
    save_tags(root, s)?;
    atomic(&root.join("notes/.migrated"), b"1")
}

/// Reconcile Markdown files with note entries in the recycle bin after an interrupted
/// cross-file/database delete or restore. The database payload remains the recovery source.
pub fn reconcile_trash(root: &Path, conn: &Connection) -> Result<(), String> {
    let dir = root.join("notes");
    let trash_dir = dir.join(".recycle");
    fs::create_dir_all(&trash_dir).map_err(|e| e.to_string())?;
    let mut statement = conn
        .prepare("SELECT json_extract(payload,'$.id') FROM recycle_bin WHERE kind='note'")
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    let trashed = rows
        .collect::<rusqlite::Result<std::collections::HashSet<_>>>()
        .map_err(|e| e.to_string())?;
    drop(statement);

    for id in &trashed {
        let staged = trash_dir.join(format!("{id}.md"));
        if let Ok(active) = find_path(root, id) {
            if staged.exists() {
                fs::remove_file(active).map_err(|e| e.to_string())?;
            } else {
                fs::rename(active, staged).map_err(|e| e.to_string())?;
            }
        }
    }

    for entry in fs::read_dir(&trash_dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if !path.is_file() || path.extension().is_none_or(|extension| extension != "md") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        if trashed.contains(id) {
            continue;
        }
        let restored = dir.join(format!("{id}.md"));
        if restored.exists() {
            // Keep the staged copy rather than overwrite a user's current file.
            continue;
        }
        fs::rename(path, restored).map_err(|e| e.to_string())?;
    }
    Ok(())
}
pub fn save_tags(root: &Path, s: &AppSettings) -> Result<(), String> {
    let items = list(root)?;
    for (n, old) in items {
        let tags = s.product.note_tags.get(&n.id).cloned().unwrap_or_default();
        if tags != old {
            write(root, &n, tags)?;
        }
    }
    let mut catalog = s
        .product
        .note_tags
        .values()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    catalog.sort();
    catalog.dedup();
    atomic(
        &root.join("tags.json"),
        &serde_json::to_vec_pretty(&catalog).map_err(|e| e.to_string())?,
    )
}
pub fn load_tags(root: &Path, s: &mut AppSettings) -> Result<(), String> {
    if !root.join("notes/.migrated").exists() {
        return Ok(());
    }
    s.product.note_tags.clear();
    for (n, tags) in list(root)? {
        s.product.note_tags.insert(n.id, tags);
    }
    if root.join("tags.json").exists() {
        let catalog: Vec<String> =
            serde_json::from_slice(&fs::read(root.join("tags.json")).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        for (i, tags) in catalog.chunks(20).enumerate() {
            s.product
                .note_tags
                .insert(format!("__tag_catalog__{i}"), tags.to_vec());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Folder(std::path::PathBuf);
    impl Folder {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!("lumo-note-test-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn migration_and_frontmatter_roundtrip() {
        let root = Folder::new();
        let mut conn = crate::db::open(&root.0.join("app.db")).unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO notes VALUES(?1,'中文笔记','正文\n---\n保留内容','created','updated')",
            [&id],
        )
        .unwrap();
        let mut s = AppSettings::default();
        s.product.note_tags.insert(id.clone(), vec!["工作".into()]);
        migrate(&root.0, &mut conn, &s).unwrap();
        save_tags(&root.0, &s).unwrap();
        crate::storage::save_settings(&root.0, &s).unwrap();
        let items = list(&root.0).unwrap();
        assert_eq!(items[0].0.content, "正文\n---\n保留内容");
        assert_eq!(items[0].1, vec!["工作"]);
        let settings: serde_json::Value =
            serde_json::from_slice(&fs::read(root.0.join("setting.json")).unwrap()).unwrap();
        assert!(settings["product"].get("note_tags").is_none());
        let mut n = items[0].0.clone();
        n.content = "外部修改".into();
        write(&root.0, &n, vec!["其他".into()]).unwrap();
        sync(&root.0, &mut conn).unwrap();
        assert_eq!(
            conn.query_row("SELECT content FROM notes", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "外部修改"
        );
        assert!(root.0.join("backups/pre-markdown-migration.db").exists());
    }
    #[test]
    fn malformed_markdown_does_not_clear_index_and_is_reported() {
        let root = Folder::new();
        let mut conn = crate::db::open(&root.0.join("app.db")).unwrap();
        conn.execute(
            "INSERT INTO notes VALUES('existing','keep','body','a','b')",
            [],
        )
        .unwrap();
        fs::create_dir_all(root.0.join("notes")).unwrap();
        fs::write(root.0.join("notes/broken.md"), "broken").unwrap();
        sync(&root.0, &mut conn).unwrap();
        let (_, issues) = inspect(&root.0).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(
            conn.query_row("SELECT count(*) FROM notes", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
    }
}
