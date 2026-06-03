use dirs;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use walkdir::WalkDir;

//
// Config related data structures
// - [ MonitorConfig, Setup, AppConfig ]
//

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitorConfig {
    pub monitor_index: u32,
    pub wallpaper_path: String,
    pub active_widgets: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Setup {
    pub name: String,
    pub monitors: Vec<MonitorConfig>,
}

impl Default for Setup {
    fn default() -> Self {
        Setup {
            name: "Default".to_string(),
            monitors: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub shortcut: String,
    pub active_setup_name: String,
    #[serde(default)]
    pub setups: Vec<Setup>,
    #[serde(default)]
    pub custom_mode: bool,
    #[serde(default)]
    pub custom_setup: Setup,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            shortcut: "alt+w".to_string(),
            active_setup_name: "Default".to_string(),
            setups: vec![Setup {
                name: "Default".to_string(),
                monitors: vec![],
            }],
            custom_mode: true,
            custom_setup: Setup {
                name: "Custom".to_string(),
                monitors: vec![],
            },
        }
    }
}

//
// Init methods: create required directories and default configs
// - [ wallpapers/, thumbnails/, widgets/, config.json, widget.json ]
//

pub fn wallpapers_dir() -> PathBuf {
    dirs::document_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("win-wallpaper")
        .join("wallpapers")
}

pub fn thumb_dir() -> PathBuf {
    dirs::document_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("win-wallpaper")
        .join("thumbnails")
}

pub fn widgets_dir() -> PathBuf {
    dirs::document_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("win-wallpaper")
        .join("widgets")
}

pub fn config_file_path() -> PathBuf {
    dirs::document_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("win-wallpaper")
        .join("config.json")
}

pub fn widgets_config_path() -> PathBuf {
    dirs::document_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("win-wallpaper")
        .join("widgets.json")
}

//
// Ensure methods: create required directories and default configs
// [ config.json, widget.json ]
//

pub fn ensure_storage_initialized() {
    let w_dir = wallpapers_dir();
    let t_dir = thumb_dir();
    let wg_dir = widgets_dir();

    if !w_dir.exists() {
        std::fs::create_dir_all(&w_dir).unwrap();
    }
    if !t_dir.exists() {
        std::fs::create_dir_all(&t_dir).unwrap();
    }
    if !wg_dir.exists() {
        std::fs::create_dir_all(&wg_dir).unwrap();
    }

    ensure_config_initialized();
    ensure_widgets_config_initialized();
}

fn ensure_config_initialized() {
    let config_path = config_file_path();
    if !config_path.exists() {
        let config = AppConfig {
            shortcut: get_legacy_shortcut(),
            active_setup_name: "Default".to_string(),
            setups: vec![Setup {
                name: "Default".to_string(),
                monitors: vec![],
            }],
            custom_mode: true,
            custom_setup: Setup {
                name: "Custom".to_string(),
                monitors: vec![],
            },
        };
        save_app_config(config);
    }
}

fn ensure_widgets_config_initialized() {
    let widgets_path = widgets_config_path();
    if !widgets_path.exists() {
        let default_widgets = json!({
            "widgets": []
        });
        if let Ok(json_str) = serde_json::to_string_pretty(&default_widgets) {
            let _ = std::fs::write(widgets_path, json_str);
        }
    }
}

// Backward compatibility function (disappear in v1)
fn get_legacy_shortcut() -> String {
    let config_path = config_file_path();
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                return val
                    .get("shortcut")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "alt+w".to_string());
            }
        }
    }
    // Fallback to default value
    "alt+w".to_string()
}

//
// Loading/saving config
// - Load [ app_config, full_config ]
// - Save [ app_config, full_config ]
//

use std::sync::{OnceLock, RwLock};

// config memory cache
static APP_CONFIG: OnceLock<RwLock<AppConfig>> = OnceLock::new();

// get config cache reference
fn get_config_cache() -> &'static RwLock<AppConfig> {
    APP_CONFIG.get_or_init(|| {
        RwLock::new(load_app_config_from_disk())
    })
}

// load config from disk
fn load_app_config_from_disk() -> AppConfig {
    let config_path = config_file_path();
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
            return config;
        }
    }
    AppConfig::default()
}

// save config to disk
fn save_app_config_to_disk(config: &AppConfig) {
    let config_path = config_file_path();
    if let Ok(json_str) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(config_path, json_str);
    }
}

