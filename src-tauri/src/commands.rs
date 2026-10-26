use crate::{app_icon, app_scan, db, models::*, storage, AppState};
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use tauri::{AppHandle, State, Emitter};

fn sql<T>(result: rusqlite::Result<T>) -> Result<T, String> {
    result.map_err(|e| format!("本地数据操作失败：{e}"))
}
fn data_root(state: &AppState) -> Result<PathBuf, String> {
    state
        .data_dir
        .lock()
        .map(|p| p.clone())
        .map_err(|_| "数据路径状态不可用".into())
}

#[tauri::command]
pub fn bootstrap_state(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let data_dir = data_root(&state)?;
    let settings = state.settings.lock().map_err(|_| "设置状态不可用")?.clone();
    let recovery_error = state
        .recovery_error
        .lock()
        .map_err(|_| "恢复状态不可用")?
        .clone();
    let clean_uninstall = state
        .clean_uninstall_requested
        .load(std::sync::atomic::Ordering::SeqCst)
        || std::env::args_os().any(|arg| arg.to_string_lossy() == "--clean-uninstall");
    Ok(
        serde_json::json!({"bootstrap": storage::bootstrap_info(&state.paths, &data_dir), "settings": settings, "clean_uninstall": clean_uninstall, "recovery_error": recovery_error}),
    )
}

#[tauri::command]
pub fn recover_data_directory(
    app: AppHandle,
    state: State<'_, AppState>,
    new_path: String,
) -> Result<BootstrapInfo, String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    if state
        .recovery_error
        .lock()
        .map_err(|_| "恢复状态不可用")?
        .is_none()
    {
        return Err("当前不需要恢复数据路径。".into());
    }
    let requested = PathBuf::from(new_path.trim());

    if !requested.is_absolute() {
        return Err("请选择绝对路径。".into());
    }
    let is_default = requested == state.paths.default_data_dir;
    if is_default {
        fs::create_dir_all(&requested).map_err(|e| format!("无法创建默认数据目录：{e}"))?;
    }
    let target = requested
        .canonicalize()
        .map_err(|e| format!("无法打开所选数据目录：{e}"))?;
    let marker = target.join(".lumo-hub-data");
    let database = target.join("app.db");
    let mut default_can_initialize = is_default && !database.exists();
    if default_can_initialize && cfg!(debug_assertions) {
        for entry in fs::read_dir(&target).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let safe = if name == ".lumo-hub-data" {
                entry.file_type().map_err(|e| e.to_string())?.is_file()
            } else if ["icons", "logs", "backups"].contains(&name.as_ref()) {
                entry.file_type().map_err(|e| e.to_string())?.is_dir()
                    && fs::read_dir(entry.path())
                        .map_err(|e| e.to_string())?
                        .next()
                        .is_none()
            } else {
                false
            };
            if !safe {
                default_can_initialize = false;
                break;
            }
        }
    }
    if default_can_initialize {
        storage::ensure_data_marker(&target)?;
    } else {
        let marker_info = fs::symlink_metadata(&marker)
            .map_err(|_| "所选目录没有 Lumo-Hub 数据标记。请选原数据目录。")?;
        let database_info = fs::symlink_metadata(&database)
            .map_err(|_| "所选目录没有 app.db 数据库。请选原数据目录。")?;
        if !marker_info.is_file()
            || marker_info.file_type().is_symlink()
            || fs::read_to_string(&marker)
                .map_err(|e| e.to_string())?
                .trim()
                != "lumo-hub-data-v1"
            || !database_info.is_file()
            || database_info.file_type().is_symlink()
        {
            return Err("所选目录不是有效的 Lumo-Hub 数据目录。".into());
        }
    }
    for folder in ["icons", "logs", "backups"] {
        fs::create_dir_all(target.join(folder)).map_err(|e| format!("无法准备数据目录：{e}"))?;
    }
    let mut new_db = db::open(&database)?;
    let integrity: String = new_db
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if integrity != "ok" {
        return Err(format!("所选数据库校验失败：{integrity}"));
    }
    let mut new_settings = storage::load_settings(&target);crate::note_files::migrate(&target,&mut new_db,&new_settings)?;crate::note_files::load_tags(&target,&mut new_settings)?;storage::save_settings(&target,&new_settings)?;
    for candidate in [
        state.paths.bootstrap_path.clone(),
        state.paths.bootstrap_path.with_extension("json.bak"),
    ] {
        if candidate.is_file() {
            let preserved = candidate.with_extension(format!("corrupt-{}", uuid::Uuid::new_v4()));
            fs::copy(&candidate, &preserved).map_err(|e| format!("无法保留损坏的路径索引：{e}"))?;
        }
    }
    storage::write_bootstrap(&state.paths.bootstrap_path, &target)?;
    *state.db.lock().map_err(|_| "数据库不可用")? = new_db;
    *state.data_dir.lock().map_err(|_| "数据路径状态不可用")? = target.clone();
    *state.settings.lock().map_err(|_| "设置状态不可用")? = new_settings;
    *state.recovery_error.lock().map_err(|_| "恢复状态不可用")? = None;
    crate::launcher::sync(&app);
    Ok(storage::bootstrap_info(&state.paths, &target))
}

