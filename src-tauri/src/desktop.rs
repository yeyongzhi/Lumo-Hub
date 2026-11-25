use crate::AppState;
use tauri::{Emitter,Manager,State};
use serde::{Serialize,Deserialize};
use std::sync::atomic::{AtomicBool,Ordering};
#[derive(Serialize,Deserialize)]
struct Geometry {x:i32,y:i32,width:u32,height:u32,maximized:bool}
/// 开机自启时先留在托盘，等用户从托盘打开时再恢复上次的窗口尺寸与最大化状态。
static GEOMETRY_PENDING: AtomicBool = AtomicBool::new(false);
pub fn is_autostart_launch() -> bool { std::env::args_os().any(|arg| arg.to_string_lossy() == "--autostart") }
fn legacy_geometry_path(app:&tauri::AppHandle)->Option<std::path::PathBuf>{app.try_state::<AppState>().and_then(|s|s.data_dir.lock().ok().map(|p|p.join("window.json")))}
fn remember(app:&tauri::AppHandle){
    if let Some(window)=app.get_webview_window("main") {
        if window.is_minimized().unwrap_or(false){return}
        if let (Ok(position),Ok(size),Some(state))=(window.outer_position(),window.outer_size(),app.try_state::<AppState>()) {
            let g=Geometry{x:position.x,y:position.y,width:size.width,height:size.height,maximized:window.is_maximized().unwrap_or(false)};
            if let (Ok(mut settings),Ok(root))=(state.settings.lock(),state.data_dir.lock()){if let Ok(value)=serde_json::to_value(&g){settings.client.insert("lumo-window".into(),value);let _=crate::storage::save_settings(&root,&settings);}}
        }
    }
}
fn restore_geometry(app:&tauri::AppHandle){
    let Some(window)=app.get_webview_window("main") else {return};
    let Some(path)=legacy_geometry_path(app) else {return};
    let saved=app.try_state::<AppState>().and_then(|s|s.settings.lock().ok().and_then(|s|s.client.get("lumo-window").cloned())).and_then(|value|serde_json::from_value::<Geometry>(value).ok()).or_else(||std::fs::read(path).ok().and_then(|bytes|serde_json::from_slice::<Geometry>(&bytes).ok()));
    let Some(g)=saved else {return};
    let visible=window.available_monitors().unwrap_or_default().iter().any(|m| {let p=m.position();let s=m.size();g.x < p.x+s.width as i32 && g.x+g.width as i32 >p.x && g.y <p.y+s.height as i32 && g.y+g.height as i32>p.y});
    if visible && (900..=10000).contains(&g.width) && (640..=10000).contains(&g.height){
        let _=window.set_size(tauri::PhysicalSize::new(g.width,g.height));
        let _=window.set_position(tauri::PhysicalPosition::new(g.x,g.y));
        if g.maximized{let _=window.maximize();}
    }
}
#[cfg(windows)]
pub fn autostart(enabled:bool)->Result<(),String>{
    use winreg::{RegKey,enums::*};
    let (key,_)=RegKey::predef(HKEY_CURRENT_USER).create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run").map_err(|e|e.to_string())?;
    if enabled {
        let exe=std::env::current_exe().map_err(|e|e.to_string())?;
        // --autostart 让登录启动时只挂在托盘，不弹出主面板。
        key.set_value("Lumo-Hub",&format!("\"{}\" --autostart",exe.display())).map_err(|e|e.to_string())
    } else { match key.delete_value("Lumo-Hub"){Ok(())=>Ok(()),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(()),Err(e)=>Err(e.to_string())} }
}
#[cfg(not(windows))]
pub fn autostart(_enabled:bool)->Result<(),String>{Err("此平台尚不支持开机启动".into())}
pub fn setup(app:&tauri::AppHandle)->Result<(),String>{
    use tauri::{menu::{Menu,MenuItem},tray::{TrayIconBuilder,TrayIconEvent,MouseButton,MouseButtonState}};
    if is_autostart_launch() {
        // 开机自启：窗口保持隐藏，位置与最大化状态留到用户从托盘打开时再恢复。
        GEOMETRY_PENDING.store(true,Ordering::SeqCst);
    } else {
        restore_geometry(app);
        if let Some(window)=app.get_webview_window("main") { let _=window.show(); }
    }
    let show=MenuItem::with_id(app,"show","打开 Lumo",true,None::<&str>).map_err(|e|e.to_string())?;
    let quick=MenuItem::with_id(app,"quick","快捷启动",true,None::<&str>).map_err(|e|e.to_string())?;
    let quit=MenuItem::with_id(app,"quit","退出",true,None::<&str>).map_err(|e|e.to_string())?;
    let menu=Menu::with_items(app,&[&show,&quick,&quit]).map_err(|e|e.to_string())?;
    // Embed the tray asset directly so icon changes invalidate the Rust build.
    let decoder=png::Decoder::new(std::io::Cursor::new(include_bytes!("../icons/128x128.png")));
    let mut reader=decoder.read_info().map_err(|e|e.to_string())?;
    let mut pixels=vec![0;reader.output_buffer_size().ok_or("托盘图标过大")?];
    let info=reader.next_frame(&mut pixels).map_err(|e|e.to_string())?;
    if info.color_type!=png::ColorType::Rgba || info.bit_depth!=png::BitDepth::Eight {return Err("托盘图标必须为 RGBA8".into());}
    pixels.truncate(info.buffer_size());
    let tray_icon=tauri::image::Image::new_owned(pixels,info.width,info.height);
    for window in app.webview_windows().values() {
        window.set_icon(tray_icon.clone()).map_err(|e|e.to_string())?;
    }
    TrayIconBuilder::with_id("lumo-tray").icon(tray_icon).tooltip("Lumo · 随时启动与记录").menu(&menu).show_menu_on_left_click(false)
    .on_menu_event(|app,event|match event.id.as_ref(){"show"=>show_main(app),"quick"=>{let _=crate::launcher::show(app);},"quit"=>{remember(app);let _=app.emit("lumo-quit-requested",());},_=>{}})
    .on_tray_icon_event(|tray,event|{if matches!(event,TrayIconEvent::Click{button:MouseButton::Left,button_state:MouseButtonState::Up,..}){show_main(tray.app_handle());}}).build(app).map_err(|e|e.to_string())?;
    Ok(())
}
pub(crate) fn show_main(app:&tauri::AppHandle){
    if GEOMETRY_PENDING.swap(false,Ordering::SeqCst){restore_geometry(app);}
    if let Some(window)=app.get_webview_window("main"){let _=window.show();let _=window.unminimize();let _=window.set_focus();}
}
#[tauri::command]
pub fn close_main(app:tauri::AppHandle,state:State<'_,AppState>)->Result<(),String>{
    remember(&app);
    let hide=state.settings.lock().map_err(|_|"设置不可用")?.product.close_to_tray;
    if hide {app.get_webview_window("main").ok_or("主窗口不可用")?.hide().map_err(|e|e.to_string())}else{app.exit(0);Ok(())}
}
#[tauri::command]
pub fn export_markdown(state:State<'_,AppState>,id:String,path:String)->Result<(),String>{
    if std::path::Path::new(&path).extension().is_none_or(|e|!e.eq_ignore_ascii_case("md")){return Err("请选择 Markdown 文件".into())}
    let root=state.data_dir.lock().map_err(|_|"目录不可用")?.clone();let source=crate::note_files::find_path(&root,&id)?;let bytes=std::fs::read(source).map_err(|e|e.to_string())?;crate::note_files::atomic(std::path::Path::new(&path),&bytes)
}
