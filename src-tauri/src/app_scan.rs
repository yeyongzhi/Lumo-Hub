use crate::{app_icon, models::ScanCandidate};
use std::{collections::{HashMap, HashSet}, path::{Path, PathBuf}};

#[derive(Default, serde::Deserialize)]
pub struct Shortcut {
    pub path: String,
    pub target: String,
    #[serde(default)]
    pub arguments: String,
}

pub fn path_key(path: &str) -> String {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));
    canonical.to_string_lossy().trim_start_matches(r"\\?\").replace('/', "\\").to_lowercase()
}

#[cfg(windows)]
pub fn resolve_shortcuts(paths: &[String]) -> Result<HashMap<String, Shortcut>, String> {
    use std::{io::Write, os::windows::process::CommandExt, process::{Command, Stdio}};
    let paths: Vec<_> = paths.iter().filter(|p| Path::new(p).extension().is_some_and(|e| e.eq_ignore_ascii_case("lnk"))).collect();
    if paths.is_empty() { return Ok(HashMap::new()); }
    let script = r#"$ErrorActionPreference='Stop'; [Console]::InputEncoding=[Text.UTF8Encoding]::new($false); [Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); $paths=ConvertFrom-Json ([Console]::In.ReadToEnd()); $shell=New-Object -ComObject WScript.Shell; $items=@(foreach($path in $paths){try{$link=$shell.CreateShortcut($path); [pscustomobject]@{path=$path;target=$link.TargetPath;arguments=$link.Arguments}}catch{}}); ConvertTo-Json -InputObject $items -Compress"#;
    let mut child = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(0x08000000)
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().map_err(|e| format!("无法解析开始菜单快捷方式：{e}"))?;
    let input = serde_json::to_vec(&paths).map_err(|e| e.to_string())?;
    child.stdin.take().ok_or("无法读取快捷方式")?.write_all(&input).map_err(|e| e.to_string())?;
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() { return Err("无法读取 Windows 快捷方式目标，请稍后重新扫描。".into()); }
    let text = String::from_utf8_lossy(&output.stdout);
    let links: Vec<Shortcut> = serde_json::from_str(text.trim().trim_start_matches('\u{feff}')).map_err(|e| format!("无法读取快捷方式结果：{e}"))?;
    Ok(links.into_iter().map(|link| (path_key(&link.path), link)).collect())
}

#[cfg(not(windows))]
pub fn resolve_shortcuts(_paths: &[String]) -> Result<HashMap<String, Shortcut>, String> { Ok(HashMap::new()) }

pub fn identity(path: &str, links: &HashMap<String, Shortcut>) -> String {
    let key = path_key(path);
    if let Some(link) = links.get(&key).filter(|link| !link.target.is_empty()) {
        format!("{}|{}", path_key(&link.target), link.arguments.trim())
    } else { format!("{key}|") }
}

fn app_name_key(name: &str) -> String {
    // Strip only trailing version/build tokens; names alone never identify an installation.
    name.split_whitespace().filter(|part| !part.chars().next().is_some_and(|c| c.is_ascii_digit()) && !matches!(part.to_lowercase().as_str(), "(x64)" | "(x86)" | "x64" | "x86"))
        .collect::<Vec<_>>().join(" ").to_lowercase()
}

pub fn category(name: &str, target: &str) -> String {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    if path_key(target).starts_with(&(path_key(&root)+"\\")) { return "system".into(); }
    let name = name.to_lowercase();
    if ["plugin", "plug-in", "extension", "runtime", "redistributable", "sdk", "插件", "扩展", "运行库"].iter().any(|word| name.contains(word)) { "extension".into() } else { "application".into() }
}
fn is_helper(name: &str) -> bool {
    let name=name.to_lowercase();
    name.contains("uninstall") || name.starts_with("unins") || name.contains("卸载")
}