pub fn load_app_config() -> AppConfig {
    // read from cache
    let cache = get_config_cache().read().unwrap();
    cache.clone()
}

pub fn save_app_config(config: AppConfig) {
    // write to cache
    {
        let mut cache = get_config_cache().write().unwrap();
        *cache = config.clone();
    }
    // write to disk
    save_app_config_to_disk(&config);
}

fn load_full_config() -> Value {
    let config_path = config_file_path();
    if let Ok(content) = std::fs::read_to_string(config_path) {
        if let Ok(val) = serde_json::from_str(&content) {
            return val;
        }
    }
    json!({})
}

fn save_full_config(config: &Value) {
    let config_path = config_file_path();
    if let Ok(json_str) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(config_path, json_str);
    }
}

//
// Setters / getters
// - GET [config_value, active_setup, custom_mode, monitor_config, monitor_wallpaper, monitor_widgets, shortcut, setups]
// - SET [config_value, active_setup, custom_mode, monitor_config, monitor_wallpaper, monitor_widgets]
// (setups and shortcut are static for now, will change w/ config UI)
//

pub fn set_config_value(key: &str, value: Value) {
    let mut config = load_full_config();
    if let Some(obj) = config.as_object_mut() {
        obj.insert(key.to_string(), value);
        save_full_config(&config);
    }
}

pub fn get_config_value(key: &str) -> Option<Value> {
    let config = load_full_config();
    config.get(key).cloned()
}

pub fn get_shortcut() -> String {
    load_app_config().shortcut
}

pub fn get_active_setup() -> Option<Setup> {
    let config = load_app_config();
    if config.custom_mode {
        Some(config.custom_setup)
    } else {
        config
            .setups
            .into_iter()
            .find(|s| s.name == config.active_setup_name)
    }
}

pub fn set_active_setup(name: String) {
    let mut config = load_app_config();
    config.active_setup_name = name.clone();
    config.custom_mode = false;

    // Copy the selected setup to custom_setup as well, so switching to custom starts from here
    if let Some(selected) = config.setups.iter().find(|s| s.name == name) {
        config.custom_setup.monitors = selected.monitors.clone();
    }

    save_app_config(config);
}

pub fn get_custom_mode() -> bool {
    load_app_config().custom_mode
}

pub fn set_custom_mode(enabled: bool) {
    let mut config = load_app_config();
    config.custom_mode = enabled;
    save_app_config(config);
}

pub fn get_setups() -> Vec<Setup> {
    load_app_config().setups
}

pub fn get_monitor_config(monitor_index: u32) -> MonitorConfig {
    let setup = get_active_setup();
    if let Some(s) = setup {
        if let Some(mc) = s
            .monitors
            .into_iter()
            .find(|m| m.monitor_index == monitor_index)
        {
            return mc;
        }
    }
    MonitorConfig {
        monitor_index,
        wallpaper_path: String::new(),
        active_widgets: vec![],
    }
}

pub fn set_monitor_wallpaper(monitor_index: u32, path: String) {
    let mut config = load_app_config();
    // Always edit the custom_setup
    let setup = &mut config.custom_setup;

    if let Some(monitor) = setup
        .monitors
        .iter_mut()
        .find(|m| m.monitor_index == monitor_index)
    {
        monitor.wallpaper_path = path.clone();
    } else {
        setup.monitors.push(MonitorConfig {
            monitor_index,
            wallpaper_path: path.clone(),
            active_widgets: vec![],
        });
    }

    save_app_config(config);
}

pub fn set_monitor_widgets(monitor_index: u32, widgets: Vec<String>) {
    let mut config = load_app_config();
    // Always edit the custom_setup
    let setup = &mut config.custom_setup;

    if let Some(monitor) = setup
        .monitors
        .iter_mut()
        .find(|m| m.monitor_index == monitor_index)
    {
        monitor.active_widgets = widgets;
    } else {
        setup.monitors.push(MonitorConfig {
            monitor_index,
            wallpaper_path: String::new(),
            active_widgets: widgets,
        });
    }

    save_app_config(config);
}

//
// Utils
//

// List files recursively with depth limit and file extension filter (optional)
pub fn list_files_recursive(
    dir: PathBuf,
    depth: usize,
    extensions: Option<&[&str]>,
) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .max_depth(depth)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let path = entry.path().to_path_buf();
            if let Some(exts) = extensions {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if exts.contains(&ext_str.as_str()) {
                        files.push(path);
                    }
                }
            } else {
                files.push(path);
            }
        }
    }

    files
}