#[tauri::command]
pub fn clean_uninstall(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let data_dir = data_root(&state)?;
    let root = data_dir
        .canonicalize()
        .map_err(|e| format!("无法解析数据目录：{e}"))?;
    let marker = root.join(".lumo-hub-data");
    let marker_info =
        fs::symlink_metadata(&marker).map_err(|_| "数据目录缺少安全标记，已停止清理。")?;
    if !marker_info.is_file() || marker_info.file_type().is_symlink() {
        return Err("数据目录标记无效，已停止清理。".into());
    }
    let database = root.join("app.db");
    let database_info =
        fs::symlink_metadata(&database).map_err(|_| "数据目录缺少数据库，已停止清理。")?;
    if fs::read_to_string(&marker)
        .map_err(|e| e.to_string())?
        .trim()
        != "lumo-hub-data-v1"
        || !database_info.is_file()
        || database_info.file_type().is_symlink()
    {
        return Err("数据目录校验失败，已停止清理。".into());
    }
    if root.parent().is_none() {
        return Err("不能清理磁盘根目录。".into());
    }
    if let Some(profile) = dirs::home_dir().and_then(|p| p.canonicalize().ok()) {
        if profile.starts_with(&root) {
            return Err("数据目录包含整个 Windows 用户目录，已停止清理。".into());
        }
    }
    let executable = std::env::current_exe().map_err(|e| format!("无法定位程序安装目录：{e}"))?;
    let install_dir = executable.parent().ok_or("程序安装目录无效")?;
    let install_real = install_dir
        .canonicalize()
        .map_err(|e| format!("无法解析程序安装目录：{e}"))?;
    if root != install_real && (install_real.starts_with(&root) || root.starts_with(&install_real)) {
        return Err("数据目录与程序安装目录重叠，已停止清理。".into());
    }
    let uninstaller = [
        install_dir.join("uninstall.exe"),
        install_dir.join("unins000.exe"),
    ]
    .into_iter()
    .find(|p| p.is_file())
    .ok_or("找不到 Windows 卸载程序。为保留程序和数据，请先重新安装或使用 Windows 设置卸载。")?;
    let bootstrap = &state.paths.bootstrap_path;
    let bootstrap_points_to_root = fs::read(bootstrap)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|value| {
            value
                .get("data_dir")
                .and_then(|x| x.as_str())
                .map(PathBuf::from)
        })
        .and_then(|path| path.canonicalize().ok())
        .is_some_and(|path| path == root);

    {
        let mut db = state.db.lock().map_err(|_| "数据库不可用")?;
        let _ = db.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| {
            r.get::<_, i64>(0)
        });
        *db = rusqlite::Connection::open_in_memory().map_err(|e| e.to_string())?;
    }
    storage::remove_data_files(&root).map_err(|e| format!("删除用户数据失败：{e}"))?;

    if bootstrap_points_to_root {
        let _ = fs::remove_file(bootstrap);
        let _ = fs::remove_file(bootstrap.with_extension("json.bak"));
        if let Some(parent) = bootstrap.parent() {
            let _ = fs::remove_dir(parent);
        }
    }
    Command::new(uninstaller)
        .spawn()
        .map_err(|e| format!("数据已清理，但无法启动 Windows 卸载程序：{e}"))?;
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn set_setting(
    app: AppHandle,
    state: State<'_, AppState>,
    key: String,
    value: serde_json::Value,
) -> Result<AppSettings, String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let mut settings = state.settings.lock().map_err(|_| "设置状态不可用")?;
    let previous = settings.clone();
    match key.as_str() {
        "launcher" => {
            let options: crate::models::LauncherSettings = serde_json::from_value(value).map_err(|_| "快捷启动尺寸无效")?;
            if options.enabled != settings.launcher.enabled || options.shortcut != settings.launcher.shortcut || options.minimal_mode != settings.launcher.minimal_mode {
                return Err("快捷键和启动模式请通过快捷启动配置保存".into());
            }
            match (options.window_width, options.window_height) {
                (None, None) => {},
                (Some(w), Some(h)) if (480..=3840).contains(&w) && (320..=2160).contains(&h) => {},
                _ => return Err("窗口宽度须为 480–3840，高度须为 320–2160".into()),
            }
            settings.launcher = options;
        }
        "product" => {
            let options: ProductSettings=serde_json::from_value(value).map_err(|_|"偏好设置无效")?;
            if options.reminder_hour>23 || options.reminders.len()>10000 || options.reminders.values().any(|days|days.len()>3||days.iter().any(|day|![0,1,7].contains(day))) || options.note_tags.values().any(|tags|tags.len()>20||tags.iter().any(|tag|tag.len()>120)) {return Err("提醒或标签设置无效".into())}
            if options.autostart != settings.product.autostart {crate::desktop::autostart(options.autostart)?;}
            if options.note_tags!=settings.product.note_tags {crate::note_files::save_tags(&data_root(&state)?,&AppSettings{product:options.clone(),..settings.clone()})?;}settings.product=options;
        }
        "theme_mode" => {
            let v = value.as_str().ok_or("主题模式无效")?;
            if !["light", "dark", "system"].contains(&v) {
                return Err("不支持的主题模式".into());
            }
            settings.theme_mode = v.to_string();
        }
        "accent" => {
            let v = value.as_str().ok_or("强调色无效")?;
            if ![
                "red", "orange", "yellow", "green", "blue", "indigo", "violet",
            ]
            .contains(&v)
            {
                return Err("不支持的强调色".into());
            }
            settings.accent = v.to_string();
        }
        "common_launchers" => {
            let ids: Vec<String> = serde_json::from_value(value).map_err(|_| "常用启动项设置无效")?;
            let mut unique = std::collections::HashSet::new();
            if ids.len() > COMMON_LAUNCHER_IDS.len() || ids.iter().any(|id| !COMMON_LAUNCHER_IDS.contains(&id.as_str()) || !unique.insert(id.clone())) { return Err("不支持的常用启动项".into()); }
            settings.common_launchers = ids;
        }
        "sidebar_width" => {
            let width = value.as_u64().and_then(|v| u16::try_from(v).ok()).ok_or("侧边栏宽度无效")?;
            if !(100..=280).contains(&width) { return Err("不支持的侧边栏宽度".into()); }
            settings.sidebar_width = width;
        }
        "border_radius" => {
            let radius = value.as_u64().and_then(|v| u8::try_from(v).ok()).ok_or("圆角值无效")?;
            if ![0, 4, 8, 12, 16, 24].contains(&radius) { return Err("不支持的圆角值".into()); }
            settings.border_radius = radius;
        }
        "font_size" => {
            let v = value
                .as_u64()
                .and_then(|size| u8::try_from(size).ok())
                .ok_or("字号无效")?;
            if !(12..=24).contains(&v) {
                return Err("字号需在 12 到 24 之间".into());
            }
            settings.font_size = v;
        }
        "home_layout" => {
            let layout: Vec<HomeWidget> = serde_json::from_value(value).map_err(|_| "首页布局格式无效")?;
            let mut ids = std::collections::HashSet::new();
            let mut kinds = std::collections::HashSet::new();
            if layout.len() > 9 || layout.iter().any(|widget|
                widget.id.is_empty() || widget.id.len() > 80 || !ids.insert(widget.id.clone()) || !kinds.insert(widget.kind.clone())
                || !["clock", "countdowns", "sticky", "note", "recent", "weather", "hitokoto", "news", "poetry"].contains(&widget.kind.as_str())
                || !["1x1", "2x1", "1x2", "2x2", "4x1"].contains(&widget.size.as_str()) || widget.selected_ids.len() > 100 || widget.selected_ids.iter().any(|id| id.len()>80) || (["sticky", "note"].contains(&widget.kind.as_str()) && widget.selected_ids.len()>1)) {
                return Err("首页模块或布局尺寸无效".into());
            }
            settings.home_layout = layout;
        }
        "auto_backup" => {
            let options: crate::models::AutoBackupSettings = serde_json::from_value(value).map_err(|_| "自动备份设置无效")?;
            if ![24,168].contains(&options.interval_hours) || !(1..=30).contains(&options.retain) { return Err("备份周期或保留数量无效".into()); }
            settings.auto_backup = options;
        }
        _ => return Err("不允许修改此设置项".into()),
    }
    if let Err(error) = storage::save_settings(&data_root(&state)?, &settings) {
        if key=="product" {let _=crate::desktop::autostart(previous.product.autostart);}
        *settings = previous;
        return Err(error);
    }
    let result=settings.clone();
    drop(settings);
    let _=app.emit("lumo-settings-changed",());
    Ok(result)
}

