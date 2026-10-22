use crate::AppState;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::sync::{atomic::{AtomicBool, Ordering}, Mutex};
use tauri::{Emitter, Manager, State};

#[derive(Clone, Serialize, Deserialize)]
pub struct Clip { id: String, kind: String, content: String, created: String }
#[derive(Default)]
pub struct ClipboardState { entries: Mutex<Vec<Clip>>, paused: AtomicBool, access: Mutex<()> }

#[tauri::command]
// WebView2 creation must run outside the synchronous IPC/main-thread handler on Windows.
pub async fn open_builtin_tool(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let title = match id.as_str() { "color"=>"取色器 + 色板", "clipboard"=>"剪贴板", "qrcode"=>"二维码工具", "text"=>"文本工具箱", "floating"=>"悬浮文本", "timestamp"=>"时间戳工具", _=>return Err("未知工具".into()) };
    let label = format!("tool-{id}");
    if let Some(w) = app.get_webview_window(&label) { w.unminimize().map_err(|e|e.to_string())?; w.show().map_err(|e|e.to_string())?; return w.set_focus().map_err(|e|e.to_string()); }
    let floating = id == "floating";
    tauri::WebviewWindowBuilder::new(&app, label, tauri::WebviewUrl::App(format!("index.html?tool={id}").into()))
        .title(title).inner_size(if floating {420.} else {800.}, if floating {360.} else {650.})
        .min_inner_size(360., 300.).decorations(false).always_on_top(floating).build().map_err(|e|e.to_string())?;
    Ok(())
}
#[tauri::command]
pub fn save_tool_png(path: String, data: String) -> Result<(), String> {
    let encoded = data.strip_prefix("data:image/png;base64,").ok_or("无效 PNG 图片")?;
    if encoded.len() > 8 * 1024 * 1024 { return Err("图片过大".into()); }
    let bytes = STANDARD.decode(encoded).map_err(|e|e.to_string())?;
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") { return Err("无效 PNG 图片".into()); }
    std::fs::write(path, bytes).map_err(|e|e.to_string())
}
fn save(app: &tauri::AppHandle, entries: &[Clip]) -> Result<(), String> {
    let state = app.state::<AppState>();
    if state.recovery_error.lock().map_err(|e|e.to_string())?.is_some() { return Err("请先恢复数据目录".into()); }
    let _op = state.data_operation.lock().map_err(|e|e.to_string())?;
    let root = state.data_dir.lock().map_err(|e|e.to_string())?;
    crate::note_files::atomic(&root.join("tools/clipboard.json"), &serde_json::to_vec(entries).map_err(|e|e.to_string())?)
}
#[tauri::command]
pub fn clipboard_history(state: State<'_, ClipboardState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({"entries": *state.entries.lock().map_err(|e|e.to_string())?, "paused": state.paused.load(Ordering::Relaxed)}))
}
#[tauri::command]
pub fn clipboard_pause(state: State<'_, ClipboardState>, app_state: State<'_, AppState>, paused: bool) -> Result<(), String> {
    let _op = app_state.data_operation.lock().map_err(|e|e.to_string())?;
    let root = app_state.data_dir.lock().map_err(|e|e.to_string())?;
    let mut settings = app_state.settings.lock().map_err(|e|e.to_string())?;
    let mut next = settings.clone();
    next.client.insert("lumo-clipboard-paused".into(), serde_json::json!(paused));
    crate::storage::save_settings(&root, &next)?; *settings = next;
    state.paused.store(paused, Ordering::Relaxed); Ok(())
}
#[tauri::command]
pub fn clipboard_delete(app: tauri::AppHandle, state: State<'_, ClipboardState>, id: Option<String>) -> Result<(), String> {
    let mut entries = state.entries.lock().map_err(|e|e.to_string())?;
    let mut next = entries.clone();
    next.retain(|entry| id.as_ref().is_some_and(|id| &entry.id != id));
    save(&app, &next)?; *entries = next; drop(entries);
    let _ = app.emit("lumo-clipboard-changed", ()); Ok(())
}
#[tauri::command]
pub fn clipboard_copy(state: State<'_, ClipboardState>, id: Option<String>, text: Option<String>) -> Result<(), String> {
    let _access = state.access.lock().map_err(|e|e.to_string())?;
    let mut clipboard = arboard::Clipboard::new().map_err(|e|e.to_string())?;
    if let Some(text) = text { return clipboard.set_text(text).map_err(|e|e.to_string()); }
    let entries = state.entries.lock().map_err(|e|e.to_string())?;
    let clip = entries.iter().find(|e|Some(&e.id)==id.as_ref()).ok_or("记录不存在")?;
    if clip.kind == "text" { return clipboard.set_text(clip.content.clone()).map_err(|e|e.to_string()); }
    let bytes = STANDARD.decode(clip.content.split(',').nth(1).ok_or("图片无效")?).map_err(|e|e.to_string())?;
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info().map_err(|e|e.to_string())?;
    let mut data = vec![0; reader.output_buffer_size().ok_or("图片过大")?];
    let info = reader.next_frame(&mut data).map_err(|e|e.to_string())?;
    data.truncate(info.buffer_size());
    clipboard.set_image(arboard::ImageData {width:info.width as usize,height:info.height as usize,bytes:data.into()}).map_err(|e|e.to_string())
}
pub fn start(app: tauri::AppHandle) {
    let entries = { let s=app.state::<AppState>(); let root=s.data_dir.lock().unwrap(); std::fs::read(root.join("tools/clipboard.json")).ok().and_then(|b|serde_json::from_slice::<Vec<Clip>>(&b).ok()).unwrap_or_default() };
    let paused = app.state::<AppState>().settings.lock().ok().and_then(|s|s.client.get("lumo-clipboard-paused").and_then(|v|v.as_bool())).unwrap_or(false);
    app.manage(ClipboardState { entries: Mutex::new(entries), paused: AtomicBool::new(paused), ..Default::default() });
    std::thread::spawn(move || {
        let mut previous = String::new();
        #[cfg(windows)] let mut sequence = 0;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(600));
            let state=app.state::<ClipboardState>();
            if state.paused.load(Ordering::Relaxed) { continue; }
            #[cfg(windows)] {
                let current = unsafe { windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber() };
                if current == sequence { continue; }
            }
            let Ok(_access)=state.access.lock() else {continue};
            let Ok(mut clipboard)=arboard::Clipboard::new() else {continue};
            let value = if let Ok(image)=clipboard.get_image() {
                if image.bytes.len()>16*1024*1024 {continue;}
                let mut bytes=Vec::new();
                let encoded=(||->Result<(),png::EncodingError>{let mut encoder=png::Encoder::new(&mut bytes,image.width as u32,image.height as u32);encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);encoder.write_header()?.write_image_data(&image.bytes)?;Ok(())})();
                if encoded.is_err(){continue;} ("image",format!("data:image/png;base64,{}",STANDARD.encode(bytes)))
            } else if let Ok(text)=clipboard.get_text() { if text.is_empty()||text.len()>1024*1024 {continue;} ("text",text) } else { previous.clear(); continue; };
            #[cfg(windows)] { sequence = unsafe { windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber() }; }
            if value.1==previous {continue;} previous=value.1.clone();
            let Ok(mut entries)=state.entries.lock() else {continue};
            let mut next=entries.clone(); next.retain(|e|e.content!=value.1);
            next.insert(0,Clip{id:uuid::Uuid::new_v4().to_string(),kind:value.0.into(),content:value.1,created:chrono::Local::now().to_rfc3339()});
            next.truncate(200);
            while next.iter().map(|e|e.content.len()).sum::<usize>()>32*1024*1024 {next.pop();}
            if save(&app,&next).is_ok(){*entries=next;let _=app.emit("lumo-clipboard-changed",());} else {previous.clear();}
        }
    });
}
