use std::path::{PathBuf};
use dirs;
use walkdir::WalkDir;
use serde_json::{Value, json};

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

pub fn ensure_storage_initialized() {
    let w_dir = wallpapers_dir();
    let t_dir = thumb_dir();

    if !w_dir.exists() {
        std::fs::create_dir_all(&w_dir).unwrap();
    }
    if !t_dir.exists() {
        std::fs::create_dir_all(&t_dir).unwrap();
    }
}

pub fn config_file_path() -> PathBuf {
    dirs::document_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("win-wallpaper")
        .join("config.json")
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


// Backward compatibility with previous version
pub fn save_config(path: String) {
    set_config_value("last_wallpaper", json!(path));
}

pub fn load_config() -> Option<String> {
    get_config_value("last_wallpaper").and_then(|v| v.as_str().map(|s| s.to_string()))
}

pub fn list_files_recursive(dir: PathBuf, depth: usize, extensions: Option<&[&str]>) -> Vec<PathBuf> {
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