#[tauri::command]
pub fn complete_welcome(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let mut settings = state.settings.lock().map_err(|_| "设置状态不可用")?;
    let previous = settings.clone();
    settings.welcome_completed = true;
    if let Err(error) = storage::save_settings(&data_root(&state)?, &settings) {
        *settings = previous;
        return Err(error);
    }
    Ok(settings.clone())
}

#[tauri::command]
pub fn list_notes(state: State<'_, AppState>) -> Result<Vec<Note>, String> { let root=data_root(&state)?;crate::note_files::load_tags(&root,&mut *state.settings.lock().map_err(|_|"设置不可用")?)?;return Ok(crate::note_files::list(&root)?.into_iter().map(|(n,_)|n).collect());}
#[tauri::command]
pub fn note_file_issues(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let root = data_root(&state)?;
    Ok(crate::note_files::inspect(&root)?.1)
}
#[tauri::command]
pub fn create_note(state: State<'_, AppState>, title: String, content: Option<String>) -> Result<Note, String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let now = db::timestamp();
    let note = Note {
        id: db::id(),
        title: if title.trim().is_empty() {
            "新笔记".into()
        } else {
            title.trim().into()
        },
        content: content.unwrap_or_default(),
        created_at: now.clone(),
        updated_at: now,
    };
    let root = data_root(&state)?;
    crate::note_files::write(&root, &note, vec![])?;
    if let Err(error) = sql(db.execute(
        "INSERT INTO notes(id,title,content,created_at,updated_at) VALUES(?1,?2,?3,?4,?4)",
        params![note.id, note.title, note.content, note.created_at],
    )) {
        if let Ok(path) = crate::note_files::find_path(&root, &note.id) {
            if let Err(rollback) = std::fs::remove_file(&path) {
                return Err(format!("{error}；新建的笔记文件仍保留在 {}：{rollback}", path.display()));
            }
        }
        return Err(error);
    }
    Ok(note)
}

#[tauri::command]
pub fn update_note(
    state: State<'_, AppState>,
    id: String,
    title: String,
    content: String,
) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let mut db = state.db.lock().map_err(|_| "数据库不可用")?;
    let root = data_root(&state)?;
    crate::note_files::sync(&root, &mut db)?;
    let (mut note, tags) = crate::note_files::list(&root)?.into_iter().find(|(n, _)| n.id == id).ok_or("找不到笔记")?;
    let original = note.clone();
    let updated_at = db::timestamp();
    note.title = title.clone();
    note.content = content.clone();
    note.updated_at = updated_at.clone();
    crate::note_files::write(&root, &note, tags.clone())?;
    let changed = match sql(db.execute(
        "UPDATE notes SET title=?1,content=?2,updated_at=?3 WHERE id=?4",
        params![title, content, updated_at, id],
    )) {
        Ok(changed) => changed,
        Err(error) => {
            if let Err(rollback) = crate::note_files::write(&root, &original, tags) {
                return Err(format!("{error}；恢复旧笔记内容失败：{rollback}"));
            }
            return Err(error);
        }
    };
    if changed == 0 {
        crate::note_files::write(&root, &original, tags)?;
        return Err("找不到这篇笔记，可能已被删除。".into());
    }
    Ok(())
}

#[tauri::command]
pub fn delete_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let _operation = state.data_operation.lock().map_err(|_| "数据操作状态不可用")?;
    let mut conn = state.db.lock().map_err(|_| "数据库不可用")?;
    let root = data_root(&state)?;
    crate::note_files::sync(&root, &mut conn)?;
    crate::recycle::trash_note(&mut conn, &root, &id)
}

#[tauri::command]
pub fn list_stickies(state: State<'_, AppState>) -> Result<Vec<Sticky>, String> {
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let mut stmt = sql(db.prepare(
        "SELECT id,title,content,created_at,updated_at FROM stickies ORDER BY updated_at DESC",
    ))?;
    let rows = sql(stmt.query_map([], |r| {
        Ok(Sticky {
            id: r.get(0)?,
            title: r.get(1)?,
            content: r.get(2)?,
            created_at: r.get(3)?,
            updated_at: r.get(4)?,
        })
    }))?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_sticky(state: State<'_, AppState>, title: String, content: Option<String>) -> Result<Sticky, String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let now = db::timestamp();
    let item = Sticky {
        id: db::id(),
        title: if title.trim().is_empty() {
            "新便签".into()
        } else {
            title.trim().into()
        },
        content: content.unwrap_or_default(),
        created_at: now.clone(),
        updated_at: now,
    };
    sql(db.execute(
        "INSERT INTO stickies(id,title,content,created_at,updated_at) VALUES(?1,?2,?3,?4,?4)",
        params![item.id, item.title, item.content, item.created_at],
    ))?;
    Ok(item)
}

#[tauri::command]
pub fn update_sticky(
    state: State<'_, AppState>,
    id: String,
    title: String,
    content: String,
) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let changed = sql(db.execute(
        "UPDATE stickies SET title=?1,content=?2,updated_at=?3 WHERE id=?4",
        params![title, content, db::timestamp(), id],
    ))?;
    if changed == 0 {
        return Err("找不到这条便签，可能已被删除。".into());
    }
    Ok(())
}

