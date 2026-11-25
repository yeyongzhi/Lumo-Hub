// 桌面卡片：卡片数据、命令接口与常驻桌面的置底窗口。
// 卡片条目是引用（原文件不移动），点击条目用系统默认方式打开。
use crate::{app_icon, app_scan, commands, db, AppState};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, path::PathBuf};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

const MAX_CARDS: usize = 30;
const MAX_ITEMS_PER_CARD: usize = 500;
const MAX_ITEMS_PER_DROP: usize = 100;
const MAX_TITLE_CHARS: usize = 60;
const DEFAULT_LOGICAL_WIDTH: f64 = 320.0;
const DEFAULT_LOGICAL_HEIGHT: f64 = 420.0;
const CARD_ICONS: &[&str] = &[
    "layout-grid",
    "folder",
    "file-text",
    "globe",
    "app-window",
    "heart",
    "star",
    "home",
    "briefcase-business",
    "book-open",
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DesktopCardItem {
    pub id: String,
    pub name: String,
    pub target: String,
    pub kind: String,
    #[serde(default)]
    pub icon_path: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub original_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DesktopCard {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub items: Vec<DesktopCardItem>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub sort_order: i64,
    #[serde(default)]
    pub locked: bool,
    #[serde(default = "desktop_visible_default")]
    pub visible_on_desktop: bool,
    #[serde(default = "default_card_icon")]
    pub icon: Option<String>,
    #[serde(default)]
    pub emoji: Option<String>,
    #[serde(default)]
    pub style: DesktopCardStyle,
    pub created_at: String,
    pub updated_at: String,
}

const CARD_OPACITIES: &[&str] = &["frosted", "translucent", "solid"];
const CARD_ICON_SIZES: &[&str] = &["small", "medium", "large"];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DesktopCardStyle {
    #[serde(default = "style_opacity_default")]
    pub opacity: String,
    #[serde(default = "style_icon_size_default")]
    pub icon_size: String,
}
fn style_opacity_default() -> String {
    "frosted".into()
}
fn style_icon_size_default() -> String {
    "medium".into()
}
impl Default for DesktopCardStyle {
    fn default() -> Self {
        Self {
            opacity: style_opacity_default(),
            icon_size: style_icon_size_default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CardsFile {
    #[serde(default = "cards_schema_version")]
    schema_version: i64,
    #[serde(default)]
    cards: Vec<DesktopCard>,
}
fn cards_schema_version() -> i64 {
    1
}
fn desktop_visible_default() -> bool {
    true
}
fn default_card_icon() -> Option<String> {
    Some("layout-grid".into())
}

#[derive(Serialize)]
pub struct DesktopCardItemView {
    pub id: String,
    pub name: String,
    pub target: String,
    pub kind: String,
    pub icon_data: Option<String>,
    pub missing: bool,
    pub managed: bool,
    pub recovery_state: String,
    pub original_path: Option<String>,
}
#[derive(Serialize)]
pub struct DesktopCardView {
    pub id: String,
    pub title: String,
    pub items: Vec<DesktopCardItemView>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub sort_order: i64,
    pub locked: bool,
    pub visible_on_desktop: bool,
    pub icon: Option<String>,
    pub emoji: Option<String>,
    pub style: DesktopCardStyle,
}

fn cards_path(data_dir: &Path) -> PathBuf {
    data_dir.join("desktop_cards.json")
}

pub fn load(data_dir: &Path) -> Result<Vec<DesktopCard>, String> {
    let bytes = match fs::read(cards_path(data_dir)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("无法读取桌面卡片数据：{error}")),
    };
    let file: CardsFile =
        serde_json::from_slice(&bytes).map_err(|_| "桌面卡片数据损坏，请从备份恢复".to_string())?;
    Ok(file.cards)
}

pub fn save(data_dir: &Path, cards: &[DesktopCard]) -> Result<(), String> {
    let file = CardsFile {
        schema_version: 3,
        cards: cards.to_vec(),
    };
    let bytes = serde_json::to_vec_pretty(&file).map_err(|e| e.to_string())?;
    crate::note_files::atomic(&cards_path(data_dir), &bytes)
}

fn window_label(id: &str) -> String {
    format!("desktop-card-{id}")
}

// 卡片写操作的统一入口：串行锁 + 读取 + 变更 + 原子保存。
fn with_cards<T>(
    state: &AppState,
    change: impl FnOnce(&mut Vec<DesktopCard>) -> Result<T, String>,
) -> Result<T, String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let data_dir = state.data_dir.lock().map_err(|_| "目录不可用")?.clone();
    let mut cards = load(&data_dir)?;
    let result = change(&mut cards)?;
    save(&data_dir, &cards)?;
    Ok(result)
}

fn card_view(card: &DesktopCard, data_dir: &Path) -> DesktopCardView {
    let items = card
        .items
        .iter()
        .map(|item| DesktopCardItemView {
            id: item.id.clone(),
            name: item.name.clone(),
            target: effective_target(item).to_string(),
            kind: item.kind.clone(),
            missing: item.kind != "web" && !Path::new(effective_target(item)).exists(),
            managed: item.original_path.is_some(),
            recovery_state: if item.original_path.is_none() {
                "reference".into()
            } else if Path::new(&item.target).exists() {
                if item
                    .original_path
                    .as_deref()
                    .is_some_and(|path| Path::new(path).exists())
                {
                    "conflict".into()
                } else {
                    "stored".into()
                }
            } else if item
                .original_path
                .as_deref()
                .is_some_and(|path| Path::new(path).exists())
            {
                "interrupted".into()
            } else {
                "missing".into()
            },
            original_path: item.original_path.clone(),
            icon_data: item
                .icon_path
                .as_deref()
                .and_then(|path| commands::saved_icon_path(data_dir, path))
                .and_then(|path| fs::read(path).ok())
                .map(|bytes| app_icon::data_url(&bytes)),
        })
        .collect();
    DesktopCardView {
        id: card.id.clone(),
        title: card.title.clone(),
        items,
        x: card.x,
        y: card.y,
        width: card.width,
        height: card.height,
        sort_order: card.sort_order,
        locked: card.locked,
        visible_on_desktop: card.visible_on_desktop,
        icon: card.icon.clone(),
        emoji: card.emoji.clone(),
        style: card.style.clone(),
    }
}

// A crash between saving the recovery record and moving the Desktop file leaves
// the source untouched. Convert that record back to a normal reference on startup.
fn recover_interrupted_moves(cards: &mut [DesktopCard]) -> bool {
    let mut changed = false;
    for item in cards.iter_mut().flat_map(|card| &mut card.items) {
        let Some(original) = item.original_path.as_deref() else {
            continue;
        };
        if !Path::new(&item.target).exists() && Path::new(original).exists() {
            item.target = original.to_string();
            item.original_path = None;
            changed = true;
        }
    }
    changed
}

// The record is written before moving a desktop item, so an interrupted move remains recoverable.
fn effective_target(item: &DesktopCardItem) -> &str {
    if item.original_path.is_some() && !Path::new(&item.target).exists() {
        if let Some(original) = item
            .original_path
            .as_deref()
            .filter(|path| Path::new(path).exists())
        {
            return original;
        }
    }
    &item.target
}

fn desktop_source(path: &Path) -> bool {
    let Ok(parent) = path
        .parent()
        .ok_or(())
        .and_then(|parent| fs::canonicalize(parent).map_err(|_| ()))
    else {
        return false;
    };
    let mut desktops = Vec::new();
    if let Some(desktop) = dirs::desktop_dir() {
        desktops.push(desktop);
    }
    if let Some(public) = std::env::var_os("PUBLIC") {
        desktops.push(PathBuf::from(public).join("Desktop"));
    }
    desktops
        .iter()
        .any(|desktop| fs::canonicalize(desktop).is_ok_and(|value| value == parent))
}

fn restore_managed(item: &DesktopCardItem) -> Result<(), String> {
    let Some(original) = item.original_path.as_deref() else {
        return Ok(());
    };
    let source = Path::new(&item.target);
    let destination = Path::new(original);
    if !source.exists() {
        // 卡片内的文件已不存在（例如被手动删除或移动）：没有可移回的内容，
        // 因此视为已完成恢复，避免留下无法移除的坏条目、也无法删除整张卡片。
        return Ok(());
    }
    if destination.exists() {
        return Err(format!(
            "无法移回：桌面已有同名项目 {}，请重命名后重试",
            destination.display()
        ));
    }
    fs::rename(source, destination)
        .map_err(|error| format!("无法将 {} 移回桌面：{error}", item.name))
}

#[tauri::command]
pub fn desktop_cards_reset_position(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let (x, y, width, height) = default_geometry(&app, 0);
    with_cards(&state, |cards| {
        let card = cards
            .iter_mut()
            .find(|card| card.id == id)
            .ok_or("卡片不存在")?;
        card.x = x;
        card.y = y;
        card.width = width;
        card.height = height;
        Ok(())
    })?;
    if let Some(window) = app.get_webview_window(&window_label(&id)) {
        window
            .set_position(tauri::PhysicalPosition::new(x, y))
            .map_err(|e| e.to_string())?;
    }
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn desktop_cards_set_geometry(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let width = width.clamp(220, 600);
    let height = height.clamp(160, 800);
    let monitor = app
        .get_webview_window("main")
        .and_then(|window| window.current_monitor().ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());
    let (x, y) = if let Some(monitor) = monitor {
        let area = monitor.work_area();
        clamp_point(
            x,
            y,
            width,
            height,
            area.position.x,
            area.position.y,
            area.position.x.saturating_add(area.size.width as i32),
            area.position.y.saturating_add(area.size.height as i32),
        )
    } else {
        (x, y)
    };
    with_cards(&state, |cards| {
        let card = cards
            .iter_mut()
            .find(|card| card.id == id)
            .ok_or("卡片不存在")?;
        card.x = x;
        card.y = y;
        card.width = width;
        card.height = height;
        card.updated_at = db::timestamp();
        Ok(())
    })?;
    if let Some(window) = app.get_webview_window(&window_label(&id)) {
        window
            .set_position(tauri::PhysicalPosition::new(x, y))
            .map_err(|error| error.to_string())?;
        window
            .set_size(tauri::PhysicalSize::new(width, height))
            .map_err(|error| error.to_string())?;
    }
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn desktop_cards_reorder(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<(), String> {
    with_cards(&state, |cards| {
        if ids.len() != cards.len()
            || ids.iter().collect::<std::collections::HashSet<_>>().len() != ids.len()
            || cards.iter().any(|card| !ids.contains(&card.id))
        {
            return Err("卡片列表已变化，请刷新后重试".into());
        }
        for (order, id) in ids.iter().enumerate() {
            let card = cards
                .iter_mut()
                .find(|card| card.id == *id)
                .ok_or("卡片不存在")?;
            card.sort_order = order as i64;
            card.updated_at = db::timestamp();
        }
        Ok(())
    })?;
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn desktop_cards_reorder_items(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    item_ids: Vec<String>,
) -> Result<(), String> {
    with_cards(&state, |cards| {
        let card = cards
            .iter_mut()
            .find(|card| card.id == id)
            .ok_or("卡片不存在")?;
        if item_ids.len() != card.items.len()
            || item_ids
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                != item_ids.len()
            || card.items.iter().any(|item| !item_ids.contains(&item.id))
        {
            return Err("条目列表已变化，请刷新后重试".into());
        }
        let mut items = std::mem::take(&mut card.items);
        card.items = item_ids
            .iter()
            .map(|item_id| {
                let index = items
                    .iter()
                    .position(|item| item.id == *item_id)
                    .expect("validated item id");
                items.remove(index)
            })
            .collect();
        card.updated_at = db::timestamp();
        Ok(())
    })?;
    emit_changed(&app);
    Ok(())
}

fn desktop_card_storage_path() -> Result<PathBuf, String> {
    let desktop = dirs::desktop_dir().ok_or("无法获取桌面目录")?;
    let folder = desktop
        .parent()
        .ok_or("无法确定桌面文件夹的父目录")?
        .join("Lumo-Hub-Desktop-Cards");
    fs::create_dir_all(&folder).map_err(|error| format!("无法准备桌面卡片文件夹：{error}"))?;
    Ok(folder)
}

#[tauri::command]
pub fn desktop_cards_open_storage_folder(app: AppHandle) -> Result<String, String> {
    let folder = desktop_card_storage_path()?;
    app.opener()
        .open_path(folder.to_string_lossy().into_owned(), None::<&str>)
        .map_err(|error| error.to_string())?;
    Ok(folder.to_string_lossy().into_owned())
}

fn card_views(cards: &[DesktopCard], data_dir: &Path) -> Vec<DesktopCardView> {
    cards.iter().map(|card| card_view(card, data_dir)).collect()
}

fn spawn_sync(app: &AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = sync_windows(&handle).await {
            let _ = handle.emit("lumo-cards-error", error);
        }
    });
}

fn emit_changed(app: &AppHandle) {
    let _ = app.emit("lumo-cards-changed", ());
}

#[tauri::command]
pub fn desktop_cards_list(app: AppHandle, state: State<'_, AppState>) -> Result<Vec<DesktopCardView>, String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let data_dir = state.data_dir.lock().map_err(|_| "目录不可用")?.clone();
    let path = cards_path(&data_dir);
    let version = match fs::read(&path) {
        Ok(bytes) => serde_json::from_slice::<CardsFile>(&bytes)
            .map_err(|_| "桌面卡片数据损坏，请从备份恢复".to_string())?
            .schema_version,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
        Err(error) => return Err(format!("无法读取桌面卡片数据：{error}")),
    };
    let needs_defaults = version < 2;
    let mut cards = load(&data_dir)?;
    let mut changed = recover_interrupted_moves(&mut cards);
    if version == 2 {
        for card in &mut cards {
            if [("工作", "briefcase-business"), ("临时处理", "folder"), ("紧急要事", "star"), ("个人资料", "file-text")]
                .iter().any(|(title, icon)| card.title == *title && card.icon.as_deref() == Some(*icon)) {
                card.visible_on_desktop = false;
            }
        }
        changed = true;
    }
    if needs_defaults {
        for (title, icon) in [
            ("工作", "briefcase-business"),
            ("临时处理", "folder"),
            ("紧急要事", "star"),
            ("个人资料", "file-text"),
        ] {
            if cards.len() >= MAX_CARDS { break; }
            if cards.iter().any(|card| card.title == title) { continue; }
            let now = db::timestamp();
            let (x, y, width, height) = default_geometry(&app, cards.len());
            cards.push(DesktopCard {
                id: db::id(), title: title.into(), items: Vec::new(), x, y, width, height,
                sort_order: cards.iter().map(|card| card.sort_order).max().unwrap_or(0) + 1,
                locked: false, visible_on_desktop: false, icon: Some(icon.into()), emoji: None,
                style: DesktopCardStyle::default(), created_at: now.clone(), updated_at: now,
            });
        }
        changed = true;
    }
    if changed { save(&data_dir, &cards)?; }
    if version < 3 { spawn_sync(&app); }
    cards.sort_by(|a, b| {
        a.sort_order
            .cmp(&b.sort_order)
            .then_with(|| a.created_at.cmp(&b.created_at))
    });
    Ok(card_views(&cards, &data_dir))
}

#[tauri::command]
pub fn desktop_cards_create(
    app: AppHandle,
    state: State<'_, AppState>,
    title: Option<String>,
    icon: Option<String>,
    emoji: Option<String>,
) -> Result<DesktopCardView, String> {
    if icon.as_deref().is_some_and(|icon| !CARD_ICONS.contains(&icon)) {
        return Err("卡片图标无效".into());
    }
    if emoji
        .as_deref()
        .is_some_and(|value| value.chars().count() > 8 || value.trim().is_empty())
    {
        return Err("卡片 Emoji 无效".into());
    }
    let data_dir = state.data_dir.lock().map_err(|_| "目录不可用")?.clone();
    let card = with_cards(&state, |cards| {
        if cards.len() >= MAX_CARDS {
            return Err(format!("最多创建 {MAX_CARDS} 张卡片"));
        }
        let sort_order = cards.iter().map(|card| card.sort_order).max().unwrap_or(0) + 1;
        let title = match title
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(value) if value.chars().count() <= MAX_TITLE_CHARS => value.to_string(),
            Some(_) => return Err(format!("卡片标题不能超过 {MAX_TITLE_CHARS} 个字符")),
            None => "新卡片".to_string(),
        };
        let now = db::timestamp();
        let (x, y, width, height) = default_geometry(&app, cards.len());
        let card = DesktopCard {
            id: db::id(),
            title,
            items: Vec::new(),
            x,
            y,
            width,
            height,
            sort_order,
            locked: false,
            visible_on_desktop: true,
            icon: if emoji.is_some() { None } else { icon },
            emoji,
            style: DesktopCardStyle::default(),
            created_at: now.clone(),
            updated_at: now,
        };
        cards.push(card.clone());
        Ok(card)
    })?;
    spawn_sync(&app);
    emit_changed(&app);
    Ok(card_view(&card, &data_dir))
}

#[tauri::command]
pub fn desktop_cards_set_locked(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    locked: bool,
) -> Result<(), String> {
    with_cards(&state, |cards| {
        let card = cards
            .iter_mut()
            .find(|card| card.id == id)
            .ok_or("卡片不存在")?;
        card.locked = locked;
        card.updated_at = db::timestamp();
        Ok(())
    })?;
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn desktop_cards_rename(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    title: String,
) -> Result<(), String> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err("卡片标题不能为空".into());
    }
    if title.chars().count() > MAX_TITLE_CHARS {
        return Err(format!("卡片标题不能超过 {MAX_TITLE_CHARS} 个字符"));
    }
    with_cards(&state, |cards| {
        let card = cards
            .iter_mut()
            .find(|card| card.id == id)
            .ok_or("卡片不存在")?;
        card.title = title.clone();
        card.updated_at = db::timestamp();
        Ok(())
    })?;
    spawn_sync(&app);
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn desktop_cards_update_identity(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    title: String,
    icon: Option<String>,
    emoji: Option<String>,
) -> Result<(), String> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err("卡片标题不能为空".into());
    }
    if title.chars().count() > MAX_TITLE_CHARS {
        return Err(format!("卡片标题不能超过 {MAX_TITLE_CHARS} 个字符"));
    }
    if icon.as_deref().is_some_and(|icon| !CARD_ICONS.contains(&icon)) {
        return Err("卡片图标无效".into());
    }
    if emoji
        .as_deref()
        .is_some_and(|value| value.chars().count() > 8 || value.trim().is_empty())
    {
        return Err("卡片 Emoji 无效".into());
    }
    with_cards(&state, |cards| {
        let card = cards
            .iter_mut()
            .find(|card| card.id == id)
            .ok_or("卡片不存在")?;
        card.title = title;
        card.icon = if emoji.is_some() { None } else { icon };
        card.emoji = emoji;
        card.updated_at = db::timestamp();
        Ok(())
    })?;
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn desktop_cards_delete(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let data_dir = state.data_dir.lock().map_err(|_| "目录不可用")?.clone();
    let icons = with_cards(&state, |cards| {
        let index = cards
            .iter()
            .position(|card| card.id == id)
            .ok_or("卡片不存在")?;
        for item in &cards[index].items {
            restore_managed(item)?;
        }
        let card = cards.remove(index);
        Ok(card
            .items
            .iter()
            .filter_map(|item| {
                item.icon_path
                    .as_deref()
                    .and_then(|path| commands::saved_icon_path(&data_dir, path))
            })
            .filter(|path| {
                !cards.iter().flat_map(|card| &card.items).any(|item| {
                    item.icon_path
                        .as_deref()
                        .and_then(|icon| commands::saved_icon_path(&data_dir, icon))
                        .as_ref()
                        == Some(path)
                })
            })
            .collect::<Vec<PathBuf>>())
    })?;
    for path in icons {
        let _ = fs::remove_file(path);
    }
    if let Some(window) = app.get_webview_window(&window_label(&id)) {
        let _ = window.destroy();
    }
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn desktop_cards_set_style(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    style: DesktopCardStyle,
) -> Result<(), String> {
    if !CARD_OPACITIES.contains(&style.opacity.as_str())
        || !CARD_ICON_SIZES.contains(&style.icon_size.as_str())
    {
        return Err("卡片样式无效".into());
    }
    with_cards(&state, |cards| {
        let card = cards
            .iter_mut()
            .find(|card| card.id == id)
            .ok_or("卡片不存在")?;
        card.style = style;
        card.updated_at = db::timestamp();
        Ok(())
    })?;
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub async fn desktop_cards_set_visibility(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    visible: bool,
) -> Result<(), String> {
    with_cards(&state, |cards| {
        let card = cards
            .iter_mut()
            .find(|card| card.id == id)
            .ok_or("卡片不存在")?;
        card.visible_on_desktop = visible;
        card.updated_at = db::timestamp();
        Ok(())
    })?;
    emit_changed(&app);
    sync_windows(&app).await
}

// 交互模式切换：卡片平时不抢焦点（WS_EX_NOACTIVATE），搜索/重命名输入时临时允许激活，结束后恢复置底。
#[tauri::command]
pub fn desktop_cards_set_interactive(
    window: tauri::WebviewWindow,
    active: bool,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE,
        };
        let hwnd = window.hwnd().map_err(|error| error.to_string())?.0;
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            if active {
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style & !(WS_EX_NOACTIVATE as isize));
            } else {
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | (WS_EX_NOACTIVATE as isize));
            }
        }
        if active {
            window.set_focus().map_err(|error| error.to_string())?;
        } else {
            pin_to_desktop(&window)?;
        }
    }
    #[cfg(not(windows))]
    let _ = (window, active);
    Ok(())
}

