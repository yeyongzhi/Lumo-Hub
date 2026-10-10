use rusqlite::Connection;
use std::path::Path;

pub fn open(path: &Path) -> Result<Connection, String> {
    let connection = Connection::open(path).map_err(|e| format!("无法打开本地数据库：{e}"))?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| e.to_string())?;
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| e.to_string())?;
    connection
        .pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| e.to_string())?;
    let schema_version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if schema_version > 3 {
        return Err(format!(
            "数据库版本 {schema_version} 高于此应用支持的版本。请安装更新版本的 Lumo-Hub。"
        ));
    }
    if schema_version < 1 {
        connection.execute_batch(
        "BEGIN IMMEDIATE;
         CREATE TABLE IF NOT EXISTS notes (
           id TEXT PRIMARY KEY, title TEXT NOT NULL, content TEXT NOT NULL DEFAULT '',
           created_at TEXT NOT NULL, updated_at TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS stickies (
           id TEXT PRIMARY KEY, title TEXT NOT NULL, content TEXT NOT NULL DEFAULT '',
           created_at TEXT NOT NULL, updated_at TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS countdowns (
           id TEXT PRIMARY KEY, title TEXT NOT NULL, description TEXT,
           target_date TEXT NOT NULL, repeat_type TEXT NOT NULL CHECK(repeat_type IN ('once','yearly')),
           created_at TEXT NOT NULL, updated_at TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS resource_groups (
           id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, sort_order INTEGER NOT NULL DEFAULT 0
         );
         CREATE TABLE IF NOT EXISTS resources (
           id TEXT PRIMARY KEY, name TEXT NOT NULL, kind TEXT NOT NULL CHECK(kind IN ('app','web','file')),
           target TEXT NOT NULL, group_id TEXT REFERENCES resource_groups(id) ON DELETE SET NULL,
           is_favorite INTEGER NOT NULL DEFAULT 0, icon_path TEXT,
           created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
           UNIQUE(kind, target)
         );
         CREATE INDEX IF NOT EXISTS idx_notes_updated ON notes(updated_at DESC);
         CREATE INDEX IF NOT EXISTS idx_countdowns_target ON countdowns(target_date);
         CREATE INDEX IF NOT EXISTS idx_resources_group ON resources(group_id);
         CREATE INDEX IF NOT EXISTS idx_resources_kind ON resources(kind);
         PRAGMA user_version=1;
         COMMIT;"
        ).map_err(|e| format!("初始化数据库失败：{e}"))?;
    }
    if schema_version < 2 {
        connection.execute_batch(
            "BEGIN IMMEDIATE;
             ALTER TABLE resources ADD COLUMN last_opened_at TEXT;
             CREATE INDEX idx_resources_last_opened ON resources(last_opened_at DESC);
             PRAGMA user_version=2;
             COMMIT;"
        ).map_err(|e| format!("升级最近打开记录失败：{e}"))?;
    }
    if schema_version < 3 {
        connection.execute_batch("BEGIN IMMEDIATE;
        ALTER TABLE resources ADD COLUMN aliases TEXT NOT NULL DEFAULT '';
        ALTER TABLE resources ADD COLUMN usage_count INTEGER NOT NULL DEFAULT 0;
        CREATE TABLE recycle_bin (id TEXT PRIMARY KEY, kind TEXT NOT NULL, title TEXT NOT NULL, payload TEXT NOT NULL, deleted_at TEXT NOT NULL);
        PRAGMA user_version=3; COMMIT;").map_err(|e| format!("升级搜索和回收站失败：{e}"))?;
    }
    Ok(connection)
}

pub fn timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}
pub fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TestDatabase(PathBuf);
    impl TestDatabase {
        fn new() -> Self {
            Self(std::env::temp_dir().join(format!("lumo-db-test-{}.db", id())))
        }
    }
    impl Drop for TestDatabase {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
            for suffix in ["-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{suffix}", self.0.display()));
            }
        }
    }

    #[test]
    fn old_database_preserves_data_and_recent_history_on_reopen() {
        let file = TestDatabase::new();
        {
            let old = Connection::open(&file.0).unwrap();
            old.execute_batch(
                "CREATE TABLE resources (
                   id TEXT PRIMARY KEY, name TEXT NOT NULL, kind TEXT NOT NULL,
                   target TEXT NOT NULL, group_id TEXT, is_favorite INTEGER NOT NULL DEFAULT 0,
                   icon_path TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
                 );
                 CREATE TABLE notes (id TEXT PRIMARY KEY, content TEXT NOT NULL);
                 INSERT INTO resources VALUES ('app-1','编辑器','app','C:/Editor.exe',NULL,1,'icons/editor.png','2026-10-01','2026-10-01');
                 INSERT INTO notes VALUES ('note-1','保留这篇笔记');
                 PRAGMA user_version=1;"
            ).unwrap();
        }
        {
            let db = open(&file.0).unwrap();
            assert_eq!(db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0)).unwrap(), 3);
            let saved: (String, i64, String, Option<String>) = db.query_row(
                "SELECT name,is_favorite,icon_path,last_opened_at FROM resources WHERE id='app-1'", [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            ).unwrap();
            assert_eq!(saved, ("编辑器".into(), 1, "icons/editor.png".into(), None));
            assert_eq!(db.query_row("SELECT content FROM notes", [], |r| r.get::<_, String>(0)).unwrap(), "保留这篇笔记");
            db.execute("UPDATE resources SET last_opened_at='2026-10-01T07:30:00Z' WHERE id='app-1'", []).unwrap();
        }
        let db = open(&file.0).unwrap();
        assert_eq!(db.query_row("SELECT last_opened_at FROM resources", [], |r| r.get::<_, String>(0)).unwrap(), "2026-10-01T07:30:00Z");
        assert_eq!(db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0)).unwrap(), 3);
    }

    #[test]
    fn new_database_has_empty_recent_history() {
        let file = TestDatabase::new();
        let db = open(&file.0).unwrap();
        db.execute("INSERT INTO resources(id,name,kind,target,created_at,updated_at) VALUES ('new-app','程序','app','C:/App.exe','2026-10-01','2026-10-01')", []).unwrap();
        let history: Option<String> = db.query_row("SELECT last_opened_at FROM resources", [], |r| r.get(0)).unwrap();
        assert!(history.is_none());
    }
}