#[tauri::command]
pub fn delete_sticky(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let _operation = state.data_operation.lock().map_err(|_| "数据操作状态不可用")?;
    let mut conn = state.db.lock().map_err(|_| "数据库不可用")?;
    crate::recycle::trash(&mut conn, "sticky", &id)
}

#[derive(serde::Deserialize)]
pub struct CountdownInput {
    pub title: String,
    pub description: Option<String>,
    pub target_date: String,
    pub repeat_type: String,
}

fn map_countdown(r: &rusqlite::Row<'_>) -> rusqlite::Result<Countdown> {
    Ok(Countdown {
        id: r.get(0)?,
        title: r.get(1)?,
        description: r.get(2)?,
        target_date: r.get(3)?,
        repeat_type: r.get(4)?,
        created_at: r.get(5)?,
        updated_at: r.get(6)?,
    })
}

#[tauri::command]
pub fn list_countdowns(state: State<'_, AppState>) -> Result<Vec<Countdown>, String> {
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let mut stmt=sql(db.prepare("SELECT id,title,description,target_date,repeat_type,created_at,updated_at FROM countdowns ORDER BY target_date"))?;
    let rows = sql(stmt.query_map([], map_countdown))?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_countdown(
    state: State<'_, AppState>,
    value: CountdownInput,
) -> Result<Countdown, String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    if !["once", "yearly"].contains(&value.repeat_type.as_str()) {
        return Err("重复方式无效".into());
    }
    chrono::NaiveDate::parse_from_str(&value.target_date, "%Y-%m-%d")
        .map_err(|_| "日期格式无效")?;
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let now = db::timestamp();
    let item = Countdown {
        id: db::id(),
        title: value.title.trim().into(),
        description: value.description,
        target_date: value.target_date,
        repeat_type: value.repeat_type,
        created_at: now.clone(),
        updated_at: now,
    };
    sql(db.execute("INSERT INTO countdowns(id,title,description,target_date,repeat_type,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?6)",params![item.id,item.title,item.description,item.target_date,item.repeat_type,item.created_at]))?;
    Ok(item)
}

#[tauri::command]
pub fn update_countdown(state: State<'_, AppState>, value: Countdown) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    if !["once", "yearly"].contains(&value.repeat_type.as_str()) {
        return Err("重复方式无效".into());
    }
    chrono::NaiveDate::parse_from_str(&value.target_date, "%Y-%m-%d")
        .map_err(|_| "日期格式无效")?;
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let n=sql(db.execute("UPDATE countdowns SET title=?1,description=?2,target_date=?3,repeat_type=?4,updated_at=?5 WHERE id=?6",params![value.title,value.description,value.target_date,value.repeat_type,db::timestamp(),value.id]))?;
    if n == 0 {
        return Err("找不到这条倒数日。".into());
    }
    Ok(())
}

#[tauri::command]
pub fn delete_countdown(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let _operation = state.data_operation.lock().map_err(|_| "数据操作状态不可用")?;
    let mut conn = state.db.lock().map_err(|_| "数据库不可用")?;
    crate::recycle::trash(&mut conn, "countdown", &id)
}

#[derive(serde::Deserialize)]
pub struct ResourceInput {
    #[serde(default)] pub allow_duplicate: bool,
    pub name: String,
    pub kind: String,
    pub target: String,
    pub group_id: Option<String>,
    #[serde(default)] pub aliases: String,
}

fn map_resource(r: &rusqlite::Row<'_>) -> rusqlite::Result<Resource> {
    Ok(Resource {
        id: r.get(0)?,
        name: r.get(1)?,
        kind: r.get(2)?,
        target: r.get(3)?,
        group_id: r.get(4)?,
        is_favorite: r.get::<_, i64>(5)? != 0,
        icon_path: r.get(6)?,
        icon_data: None,
        created_at: r.get(7)?,
        updated_at: r.get(8)?,
        last_opened_at: r.get(9)?, aliases: r.get(10)?, usage_count: r.get(11)?,
    })
}

fn saved_icon_path(data_dir: &Path, icon_path: &str) -> Option<PathBuf> {
    let relative = Path::new(icon_path);
    if relative.parent() == Some(Path::new("icons"))
        && relative.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
    {
        Some(data_dir.join(relative))
    } else {
        None
    }
}

fn save_resource_icon(data_dir: &Path, target: &str) -> Result<Option<String>, String> {
    let resolved = if Path::new(target).extension().is_some_and(|e| e.eq_ignore_ascii_case("lnk")) {
        app_scan::resolve_shortcuts(&[target.to_string()]).ok()
            .and_then(|mut links| links.remove(&app_scan::path_key(target)))
            .filter(|link| !link.target.is_empty()).map(|link| link.target)
    } else { None };
    let bytes = resolved.as_deref().and_then(|path| app_icon::extract_icon_png(Path::new(path)))
        .or_else(|| app_icon::extract_icon_png(Path::new(target)));
    let Some(bytes) = bytes else {
        return Ok(None);
    };
    let relative = PathBuf::from("icons").join(format!("{}.png", db::id()));
    let destination = data_dir.join(&relative);
    fs::create_dir_all(data_dir.join("icons")).map_err(|e| format!("无法创建图标目录：{e}"))?;
    fs::write(&destination, bytes).map_err(|e| format!("无法保存应用图标：{e}"))?;
    Ok(Some(relative.to_string_lossy().into_owned()))
}

#[tauri::command]
pub fn list_resources(state: State<'_, AppState>) -> Result<Vec<Resource>, String> {
    let _operation = state.data_operation.lock().map_err(|_| "数据操作状态不可用")?;
    let data_dir = data_root(&state)?;
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let mut s=sql(db.prepare("SELECT id,name,kind,target,group_id,is_favorite,icon_path,created_at,updated_at,last_opened_at,aliases,usage_count FROM resources ORDER BY is_favorite DESC,name COLLATE NOCASE"))?;
    let rows = sql(s.query_map([], map_resource))?;
    let mut resources = rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    drop(s);
    for resource in &mut resources {
        if let Some(path) = resource.icon_path.as_deref().and_then(|path| saved_icon_path(&data_dir, path)) {
            if let Ok(bytes) = fs::read(path) {
                resource.icon_data = Some(app_icon::data_url(&bytes));
            }
        }
        if resource.icon_data.is_none() && resource.kind != "web" {
            if let Some(icon_path) = save_resource_icon(&data_dir, &resource.target)? {
                sql(db.execute("UPDATE resources SET icon_path=?1 WHERE id=?2", params![icon_path, resource.id]))?;
                resource.icon_data = saved_icon_path(&data_dir, &icon_path)
                    .and_then(|path| fs::read(path).ok()).map(|bytes| app_icon::data_url(&bytes));
                resource.icon_path = Some(icon_path);
            }
        }
    }
    Ok(resources)
}