struct PendingItem {
    name: String,
    target: String,
    kind: String,
}

fn resolve_drop_path(raw: &str) -> Result<PendingItem, String> {
    let path = Path::new(raw);
    if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("url"))
    {
        let content = fs::read_to_string(path).map_err(|_| "无法读取网址快捷方式")?;
        let address = content
            .lines()
            .find_map(|line| line.trim().strip_prefix("URL="))
            .ok_or("网址快捷方式缺少 URL")?;
        let parsed = url::Url::parse(address).map_err(|_| "网址快捷方式无效")?;
        if !["http", "https"].contains(&parsed.scheme()) {
            return Err("网址只支持 HTTP 或 HTTPS".into());
        }
        return Ok(PendingItem {
            name: path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            target: parsed.to_string(),
            kind: "web".into(),
        });
    }
    if !path.exists() {
        return Err(format!("目标不存在：{raw}"));
    }
    let name = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let is_lnk = path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("lnk"));
    let kind = if is_lnk
        || path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
    {
        "app"
    } else {
        "file"
    };
    // 快捷方式优先解析出指向的程序本体：桌面快捷方式被删除后卡片仍可启动。
    // 带启动参数的快捷方式仍指向原快捷方式，保证参数生效。
    if is_lnk {
        if let Ok(resolved) = app_scan::resolve_shortcuts(&[raw.to_string()]) {
            if let Some(link) = resolved.get(&app_scan::path_key(raw)) {
                let target = link.target.trim();
                if !target.is_empty()
                    && link.arguments.trim().is_empty()
                    && Path::new(target).exists()
                {
                    return Ok(PendingItem {
                        name,
                        target: target.to_string(),
                        kind: "app".into(),
                    });
                }
            }
        }
    }
    Ok(PendingItem {
        name,
        target: path.to_string_lossy().to_string(),
        kind: kind.into(),
    })
}

