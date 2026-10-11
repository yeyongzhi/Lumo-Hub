mod local_files;
mod note_files;
mod desktop;
mod reminders;
mod home_feeds;
mod backup;
mod search;
mod recycle;
mod launcher;
mod automatic_backup;
mod library_actions;
mod app_icon;
mod app_scan;
mod commands;
mod db;
mod models;
mod storage;

use models::AppSettings;
use rusqlite::Connection;
use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Mutex},
};
use tauri::{Emitter, Manager};

pub struct AppState {
    pub db: Mutex<Connection>,
    pub data_dir: Mutex<PathBuf>,
    pub data_operation: Mutex<()>,
    pub clean_uninstall_requested: AtomicBool,
    pub recovery_error: Mutex<Option<String>>,
    pub paths: storage::StoragePaths,
    pub settings: Mutex<AppSettings>,
    pub shortcut_error: Mutex<Option<String>>,
    pub backup_error: Mutex<Option<String>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            let minimal=app.try_state::<AppState>().and_then(|state|state.settings.lock().ok().map(|s|s.launcher.minimal_mode)).unwrap_or(false);
            if minimal && !argv.iter().any(|arg|arg=="--clean-uninstall") {
                let app=app.clone(); tauri::async_runtime::spawn_blocking(move||{let _=launcher::show(&app);});
            } else if let Some(window)=app.get_webview_window("main") {
                let _=window.unminimize(); let _=window.show(); let _=window.set_focus();
            }
            if argv.iter().any(|arg| arg == "--clean-uninstall") {
                if let Some(state) = app.try_state::<AppState>() {
                    state
                        .clean_uninstall_requested
                        .store(true, std::sync::atomic::Ordering::SeqCst);
                }
                let _ = app.emit("lumo-clean-uninstall", ());
            }
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(|app,_,event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed { launcher::toggle(app); }
        }).build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let paths = storage::resolve_paths().map_err(std::io::Error::other)?;
            let mut recovery_error = paths.recovery_error.clone();
            let database = if recovery_error.is_none() {
                (|| {
                    for folder in ["icons", "logs", "backups"] {
                        std::fs::create_dir_all(paths.data_dir.join(folder))
                            .map_err(|e| format!("无法创建数据目录：{e}"))?;
                    }
                    db::open(&paths.data_dir.join("app.db"))
                })()
            } else {
                Err("需要恢复数据目录".to_string())
            };
            let (connection, settings) = match database {
                Ok(mut connection) => { let mut settings=storage::load_settings(&paths.data_dir); note_files::migrate(&paths.data_dir,&mut connection,&settings).map_err(std::io::Error::other)?; note_files::load_tags(&paths.data_dir,&mut settings).map_err(std::io::Error::other)?; storage::save_settings(&paths.data_dir,&settings).map_err(std::io::Error::other)?; (connection,settings) },
                Err(error) => {
                    if recovery_error.is_none() {
                        recovery_error = Some(error);
                    }
                    (
                        db::open(std::path::Path::new(":memory:"))
                            .map_err(std::io::Error::other)?,
                        AppSettings::default(),
                    )
                }
            };
            app.manage(AppState {
                db: Mutex::new(connection),
                data_dir: Mutex::new(paths.data_dir.clone()),
                data_operation: Mutex::new(()),
                clean_uninstall_requested: AtomicBool::new(false),
                recovery_error: Mutex::new(recovery_error),
                paths,
                settings: Mutex::new(settings),
                shortcut_error: Mutex::new(None), backup_error: Mutex::new(None),
            });
            desktop::setup(app.handle()).map_err(std::io::Error::other)?;
            reminders::start(app.handle().clone());
            launcher::sync(app.handle());
            automatic_backup::start(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label()=="quick-launch" {
                match event { tauri::WindowEvent::Focused(false)=>{let _=window.hide();},
                    tauri::WindowEvent::CloseRequested { api,.. }=>{api.prevent_close();let _=window.hide();}, _=>{} }
            }
            if window.label() == "main" && matches!(event, tauri::WindowEvent::Destroyed) {
                window.app_handle().exit(0);
            }
        })
        .invoke_handler(tauri::generate_handler![
            local_files::read_local,local_files::write_local,desktop::close_main, desktop::export_markdown,
            search::search_resources,
            recycle::list_trash, recycle::restore_trash, recycle::purge_trash,
            launcher::show_launcher, launcher::hide_launcher, launcher::open_startup_view, launcher::open_main_page, launcher::configure_launcher, launcher::launcher_status,
            automatic_backup::run_auto_backup, automatic_backup::auto_backup_status,
            library_actions::resource_health, library_actions::reveal_resource, library_actions::batch_resources,
            commands::bootstrap_state,
            commands::recover_data_directory,
            commands::clean_uninstall,
            commands::set_setting,
            commands::complete_welcome,
            commands::list_notes,
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            commands::list_stickies,
            commands::create_sticky,
            commands::update_sticky,
            commands::delete_sticky,
            commands::list_countdowns,
            commands::create_countdown,
            commands::update_countdown,
            commands::delete_countdown,
            commands::list_resources,
            commands::create_resource,
            commands::update_resource,
            commands::delete_resource,
            commands::toggle_favorite,
            commands::list_groups,
            commands::create_group,
            commands::rename_group,
            commands::delete_group,
            commands::move_resource,
            commands::scan_applications,
            commands::resource_publishers,
            commands::import_scan_candidates,
            commands::launch_resource,
            commands::launch_common,
            home_feeds::fetch_home_feed,
            commands::common_launcher_icons,
            commands::relocate_resource,
            commands::search_all,
            commands::move_data_directory,
            commands::restore_default_data_directory,
            backup::export_backup,
            backup::restore_backup,
        ])
        .run(tauri::generate_context!())
        .expect("Lumo-Hub 启动失败");
}