#[tauri::command]
pub fn create_resource(
    state: State<'_, AppState>,
    value: ResourceInput,
) -> Result<Resource, String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    validate_resource(&value.kind, &value.target)?;
    let data_dir = data_root(&state)?;
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    if value.name.trim().is_empty(){return Err("名称不能为空".into())}
    if !value.allow_duplicate {
        let mut stmt=db.prepare("SELECT id,kind,target FROM resources").map_err(|e|e.to_string())?;
        let rows=stmt.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).map_err(|e|e.to_string())?.collect::<rusqlite::Result<Vec<_>>>().map_err(|e|e.to_string())?;
        if crate::library_tools::duplicate_id(&rows,&value.kind,&value.target).is_some(){return Err("该目标已存在，请定位已有记录或明确选择创建另一个入口".into())}
    }
    let now = db::timestamp();
    let id = db::id();
    let icon_path = if value.kind != "web" {
        save_resource_icon(&data_dir, &value.target)?
    } else {
        None
    };
    let item = Resource {
        id,
        name: value.name.trim().into(),
        kind: value.kind,
        target: value.target,
        group_id: value.group_id,
        is_favorite: false,
        last_opened_at: None, aliases: value.aliases.trim().into(), usage_count: 0,
        icon_data: icon_path.as_deref().and_then(|path| saved_icon_path(&data_dir, path))
            .and_then(|path| fs::read(path).ok()).map(|bytes| app_icon::data_url(&bytes)),
        icon_path,
        created_at: now.clone(),
        updated_at: now,
    };
    if let Err(error) = sql(db.execute("INSERT INTO resources(id,name,kind,target,group_id,is_favorite,icon_path,created_at,updated_at,aliases) VALUES(?1,?2,?3,?4,?5,0,?6,?7,?7,?8)",params![item.id,item.name,item.kind,item.target,item.group_id,item.icon_path,item.created_at,item.aliases])) {
        if let Some(path) = item.icon_path.as_deref().and_then(|path| saved_icon_path(&data_dir, path)) {
            let _ = fs::remove_file(path);
        }
        return Err(error);
    }
    Ok(item)
}

fn validate_resource(kind: &str, target: &str) -> Result<(), String> {
    match kind {
        "app" | "file" => {
            if target.trim().is_empty() {
                return Err("路径不能为空".into());
            }
        }
        "web" => {
            let u = url::Url::parse(target).map_err(|_| "网页地址无效")?;
            if !["http", "https"].contains(&u.scheme()) {
                return Err("网页只支持 HTTP 或 HTTPS 地址".into());
            }
        }
        _ => return Err("启动项类型无效".into()),
    };
    Ok(())
}

#[tauri::command]
pub fn update_resource(state: State<'_, AppState>, value: Resource) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    validate_resource(&value.kind, &value.target)?;
    let data_dir = data_root(&state)?;
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let previous: Option<(String, String, Option<String>)> = sql(db.query_row(
        "SELECT kind,target,icon_path FROM resources WHERE id=?1", [&value.id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).optional())?;
    let (old_kind, old_target, old_icon) = previous.ok_or("找不到这个启动项。")?;
    let refresh_icon = old_kind != value.kind || old_target != value.target;
    let icon_path = if refresh_icon {
        if value.kind != "web" { save_resource_icon(&data_dir, &value.target)? } else { None }
    } else { old_icon.clone() };
    let updated = sql(db.execute(
        "UPDATE resources SET name=?1,kind=?2,target=?3,group_id=?4,icon_path=?5,updated_at=?6,aliases=?8 WHERE id=?7",
        params![
            value.name,
            value.kind,
            value.target,
            value.group_id,
            icon_path,
            db::timestamp(),
            value.id, value.aliases.trim()
        ],
    ));
    if let Err(error) = updated {
        if refresh_icon {
            if let Some(path) = icon_path.as_deref().and_then(|path| saved_icon_path(&data_dir, path)) { let _ = fs::remove_file(path); }
        }
        return Err(error);
    }
    if refresh_icon {
        if let Some(path) = old_icon.as_deref().and_then(|path| saved_icon_path(&data_dir, path)) { let _ = fs::remove_file(path); }
    }
    Ok(())
}

#[tauri::command]
pub fn delete_resource(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let data_dir = data_root(&state)?;
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let icon_path: Option<String> = sql(db.query_row("SELECT icon_path FROM resources WHERE id=?1", [&id], |row| row.get(0)).optional())?.flatten();
    sql(db.execute("DELETE FROM resources WHERE id=?1", [id]))?;
    if let Some(path) = icon_path.as_deref().and_then(|path| saved_icon_path(&data_dir, path)) { let _ = fs::remove_file(path); }
    Ok(())
}

#[tauri::command]
pub fn toggle_favorite(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let n = sql(db.execute(
        "UPDATE resources SET is_favorite=1-is_favorite,updated_at=?1 WHERE id=?2",
        params![db::timestamp(), id],
    ))?;
    if n == 0 {
        Err("找不到这个启动项。".into())
    } else {
        Ok(())
    }
}

#[tauri::command]
pub fn list_groups(state: State<'_, AppState>) -> Result<Vec<ResourceGroup>, String> {
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let mut s = sql(db.prepare(
        "SELECT id,name,sort_order FROM resource_groups ORDER BY sort_order,name COLLATE NOCASE",
    ))?;
    let rows = sql(s.query_map([], |r| {
        Ok(ResourceGroup {
            id: r.get(0)?,
            name: r.get(1)?,
            sort_order: r.get(2)?,
        })
    }))?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_group(state: State<'_, AppState>, name: String) -> Result<ResourceGroup, String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    if name.trim().is_empty() {
        return Err("分组名称不能为空".into());
    }
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let order: i64 = sql(db.query_row(
        "SELECT COALESCE(MAX(sort_order),-1)+1 FROM resource_groups",
        [],
        |r| r.get(0),
    ))?;
    let group = ResourceGroup {
        id: db::id(),
        name: name.trim().into(),
        sort_order: order,
    };
    sql(db.execute(
        "INSERT INTO resource_groups(id,name,sort_order) VALUES(?1,?2,?3)",
        params![group.id, group.name, group.sort_order],
    ))?;
    Ok(group)
}

#[tauri::command]
pub fn rename_group(state: State<'_, AppState>, id: String, name: String) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    if name.trim().is_empty() {
        return Err("分组名称不能为空".into());
    }
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let n = sql(db.execute(
        "UPDATE resource_groups SET name=?1 WHERE id=?2",
        params![name.trim(), id],
    ))?;
    if n == 0 {
        Err("找不到这个分组。".into())
    } else {
        Ok(())
    }
}

