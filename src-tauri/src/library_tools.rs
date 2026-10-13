use crate::{app_icon, app_scan, AppState};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use tauri::State;

#[derive(Serialize)]
pub struct TargetPreview { name: String, target: String, kind: String, icon_data: Option<String> }

#[tauri::command]
pub async fn preview_targets(paths: Vec<String>) -> Result<Vec<TargetPreview>, String> {
    if paths.len() > 100 { return Err("一次最多添加 100 个目标".into()); }
    tauri::async_runtime::spawn_blocking(move || paths.into_iter().map(|target| {
        let path = Path::new(&target);
        if !path.exists() { return Err(format!("目标不存在：{target}")); }
        let name=path.file_stem().unwrap_or_default().to_string_lossy().into();
        if path.extension().is_some_and(|e|e.eq_ignore_ascii_case("url")) {
            let content=fs::read_to_string(path).map_err(|_|"无法读取网址快捷方式")?;
            let address=content.lines().find_map(|line|line.trim().strip_prefix("URL=")).ok_or("网址快捷方式缺少 URL")?;
            let url=url::Url::parse(address).map_err(|_|"网址快捷方式无效")?;
            if !["http","https"].contains(&url.scheme()){return Err("网址只支持 HTTP 或 HTTPS".into())}
            return Ok(TargetPreview{name,target:url.to_string(),kind:"web".into(),icon_data:None})
        }
        let kind = if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe") || e.eq_ignore_ascii_case("lnk")) { "app" } else { "file" };
        let resolved=app_scan::resolve_shortcuts(&[target.clone()]).unwrap_or_default();
        let image_target=resolved.get(&app_scan::path_key(&target)).map(|link|Path::new(&link.target)).unwrap_or(path);
        let icon_data = app_icon::extract_icon_png(image_target).map(|bytes| app_icon::data_url(&bytes));
        Ok(TargetPreview { name, target, kind: kind.into(), icon_data })
    }).collect()).await.map_err(|e|e.to_string())?
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LaunchOptions { pub arguments: Vec<String>, pub working_directory: String, pub icon_data: Option<String> }

#[tauri::command]
pub fn save_launch_options(state: State<'_, AppState>, id: String, options: LaunchOptions) -> Result<(), String> {
    if options.arguments.len()>64 || options.arguments.iter().any(|a|a.len()>8192 || a.contains('\0')) { return Err("启动参数过长或无效".into()); }
    if !options.working_directory.is_empty() && !Path::new(&options.working_directory).is_dir() { return Err("工作目录不存在".into()); }
    if options.icon_data.as_ref().is_some_and(|s|s.len()>3_000_000 || !s.starts_with("data:image/png;base64,")) { return Err("图标格式无效，请使用 PNG".into()); }
    let _op=state.data_operation.lock().map_err(|_|"操作不可用")?;
    let conn=state.db.lock().map_err(|_|"数据库不可用")?;
    let (kind,target):(String,String)=conn.query_row("SELECT kind,target FROM resources WHERE id=?1",[&id],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|e|e.to_string())?;
    if (!options.arguments.is_empty() || !options.working_directory.is_empty()) && (kind!="app" || !Path::new(&target).extension().is_some_and(|e|e.eq_ignore_ascii_case("exe"))) { return Err("高级启动选项仅支持 EXE 程序".into()); }
    drop(conn);
    let root=state.data_dir.lock().map_err(|_|"目录不可用")?.clone();
    let mut settings=state.settings.lock().map_err(|_|"设置不可用")?;
    let previous=settings.clone();
    settings.client.insert(format!("lumo-resource-options-{id}"),serde_json::to_value(options).map_err(|e|e.to_string())?);
    if let Err(e)=crate::storage::save_settings(&root,&settings){*settings=previous;return Err(e)}
    Ok(())
}

#[tauri::command]
pub fn read_icon(path: String) -> Result<String,String> {
    let file=Path::new(&path);
    if !file.extension().is_some_and(|e|e.eq_ignore_ascii_case("png")) {return Err("请选择 PNG 图片".into())}
    if fs::metadata(file).map_err(|e|e.to_string())?.len()>2_000_000 {return Err("图标不能超过 2 MB".into())}
    let bytes=fs::read(file).map_err(|e|e.to_string())?;
    let decoder=png::Decoder::new(std::io::Cursor::new(&bytes));
    let reader=decoder.read_info().map_err(|_|"PNG 图片无效")?;
    if reader.info().width>4096 || reader.info().height>4096 {return Err("图片尺寸不能超过 4096 px".into())}
    Ok(app_icon::data_url(&bytes))
}

#[tauri::command]
pub fn reorder_groups(state: State<'_,AppState>, ids: Vec<String>) -> Result<(),String> {
    let _op=state.data_operation.lock().map_err(|_|"操作不可用")?;
    let mut conn=state.db.lock().map_err(|_|"数据库不可用")?;
    let count:i64=conn.query_row("SELECT COUNT(*) FROM resource_groups",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    if ids.len() as i64!=count || ids.iter().collect::<std::collections::HashSet<_>>().len() as i64!=count {return Err("请刷新分组后重试".into())}
    let tx=conn.transaction().map_err(|e|e.to_string())?;
    for (i,id) in ids.iter().enumerate(){if tx.execute("UPDATE resource_groups SET sort_order=?1 WHERE id=?2",rusqlite::params![i as i64,id]).map_err(|e|e.to_string())?!=1{return Err("分组已不存在".into())}}
    tx.commit().map_err(|e|e.to_string())
}

pub fn target_key(kind:&str,target:&str)->String {
    if kind=="web" {url::Url::parse(target).map(|u|u.to_string()).unwrap_or_else(|_|target.into())} else {app_scan::path_key(target)}
}

pub fn duplicate_id(rows:&[(String,String,String)],kind:&str,target:&str)->Option<String>{
    if kind=="web" {let key=target_key(kind,target);return rows.iter().find(|(_,k,t)|k==kind&&target_key(k,t)==key).map(|(id,_,_)|id.clone())}
    let targets:Vec<String>=rows.iter().filter(|(_,k,_)|k!="web").map(|(_,_,t)|t.clone()).chain(std::iter::once(target.into())).collect();
    let links=app_scan::resolve_shortcuts(&targets).unwrap_or_default();
    let key=app_scan::identity(target,&links);
    rows.iter().find(|(_,k,t)|k!="web"&&app_scan::identity(t,&links)==key).map(|(id,_,_)|id.clone())
}

#[tauri::command]
pub async fn find_duplicate(state: State<'_,AppState>, kind:String, target:String, exclude:Option<String>)->Result<Option<String>,String>{
    if target.trim().is_empty(){return Ok(None)}
    let rows={let conn=state.db.lock().map_err(|_|"数据库不可用")?;let mut stmt=conn.prepare("SELECT id,kind,target FROM resources").map_err(|e|e.to_string())?;let rows=stmt.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).map_err(|e|e.to_string())?;rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e|e.to_string())?};
    tauri::async_runtime::spawn_blocking(move||duplicate_id(&rows.into_iter().filter(|(id,_,_)|exclude.as_ref()!=Some(id)).collect::<Vec<_>>(),&kind,&target)).await.map_err(|e|e.to_string())
}
