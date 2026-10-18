#[cfg(windows)]
fn windows_app_id(app: &tauri::AppHandle) -> Result<Option<String>, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let parent = exe.parent().ok_or("无法读取程序目录")?;
    let development = parent.ends_with("target/debug") || parent.ends_with("target/release");
    Ok((!development).then(|| app.config().identifier.clone()))
}

#[cfg(windows)]
fn ensure_enabled(app_id: Option<&str>) -> Result<(), String> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    let user = RegKey::predef(HKEY_CURRENT_USER);
    let disabled = |path: &str, value: &str| {
        user.open_subkey(path).ok().and_then(|key| key.get_value::<u32, _>(value).ok()) == Some(0)
    };
    if disabled(r"Software\Microsoft\Windows\CurrentVersion\PushNotifications", "ToastEnabled") {
        return Err("Windows 通知总开关已关闭。请点击「打开通知设置」开启通知，然后重新发送测试。".into());
    }
    let id = app_id.unwrap_or("{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe");
    if disabled(&format!(r"Software\Microsoft\Windows\CurrentVersion\Notifications\Settings\{id}"), "Enabled") {
        return Err("此应用的 Windows 通知已关闭。请在通知设置中允许通知后重试。".into());
    }
    Ok(())
}

pub fn send(app: &tauri::AppHandle, title: &str, body: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        let id = windows_app_id(app)?;
        ensure_enabled(id.as_deref())?;
        let mut notification = notify_rust::Notification::new();
        notification.summary(title).body(body).auto_icon();
        if let Some(id) = id { notification.app_id(&id); }
        // Wait for Windows to accept the toast, so failures reach the caller.
        notification.show().map_err(|e| format!("Windows 通知发送失败：{e}"))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        use tauri_plugin_notification::NotificationExt;
        app.notification().builder().title(title).body(body).show().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn open_notification_settings(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(windows)]
    {
        use tauri_plugin_opener::OpenerExt;
        app.opener().open_url("ms-settings:notifications", None::<&str>).map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    { let _ = app; Err("请在系统设置中打开通知权限".into()) }
}

