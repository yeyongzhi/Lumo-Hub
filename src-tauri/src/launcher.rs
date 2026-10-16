use crate::{models::LauncherSettings, storage, AppState};
use tauri::{Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

pub fn show(app: &tauri::AppHandle) -> Result<(), String> {
    let minimal = app
        .state::<AppState>()
        .settings
        .lock()
        .map_err(|_| "设置不可用")?
        .launcher
        .minimal_mode;
    let window = match app.get_webview_window("quick-launch") {
        Some(window) => window,
        None => tauri::WebviewWindowBuilder::new(
            app,
            "quick-launch",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .data_directory(
            app.path()
                .app_local_data_dir()
                .map_err(|e| e.to_string())?
                .join("quick-launch-webview"),
        )
        .title("Lumo 快捷启动")
        .inner_size(800.0, 560.0)
        .resizable(false)
        .decorations(false)
        .devtools(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .focused(false)
        .build()
        .map_err(|e| format!("无法打开快捷面板：{e}"))?,
    };
    window
        .set_size(tauri::LogicalSize::new(
            if minimal { 1040.0 } else { 800.0 },
            if minimal { 720.0 } else { 560.0 },
        ))
        .map_err(|e| e.to_string())?;
    window
        .set_skip_taskbar(!minimal)
        .map_err(|e| e.to_string())?;
    window.center().map_err(|e| e.to_string())?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    window
        .emit("quick-launch-open", ())
        .map_err(|e| e.to_string())
}

pub fn toggle(app: &tauri::AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || toggle_inner(&app));
}

fn toggle_inner(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("quick-launch") {
        if window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false) {
            let _ = window.hide();
            return;
        }
    }
    let _ = show(app);
}

#[tauri::command]
pub async fn show_launcher(app: tauri::AppHandle) -> Result<(), String> {
    show(&app)
}

#[tauri::command]
pub async fn open_startup_view(app: tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let minimal = {
        let settings = state.settings.lock().map_err(|_| "设置不可用")?;
        settings.launcher.minimal_mode && settings.welcome_completed
    };
    if minimal
        && state
            .recovery_error
            .lock()
            .map_err(|_| "状态不可用")?
            .is_none()
    {
        show(&app)?;
        if let Some(main) = app.get_webview_window("main") {
            main.hide().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn open_main_page(
    app: tauri::AppHandle,
    page: String,
    id: Option<String>,
) -> Result<(), String> {
    if ![
        "home",
        "countdowns",
        "stickies",
        "notes",
        "library",
        "settings",
        "about",
    ]
    .contains(&page.as_str())
    {
        return Err("页面无效".into());
    }
    let main = app.get_webview_window("main").ok_or("主窗口不可用")?;
    main.unminimize().map_err(|e| e.to_string())?;
    main.show().map_err(|e| e.to_string())?;
    main.set_focus().map_err(|e| e.to_string())?;
    main.emit("lumo-navigate", serde_json::json!({"page":page,"id":id}))
        .map_err(|e| e.to_string())?;
    if let Some(panel) = app.get_webview_window("quick-launch") {
        let _ = panel.hide();
    }
    Ok(())
}
#[tauri::command]
pub fn hide_launcher(app: tauri::AppHandle) -> Result<(), String> {
    app.get_webview_window("quick-launch")
        .ok_or("快捷面板不可用")?
        .hide()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn configure_launcher(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    value: LauncherSettings,
) -> Result<crate::models::AppSettings, String> {
    let _op = state.data_operation.lock().map_err(|_| "数据操作不可用")?;
    let shortcut: Shortcut = value
        .shortcut
        .parse()
        .map_err(|e| format!("快捷键格式无效：{e}"))?;
    if value.shortcut.len() > 80 || !value.shortcut.contains('+') {
        return Err("请使用带修饰键的组合，例如 Alt+Space".into());
    }
    let mut settings = state.settings.lock().map_err(|_| "设置不可用")?;
    let old = settings.launcher.clone();
    let registered = app.global_shortcut().is_registered(shortcut);
    if value.enabled && !registered {
        app.global_shortcut()
            .register(shortcut)
            .map_err(|e| format!("无法注册快捷键，可能被其他软件占用：{e}"))?;
    }
    let mut next = settings.clone();
    next.launcher = value.clone();
    let root = state.data_dir.lock().map_err(|_| "数据目录不可用")?.clone();
    if let Err(error) = storage::save_settings(&root, &next) {
        if value.enabled && !registered {
            let _ = app.global_shortcut().unregister(shortcut);
        }
        return Err(error);
    }
    if old.enabled {
        if let Ok(key) = old.shortcut.parse::<Shortcut>() {
            if key != shortcut || !value.enabled {
                let _ = app.global_shortcut().unregister(key);
            }
        }
    }
    *settings = next;
    *state.shortcut_error.lock().map_err(|_| "状态不可用")? = None;
    Ok(settings.clone())
}

#[tauri::command]
pub fn launcher_status(state: State<'_, AppState>) -> Result<Option<String>, String> {
    Ok(state
        .shortcut_error
        .lock()
        .map_err(|_| "状态不可用")?
        .clone())
}

pub fn sync(app: &tauri::AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || sync_inner(&app));
}

fn sync_inner(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    let settings = match state.settings.lock() {
        Ok(s) => s.launcher.clone(),
        Err(_) => return,
    };
    // Called only at startup or after loading a different data directory.
    let _ = app.global_shortcut().unregister_all();
    let error = if settings.enabled {
        app.global_shortcut()
            .register(settings.shortcut.as_str())
            .err()
            .map(|e| format!("快捷键注册失败：{e}。请在设置中更换组合键。"))
    } else {
        None
    };
    if let Ok(mut status) = state.shortcut_error.lock() {
        *status = error;
    };
}