pub fn scan() -> Result<Vec<ScanCandidate>, String> {
    let mut entries = Vec::new();
    let mut roots = Vec::new();
    if let Some(user) = dirs::data_dir() { roots.push((user.join("Microsoft\\Windows\\Start Menu\\Programs"), "当前用户开始菜单")); }
    if let Ok(program_data) = std::env::var("ProgramData") { roots.push((PathBuf::from(program_data).join("Microsoft\\Windows\\Start Menu\\Programs"), "所有用户开始菜单")); }
    for (root, source) in roots {
        for entry in walkdir::WalkDir::new(root).follow_links(false).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e.eq_ignore_ascii_case("lnk")) {
                entries.push((entry.path().to_string_lossy().into_owned(), source));
            }
        }
    }
    let paths = entries.iter().map(|(path, _)| path.clone()).collect::<Vec<_>>();
    let links = resolve_shortcuts(&paths)?;
    let mut seen = HashSet::new();
    let mut installed = HashSet::new();
    let mut candidates = Vec::new();
    for (path, source) in entries {
        let Some(link) = links.get(&path_key(&path)).filter(|link| !link.target.is_empty() && Path::new(&link.target).is_file()) else { continue; };
        if !Path::new(&link.target).extension().is_some_and(|e| e.eq_ignore_ascii_case("exe")) { continue; }
        if !link.arguments.trim().is_empty() || !seen.insert(path_key(&link.target)) { continue; }
        let name = Path::new(&path).file_stem().unwrap_or_default().to_string_lossy().into_owned();
        if is_helper(&name) { continue; }
        installed.insert((path_key(&link.target), app_name_key(&name)));
        let icon = app_icon::extract_icon_png(Path::new(&link.target)).or_else(|| app_icon::extract_icon_png(Path::new(&path)));
        candidates.push(ScanCandidate { imported: false, category: category(&name, &link.target), name, target: link.target.clone(), kind: "app".into(), source: source.into(), icon_data: icon.map(|bytes| app_icon::data_url(&bytes)), installation_path: Path::new(&link.target).parent().map(|dir| dir.to_string_lossy().into_owned()) });
    }
    #[cfg(windows)]
    for item in registry_candidates() {
        if installed.contains(&(path_key(&item.target), app_name_key(&item.name))) || !seen.insert(path_key(&item.target)) { continue; }
        candidates.push(item);
    }
    candidates.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(candidates)
}

#[cfg(windows)]
fn registry_candidates() -> Vec<ScanCandidate> {
    use winreg::{enums::*, RegKey};
    let mut out = Vec::new();
    for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        for view in [KEY_WOW64_64KEY, KEY_WOW64_32KEY] {
            let Ok(uninstall) = RegKey::predef(hive).open_subkey_with_flags(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall", KEY_READ | view) else { continue; };
            for key in uninstall.enum_keys().filter_map(Result::ok) {
                let Ok(app) = uninstall.open_subkey(&key) else { continue; };
                if app.get_value::<u32, _>("SystemComponent").unwrap_or(0) == 1 { continue; }
                let (Ok(name), Ok(raw)) = (app.get_value::<String, _>("DisplayName"), app.get_value::<String, _>("DisplayIcon")) else { continue; };
                let target = display_icon_path(&raw);
                let path = Path::new(&target);
                if !path.is_file() || !path.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe")) { continue; }
                let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_lowercase();
                if is_helper(&stem) || is_helper(&name) { continue; }
                let location = app.get_value::<String, _>("InstallLocation").ok().filter(|value| !value.trim().is_empty()).unwrap_or_else(|| path.parent().unwrap_or(path).to_string_lossy().into_owned());
                out.push(ScanCandidate { imported: false, category: category(&name, &target), name, target: target.clone(), kind: "app".into(), source: "Windows 已安装应用".into(), icon_data: app_icon::extract_icon_png(path).map(|bytes| app_icon::data_url(&bytes)), installation_path: Some(location) });
            }
        }
    }
    out
}

fn display_icon_path(raw: &str) -> String {
    let raw = raw.trim();
    if let Some(quoted) = raw.strip_prefix('"') { return quoted.split('"').next().unwrap_or_default().into(); }
    match raw.rsplit_once(',') {
        Some((path, index)) if index.trim().parse::<i32>().is_ok() => path.trim().into(),
        _ => raw.into(),
    }
}

