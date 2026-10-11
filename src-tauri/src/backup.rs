use crate::{db, storage, AppState};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    time::Duration,
};
use tauri::State;
use walkdir::WalkDir;
use zip::{write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

fn add_file(
    zip: &mut ZipWriter<File>,
    archive_name: &str,
    path: &Path,
    options: FileOptions,
) -> Result<(), String> {
    zip.start_file(archive_name, options)
        .map_err(|e| format!("写入备份失败：{e}"))?;
    let mut file = File::open(path).map_err(|e| format!("读取备份文件失败：{e}"))?;
    std::io::copy(&mut file, zip).map_err(|e| format!("写入备份失败：{e}"))?;
    Ok(())
}

pub fn write_backup(state: &AppState, destination: &Path) -> Result<(), String> {
    let root = state
        .data_dir
        .lock()
        .map_err(|_| "数据路径状态不可用")?
        .clone();
    let db_guard = state.db.lock().map_err(|_| "数据库不可用")?;
    let snapshot_path =
        destination.with_file_name(format!(".lumo-hub-snapshot-{}.db", uuid::Uuid::new_v4()));
    let snapshot_result = (|| {
        let mut snapshot = rusqlite::Connection::open(&snapshot_path)
            .map_err(|e| format!("无法创建数据库快照：{e}"))?;
        let backup = rusqlite::backup::Backup::new(&db_guard, &mut snapshot)
            .map_err(|e| format!("无法开始数据库快照：{e}"))?;
        backup
            .run_to_completion(64, Duration::from_millis(25), None)
            .map_err(|e| format!("数据库快照失败：{e}"))?;
        drop(backup);
        drop(snapshot);
        Ok::<(), String>(())
    })();
    if let Err(error) = snapshot_result {
        let _ = fs::remove_file(&snapshot_path);
        return Err(error);
    }
    drop(db_guard);
    let result = write_backup_archive(&root, &snapshot_path, destination);
    let _ = fs::remove_file(snapshot_path);
    result
}

fn write_backup_archive(
    root: &Path,
    snapshot_path: &Path,
    destination: &Path,
) -> Result<(), String> {
    let temp = destination.with_extension("lumo.tmp");
    if let Some(parent) = temp.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?
    }
    let file = File::create(&temp).map_err(|e| format!("无法创建备份文件：{e}"))?;
    let mut files = vec![
        serde_json::json!({"path":"app.db","size":fs::metadata(snapshot_path).map_err(|e|e.to_string())?.len()}),
    ];
    if root.join("setting.json").exists() {
        files.push(serde_json::json!({"path":"setting.json","size":fs::metadata(root.join("setting.json")).map_err(|e|e.to_string())?.len()}));
    }
    let icons = root.join("icons");
    if icons.exists() {
        for entry in WalkDir::new(&icons).follow_links(false) {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_type().is_file() {
                let rel = entry
                    .path()
                    .strip_prefix(&root)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push(serde_json::json!({"path":rel,"size":entry.metadata().map_err(|e|e.to_string())?.len()}));
            }
        }
    }
    for name in ["tags.json","notes","cache"] {let p=root.join(name);if p.exists(){for entry in WalkDir::new(&p).follow_links(false){let entry=entry.map_err(|e|e.to_string())?;if entry.file_type().is_file(){let rel=entry.path().strip_prefix(root).map_err(|e|e.to_string())?.to_string_lossy().replace('\\',"/");files.push(serde_json::json!({"path":rel,"size":entry.metadata().map_err(|e|e.to_string())?.len()}));}}}}
    let extra=files.clone();let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
    let manifest = serde_json::json!({"format":"lumo-hub-backup","format_version":1,"created_at":chrono::Utc::now().to_rfc3339(),"app_version":env!("CARGO_PKG_VERSION"),"files":files});
    zip.start_file("manifest.json", options)
        .map_err(|e| e.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(&manifest)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
    .map_err(|e| e.to_string())?;
    add_file(&mut zip, "app.db", snapshot_path, options)?;for entry in extra {let name=entry["path"].as_str().ok_or("无效路径")?;if name=="tags.json"||name.starts_with("notes/")||name.starts_with("cache/"){add_file(&mut zip,name,&root.join(name),options)?;}}
    if root.join("setting.json").exists() {
        add_file(&mut zip, "setting.json", &root.join("setting.json"), options)?;
    }
    let icons = root.join("icons");
    if icons.exists() {
        for entry in WalkDir::new(&icons)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file() {
                let rel = entry
                    .path()
                    .strip_prefix(&root)
                    .map_err(|e| e.to_string())?;
                add_file(
                    &mut zip,
                    &rel.to_string_lossy().replace('\\', "/"),
                    entry.path(),
                    options,
                )?;
            }
        }
    }
    zip.finish().map_err(|e| format!("完成备份失败：{e}"))?;
    let previous = destination.with_extension("lumo.bak");
    if previous.exists() {
        fs::remove_file(&previous).map_err(|e| format!("清理旧备份暂存文件失败：{e}"))?
    }
    if destination.exists() {
        fs::rename(destination, &previous).map_err(|e| format!("无法暂存旧备份文件：{e}"))?
    }
    if let Err(error) = fs::rename(&temp, destination) {
        if previous.exists() {
            let _ = fs::rename(&previous, destination);
        }
        let _ = fs::remove_file(&temp);
        return Err(format!("保存备份文件失败：{error}"));
    }
    if previous.exists() {
        let _ = fs::remove_file(previous);
    }
    Ok(())
}