#[tauri::command]
pub fn delete_group(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    sql(db.execute("DELETE FROM resource_groups WHERE id=?1", [id])).map(|_| ())
}

#[tauri::command]
pub fn move_resource(
    state: State<'_, AppState>,
    resource_id: String,
    group_id: Option<String>,
) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    sql(db.execute(
        "UPDATE resources SET group_id=?1,updated_at=?2 WHERE id=?3",
        params![group_id, db::timestamp(), resource_id],
    ))
    .map(|_| ())
}

#[tauri::command]
pub fn launch_resource(app: tauri::AppHandle, state: State<'_, AppState>, id: String) -> Result<String, String> {
    let _operation = state.data_operation.lock().map_err(|_| "数据操作状态不可用")?;
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let (kind, target): (String, String) = sql(db.query_row(
        "SELECT kind,target FROM resources WHERE id=?1",
        [&id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ))?;
    drop(db);
    validate_resource(&kind, &target)?;
    if kind == "web" || (kind == "app" && app_scan::apps_folder_id(&target).is_some()) {
        Command::new("explorer.exe")
            .arg(target)
            .spawn()
            .map_err(|e| format!("无法打开应用或网页：{e}"))?;
    } else {
        let path = Path::new(&target);
        if !path.exists() {
            return Err(format!("路径已失效：{}", path.display()));
        }
        if kind == "file"
            || path.extension().is_some_and(|x| x.eq_ignore_ascii_case("lnk"))
        {
            Command::new("explorer.exe")
                .arg(path)
                .spawn()
                .map_err(|e| format!("无法打开目标：{e}"))?;
        } else {
            let options: crate::library_tools::LaunchOptions = state.settings.lock().map_err(|_|"设置不可用")?.client.get(&format!("lumo-resource-options-{id}")).cloned().map(serde_json::from_value).transpose().map_err(|_|"启动选项无效")?.unwrap_or_default();
            let mut command=Command::new(path);
            command.args(&options.arguments);
            if !options.working_directory.is_empty(){command.current_dir(&options.working_directory);}
            command.spawn()
                .map_err(|e| format!("无法启动程序：{e}"))?;
        }
    }
    let opened_at = db::timestamp();
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    sql(db.execute("UPDATE resources SET last_opened_at=?1,usage_count=usage_count+1 WHERE id=?2", params![opened_at, id]))?;
    let _ = tauri::Emitter::emit(&app, "lumo-resources-changed", ());
    Ok(opened_at)
}

#[tauri::command]
pub fn relocate_resource(
    state: State<'_, AppState>,
    id: String,
    target: String,
) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let data_dir = data_root(&state)?;
    let db = state.db.lock().map_err(|_| "数据库不可用")?;
    let (kind, old_icon): (String, Option<String>) = sql(
        db.query_row("SELECT kind,icon_path FROM resources WHERE id=?1", [&id], |r| {
            Ok((r.get(0)?, r.get(1)?))
        }),
    )?;
    validate_resource(&kind, &target)?;
    if kind != "web" && app_scan::apps_folder_id(&target).is_none() && !Path::new(&target).exists() {
        return Err("所选路径不存在。".into());
    }
    let icon_path = if kind != "web" { save_resource_icon(&data_dir, &target)? } else { None };
    if let Err(error) = sql(db.execute(
        "UPDATE resources SET target=?1,icon_path=?2,updated_at=?3 WHERE id=?4",
        params![target, icon_path, db::timestamp(), id],
    )) {
        if let Some(path) = icon_path.as_deref().and_then(|path| saved_icon_path(&data_dir, path)) { let _ = fs::remove_file(path); }
        return Err(error);
    }
    if let Some(path) = old_icon.as_deref().and_then(|path| saved_icon_path(&data_dir, path)) { let _ = fs::remove_file(path); }
    Ok(())
}

#[tauri::command]
pub fn search_all(state: State<'_, AppState>, query: String) -> Result<Vec<SearchResult>, String> {
    let db=state.db.lock().map_err(|_|"数据库不可用")?;
    let mut result=crate::search::find_all(&db,&query)?;result.retain(|item|item.kind!="note");for (n,_) in crate::note_files::list(&data_root(&state)?)?{if format!("{} {}",n.title,n.content).to_lowercase().contains(&query.to_lowercase()){result.push(SearchResult{id:n.id,title:n.title,subtitle:crate::search::excerpt(&n.content,&query),kind:"note".into()});}}crate::search::rank(&mut result,&query);Ok(result)
}

#[tauri::command]
pub async fn resource_publishers(state: State<'_,AppState>) -> Result<std::collections::HashMap<String,String>,String> {
    let resources={
        let db=state.db.lock().map_err(|_|"数据库不可用")?;
        let mut statement=sql(db.prepare("SELECT id,target FROM resources WHERE kind='app'"))?;
        let rows=sql(statement.query_map([],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?))))?;
        sql(rows.collect::<rusqlite::Result<Vec<_>>>())?
    };
    tauri::async_runtime::spawn_blocking(move||app_scan::publishers(resources)).await.map_err(|e|e.to_string())?
}