#[cfg(windows)]
pub fn publishers(resources: Vec<(String,String)>) -> Result<HashMap<String,String>,String> {
    use std::{io::Write, os::windows::process::CommandExt, process::{Command,Stdio}};
    let links=resolve_shortcuts(&resources.iter().map(|(_,path)|path.clone()).collect::<Vec<_>>())?;
    let paths: Vec<_>=resources.into_iter().map(|(id,path)| {let target=links.get(&path_key(&path)).map(|link|link.target.clone()).unwrap_or(path); serde_json::json!({"id":id,"path":target})}).collect();
    if paths.is_empty(){return Ok(HashMap::new())}
    let script=r#"[Console]::InputEncoding=[Text.UTF8Encoding]::new($false); [Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); $items=ConvertFrom-Json ([Console]::In.ReadToEnd()); $result=@{}; foreach($item in $items){try{$company=(Get-Item -LiteralPath $item.path -ErrorAction Stop).VersionInfo.CompanyName; if($company -and $company.Trim()){$result[$item.id]=$company.Trim()}}catch{}}; ConvertTo-Json -InputObject $result -Compress"#;
    let mut child=Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-Command",script]).creation_flags(0x08000000).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e|e.to_string())?;
    child.stdin.take().ok_or("无法读取应用元数据")?.write_all(&serde_json::to_vec(&paths).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let output=child.wait_with_output().map_err(|e|e.to_string())?;
    if !output.status.success(){return Err("无法读取应用开发者信息".into())}
    serde_json::from_str(String::from_utf8_lossy(&output.stdout).trim().trim_start_matches('\u{feff}')).map_err(|e|e.to_string())
}
#[cfg(not(windows))]
pub fn publishers(_resources:Vec<(String,String)>)->Result<HashMap<String,String>,String>{Ok(HashMap::new())}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shortcuts_to_same_program_merge_but_arguments_remain_distinct() {
        let a = "C:/Start/A.lnk"; let b = "C:/Other/A.lnk";
        let mut links = HashMap::from([
            (path_key(a), Shortcut { path: a.into(), target: "C:/Apps/App.exe".into(), arguments: "".into() }),
            (path_key(b), Shortcut { path: b.into(), target: "c:/apps/app.exe".into(), arguments: "".into() })
        ]);
        assert_eq!(identity(a, &links), identity(b, &links));
        assert_eq!(identity(a, &links), identity("C:/Apps/App.exe", &links));
        links.get_mut(&path_key(b)).unwrap().arguments = "--profile work".into();
        assert_ne!(identity(a, &links), identity(b, &links));
        assert_ne!(identity(a, &links), identity("D:/Apps/App.exe", &links));
    }
    #[test]
    fn scan_categories_and_helpers() {
        let root=std::env::var("SystemRoot").unwrap_or_else(|_|r"C:\Windows".into());
        assert_eq!(category("记事本", &format!("{root}\\system32\\notepad.exe")),"system");
        assert_eq!(category("Example Plugin", r"D:\Apps\plugin.exe"),"extension");
        assert_eq!(category("Example", r"D:\Apps\app.exe"),"application");
        assert!(is_helper("卸载 Example"));
        assert!(!is_helper("Example"));
    }
    #[test]
    fn registry_icon_paths_and_versioned_names_are_normalized() {
        assert_eq!(display_icon_path(r#""C:\Apps\App.exe",0"#), r"C:\Apps\App.exe");
        assert_eq!(display_icon_path(r"C:\Apps\App.exe,-1"), r"C:\Apps\App.exe");
        assert_eq!(display_icon_path(r"C:\Apps, Inc\App.exe"), r"C:\Apps, Inc\App.exe");
        assert_eq!(app_name_key("语雀 4.2.4"), app_name_key("语雀"));
        assert_ne!(app_name_key("豆包工作"), app_name_key("豆包输入法"));
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "Reads installed applications on the development machine"]
    fn real_windows_scan_contains_locations_and_unique_targets() {
        let candidates = scan().unwrap();
        assert!(!candidates.is_empty());
        let links = resolve_shortcuts(&candidates.iter().map(|item| item.target.clone()).collect::<Vec<_>>()).unwrap();
        let mut identities = HashSet::new();
        for item in &candidates {
            assert!(Path::new(&item.target).extension().is_some_and(|ext|ext.eq_ignore_ascii_case("exe")));
            assert!(identities.insert(identity(&item.target, &links)), "重复扫描项：{}", item.name);
            assert!(item.installation_path.as_deref().is_some_and(|value| !value.is_empty()));
        }
        let companies=publishers(candidates.iter().map(|item|(item.target.clone(),item.target.clone())).collect()).unwrap();
        assert!(!companies.is_empty(), "未读取到任何程序开发者信息");
        println!("已读取 {} 个程序的开发者信息",companies.len());
        println!("扫描 {} 个入口，其中 {} 个带软件图标", candidates.len(), candidates.iter().filter(|item| item.icon_data.is_some()).count());
        for prefix in ["语雀", "豆包工作"] {
            println!("{prefix}：{} 个入口", candidates.iter().filter(|item| item.name.starts_with(prefix)).count());
        }
    }
}