#[tauri::command]
pub fn export_backup(state: State<'_, AppState>, destination: String) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let destination = Path::new(&destination);
    if !destination
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("lumo"))
    {
        return Err("备份文件必须使用 .lumo 扩展名。".into());
    }
    write_backup(&state, destination)
}

fn rollback_restore_files(
    current: &Path,
    db_file: &Path,
    db_hold: &Path,
    icons_hold: &Path,
    settings_hold: &Path,
    db_held: bool,
    had_icons: bool,
    had_settings: bool,
) {
    if db_held {
        for suffix in ["", "-wal", "-shm"] {
            let _ = fs::remove_file(PathBuf::from(format!("{}{}", db_file.display(), suffix)));
        }
        let _ = fs::rename(db_hold, db_file);
    }
    let icons = current.join("icons");
    if had_icons {
        if icons_hold.exists() {
            let _ = fs::remove_dir_all(&icons);
            let _ = fs::rename(icons_hold, &icons);
        }
    } else {
        let _ = fs::remove_dir_all(&icons);
    }
    let settings = current.join("setting.json");
    if had_settings {
        if settings_hold.exists() {
            let _ = fs::remove_file(&settings);
            let _ = fs::rename(settings_hold, &settings);
        }
    } else {
        let _ = fs::remove_file(settings);
    }
}