#[tauri::command]
pub fn desktop_cards_add_items(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    paths: Vec<String>,
) -> Result<usize, String> {
    if paths.is_empty() {
        return Ok(0);
    }
    if paths.len() > MAX_ITEMS_PER_DROP {
        return Err(format!("一次最多添加 {MAX_ITEMS_PER_DROP} 个目标"));
    }
    let data_dir = state.data_dir.lock().map_err(|_| "目录不可用")?.clone();
    let mut pending = Vec::with_capacity(paths.len());
    for raw in &paths {
        pending.push(resolve_drop_path(raw)?);
    }
    // 图标提取是耗时的 COM/文件操作，放在数据锁之外进行。
    let existing: Vec<(String, String)> = load(&data_dir)?
        .iter()
        .find(|card| card.id == id)
        .map(|card| {
            card.items
                .iter()
                .map(|item| (item.kind.clone(), item.target.clone()))
                .collect()
        })
        .ok_or("卡片不存在")?;
    let mut prepared = Vec::with_capacity(pending.len());
    for item in pending {
        if existing
            .iter()
            .any(|(kind, target)| *kind == item.kind && *target == item.target)
        {
            continue;
        }
        let icon_path = if item.kind != "web" {
            commands::save_resource_icon(&data_dir, &item.target)?
        } else {
            None
        };
        prepared.push((item, icon_path));
    }
    let added = with_cards(&state, |cards| {
        let card = cards
            .iter_mut()
            .find(|card| card.id == id)
            .ok_or("卡片不存在")?;
        let mut added = 0usize;
        for (item, icon_path) in prepared {
            if card.items.len() >= MAX_ITEMS_PER_CARD {
                return Err(format!("每张卡片最多容纳 {MAX_ITEMS_PER_CARD} 个条目"));
            }
            if card
                .items
                .iter()
                .any(|current| current.kind == item.kind && current.target == item.target)
            {
                continue;
            }
            card.items.push(DesktopCardItem {
                id: db::id(),
                name: item.name,
                target: item.target,
                kind: item.kind,
                icon_path,
                created_at: db::timestamp(),
                original_path: None,
            });
            added += 1;
        }
        if added > 0 {
            card.updated_at = db::timestamp();
        }
        Ok(added)
    })?;
    if added > 0 {
        spawn_sync(&app);
        emit_changed(&app);
    }
    Ok(added)
}

