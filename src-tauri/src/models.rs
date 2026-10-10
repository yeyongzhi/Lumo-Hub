use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme_mode: String,
    pub accent: String,
    pub font_size: u8,
    #[serde(default = "default_border_radius")]
    pub border_radius: u8,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u16,
    #[serde(default = "default_common_launchers")]
    pub common_launchers: Vec<String>,
    pub welcome_completed: bool,
    #[serde(default = "default_home_layout")]
    pub home_layout: Vec<HomeWidget>,
    #[serde(default)] pub launcher: LauncherSettings,
    #[serde(default)] pub auto_backup: AutoBackupSettings,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HomeWidget { pub id: String, pub kind: String, pub size: String, #[serde(default)] pub selected_ids: Vec<String> }

pub const COMMON_LAUNCHER_IDS: &[&str] = &["computer", "recycle_bin", "explorer", "cmd", "powershell", "notepad", "calculator"];
fn default_common_launchers() -> Vec<String> { COMMON_LAUNCHER_IDS.iter().map(|id| (*id).to_string()).collect() }

fn default_sidebar_width() -> u16 { 240 }

fn default_border_radius() -> u8 { 16 }

fn default_home_layout() -> Vec<HomeWidget> {
    [("clock", "2x1"), ("countdowns", "2x1"), ("sticky", "1x1"), ("note", "1x1"), ("recent", "2x1")]
        .into_iter().map(|(kind, size)| HomeWidget { id: kind.into(), kind: kind.into(), size: size.into(), selected_ids: vec![] }).collect()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_mode: "system".into(),
            accent: "blue".into(),
            font_size: 14,
            border_radius: default_border_radius(),
            sidebar_width: default_sidebar_width(),
            common_launchers: default_common_launchers(),
            welcome_completed: false,
            home_layout: default_home_layout(),
            launcher: LauncherSettings::default(), auto_backup: AutoBackupSettings::default(),
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn previous_settings_receive_default_home_modules() {
        let settings: AppSettings = serde_json::from_str(r#"{"theme_mode":"dark","accent":"violet","font_size":14,"welcome_completed":true}"#).unwrap();
        assert_eq!(settings.home_layout.len(), 5);
        assert_eq!(settings.home_layout[0].kind, "clock");
        assert_eq!(settings.accent, "violet");
        assert_eq!(settings.border_radius, 16);
        assert_eq!(settings.sidebar_width, 240);
        assert_eq!(settings.common_launchers, default_common_launchers());
    }

    #[test]
    fn common_launchers_preserve_custom_and_empty_selection() {
        for ids in [vec!["computer".to_string(), "cmd".to_string()], vec![]] {
            let mut settings = AppSettings::default();
            settings.common_launchers = ids.clone();
            let restored: AppSettings = serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
            assert_eq!(restored.common_launchers, ids);
        }
    }

    #[test]
    fn sidebar_width_survives_roundtrip() {
        for width in [100, 140, 180, 200, 220, 240, 260, 280] {
            let mut settings = AppSettings::default();
            settings.sidebar_width = width;
            let restored: AppSettings = serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
            assert_eq!(restored.sidebar_width, width);
        }
    }

    #[test]
    fn radius_settings_survive_roundtrip() {
        for radius in [0, 4, 8, 12, 16, 24] {
            let mut settings = AppSettings::default();
            settings.border_radius = radius;
            let restored: AppSettings = serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
            assert_eq!(restored.border_radius, radius);
        }
    }

    #[test]
    fn custom_and_empty_layouts_survive_settings_roundtrip() {
        let mut settings = AppSettings::default();
        settings.home_layout = vec![HomeWidget { id: "custom-note".into(), kind: "note".into(), size: "2x2".into(), selected_ids: vec![] }];
        let restored: AppSettings = serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert_eq!(restored.home_layout[0].id, "custom-note");
        assert_eq!(restored.home_layout[0].size, "2x2");
        settings.home_layout.clear();
        let restored: AppSettings = serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert!(restored.home_layout.is_empty());
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Sticky {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Countdown {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub target_date: String,
    pub repeat_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub target: String,
    pub group_id: Option<String>,
    pub is_favorite: bool,
    pub icon_path: Option<String>,
    #[serde(default)]
    pub icon_data: Option<String>,
    #[serde(default)]
    pub last_opened_at: Option<String>,
    #[serde(default)] pub aliases: String,
    #[serde(default)] pub usage_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResourceGroup {
    pub id: String,
    pub name: String,
    pub sort_order: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ScanCandidate {
    pub name: String,
    pub target: String,
    pub kind: String,
    pub source: String,
    #[serde(default)]
    pub icon_data: Option<String>,
    #[serde(default)]
    pub installation_path: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BootstrapInfo {
    pub data_dir: String,
    pub default_data_dir: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub kind: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LauncherSettings { pub enabled: bool, pub shortcut: String, #[serde(default)] pub minimal_mode: bool }
impl Default for LauncherSettings { fn default() -> Self { Self { enabled: true, shortcut: "Alt+Space".into(), minimal_mode: false } } }
#[derive(Clone, Serialize, Deserialize)]
pub struct AutoBackupSettings { pub enabled: bool, pub interval_hours: u32, pub retain: usize }
impl Default for AutoBackupSettings { fn default() -> Self { Self { enabled: true, interval_hours: 24, retain: 7 } } }