#[tauri::command]
pub fn restore_backup(app: tauri::AppHandle, state: State<'_, AppState>, source: String) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let source = PathBuf::from(source);
    let current = state
        .data_dir
        .lock()
        .map_err(|_| "数据路径状态不可用")?
        .clone();
    let stage = current.join(format!(".restore-stage-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&stage).map_err(|e| e.to_string())?;
    let restore_result = (|| {
        let file = File::open(&source).map_err(|e| format!("无法读取备份文件：{e}"))?;
        let mut archive = ZipArchive::new(file).map_err(|e| format!("备份格式无效：{e}"))?;
        let mut has_manifest = false;
        let mut has_db = false;
        let mut total_size = 0_u64;
        let mut extracted_size = 0_u64;
        let mut expected_files = None;
        let mut actual_files = BTreeMap::new();
        const MAX_ARCHIVE_SIZE: u64 = 512 * 1024 * 1024;
        const MAX_MANIFEST_SIZE: u64 = 1024 * 1024;
        if archive.len() > 10_000 {
            return Err("备份文件包含过多条目。".into());
        }
        for i in 0..archive.len() {
            let entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = entry.name().to_string();
            let rel = Path::new(&name);
            let declared_size = entry.size();
            total_size = total_size.saturating_add(declared_size);
            if total_size > MAX_ARCHIVE_SIZE {
                return Err("备份解压内容超过 512 MB，已停止恢复。".into());
            }
            if entry
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
            {
                return Err("备份包含符号链接，已停止恢复。".into());
            }
            if rel.is_absolute()
                || rel
                    .components()
                    .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_)))
            {
                return Err("备份包含不安全的文件路径。".into());
            }
            if name == "manifest.json" {
                if has_manifest {
                    return Err("备份包含重复清单。".into());
                }
                let mut text = String::new();
                entry
                    .take(MAX_MANIFEST_SIZE + 1)
                    .read_to_string(&mut text)
                    .map_err(|e| e.to_string())?;
                if text.len() as u64 > MAX_MANIFEST_SIZE {
                    return Err("备份清单大小异常。".into());
                }
                let v: serde_json::Value =
                    serde_json::from_str(&text).map_err(|_| "备份清单损坏")?;
                if v["format"] != "lumo-hub-backup" || v["format_version"] != 1 {
                    return Err("该备份格式不受支持。".into());
                }
                if let Some(files) = v["files"].as_array() {
                    let mut listed = BTreeMap::new();
                    for file in files {
                        let path = file["path"].as_str().ok_or("备份清单文件路径无效")?;
                        let size = file["size"].as_u64().ok_or("备份清单文件大小无效")?;
                        if listed.insert(path.to_string(), size).is_some() {
                            return Err("备份清单存在重复文件。".into());
                        }
                    }
                    expected_files = Some(listed)
                }
                has_manifest = true;
                continue;
            }
            if name != "app.db" && name != "setting.json" && name != "app.json" && name != "tags.json" && !name.starts_with("notes/") && !name.starts_with("cache/") && !name.starts_with("icons/") {
                return Err("备份包含未知文件，已停止恢复。".into());
            }
            if actual_files.contains_key(&name) {
                return Err("备份包含重复文件。".into());
            }
            if name == "app.db" {
                has_db = true
            }
            let destination = stage.join(rel);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?
            }
            let mut out = File::create(&destination).map_err(|e| e.to_string())?;
            let remaining = MAX_ARCHIVE_SIZE.saturating_sub(extracted_size);
            let copied = std::io::copy(&mut entry.take(remaining + 1), &mut out)
                .map_err(|e| e.to_string())?;
            if copied > remaining || copied != declared_size {
                return Err("备份文件大小与清单不符，或解压内容超过限制。".into());
            }
            extracted_size += copied;
            actual_files.insert(name, copied);
        }
        if !has_manifest || !has_db {
            return Err("备份缺少必要文件。".into());
        }
        if expected_files.as_ref() != Some(&actual_files) {
            return Err("备份清单与文件内容不一致，可能已损坏。".into());
        }
        storage::ensure_data_marker(&stage)?;
        let check = db::open(&stage.join("app.db"))?;
        let integrity: String = check
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if integrity != "ok" {
            return Err(format!("备份数据库校验失败：{integrity}"));
        }
        drop(check);if !stage.join("setting.json").exists()&&stage.join("app.json").exists(){fs::rename(stage.join("app.json"),stage.join("setting.json")).map_err(|e|e.to_string())?;}let mut check=db::open(&stage.join("app.db"))?;let settings=storage::load_settings(&stage);crate::note_files::migrate(&stage,&mut check,&settings)?;storage::save_settings(&stage,&settings)?;
        Ok::<(), String>(())
    })();
    if let Err(error) = restore_result {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }

    let safety = current.join("backups").join(format!(
        "pre-restore-{}-{}.lumo",
        chrono::Utc::now().format("%Y%m%d-%H%M%S"),
        uuid::Uuid::new_v4()
    ));
    if let Err(error) = write_backup(&state, &safety) {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    let mut extras=ExtraSwap::new(&current,&stage)?;
    let mut conn = state.db.lock().map_err(|_| "数据库不可用")?;
    let _ = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| {
        r.get::<_, i64>(0)
    });
    *conn = rusqlite::Connection::open_in_memory().map_err(|e| e.to_string())?;
    let old_db = current.join("app.db");
    for suffix in ["-wal", "-shm"] {
        let _ = fs::remove_file(PathBuf::from(format!("{}{}", old_db.display(), suffix)));
    }
    let db_hold = current.join(format!(".app-db-pre-restore-{}", uuid::Uuid::new_v4()));
    let icons = current.join("icons");
    let icons_hold = current.join(format!(".icons-pre-restore-{}", uuid::Uuid::new_v4()));
    let settings = current.join("setting.json");
    let settings_hold = current.join(format!(".app-json-pre-restore-{}", uuid::Uuid::new_v4()));
    let had_icons = icons.exists();
    let had_settings = settings.exists();
    if let Err(error) = fs::rename(&old_db, &db_hold) {
        if let Ok(old) = db::open(&old_db) {
            *conn = old
        }
        let _ = fs::remove_dir_all(&stage);
        return Err(format!("无法切换数据库：{error}"));
    }
    let db_held = true;
    if let Err(error) = fs::rename(stage.join("app.db"), &old_db) {
        rollback_restore_files(
            &current,
            &old_db,
            &db_hold,
            &icons_hold,
            &settings_hold,
            db_held,
            had_icons,
            had_settings,
        );
        if let Ok(old) = db::open(&old_db) {
            *conn = old
        }
        let _ = fs::remove_dir_all(&stage);
        return Err(format!("恢复数据库失败：{error}"));
    }
    let filesystem_result = (|| {
        if had_icons {
            fs::rename(&icons, &icons_hold).map_err(|e| format!("无法暂存旧图标：{e}"))?
        }
        if stage.join("icons").exists() {
            fs::rename(stage.join("icons"), &icons).map_err(|e| format!("无法恢复图标：{e}"))?
        } else {
            fs::create_dir_all(&icons).map_err(|e| format!("无法初始化图标目录：{e}"))?
        }
        if had_settings {
            fs::rename(&settings, &settings_hold).map_err(|e| format!("无法暂存旧设置：{e}"))?
        }
        if stage.join("setting.json").exists() {
            fs::rename(stage.join("setting.json"), &settings)
                .map_err(|e| format!("无法恢复设置：{e}"))?
        }
        Ok::<(), String>(())
    })();
    if let Err(error) = filesystem_result {
        rollback_restore_files(
            &current,
            &old_db,
            &db_hold,
            &icons_hold,
            &settings_hold,
            db_held,
            had_icons,
            had_settings,
        );
        if let Ok(old) = db::open(&old_db) {
            *conn = old
        }
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    let new_conn = match db::open(&old_db) {
        Ok(c) => c,
        Err(error) => {
            rollback_restore_files(
                &current,
                &old_db,
                &db_hold,
                &icons_hold,
                &settings_hold,
                db_held,
                had_icons,
                had_settings,
            );
            if let Ok(old) = db::open(&old_db) {
                *conn = old
            }
            let _ = fs::remove_dir_all(&stage);
            return Err(error);
        }
    };
    *conn = new_conn;extras.commit();
    let new_settings = storage::load_settings(&current);
    *state.settings.lock().map_err(|_| "设置状态不可用")? = new_settings;
    crate::launcher::sync(&app);
    let _ = fs::remove_file(db_hold);
    let _ = fs::remove_dir_all(icons_hold);
    let _ = fs::remove_file(settings_hold);
    let _ = fs::remove_dir_all(stage);
    Ok(())
}

