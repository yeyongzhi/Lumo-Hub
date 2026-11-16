use serde::Serialize;
use std::{sync::Mutex, time::Instant};
use sysinfo::{Disks, Networks, System};
use tauri::State;
use tauri_plugin_opener::OpenerExt;

pub struct MetricsState(Mutex<Metrics>);
struct Metrics { system: System, networks: Networks, last: Instant, cached: Option<MetricsSnapshot> }
impl Default for MetricsState {
    fn default() -> Self {
        let mut system = System::new();
        system.refresh_cpu_usage();
        Self(Mutex::new(Metrics { system, networks: Networks::new_with_refreshed_list(), last: Instant::now(), cached: None }))
    }
}
#[derive(Clone, Serialize)]
pub struct DiskInfo { name: String, path: String, total: u64, available: u64 }
#[derive(Clone, Serialize)]
pub struct MetricsSnapshot { cpu: f32, download: u64, upload: u64, interfaces: usize, connected: Option<bool>, disks: Vec<DiskInfo> }
#[tauri::command]
pub fn home_metrics(state: State<'_, MetricsState>) -> Result<MetricsSnapshot, String> {
    let mut metrics = state.0.lock().map_err(|_| "系统指标暂不可用")?;
    let elapsed = metrics.last.elapsed().as_secs_f64().max(0.1);
    if elapsed < 3.0 { if let Some(snapshot) = &metrics.cached { return Ok(snapshot.clone()); } }
    metrics.system.refresh_cpu_usage();
    metrics.networks.refresh(true);
    let (mut download, mut upload, mut interfaces) = (0u64, 0u64, 0usize);
    for (name, data) in &metrics.networks {
        if name.to_lowercase().contains("loopback") { continue; }
        interfaces += 1;
        download += data.received(); upload += data.transmitted();
    }
    metrics.last = Instant::now();
    let disks = Disks::new_with_refreshed_list().list().iter().filter(|disk| disk.total_space() > 0).map(|disk| DiskInfo {
        name: disk.name().to_string_lossy().into_owned(), path: disk.mount_point().to_string_lossy().into_owned(), total: disk.total_space(), available: disk.available_space(),
    }).collect();
    #[cfg(windows)]
    let connected = Some(unsafe { windows_sys::Win32::Networking::WinInet::InternetGetConnectedState(std::ptr::null_mut(), 0) != 0 });
    #[cfg(not(windows))]
    let connected = None;
    let snapshot = MetricsSnapshot { connected, cpu: metrics.system.global_cpu_usage().clamp(0.0, 100.0), download: (download as f64 / elapsed) as u64, upload: (upload as f64 / elapsed) as u64, interfaces, disks };
    metrics.cached = Some(snapshot.clone());
    Ok(snapshot)
}
#[tauri::command]
pub fn open_home_disk(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let valid = Disks::new_with_refreshed_list().list().iter().any(|disk| disk.mount_point().to_string_lossy() == path);
    if !valid { return Err("磁盘不可用，请刷新后重试".into()); }
    app.opener().open_path(path, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_network_settings(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(windows)]
    { app.opener().open_url("ms-settings:network-status", None::<&str>).map_err(|e| e.to_string()) }
    #[cfg(not(windows))]
    { let _ = app; Err("当前平台暂不支持打开网络设置".into()) }
}