#[derive(Serialize)]
pub struct DropResult {
    added: usize,
    errors: Vec<String>,
}

// Desktop files are stored alongside the Desktop folder, on the same volume. Non-desktop
// paths remain references. Each item is committed independently to report partial failures.
#[tauri::command]
pub fn desktop_cards_drop_items(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    paths: Vec<String>,
) -> Result<DropResult, String> {
    if paths.len() > MAX_ITEMS_PER_DROP {
        return Err(format!("一次最多添加 {MAX_ITEMS_PER_DROP} 个目标"));
    }
    let mut result = DropResult {
        added: 0,
        errors: Vec::new(),
    };
    for raw in paths {
        let source = Path::new(&raw);
        if !desktop_source(source) {
            match desktop_cards_add_items(app.clone(), state.clone(), id.clone(), vec![raw.clone()])
            {
                Ok(n) => result.added += n,
                Err(error) => result.errors.push(format!("{raw}：{error}")),
            }
            continue;
        }
        let process = || -> Result<(), String> {
            let metadata =
                fs::symlink_metadata(source).map_err(|e| format!("无法读取文件：{e}"))?;
            if metadata.file_type().is_symlink() {
                return Err("不支持移动符号链接".into());
            }
            let filename = source.file_name().ok_or("无效的文件名")?;
            let root = source
                .parent()
                .and_then(Path::parent)
                .ok_or("无法确定桌面目录")?
                .join("Lumo-Hub-Desktop-Cards");
            let folder = root.join(db::id());
            fs::create_dir_all(&folder).map_err(|e| format!("无法创建卡片文件目录：{e}"))?;
            let destination = folder.join(filename);
            let data_dir = state.data_dir.lock().map_err(|_| "目录不可用")?.clone();
            let name = source
                .file_stem()
                .unwrap_or(filename)
                .to_string_lossy()
                .into_owned();
            let kind = if source.extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("lnk") || ext.eq_ignore_ascii_case("exe")
            }) {
                "app"
            } else {
                "file"
            };
            let icon = commands::save_resource_icon(&data_dir, &raw).ok().flatten();
            let _operation = state
                .data_operation
                .lock()
                .map_err(|_| "数据操作状态不可用")?;
            let mut cards = load(&data_dir)?;
            let card = cards
                .iter_mut()
                .find(|card| card.id == id)
                .ok_or("卡片不存在")?;
            let previous = card.items.iter().position(|item| {
                item.original_path.is_none() && item.target.eq_ignore_ascii_case(&raw)
            });
            if previous.is_none() && card.items.len() >= MAX_ITEMS_PER_CARD {
                return Err(format!("每张卡片最多容纳 {MAX_ITEMS_PER_CARD} 个条目"));
            }
            let item = DesktopCardItem {
                id: previous
                    .map(|index| card.items[index].id.clone())
                    .unwrap_or_else(db::id),
                name,
                target: destination.to_string_lossy().into_owned(),
                kind: kind.into(),
                icon_path: icon,
                created_at: db::timestamp(),
                original_path: Some(raw.clone()),
            };
            if let Some(index) = previous {
                card.items[index] = item;
            } else {
                card.items.push(item);
            }
            card.updated_at = db::timestamp();
            save(&data_dir, &cards)?;
            // If the move fails or the process stops, effective_target still points to the original.
            fs::rename(source, &destination)
                .map_err(|e| format!("文件未移入卡片（原文件仍在桌面）：{e}"))?;
            Ok(())
        };
        match process() {
            Ok(()) => result.added += 1,
            Err(error) => result.errors.push(format!("{raw}：{error}")),
        }
    }
    emit_changed(&app);
    Ok(result)
}

#[tauri::command]
pub fn desktop_cards_remove_item(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    item_id: String,
) -> Result<(), String> {
    let data_dir = state.data_dir.lock().map_err(|_| "目录不可用")?.clone();
    let icon = with_cards(&state, |cards| {
        let card = cards
            .iter_mut()
            .find(|card| card.id == id)
            .ok_or("卡片不存在")?;
        let index = card
            .items
            .iter()
            .position(|item| item.id == item_id)
            .ok_or("条目不存在")?;
        restore_managed(&card.items[index])?;
        let item = card.items.remove(index);
        card.updated_at = db::timestamp();
        let icon = item
            .icon_path
            .and_then(|path| commands::saved_icon_path(&data_dir, &path));
        Ok(icon.filter(|path| {
            !cards.iter().flat_map(|card| &card.items).any(|item| {
                item.icon_path
                    .as_deref()
                    .and_then(|icon| commands::saved_icon_path(&data_dir, icon))
                    .as_ref()
                    == Some(path)
            })
        }))
    })?;
    if let Some(path) = icon {
        let _ = fs::remove_file(path);
    }
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn desktop_cards_relocate_item(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    item_id: String,
    path: String,
) -> Result<(), String> {
    let pending = resolve_drop_path(&path)?;
    let data_dir = state.data_dir.lock().map_err(|_| "目录不可用")?.clone();
    let icon = if pending.kind != "web" {
        commands::save_resource_icon(&data_dir, &pending.target)?
    } else {
        None
    };
    with_cards(&state, |cards| {
        let card = cards
            .iter_mut()
            .find(|card| card.id == id)
            .ok_or("卡片不存在")?;
        if card.items.iter().any(|item| {
            item.id != item_id && item.kind == pending.kind && item.target == pending.target
        }) {
            return Err("该目标已在卡片中".into());
        }
        let item = card
            .items
            .iter_mut()
            .find(|item| item.id == item_id)
            .ok_or("条目不存在")?;
        if item.original_path.is_some() {
            return Err("这是移入卡片的文件，请先移回桌面后再重新添加".into());
        }
        item.name = pending.name;
        item.target = pending.target;
        item.kind = pending.kind;
        item.icon_path = icon;
        card.updated_at = db::timestamp();
        Ok(())
    })?;
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn desktop_cards_open_item(
    state: State<'_, AppState>,
    id: String,
    item_id: String,
) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let data_dir = state.data_dir.lock().map_err(|_| "目录不可用")?.clone();
    let cards = load(&data_dir)?;
    let card = cards
        .iter()
        .find(|card| card.id == id)
        .ok_or("卡片不存在")?;
    let item = card
        .items
        .iter()
        .find(|item| item.id == item_id)
        .ok_or("条目不存在")?;
    if item.kind == "web" {
        let parsed = url::Url::parse(&item.target).map_err(|_| "网址无效")?;
        if !["http", "https"].contains(&parsed.scheme()) {
            return Err("网址只支持 HTTP 或 HTTPS".into());
        }
    } else if !Path::new(effective_target(item)).exists() {
        return Err(format!("路径已失效：{}", effective_target(item)));
    }
    std::process::Command::new("explorer.exe")
        .arg(effective_target(item))
        .spawn()
        .map_err(|e| format!("无法打开目标：{e}"))?;
    Ok(())
}

#[tauri::command]
pub fn desktop_cards_reveal_item(
    state: State<'_, AppState>,
    id: String,
    item_id: String,
) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let data_dir = state.data_dir.lock().map_err(|_| "目录不可用")?.clone();
    let cards = load(&data_dir)?;
    let item = cards
        .iter()
        .find(|card| card.id == id)
        .and_then(|card| card.items.iter().find(|item| item.id == item_id))
        .ok_or("条目不存在")?;
    if item.kind == "web" {
        return Err("网址没有本地文件夹".into());
    }
    let target = effective_target(item);
    if !Path::new(target).exists() {
        return Err("路径已失效，请先找回文件".into());
    }
    tauri_plugin_opener::reveal_item_in_dir(target)
        .map_err(|error| format!("无法打开所在位置：{error}"))
}

