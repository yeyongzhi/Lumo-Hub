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

pub fn bootstrap_info(paths: &StoragePaths, data_dir: &Path) -> BootstrapInfo {
    BootstrapInfo {
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
    data_dir.join("app.json")
}

pub fn load_settings(data_dir: &Path) -> AppSettings {
    let path = settings_path(data_dir);
    if let Ok(bytes) = fs::read(&path) {
        if let Ok(settings) = serde_json::from_slice(&bytes) {
            return settings;
        }
    }
    let backup = path.with_extension("json.bak");
    if let Ok(bytes) = fs::read(&backup) {
        if let Ok(settings) = serde_json::from_slice(&bytes) {
            let _ = fs::write(&path, bytes);
            return settings;
        }
    }
    AppSettings::default()
}

pub fn save_settings(data_dir: &Path, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(data_dir);
    let temp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?;
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
    for entry in walkdir::WalkDir::new(source).follow_links(false) {
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