struct ExtraSwap {root:PathBuf,held:Vec<(String,Option<PathBuf>)>,committed:bool}
impl ExtraSwap {
 fn new(root:&Path,stage:&Path)->Result<Self,String>{let mut swap=Self{root:root.into(),held:vec![],committed:false};for name in ["notes","tags.json","cache"]{let p=root.join(name);let hold=if p.exists(){let hold=root.join(format!(".restore-{}-{}",name,uuid::Uuid::new_v4()));fs::rename(&p,&hold).map_err(|e|e.to_string())?;Some(hold)}else{None};swap.held.push((name.into(),hold));if stage.join(name).exists(){fs::rename(stage.join(name),p).map_err(|e|e.to_string())?;}}Ok(swap)}
 fn commit(&mut self){self.committed=true;}
}
impl Drop for ExtraSwap {fn drop(&mut self){for (name,hold) in &self.held{let p=self.root.join(name);let remove=|p:&Path|{if p.is_dir(){let _=fs::remove_dir_all(p);}else{let _=fs::remove_file(p);}};if self.committed{if let Some(hold)=hold{remove(hold)}}else{remove(&p);if let Some(hold)=hold{let _=fs::rename(hold,p);}}}}}
#[cfg(test)]
mod file_backup_tests {
 use super::*;
 #[test]
 fn archive_contains_markdown_tags_and_unified_settings(){
  let root=std::env::temp_dir().join(format!("lumo-backup-test-{}",uuid::Uuid::new_v4()));fs::create_dir_all(root.join("notes")).unwrap();
  fs::write(root.join("notes/example.md"),"---\nid: example\ntags: [工作]\n---\n正文").unwrap();fs::write(root.join("notes/.migrated"),"1").unwrap();fs::write(root.join("setting.json"),"{}").unwrap();fs::write(root.join("tags.json"),"[\"工作\"]").unwrap();fs::write(root.join("snapshot.db"),"snapshot").unwrap();
  let target=root.join("test.lumo");write_backup_archive(&root,&root.join("snapshot.db"),&target).unwrap();let mut zip=ZipArchive::new(File::open(target).unwrap()).unwrap();
  for name in ["app.db","setting.json","tags.json","notes/example.md","notes/.migrated"]{assert!(zip.by_name(name).is_ok(),"{name}");}
  drop(zip);fs::remove_dir_all(root).unwrap();
 }
}