#[tauri::command]
pub async fn scan_applications(state: State<'_,AppState>) -> Result<Vec<ScanCandidate>, String> {
    let targets={let db=state.db.lock().map_err(|_|"数据库不可用")?;let mut stmt=sql(db.prepare("SELECT target FROM resources WHERE kind='app'"))?;let rows=sql(stmt.query_map([],|r|r.get::<_,String>(0)))?;sql(rows.collect::<rusqlite::Result<Vec<_>>>())?};
    tauri::async_runtime::spawn_blocking(move||{
        let links=app_scan::resolve_shortcuts(&targets)?;
        let known:std::collections::HashSet<_>=targets.iter().map(|target|app_scan::identity(target,&links)).collect();
        let mut candidates=app_scan::scan()?;
        for candidate in &mut candidates{candidate.imported=known.contains(&app_scan::identity(&candidate.target,&links));}
        Ok(candidates)
    }).await.map_err(|error|format!("应用扫描任务失败：{error}"))?
}

#[tauri::command]
pub fn import_scan_candidates(
    state: State<'_, AppState>,
    candidates: Vec<ScanCandidate>,
) -> Result<usize, String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let data_dir = data_root(&state)?;
    let mut created_icons = Vec::<PathBuf>::new();
    let result = (|| {
        let mut db = state.db.lock().map_err(|_| "数据库不可用")?;
        let tx = sql(db.transaction())?;
        let mut count = 0;
        let now = db::timestamp();
        let existing_targets = {
            let mut statement = sql(tx.prepare("SELECT id,target FROM resources"))?;
            let rows = sql(statement.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))))?;
            rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())?
        };
        let all_paths = existing_targets.iter().map(|(_, target)| target.clone())
            .chain(candidates.iter().map(|item| item.target.clone())).collect::<Vec<_>>();
        let links = app_scan::resolve_shortcuts(&all_paths)?;
        let mut existing_ids = existing_targets.into_iter().map(|(id, target)| (app_scan::identity(&target, &links), id))
            .collect::<std::collections::HashMap<_, _>>();
        for item in candidates {
            validate_resource(&item.kind, &item.target)?;
            let identity = app_scan::identity(&item.target, &links);
            let existing_id = existing_ids.get(&identity).cloned();
            let exists: Option<(String, Option<String>)> = sql(tx
                .query_row(
                    "SELECT id,icon_path FROM resources WHERE id=?1 OR lower(target)=lower(?2)",
                    params![existing_id, item.target],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional())?;
            if let Some((id, existing_icon)) = exists {
                if existing_icon.is_none() && item.kind != "web" {
                    if let Some(path) = save_resource_icon(&data_dir, &item.target)? {
                        if let Some(absolute) = saved_icon_path(&data_dir, &path) {
                            created_icons.push(absolute);
                        }
                        sql(tx.execute("UPDATE resources SET icon_path=?1,updated_at=?2 WHERE id=?3", params![path, now, id]))?;
                        count += 1;
                    }
                }
                continue;
            }
            let id = db::id();
            let icon_path = if item.kind != "web" {
                save_resource_icon(&data_dir, &item.target)?
            } else {
                None
            };
            if let Some(path) = icon_path.as_deref().and_then(|path| saved_icon_path(&data_dir, path)) {
                created_icons.push(path);
            }
            sql(tx.execute("INSERT INTO resources(id,name,kind,target,icon_path,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?6)",params![id,item.name,item.kind,item.target,icon_path,now]))?;
            existing_ids.insert(identity, id);
            count += 1;
        }
        sql(tx.commit())?;
        Ok::<usize, String>(count)
    })();
    if result.is_err() {
        for path in created_icons {
            let _ = fs::remove_file(path);
        }
    }
    result
}

#[tauri::command]
pub fn move_data_directory(
    state: State<'_, AppState>,
    new_path: String,
) -> Result<BootstrapInfo, String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    move_data_directory_inner(&state, &new_path)
}