#[tauri::command]
pub fn desktop_cards_reveal_original(
    state: State<'_, AppState>,
    id: String,
    item_id: String,
) -> Result<(), String> {
    let _operation = state
        .data_operation
        .lock()
        .map_err(|_| "数据操作状态不可用")?;
    let data_dir = state.data_dir.lock().map_err(|_| "目录不可用")?.clone();
    let cards = load(&data_dir)?;
    let item = cards
        .iter()
        .find(|card| card.id == id)
        .and_then(|card| card.items.iter().find(|item| item.id == item_id))
        .ok_or("条目不存在")?;
    let target = item
        .original_path
        .as_deref()
        .ok_or("此条目没有原桌面位置")?;
    if !Path::new(target).exists() {
        return Err("原位置不存在，请检查文件是否已移动".into());
    }
    tauri_plugin_opener::reveal_item_in_dir(target)
        .map_err(|error| format!("无法打开原位置：{error}"))
}

#[tauri::command]
pub fn desktop_cards_move_items(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    item_ids: Vec<String>,
    destination_id: String,
    before_id: Option<String>,
) -> Result<(), String> {
    if item_ids.is_empty() {
        return Ok(());
    }
    with_cards(&state, |cards| {
        let from = cards
            .iter()
            .position(|card| card.id == id)
            .ok_or("来源卡片不存在")?;
        let to = cards
            .iter()
            .position(|card| card.id == destination_id)
            .ok_or("目标卡片不存在")?;
        let selected: std::collections::HashSet<&str> =
            item_ids.iter().map(String::as_str).collect();
        if selected.len() != item_ids.len()
            || cards[from]
                .items
                .iter()
                .filter(|item| selected.contains(item.id.as_str()))
                .count()
                != selected.len()
        {
            return Err("条目已变化，请刷新后重试".into());
        }
        if from != to && cards[to].items.len() + selected.len() > MAX_ITEMS_PER_CARD {
            return Err("目标卡片空间不足".into());
        }
        if before_id
            .as_deref()
            .is_some_and(|before| selected.contains(before))
        {
            return Ok(());
        }
        if before_id
            .as_deref()
            .is_some_and(|before| !cards[to].items.iter().any(|item| item.id == before))
        {
            return Err("目标位置已变化，请刷新后重试".into());
        }
        let mut moving = Vec::new();
        cards[from].items.retain(|item| {
            if selected.contains(item.id.as_str()) {
                moving.push(item.clone());
                false
            } else {
                true
            }
        });
        let position = before_id
            .as_deref()
            .and_then(|before| cards[to].items.iter().position(|item| item.id == before))
            .unwrap_or(cards[to].items.len());
        cards[to].items.splice(position..position, moving);
        cards[from].updated_at = db::timestamp();
        cards[to].updated_at = db::timestamp();
        Ok(())
    })?;
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn desktop_cards_save_geometry(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let id = window
        .label()
        .strip_prefix("desktop-card-")
        .filter(|label| !label.is_empty())
        .ok_or("窗口与卡片不匹配")?
        .to_string();
    let width = width.round().clamp(160.0, 6000.0) as u32;
    let height = height.round().clamp(160.0, 6000.0) as u32;
    let (mut x, mut y) = (x.round() as i32, y.round() as i32);
    if let Ok(Some(monitor)) = window.current_monitor() {
        let area = monitor.work_area();
        let left = area.position.x;
        let top = area.position.y;
        let right = left + area.size.width as i32;
        let bottom = top + area.size.height as i32;
        (x, y) = clamp_point(x, y, width, height, left, top, right, bottom);
    }
    with_cards(&state, |cards| {
        let card = cards
            .iter_mut()
            .find(|card| card.id == id)
            .ok_or("卡片不存在")?;
        card.x = x;
        card.y = y;
        card.width = width;
        card.height = height;
        card.updated_at = db::timestamp();
        Ok(())
    })
}

#[derive(Clone, Copy, Debug)]
struct CardRect { x: i32, y: i32, width: u32, height: u32 }

fn card_rects_overlap(a: CardRect, b: CardRect) -> bool {
    let (ax, ay, bx, by) = (a.x as i64, a.y as i64, b.x as i64, b.y as i64);
    ax < bx + b.width as i64 && bx < ax + a.width as i64
        && ay < by + b.height as i64 && by < ay + a.height as i64
}

fn nearest_free_position(target: CardRect, area: CardRect, others: &[CardRect]) -> Option<(i32, i32)> {
    let max_x = area.x as i64 + area.width as i64 - target.width as i64;
    let max_y = area.y as i64 + area.height as i64 - target.height as i64;
    if max_x < area.x as i64 || max_y < area.y as i64 { return None; }
    let mut xs = vec![target.x as i64, area.x as i64, max_x];
    let mut ys = vec![target.y as i64, area.y as i64, max_y];
    for other in others {
        xs.extend([other.x as i64 - target.width as i64, other.x as i64 + other.width as i64]);
        ys.extend([other.y as i64 - target.height as i64, other.y as i64 + other.height as i64]);
    }
    let mut best = None;
    for x in xs { for &y in &ys {
        let x = x.clamp(area.x as i64, max_x) as i32;
        let y = y.clamp(area.y as i64, max_y) as i32;
        let rect = CardRect { x, y, ..target };
        if others.iter().any(|other| card_rects_overlap(rect, *other)) { continue; }
        let dx = x as i64 - target.x as i64;
        let dy = y as i64 - target.y as i64;
        let distance = dx * dx + dy * dy;
        if best.as_ref().is_none_or(|(old, _, _)| distance < *old) { best = Some((distance, x, y)); }
    }}
    best.map(|(_, x, y)| (x, y))
}

fn avoid_card_overlap(app: &AppHandle, window: &tauri::WebviewWindow) -> Result<(), String> {
    let monitor = window.current_monitor().map_err(|e| e.to_string())?
        .or_else(|| app.primary_monitor().ok().flatten()).ok_or("无法获取显示器工作区域")?;
    let work = monitor.work_area();
    let position = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let target = CardRect { x: position.x, y: position.y, width: size.width, height: size.height };
    let area = CardRect { x: work.position.x, y: work.position.y, width: work.size.width, height: work.size.height };
    let mut others = Vec::new();
    for (label, other) in app.webview_windows() {
        if label == window.label() || !label.starts_with("desktop-card-") { continue; }
        if let (Ok(pos), Ok(size)) = (other.outer_position(), other.outer_size()) {
            others.push(CardRect { x: pos.x, y: pos.y, width: size.width, height: size.height });
        }
    }
    if !others.iter().any(|other| card_rects_overlap(target, *other)) { return Ok(()); }
    let (x, y) = nearest_free_position(target, area, &others)
        .ok_or("桌面空间不足，无法避免卡片重叠；请移动、缩小或隐藏部分卡片")?;
    window.set_position(tauri::PhysicalPosition::new(x, y)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn desktop_cards_avoid_overlap(window: tauri::WebviewWindow, state: State<'_, AppState>) -> Result<(), String> {
    if !window.label().starts_with("desktop-card-") { return Err("窗口与卡片不匹配".into()); }
    if state.settings.lock().map_err(|_| "设置不可用")?.desktop_cards.allow_overlap { return Ok(()); }
    let _operation = state.data_operation.lock().map_err(|_| "数据操作状态不可用")?;
    avoid_card_overlap(window.app_handle(), &window)
}

// 保证卡片至少有 120 物理像素留在所在显示器工作区内。
fn clamp_point(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
) -> (i32, i32) {
    const VISIBLE: i32 = 120;
    let clamp = |value: i32, min: i32, max: i32| value.clamp(min.min(max), max.max(min));
    let min_x = left - (width as i32 - VISIBLE);
    let min_y = top - (height as i32 - VISIBLE);
    (
        clamp(x, min_x, right - VISIBLE),
        clamp(y, min_y, bottom - VISIBLE),
    )
}

fn default_geometry(app: &AppHandle, index: usize) -> (i32, i32, u32, u32) {
    let monitor = app
        .get_webview_window("main")
        .and_then(|main| main.current_monitor().ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return (
            200 + (index % 6) as i32 * 28,
            120 + (index % 6) as i32 * 28,
            480,
            630,
        );
    };
    let scale = monitor.scale_factor();
    let width = (DEFAULT_LOGICAL_WIDTH * scale) as u32;
    let height = (DEFAULT_LOGICAL_HEIGHT * scale) as u32;
    let area = monitor.work_area();
    let cascade = (index % 6) as i32 * (28.0 * scale) as i32;
    let x =
        area.position.x + area.size.width as i32 - width as i32 - (48.0 * scale) as i32 - cascade;
    let y = area.position.y + (48.0 * scale) as i32 + cascade;
    (x, y, width, height)
}

// 按当前设置与数据同步桌面卡片窗口：开启时逐一创建/复位并置底，关闭或删除时销毁。
#[tauri::command]
pub async fn desktop_cards_sync(app: AppHandle) -> Result<(), String> {
    sync_windows(&app).await
}

pub async fn sync_windows(app: &AppHandle) -> Result<(), String> {
    let state = app.try_state::<AppState>().ok_or("应用状态不可用")?;
    if state
        .recovery_error
        .lock()
        .map_err(|_| "恢复状态不可用")?
        .is_some()
    {
        return Err("数据目录需要恢复，无法显示桌面卡片".into());
    }
    let enabled = state
        .settings
        .lock()
        .map_err(|_| "设置不可用")?
        .desktop_cards
        .enabled;
    let data_dir = state.data_dir.lock().map_err(|_| "目录不可用")?.clone();
    let mut stored_cards = load(&data_dir)?;
    let mut changed = recover_interrupted_moves(&mut stored_cards);
    let main = app.get_webview_window("main");
    let monitors = main
        .as_ref()
        .and_then(|window| window.available_monitors().ok())
        .unwrap_or_default();
    let fallback_monitor = main
        .as_ref()
        .and_then(|window| window.current_monitor().ok().flatten())
        .or_else(|| monitors.first().cloned());
    if let Some(monitor) = fallback_monitor {
        let area = monitor.work_area();
        for card in &mut stored_cards {
            let intersects = monitors.iter().any(|monitor| {
                let area = monitor.work_area();
                card.x as i64 + card.width as i64 > area.position.x as i64
                    && (card.x as i64) < area.position.x as i64 + area.size.width as i64
                    && card.y as i64 + card.height as i64 > area.position.y as i64
                    && (card.y as i64) < area.position.y as i64 + area.size.height as i64
            });
            if !intersects {
                let (x, y) = clamp_point(
                    area.position.x + 48,
                    area.position.y + 48,
                    card.width,
                    card.height,
                    area.position.x,
                    area.position.y,
                    area.position.x.saturating_add(area.size.width as i32),
                    area.position.y.saturating_add(area.size.height as i32),
                );
                card.x = x;
                card.y = y;
                card.updated_at = db::timestamp();
                changed = true;
            }
        }
    }
    if changed {
        save(&data_dir, &stored_cards)?;
    }
    let cards = if enabled {
        stored_cards
            .into_iter()
            .filter(|card| card.visible_on_desktop)
            .collect()
    } else {
        Vec::new()
    };
    let active: Vec<String> = cards.iter().map(|card| window_label(&card.id)).collect();
    let windows: Vec<(String, tauri::WebviewWindow)> = app
        .webview_windows()
        .into_iter()
        .filter(|(label, _)| label.starts_with("desktop-card-"))
        .collect();
    for (label, window) in &windows {
        if !active.contains(label) {
            let _ = window.destroy();
        }
    }
    for card in &cards {
        ensure_window(app, card)
            .await
            .map_err(|error| format!("卡片「{}」显示失败：{error}", card.title))?;
    }
    let allow_overlap = state.settings.lock().map_err(|_| "设置不可用")?.desktop_cards.allow_overlap;
    if enabled && !allow_overlap {
        let _operation = state.data_operation.lock().map_err(|_| "数据操作状态不可用")?;
        for card in &cards {
            if let Some(window) = app.get_webview_window(&window_label(&card.id)) {
                avoid_card_overlap(app, &window)?;
            }
        }
    }
    Ok(())
}

async fn ensure_window(app: &AppHandle, card: &DesktopCard) -> Result<(), String> {
    let label = window_label(&card.id);
    let window = match app.get_webview_window(&label) {
        Some(window) => window,
        None => {
            // A disconnected display must not leave a restored card completely unreachable.
            let main = app.get_webview_window("main");
            let monitors = main
                .as_ref()
                .and_then(|window| window.available_monitors().ok())
                .unwrap_or_default();
            let intersects = monitors.iter().any(|monitor| {
                let area = monitor.work_area();
                card.x as i64 + card.width as i64 > area.position.x as i64
                    && (card.x as i64) < area.position.x as i64 + area.size.width as i64
                    && card.y as i64 + card.height as i64 > area.position.y as i64
                    && (card.y as i64) < area.position.y as i64 + area.size.height as i64
            });
            let (x, y) = if !intersects && !monitors.is_empty() {
                let monitor = main
                    .as_ref()
                    .and_then(|window| window.current_monitor().ok().flatten())
                    .unwrap_or_else(|| monitors[0].clone());
                let area = monitor.work_area();
                clamp_point(
                    area.position.x + 48,
                    area.position.y + 48,
                    card.width,
                    card.height,
                    area.position.x,
                    area.position.y,
                    area.position.x.saturating_add(area.size.width as i32),
                    area.position.y.saturating_add(area.size.height as i32),
                )
            } else {
                (card.x, card.y)
            };
            // 仅在新建时应用存储的几何信息，避免覆盖用户正在进行的拖动。
            let scale = app
                .get_webview_window("main")
                .and_then(|main| main.current_monitor().ok().flatten())
                .map(|monitor| monitor.scale_factor())
                .unwrap_or(1.0);
            // 以可见状态创建窗口：隐藏窗口上 WebView2 可能延迟初始化拖放子窗口。
            let window = tauri::WebviewWindowBuilder::new(
                app,
                &label,
                tauri::WebviewUrl::App(format!("index.html?desktop-card={}", card.id).into()),
            )
            .title("Lumo 桌面卡片")
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .background_color(tauri::webview::Color(0, 0, 0, 0))
            .min_inner_size(220.0, 60.0)
            .max_inner_size(600.0, 800.0)
            .maximizable(false)
            .resizable(true)
            .skip_taskbar(true)
            .visible(true)
            .focused(false)
            .position(x as f64 / scale, y as f64 / scale)
            .inner_size(
                (card.width as f64 / scale).clamp(220.0, 600.0),
                (card.height as f64 / scale).clamp(120.0, 800.0),
            )
            .build()
            .map_err(|error| format!("无法创建桌面卡片窗口：{error}"))?;
            window
                .set_position(tauri::PhysicalPosition::new(x, y))
                .map_err(|error| error.to_string())?;
            window
                .set_size(tauri::LogicalSize::new(
                    (card.width as f64 / scale).clamp(220.0, 600.0),
                    (card.height as f64 / scale).clamp(120.0, 800.0),
                ))
                .map_err(|error| error.to_string())?;
            window
        }
    };
    window
        .set_maximizable(false)
        .map_err(|error| error.to_string())?;
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize().map_err(|error| error.to_string())?;
    }
    window.show().map_err(|error| error.to_string())?;
    pin_to_desktop(&window)?;
    if let Ok(raw_hwnd) = window.hwnd() {
        card_drop_target::register(&window.app_handle(), &label, raw_hwnd.0 as isize);
    }
    Ok(())
}

#[cfg(not(windows))]
fn pin_to_desktop(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}

// wry 在创建 webview 时一次性向子窗口注入拖放目标，但 WebView2 的输入窗口（Chrome_WidgetWin_1）
// 在桌面卡片上未被注入（主窗口正常），导致拖放命中测试落空。这里自行实现 IDropTarget 并补注册，
// 直接向卡片 webview 转发 tauri 拖放事件。
#[cfg(windows)]
mod card_drop_target {
    use std::{
        ffi::c_void,
        sync::atomic::{AtomicBool, AtomicU32, Ordering},
    };
    use tauri::Emitter;
    use windows::{
        core::implement,
        Win32::{
            Foundation::{HWND, POINT, POINTL},
            Graphics::Gdi::ScreenToClient,
            System::{
                Com::{IDataObject, DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL},
                Ole::{
                    IDropTarget, IDropTarget_Impl, RegisterDragDrop, ReleaseStgMedium,
                    RevokeDragDrop, CF_HDROP, DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_NONE,
                },
                SystemServices::MODIFIERKEYS_FLAGS,
            },
            UI::Shell::{DragQueryFileW, HDROP},
        },
    };

    #[implement(IDropTarget)]
    struct CardDropTarget {
        app: tauri::AppHandle,
        label: String,
        window: isize,
        enter_is_valid: AtomicBool,
        cursor_effect: AtomicU32,
    }

    fn extract_paths(data_obj: windows::core::Ref<'_, IDataObject>) -> Option<Vec<String>> {
        let data = data_obj.as_ref()?;
        let format = FORMATETC {
            cfFormat: CF_HDROP.0,
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0,
            lindex: -1,
            tymed: TYMED_HGLOBAL.0 as u32,
        };
        let mut medium = unsafe { data.GetData(&format) }.ok()?;
        let hdrop = HDROP(unsafe { medium.u.hGlobal }.0);
        let count = unsafe { DragQueryFileW(hdrop, u32::MAX, None) };
        let mut paths = Vec::with_capacity(count as usize);
        for index in 0..count {
            let length = unsafe { DragQueryFileW(hdrop, index, None) } as usize;
            let mut buffer = vec![0u16; length + 1];
            unsafe { DragQueryFileW(hdrop, index, Some(&mut buffer)) };
            paths.push(String::from_utf16_lossy(&buffer[..length]));
        }
        // GetData transfers ownership of the STGMEDIUM to this caller. CF_HDROP
        // obtained this way must be released with ReleaseStgMedium; DragFinish is
        // only for handles delivered through WM_DROPFILES.
        unsafe { ReleaseStgMedium(&mut medium) };
        Some(paths)
    }

    fn client_position(window: isize, point: &POINTL) -> (i32, i32) {
        let mut client = POINT {
            x: point.x,
            y: point.y,
        };
        let _ = unsafe { ScreenToClient(HWND(window as *mut c_void), &mut client) };
        (client.x, client.y)
    }

    impl CardDropTarget {
        fn emit(&self, event: &str, payload: serde_json::Value) {
            let _ = self.app.emit_to(
                tauri::EventTarget::labeled(self.label.clone()),
                event,
                payload,
            );
        }
    }

    impl IDropTarget_Impl for CardDropTarget_Impl {
        fn DragEnter(
            &self,
            pdataobj: windows::core::Ref<'_, IDataObject>,
            _grfkeystate: MODIFIERKEYS_FLAGS,
            pt: &POINTL,
            pdweffect: *mut DROPEFFECT,
        ) -> windows::core::Result<()> {
            let Some(paths) = extract_paths(pdataobj) else {
                self.enter_is_valid.store(false, Ordering::SeqCst);
                self.cursor_effect.store(0, Ordering::SeqCst);
                unsafe { *pdweffect = DROPEFFECT_NONE };
                return Ok(());
            };
            self.enter_is_valid.store(true, Ordering::SeqCst);
            self.cursor_effect
                .store(DROPEFFECT_COPY.0, Ordering::SeqCst);
            let (x, y) = client_position(self.window, pt);
            self.emit(
                "tauri://drag-enter",
                serde_json::json!({"paths": paths, "position": {"x": x, "y": y}}),
            );
            unsafe { *pdweffect = DROPEFFECT_COPY };
            Ok(())
        }

        fn DragOver(
            &self,
            _grfkeystate: MODIFIERKEYS_FLAGS,
            pt: &POINTL,
            pdweffect: *mut DROPEFFECT,
        ) -> windows::core::Result<()> {
            if self.enter_is_valid.load(Ordering::SeqCst) {
                let (x, y) = client_position(self.window, pt);
                self.emit(
                    "tauri://drag-over",
                    serde_json::json!({"position": {"x": x, "y": y}}),
                );
            }
            unsafe { *pdweffect = DROPEFFECT(self.cursor_effect.load(Ordering::SeqCst)) };
            Ok(())
        }

        fn DragLeave(&self) -> windows::core::Result<()> {
            if self.enter_is_valid.swap(false, Ordering::SeqCst) {
                self.emit("tauri://drag-leave", serde_json::json!(null));
            }
            Ok(())
        }

        fn Drop(
            &self,
            pdataobj: windows::core::Ref<'_, IDataObject>,
            _grfkeystate: MODIFIERKEYS_FLAGS,
            pt: &POINTL,
            _pdweffect: *mut DROPEFFECT,
        ) -> windows::core::Result<()> {
            if !self.enter_is_valid.swap(false, Ordering::SeqCst) {
                return Ok(());
            }
            let paths = extract_paths(pdataobj).unwrap_or_default();
            let (x, y) = client_position(self.window, pt);
            self.emit(
                "tauri://drag-drop",
                serde_json::json!({"paths": paths, "position": {"x": x, "y": y}}),
            );
            Ok(())
        }
    }

    // 在卡片窗口及其全部子窗口上注册自实现拖放目标（补齐 wry 的注入）。
    pub fn register(app: &tauri::AppHandle, label: &str, window: isize) {
        use windows_sys::Win32::UI::WindowsAndMessaging::EnumChildWindows as SysEnumChildren;
        let target: IDropTarget = CardDropTarget {
            app: app.clone(),
            label: label.to_string(),
            window,
            enter_is_valid: AtomicBool::new(false),
            cursor_effect: AtomicU32::new(0),
        }
        .into();
        let keep_alive = target.clone();
        std::mem::forget(keep_alive);
        let mut callback = |child: *mut c_void| unsafe {
            let _ = RevokeDragDrop(HWND(child));
            let _ = RegisterDragDrop(HWND(child), &target);
        };
        let mut trait_obj: &mut dyn FnMut(*mut c_void) = &mut callback;
        let closure_pointer_pointer: *mut c_void = unsafe { std::mem::transmute(&mut trait_obj) };
        unsafe extern "system" fn enumerate(child: *mut c_void, lparam: isize) -> i32 {
            let closure = &mut *(lparam as *mut c_void as *mut &mut dyn FnMut(*mut c_void));
            closure(child);
            1
        }
        unsafe {
            SysEnumChildren(
                window as *mut c_void,
                Some(enumerate),
                closure_pointer_pointer as isize,
            );
            let _ = RevokeDragDrop(HWND(window as *mut c_void));
            let _ = RegisterDragDrop(HWND(window as *mut c_void), &target);
        }
    }
}

// 桌面置底：工具窗口样式（不进任务栏/Alt+Tab、点击不抢焦点），并插入到桌面窗口之上、其余窗口之下。
#[cfg(windows)]
fn pin_to_desktop(window: &tauri::WebviewWindow) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FindWindowW, GetWindow, GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE,
        GW_HWNDPREV, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE,
        WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    };
    let hwnd = window.hwnd().map_err(|error| error.to_string())?.0;
    let class = wide("Progman");
    unsafe {
        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            GetWindowLongPtrW(hwnd, GWL_EXSTYLE) | (WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE) as isize,
        );
        // 插入到 Progman（桌面）正上方：取其 Z 序上一邻窗口作为插入锚点，
        // 保证卡片位于桌面之上、其余普通窗口之下。
        let desktop = FindWindowW(class.as_ptr(), std::ptr::null());
        let anchor = if desktop.is_null() {
            desktop
        } else {
            GetWindow(desktop, GW_HWNDPREV)
        };
        let anchor = if anchor.is_null() {
            HWND_BOTTOM
        } else {
            anchor
        };
        SetWindowPos(
            hwnd,
            anchor,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        );
    }
    Ok(())
}

