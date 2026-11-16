use serde::Serialize;

#[derive(Serialize)]
pub struct SystemOverview {
    cpu: String,
    logical_cores: usize,
    architecture: String,
    total_memory: u64,
    used_memory: u64,
}

#[tauri::command]
pub fn system_overview() -> Result<SystemOverview, String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        let mut memory: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
        memory.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if unsafe { GlobalMemoryStatusEx(&mut memory) } == 0 {
            return Err("无法读取系统内存信息".into());
        }
        let cpu = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
            .open_subkey(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0")
            .and_then(|key| key.get_value::<String, _>("ProcessorNameString"))
            .unwrap_or_else(|_| "未知处理器".into());
        Ok(SystemOverview {
            cpu: cpu.trim().into(),
            logical_cores: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0),
            architecture: std::env::consts::ARCH.into(),
            total_memory: memory.ullTotalPhys,
            used_memory: memory.ullTotalPhys.saturating_sub(memory.ullAvailPhys),
        })
    }
    #[cfg(not(windows))]
    { Err("当前平台暂不支持读取硬件信息".into()) }
}