fn move_data_directory_inner(state: &AppState, new_path: &str) -> Result<BootstrapInfo, String> {

    let requested = PathBuf::from(new_path.trim());
    if !requested.is_absolute() {
        return Err("请选择绝对路径。".into());
    }
    let old = data_root(state)?;
    let old_real = old
        .canonicalize()
        .map_err(|e| format!("无法解析当前数据目录：{e}"))?;
    if requested == old || requested == old_real {
        return Ok(storage::bootstrap_info(&state.paths, &old));
    }
    if requested.starts_with(&old) || old.starts_with(&requested) {
        return Err(
            "新数据目录不能位于当前数据目录内，也不能包含当前数据目录。请另选一个独立目录。".into(),
        );
    }
    fs::create_dir_all(&requested).map_err(|e| format!("无法创建目标目录：{e}"))?;
    let target = requested
        .canonicalize()
        .map_err(|e| format!("无法解析目标目录：{e}"))?;
    if target == old_real || target.starts_with(&old_real) || old_real.starts_with(&target) {
        return Err("新数据目录与当前数据目录重叠，请选择另一个位置。".into());
    }
    let target_empty = fs::read_dir(&target)
        .map_err(|e| e.to_string())?
        .next()
        .is_none();
    if !target_empty {
        return Err("目标目录不是空目录。为避免覆盖已有文件，请选择空目录。".into());
    }

    let safety = old.join("backups").join(format!(
        "pre-migration-{}-{}.lumo",
        Utc::now().format("%Y%m%d-%H%M%S"),
        uuid::Uuid::new_v4()
    ));
    crate::backup::write_backup(state, &safety)?;
    let parent = target.parent().ok_or("目标目录路径无效")?;
    let stage = parent.join(format!(".lumo-hub-migration-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&stage).map_err(|e| format!("无法准备迁移暂存目录：{e}"))?;
    let mut target_installed = false;
    let result = (|| {
        let mut current_db = state.db.lock().map_err(|_| "数据库不可用")?;
        storage::copy_directory(&old, &stage)?;
        storage::ensure_data_marker(&stage)?;
        let mut staged_db = rusqlite::Connection::open(stage.join("app.db"))
            .map_err(|e| format!("无法创建迁移数据库：{e}"))?;
        {
            let backup = rusqlite::backup::Backup::new(&current_db, &mut staged_db)
                .map_err(|e| format!("无法开始迁移数据库快照：{e}"))?;
            backup
                .run_to_completion(64, Duration::from_millis(25), None)
                .map_err(|e| format!("迁移数据库快照失败：{e}"))?;
        }
        let integrity: String = staged_db
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if integrity != "ok" {
            return Err(format!("迁移后的数据库校验失败：{integrity}"));
        }
        drop(staged_db);
        fs::remove_dir(&target).map_err(|e| format!("无法准备空目标目录：{e}"))?;
        fs::rename(&stage, &target).map_err(|e| format!("无法切换迁移后的数据目录：{e}"))?;
        target_installed = true;
        let new_db = db::open(&target.join("app.db"))?;
        let mut data_dir = state.data_dir.lock().map_err(|_| "数据路径状态不可用")?;
        storage::write_bootstrap(&state.paths.bootstrap_path, &target)?;
        *current_db = new_db;
        *data_dir = target.clone();
        Ok(storage::bootstrap_info(&state.paths, &target))
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&stage);
        if target_installed {
            let _ = fs::remove_dir_all(&target);
        }
    }
    result
}

#[tauri::command]
pub fn restore_default_data_directory(app: AppHandle, state: State<'_, AppState>) -> Result<BootstrapInfo, String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let default = state.paths.default_data_dir.clone();
    let old = data_root(&state)?;
    if old != default {
        if default.join("app.db").exists() {
            if !default.join(".lumo-hub-data").exists() {
                return Err(
                    "默认数据目录缺少 Lumo-Hub 标记，已停止切换以避免打开错误目录。".into(),
                );
            }
            storage::ensure_data_marker(&default)?;
            let database_info = fs::symlink_metadata(default.join("app.db"))
                .map_err(|e| format!("无法检查默认目录数据库：{e}"))?;
            if !database_info.is_file() || database_info.file_type().is_symlink() {
                return Err("默认目录数据库不是普通文件，已停止切换。".into());
            }
            let mut new_db = db::open(&default.join("app.db"))?;
            let integrity: String = new_db
                .query_row("PRAGMA integrity_check", [], |r| r.get(0))
                .map_err(|e| e.to_string())?;
            if integrity != "ok" {
                return Err(format!("默认目录数据库校验失败：{integrity}"));
            }
            let mut new_settings = storage::load_settings(&default);crate::note_files::migrate(&default,&mut new_db,&new_settings)?;crate::note_files::load_tags(&default,&mut new_settings)?;storage::save_settings(&default,&new_settings)?;
            let mut db = state.db.lock().map_err(|_| "数据库不可用")?;
            let mut data_dir = state.data_dir.lock().map_err(|_| "数据路径状态不可用")?;
            storage::write_bootstrap(&state.paths.bootstrap_path, &default)?;
            *db = new_db;
            *data_dir = default.clone();
            *state.settings.lock().map_err(|_| "设置状态不可用")? = new_settings;
        } else {
            return move_data_directory_inner(&state, &default.to_string_lossy());
        }
    }
    crate::launcher::sync(&app);
    Ok(storage::bootstrap_info(&state.paths, &default))
}

fn common_launcher_target(id: &str) -> Result<(&'static str, &'static [&'static str], bool), String> {
    match id {
        "computer" => Ok(("explorer.exe", &["shell:MyComputerFolder"], false)),
        "recycle_bin" => Ok(("explorer.exe", &["shell:RecycleBinFolder"], false)),
        "explorer" => Ok(("explorer.exe", &[], false)),
        "cmd" => Ok(("System32/cmd.exe", &[], true)),
        "powershell" => Ok(("System32/WindowsPowerShell/v1.0/powershell.exe", &[], true)),
        "control_panel" => Ok(("System32/control.exe", &[], false)),
        "notepad" => Ok(("System32/notepad.exe", &[], false)),
        "calculator" => Ok(("System32/calc.exe", &[], false)),
        "task_manager" => Ok(("System32/taskmgr.exe", &[], false)),
        "windows_settings" => Ok(("explorer.exe", &["ms-settings:"], false)),
        _ => Err("不支持的常用启动项".into()),
    }
}

#[tauri::command]
pub fn common_launcher_icons() -> Result<std::collections::HashMap<String, String>, String> {
    let windows = std::env::var_os("SystemRoot").ok_or("无法读取 Windows 系统目录")?;
    let mut icons = std::collections::HashMap::new();
    for id in COMMON_LAUNCHER_IDS {
        let (relative, args, _) = common_launcher_target(id)?;
        let target = if matches!(*id, "computer" | "recycle_bin") {
            PathBuf::from(args[0])
        } else if *id == "control_panel" {
            PathBuf::from("shell:ControlPanelFolder")
        } else if *id == "windows_settings" {
            PathBuf::from(&windows).join("ImmersiveControlPanel/SystemSettings.exe")
        } else { PathBuf::from(&windows).join(relative) };
        if let Some(bytes) = crate::app_icon::extract_icon_png(&target) {
            icons.insert((*id).to_string(), crate::app_icon::data_url(&bytes));
        }
    }
    Ok(icons)
}

#[tauri::command]
pub fn launch_common(id: String) -> Result<(), String> {
    let (relative, args, console) = common_launcher_target(&id)?;
    let windows = std::env::var_os("SystemRoot").ok_or("无法读取 Windows 系统目录")?;
    let mut command = Command::new(PathBuf::from(windows).join(relative));
    command.args(args);
    if let Some(home) = std::env::var_os("USERPROFILE") { command.current_dir(home); }
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        if console { command.creation_flags(0x00000010); }
    }
    #[cfg(not(windows))] let _ = console;
    command.spawn().map_err(|e| format!("无法打开常用启动项：{e}"))?;
    Ok(())
}

#[cfg(test)]
mod common_launcher_tests {
    #[cfg(windows)]
    #[test]
    fn windows_common_launchers_have_native_icons() {
        let icons = super::common_launcher_icons().unwrap();
        for id in super::COMMON_LAUNCHER_IDS {
            assert!(icons.get(*id).is_some_and(|icon| icon.starts_with("data:image/png;base64,")), "Missing Windows icon: {id}");
        }
    }
    use super::*;
    #[test]
    fn builtins_are_whitelisted_and_have_no_user_commands() {
        for id in COMMON_LAUNCHER_IDS { assert!(common_launcher_target(id).is_ok()); }
        assert!(common_launcher_target("cmd /c del").is_err());
        assert!(common_launcher_target("unknown").is_err());
        assert_eq!(common_launcher_target("recycle_bin").unwrap().1, &["shell:RecycleBinFolder"]);
        assert!(common_launcher_target("cmd").unwrap().2);
    }
}
