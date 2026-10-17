use crate::models::{AppSettings, BootstrapInfo};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
struct BootstrapFile {
    schema_version: u32,
    data_dir: PathBuf,
}

pub struct StoragePaths {
    pub bootstrap_path: PathBuf,
    pub default_data_dir: PathBuf,
    pub data_dir: PathBuf,
    pub recovery_error: Option<String>,
}

pub fn resolve_paths() -> Result<StoragePaths, String> {
    let roaming = dirs::data_dir()
        .ok_or("无法获取 Windows 用户 Roaming AppData 目录")?
        .join("Lumo-Hub");
    let local = dirs::config_local_dir()
        .ok_or("无法获取 Windows 用户 Local AppData 目录")?
        .join("Lumo-Hub");
    if !cfg!(debug_assertions) {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let root = exe.parent().ok_or("无法定位程序安装目录")?.to_path_buf();
        let legacy_pointer = local.join("bootstrap.json");
        let source = if root.join("app.db").exists() { None } else if legacy_pointer.exists() {
            let pointer: BootstrapFile = serde_json::from_slice(&fs::read(&legacy_pointer).map_err(|e| e.to_string())?)
                .map_err(|e| format!("旧数据目录索引损坏，已保留原文件：{e}"))?;
            if pointer.schema_version != 1 || !pointer.data_dir.is_absolute() {
                return Err("旧数据目录索引无效，已停止自动迁移。".into());
            }
            Some(pointer.data_dir)
        } else if roaming.join("app.db").exists() { Some(roaming) } else { None };
        return installed_paths(&root, source.as_deref());
    }
    fs::create_dir_all(&local).map_err(|e| format!("无法创建本地配置目录：{e}"))?;
    let bootstrap_path = local.join("bootstrap.json");
    let backup_path = bootstrap_path.with_extension("json.bak");
    let had_pointer = bootstrap_path.exists() || backup_path.exists();
    let mut recovery_error = None;
    let default_file = || BootstrapFile {
        schema_version: 1,
        data_dir: roaming.clone(),
    };
    let (data, recovered_backup) = if bootstrap_path.exists() {
        let primary = fs::read(&bootstrap_path)
            .map_err(|e| e.to_string())
            .and_then(|bytes| {
                serde_json::from_slice::<BootstrapFile>(&bytes).map_err(|e| e.to_string())
            });
        match primary {
            Ok(data) => (data, false),
            Err(primary_error) if backup_path.exists() => {
                let backup = fs::read(&backup_path)
                    .map_err(|e| e.to_string())
                    .and_then(|bytes| {
                        serde_json::from_slice::<BootstrapFile>(&bytes).map_err(|e| e.to_string())
                    });
                match backup {
                    Ok(backup) => (backup, true),
                    Err(error) => {
                        recovery_error = Some(format!(
                            "数据路径索引及其备份均无法读取：{primary_error}; {error}"
                        ));
                        (default_file(), false)
                    }
                }
            }
            Err(error) => {
                recovery_error = Some(format!("数据路径索引无法读取：{error}"));
                (default_file(), false)
            }
        }
    } else if backup_path.exists() {
        let backup = fs::read(&backup_path)
            .map_err(|e| e.to_string())
            .and_then(|bytes| {
                serde_json::from_slice::<BootstrapFile>(&bytes).map_err(|e| e.to_string())
            });
        match backup {
            Ok(data) => (data, true),
            Err(error) => {
                recovery_error = Some(format!("数据路径索引备份无法读取：{error}"));
                (default_file(), false)
            }
        }
    } else {
        (default_file(), false)
    };
    let data_dir = {
        if data.schema_version != 1 || !data.data_dir.is_absolute() {
            recovery_error = Some("数据路径索引版本无效或路径不是绝对路径。".into());
        }
        if recovered_backup && recovery_error.is_none() {
            let temp = bootstrap_path.with_extension("json.recovering");
            fs::copy(&backup_path, &temp)
                .map_err(|e| format!("已找到有效的数据路径备份，但恢复失败：{e}"))?;
            let corrupt = bootstrap_path.with_extension("json.corrupt");
            if corrupt.exists() {
                fs::remove_file(&corrupt).map_err(|e| e.to_string())?;
            }
            if bootstrap_path.exists() {
                fs::rename(&bootstrap_path, &corrupt)
                    .map_err(|e| format!("无法暂存损坏的路径配置：{e}"))?;
            }
            if let Err(error) = fs::rename(&temp, &bootstrap_path) {
                if corrupt.exists() {
                    let _ = fs::rename(&corrupt, &bootstrap_path);
                }
                return Err(format!("无法恢复路径配置备份：{error}"));
            }
        }
        if recovery_error.is_some() {
            roaming.clone()
        } else {
            data.data_dir
        }
    };
    if recovery_error.is_none() {
        let preparation = (|| {
            if data_dir.exists() {
                let has_entries = fs::read_dir(&data_dir)
                    .map_err(|e| format!("无法读取数据目录：{e}"))?
                    .next()
                    .is_some();
                if has_entries || had_pointer {
                    let marker = fs::symlink_metadata(data_dir.join(".lumo-hub-data"));
                    let database = fs::symlink_metadata(data_dir.join("app.db"));
                    if !marker.is_ok_and(|info| info.is_file() && !info.file_type().is_symlink())
                        || !database
                            .is_ok_and(|info| info.is_file() && !info.file_type().is_symlink())
                    {
                        return Err(
                            "数据路径指向的目录缺少标记或数据库。请从恢复页面选择原数据目录。"
                                .to_string(),
                        );
                    }
                }
            } else if had_pointer {
                return Err("数据路径指向的目录不存在。请从恢复页面选择原数据目录。".to_string());
            }
            fs::create_dir_all(&data_dir)
                .map_err(|e| format!("无法创建数据目录 {}：{e}", data_dir.display()))?;
            ensure_data_marker(&data_dir)?;
            if !bootstrap_path.exists() {
                write_bootstrap(&bootstrap_path, &data_dir)?;
            }
            Ok::<(), String>(())
        })();
        if let Err(error) = preparation {
            recovery_error = Some(error);
        }
    }
    Ok(StoragePaths {
        bootstrap_path,
        default_data_dir: roaming,
        data_dir,
        recovery_error,
    })
}