#[cfg(windows)]
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Folder(std::path::PathBuf);
    impl Folder {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("lumo-cards-test-{}", db::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn free_position_avoids_overlap_and_reports_full_area() {
        let area = CardRect { x: 0, y: 0, width: 600, height: 400 };
        let card = CardRect { x: 0, y: 0, width: 300, height: 200 };
        let occupied = CardRect { x: 0, y: 0, width: 300, height: 200 };
        let (x, y) = nearest_free_position(card, area, &[occupied]).unwrap();
        assert!(!card_rects_overlap(CardRect { x, y, ..card }, occupied));
        assert!(nearest_free_position(card, area, &[area]).is_none());
        assert_eq!(nearest_free_position(card, area, &[]), Some((0, 0)));
    }

    fn sample_card(id: &str) -> DesktopCard {
        DesktopCard {
            id: id.into(),
            title: "常用软件".into(),
            items: vec![DesktopCardItem {
                id: "item-1".into(),
                name: "编辑器".into(),
                target: "C:/Editor.exe".into(),
                kind: "app".into(),
                icon_path: Some("icons/a.png".into()),
                created_at: "2026-11-22T00:00:00Z".into(),
                original_path: None,
            }],
            x: 800,
            y: 120,
            width: 480,
            height: 630,
            sort_order: 1,
            locked: false,
            visible_on_desktop: true,
            icon: None,
            emoji: None,
            style: DesktopCardStyle::default(),
            created_at: "2026-11-22T00:00:00Z".into(),
            updated_at: "2026-11-22T00:00:00Z".into(),
        }
    }

    #[test]
    fn cards_survive_save_and_load_roundtrip() {
        let folder = Folder::new();
        let cards = vec![
            sample_card("card-1"),
            DesktopCard {
                id: "card-2".into(),
                title: "".into(),
                items: vec![],
                x: 0,
                y: 0,
                width: 480,
                height: 630,
                sort_order: 2,
                locked: false,
                visible_on_desktop: true,
                icon: None,
                emoji: None,
                style: DesktopCardStyle::default(),
                created_at: String::new(),
                updated_at: String::new(),
            },
        ];
        save(&folder.0, &cards).unwrap();
        assert_eq!(load(&folder.0).unwrap(), cards);
    }

    #[test]
    fn missing_cards_file_loads_empty_and_corrupt_file_errors() {
        let folder = Folder::new();
        assert!(load(&folder.0).unwrap().is_empty());
        fs::write(folder.0.join("desktop_cards.json"), b"{not json").unwrap();
        assert!(load(&folder.0).is_err());
    }

    #[test]
    fn unreadable_cards_path_is_not_treated_as_empty() {
        let folder = Folder::new();
        fs::create_dir(folder.0.join("desktop_cards.json")).unwrap();
        assert!(load(&folder.0).is_err());
    }

    #[test]
    fn item_view_marks_only_missing_local_targets() {
        let folder = Folder::new();
        let path = folder.0.join("present.txt");
        fs::write(&path, "data").unwrap();
        let mut card = sample_card("card-1");
        card.items[0].target = path.to_string_lossy().into_owned();
        assert!(!card_view(&card, &folder.0).items[0].missing);
        fs::remove_file(path).unwrap();
        assert!(card_view(&card, &folder.0).items[0].missing);
        card.items[0].kind = "web".into();
        card.items[0].target = "https://example.com".into();
        assert!(!card_view(&card, &folder.0).items[0].missing);
    }

    #[test]
    fn managed_item_restores_without_overwriting_desktop_file() {
        let folder = Folder::new();
        let original = folder.0.join("desktop.txt");
        let stored = folder.0.join("stored.txt");
        fs::write(&stored, "card data").unwrap();
        fs::write(&original, "existing data").unwrap();
        let mut item = sample_card("test").items.remove(0);
        item.original_path = Some(original.to_string_lossy().into_owned());
        item.target = stored.to_string_lossy().into_owned();
        assert!(restore_managed(&item).is_err());
        assert_eq!(fs::read_to_string(&original).unwrap(), "existing data");
        assert_eq!(fs::read_to_string(&stored).unwrap(), "card data");
        fs::remove_file(&original).unwrap();
        restore_managed(&item).unwrap();
        assert_eq!(fs::read_to_string(&original).unwrap(), "card data");
        assert!(!stored.exists());
        assert_eq!(effective_target(&item), original.to_str().unwrap());
        restore_managed(&item).unwrap();
    }

    #[test]
    fn lost_managed_item_can_be_removed_without_error() {
        let folder = Folder::new();
        let original = folder.0.join("desktop.txt");
        let mut item = sample_card("test").items.remove(0);
        item.original_path = Some(original.to_string_lossy().into_owned());
        item.target = folder.0.join("stored.txt").to_string_lossy().into_owned();
        // 卡片文件与原桌面文件都已不存在时，移除条目/删除卡片仍应成功。
        restore_managed(&item).unwrap();
        assert!(!original.exists());
    }

    #[test]
    fn managed_folder_restores_with_contents() {
        let folder = Folder::new();
        let original = folder.0.join("original-folder");
        let stored = folder.0.join("stored-folder");
        fs::create_dir(&stored).unwrap();
        fs::write(stored.join("file.txt"), "content").unwrap();
        let mut item = sample_card("test").items.remove(0);
        item.original_path = Some(original.to_string_lossy().into_owned());
        item.target = stored.to_string_lossy().into_owned();
        restore_managed(&item).unwrap();
        assert_eq!(
            fs::read_to_string(original.join("file.txt")).unwrap(),
            "content"
        );
    }

    #[test]
    fn interrupted_move_is_recovered_as_a_reference_without_moving_user_file() {
        let folder = Folder::new();
        let original = folder.0.join("desktop.txt");
        let stored = folder.0.join("cards").join("desktop.txt");
        fs::write(&original, "still on desktop").unwrap();
        let mut card = sample_card("test");
        let item = &mut card.items[0];
        item.target = stored.to_string_lossy().into_owned();
        item.original_path = Some(original.to_string_lossy().into_owned());

        assert!(recover_interrupted_moves(std::slice::from_mut(&mut card)));
        assert_eq!(card.items[0].target, original.to_string_lossy());
        assert_eq!(card.items[0].original_path, None);
        assert_eq!(fs::read_to_string(original).unwrap(), "still on desktop");
        assert!(!stored.exists());
        assert!(!recover_interrupted_moves(std::slice::from_mut(&mut card)));
    }

    #[test]
    fn card_points_stay_visible_in_work_area() {
        assert_eq!(
            clamp_point(5000, 5000, 480, 630, 0, 0, 1920, 1040),
            (1800, 920)
        );
        assert_eq!(
            clamp_point(-2000, -2000, 480, 630, 0, 0, 1920, 1040),
            (-360, -510)
        );
        assert_eq!(
            clamp_point(100, 100, 480, 630, 0, 0, 1920, 1040),
            (100, 100)
        );
        // 超大卡片也不会让 clamp 区间倒置而 panic。
        assert_eq!(clamp_point(10, 10, 4000, 3000, 0, 0, 1920, 1040), (10, 10));
    }

    #[test]
    fn drop_paths_are_resolved_and_validated() {
        let folder = Folder::new();
        let text = folder.0.join("资料.txt");
        fs::write(&text, "x").unwrap();
        let url_ok = folder.0.join("站点.url");
        fs::write(&url_ok, "[InternetShortcut]\nURL=https://example.com\n").unwrap();
        let url_bad = folder.0.join("本地.url");
        fs::write(&url_bad, "[InternetShortcut]\nURL=ftp://example.com\n").unwrap();

        let file = resolve_drop_path(text.to_str().unwrap()).unwrap();
        assert_eq!((file.name.as_str(), file.kind.as_str()), ("资料", "file"));
        let web = resolve_drop_path(url_ok.to_str().unwrap()).unwrap();
        assert_eq!(
            (web.kind.as_str(), web.target.as_str()),
            ("web", "https://example.com/")
        );
        assert!(resolve_drop_path(url_bad.to_str().unwrap()).is_err());
        assert!(resolve_drop_path(folder.0.join("缺失.txt").to_str().unwrap()).is_err());
    }
}