const DATA_ENTRIES: &[&str] = &["setting.json", "app.json", "tags.json", "window.json", "notes", "cache", "icons", "backups", "logs"];

pub fn remove_data_files(root: &Path) -> Result<(), String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    for name in DATA_ENTRIES.iter().copied().chain(["app.db","app.db-wal","app.db-shm",".lumo-hub-data","bootstrap.json","bootstrap.json.bak"]) {
        let path = root.join(name);
        if !path.exists() { continue; }
        let info = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if info.file_type().is_symlink() || !path.canonicalize().map_err(|e|e.to_string())?.starts_with(&root) { return Err("数据文件路径无效，已停止清理。".into()); }
        if info.is_dir() { fs::remove_dir_all(path).map_err(|e| e.to_string())?; }
        else { fs::remove_file(path).map_err(|e| e.to_string())?; }
    }
    Ok(())
}

fn writable_directory(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let probe = root.join(format!(".write-check-{}", uuid::Uuid::new_v4()));
    fs::write(&probe, b"test").map_err(|e| format!("安装目录 {} 不可写：{e}。请将程序安装到当前用户可写的位置，数据不会转存到 AppData。", root.display()))?;
    fs::remove_file(probe).map_err(|e| e.to_string())
}

pub fn import_legacy_data(source: &Path, root: &Path) -> Result<(), String> {
    writable_directory(root)?;
    if source == root { return Ok(()); }
    if root.join("app.db").exists() { return Err("安装目录已有数据，不会覆盖或合并旧数据。".into()); }
    if fs::read_to_string(source.join(".lumo-hub-data")).map_err(|e| format!("无法读取旧数据标记：{e}"))?.trim() != "lumo-hub-data-v1" {
        return Err("旧数据目录标记无效。".into());
    }
    for name in DATA_ENTRIES { if root.join(name).exists() { return Err(format!("安装目录已存在 {name}，已停止迁移以保留现有文件。")); } }
    let stage = root.join(format!(".data-import-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&stage).map_err(|e| e.to_string())?;
    let result = (|| {
        let source_db = rusqlite::Connection::open_with_flags(source.join("app.db"), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| e.to_string())?;
        let mut snapshot = rusqlite::Connection::open(stage.join("app.db")).map_err(|e| e.to_string())?;
        rusqlite::backup::Backup::new(&source_db, &mut snapshot).map_err(|e| e.to_string())?
            .run_to_completion(64, std::time::Duration::from_millis(10), None).map_err(|e| e.to_string())?;
        let integrity: String = snapshot.query_row("PRAGMA integrity_check", [], |r| r.get(0)).map_err(|e| e.to_string())?;
        if integrity != "ok" { return Err("旧数据库校验失败。".into()); }
        drop(snapshot);
        for name in DATA_ENTRIES {
            let original = source.join(name);
            if !original.exists() { continue; }
            for entry in walkdir::WalkDir::new(&original).follow_links(false) {
                let entry = entry.map_err(|e| e.to_string())?;
                if entry.file_type().is_symlink() { return Err("旧数据包含符号链接，已停止迁移。".into()); }
                let target = stage.join(entry.path().strip_prefix(source).map_err(|e| e.to_string())?);
                if entry.file_type().is_dir() { fs::create_dir_all(&target).map_err(|e| e.to_string())?; }
                else if entry.file_type().is_file() { if let Some(parent) = target.parent() {fs::create_dir_all(parent).map_err(|e| e.to_string())?;} fs::copy(entry.path(), target).map_err(|e| e.to_string())?; }
            }
        }
        for name in DATA_ENTRIES { if stage.join(name).exists() { fs::rename(stage.join(name),root.join(name)).map_err(|e|e.to_string())?; } }
        fs::rename(stage.join("app.db"),root.join("app.db")).map_err(|e| e.to_string())?;
        ensure_data_marker(root)?;
        Ok(())
    })();
    // The staging directory is created directly below the verified installation root.
    if result.is_ok() { let _ = fs::remove_dir(&stage); }
    result
}

fn installed_paths(root: &Path, legacy: Option<&Path>) -> Result<StoragePaths, String> {
    let pointer_path = root.join("bootstrap.json");
    if pointer_path.exists() {
        let selected = (|| {
            let pointer: BootstrapFile = serde_json::from_slice(&fs::read(&pointer_path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
            if pointer.schema_version != 1 || !pointer.data_dir.is_absolute() { return Err("数据目录索引无效。".into()); }
            if pointer.data_dir == root { return Ok(None); }
            let directory = pointer.data_dir;
            if !directory.join("app.db").is_file() || !directory.join(".lumo-hub-data").is_file() { return Err("自定义数据目录不可用，请连接对应磁盘或恢复数据目录。".into()); }
            ensure_data_marker(&directory)?;
            writable_directory(&directory)?;
            Ok(Some(directory))
        })();
        match selected {
            Ok(Some(directory)) => return Ok(StoragePaths {bootstrap_path:pointer_path,default_data_dir:root.into(),data_dir:directory,recovery_error:None}),
            Err(error) => return Ok(StoragePaths {bootstrap_path:pointer_path,default_data_dir:root.into(),data_dir:root.into(),recovery_error:Some(error)}),
            Ok(None) => {},
        }
    }
    let preparation = (|| {
        writable_directory(root)?;
        if root.join("app.db").exists() {
            if !root.join(".lumo-hub-data").is_file() { return Err("安装目录已有 app.db，但缺少 Lumo-Hub 数据标记，已停止操作以保留原文件。".into()); }
            ensure_data_marker(root)?;
        }
        else if let Some(source) = legacy { import_legacy_data(source, root)?; }
        else { ensure_data_marker(root)?; }
        write_bootstrap(&root.join("bootstrap.json"), root)?;
        Ok::<(),String>(())
    })();
    Ok(StoragePaths { bootstrap_path:root.join("bootstrap.json"), default_data_dir:root.into(), data_dir:root.into(), recovery_error:preparation.err() })
}

pub fn bootstrap_info(paths: &StoragePaths, data_dir: &Path) -> BootstrapInfo {
    BootstrapInfo {
        data_follows_install: !cfg!(debug_assertions) && data_dir == paths.default_data_dir,
        data_dir: data_dir.to_string_lossy().into_owned(),
        default_data_dir: paths.default_data_dir.to_string_lossy().into_owned(),
    }
}

pub fn write_bootstrap(path: &Path, data_dir: &Path) -> Result<(), String> {
    let parent = path.parent().ok_or("引导配置路径无效")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temp = path.with_extension("json.tmp");
    let backup = path.with_extension("json.bak");
    let payload = serde_json::to_vec_pretty(&BootstrapFile {
        schema_version: 1,
        data_dir: data_dir.to_path_buf(),
    })
    .map_err(|e| e.to_string())?;
    fs::write(&temp, payload).map_err(|e| format!("写入数据路径配置失败：{e}"))?;
    if backup.exists() {
        fs::remove_file(&backup).map_err(|e| format!("清理旧路径配置备份失败：{e}"))?;
    }
    if path.exists() {
        fs::rename(path, &backup).map_err(|e| format!("无法暂存旧路径配置：{e}"))?;
    }
    if let Err(error) = fs::rename(&temp, path) {
        if backup.exists() {
            let _ = fs::rename(&backup, path);
        }
        let _ = fs::remove_file(&temp);
        return Err(format!("无法切换数据路径配置：{error}"));
    }
    if backup.exists() {
        let _ = fs::remove_file(backup);
    }
    Ok(())
}

pub fn settings_path(data_dir: &Path) -> PathBuf {
    data_dir.join("setting.json")
}

pub fn load_settings(data_dir: &Path) -> AppSettings {
    let path = if settings_path(data_dir).exists() { settings_path(data_dir) } else { data_dir.join("app.json") };
    if let Ok(bytes) = fs::read(&path) {
        if let Ok(settings) = serde_json::from_slice(&bytes) {
            let mut settings=settings;crate::models::migrate_common_launchers(&mut settings);let _=crate::note_files::load_tags(data_dir,&mut settings);return settings;
        }
    }
    let backup = path.with_extension("json.bak");
    if let Ok(bytes) = fs::read(&backup) {
        if let Ok(settings) = serde_json::from_slice(&bytes) {
            let _ = fs::write(&path, bytes);
            let mut settings=settings;crate::models::migrate_common_launchers(&mut settings);let _=crate::note_files::load_tags(data_dir,&mut settings);return settings;
        }
    }
    AppSettings::default()
}

pub fn save_settings(data_dir: &Path, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(data_dir);
    let temp = path.with_extension("json.tmp");
    let mut value=serde_json::to_value(settings).map_err(|e|e.to_string())?;value["product"].as_object_mut().ok_or("设置格式无效")?.remove("note_tags");let bytes = serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?;
    fs::write(&temp, bytes).map_err(|e| format!("保存设置失败：{e}"))?;
    let backup = path.with_extension("json.bak");
    if backup.exists() {
        fs::remove_file(&backup).map_err(|e| e.to_string())?;
    }
    if path.exists() {
        fs::rename(&path, &backup).map_err(|e| format!("暂存旧设置失败：{e}"))?;
    }
    if let Err(error) = fs::rename(&temp, &path) {
        if backup.exists() {
            let _ = fs::rename(&backup, &path);
        }
        let _ = fs::remove_file(temp);
        return Err(format!("保存设置失败：{error}"));
    }
    if backup.exists() {
        let _ = fs::remove_file(backup);
    }
    Ok(())
}

pub fn ensure_data_marker(data_dir: &Path) -> Result<(), String> {
    let marker = data_dir.join(".lumo-hub-data");
    match fs::symlink_metadata(&marker) {
        Ok(info) => {
            if !info.is_file() || info.file_type().is_symlink() {
                return Err("数据目录标记不是普通文件，已停止操作。".into());
            }
            let content =
                fs::read_to_string(&marker).map_err(|e| format!("无法读取数据目录标记：{e}"))?;
            if content.trim() != "lumo-hub-data-v1" {
                return Err("数据目录标记无效，已停止可能造成误删的操作。".into());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::write(marker, "lumo-hub-data-v1\n")
                .map_err(|e| format!("无法初始化数据目录：{e}"))?;
        }
        Err(error) => return Err(format!("无法检查数据目录标记：{error}")),
    }
    Ok(())
}

pub fn copy_directory(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target).map_err(|e| format!("无法创建目标目录：{e}"))?;
    for entry in walkdir::WalkDir::new(source).follow_links(false).into_iter().filter_entry(|entry|{if entry.depth()==0{return true}entry.path().strip_prefix(source).ok().and_then(|p|p.components().next()).and_then(|c|c.as_os_str().to_str()).is_some_and(|name|DATA_ENTRIES.contains(&name)||name==".lumo-hub-data")}) {
        let entry = entry.map_err(|e| format!("读取数据目录失败：{e}"))?;
        let relative = entry
            .path()
            .strip_prefix(source)
            .map_err(|e| e.to_string())?;
        let destination = target.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&destination).map_err(|e| e.to_string())?;
        } else if entry.file_type().is_file() {
            if relative
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| matches!(name, "app.db" | "app.db-wal" | "app.db-shm"))
            {
                continue;
            }
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(entry.path(), destination).map_err(|e| format!("复制数据失败：{e}"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod installed_storage_tests {
 use super::*;
 struct Folder(PathBuf);
 impl Folder {fn new()->Self{let p=std::env::temp_dir().join(format!("lumo-installed-test-{}",uuid::Uuid::new_v4()));fs::create_dir_all(&p).unwrap();Self(p)}}
 impl Drop for Folder{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
 #[test]fn fresh_install_uses_executable_directory(){let root=Folder::new();fs::write(root.0.join("lumo-hub.exe"),b"program").unwrap();let paths=installed_paths(&root.0,None).unwrap();assert!(paths.recovery_error.is_none());assert_eq!(paths.data_dir,root.0);assert_eq!(paths.default_data_dir,root.0);assert_eq!(paths.bootstrap_path,root.0.join("bootstrap.json"));}
 #[test]fn migration_preserves_legacy_and_copies_live_database(){let source=Folder::new();let root=Folder::new();ensure_data_marker(&source.0).unwrap();let conn=crate::db::open(&source.0.join("app.db")).unwrap();conn.execute("INSERT INTO stickies VALUES('id','便签','保留正文','a','b')",[]).unwrap();fs::write(source.0.join("setting.json"),b"{}").unwrap();fs::write(source.0.join("other-program.exe"),b"do not copy").unwrap();let paths=installed_paths(&root.0,Some(&source.0)).unwrap();assert!(paths.recovery_error.is_none(),"{:?}",paths.recovery_error);let db=crate::db::open(&root.0.join("app.db")).unwrap();assert_eq!(db.query_row("SELECT content FROM stickies",[],|r|r.get::<_,String>(0)).unwrap(),"保留正文");assert!(source.0.join("app.db").exists());assert!(!root.0.join("other-program.exe").exists());}
 #[test]fn existing_install_does_not_replace_its_database(){let root=Folder::new();ensure_data_marker(&root.0).unwrap();let source=Folder::new();let db=crate::db::open(&root.0.join("app.db")).unwrap();db.execute("INSERT INTO stickies VALUES('id','已有便签','已有正文','a','b')",[]).unwrap();let paths=installed_paths(&root.0,Some(&source.0)).unwrap();assert!(paths.recovery_error.is_none());assert_eq!(db.query_row("SELECT content FROM stickies",[],|r|r.get::<_,String>(0)).unwrap(),"已有正文");}
 #[test]fn explicit_cleanup_preserves_program_and_unrelated_files(){let root=Folder::new();fs::write(root.0.join("lumo-hub.exe"),b"program").unwrap();fs::write(root.0.join("uninstall.exe"),b"program").unwrap();fs::write(root.0.join("keep.txt"),b"unrelated").unwrap();fs::write(root.0.join("setting.json"),b"{}").unwrap();fs::create_dir(root.0.join("notes")).unwrap();fs::write(root.0.join("notes/test.md"),b"note").unwrap();remove_data_files(&root.0).unwrap();assert!(root.0.join("lumo-hub.exe").exists());assert!(root.0.join("uninstall.exe").exists());assert!(root.0.join("keep.txt").exists());assert!(!root.0.join("notes").exists());}
}

#[cfg(test)]
mod custom_directory_tests {
 use super::*;
 #[test]fn installed_pointer_restores_custom_directory(){let root=std::env::temp_dir().join(format!("lumo-custom-test-{}",uuid::Uuid::new_v4()));let custom=root.join("external");fs::create_dir_all(&custom).unwrap();ensure_data_marker(&custom).unwrap();let db=crate::db::open(&custom.join("app.db")).unwrap();write_bootstrap(&root.join("bootstrap.json"),&custom).unwrap();let paths=installed_paths(&root,None).unwrap();assert_eq!(paths.data_dir,custom);assert!(paths.recovery_error.is_none());drop(db);fs::remove_dir_all(&root).unwrap();}
 #[test]fn missing_custom_directory_requires_recovery(){let root=std::env::temp_dir().join(format!("lumo-missing-test-{}",uuid::Uuid::new_v4()));fs::create_dir(&root).unwrap();write_bootstrap(&root.join("bootstrap.json"),&root.join("missing")).unwrap();let paths=installed_paths(&root,None).unwrap();assert!(paths.recovery_error.is_some());assert!(!root.join("app.db").exists());fs::remove_dir_all(root).unwrap();}
}
