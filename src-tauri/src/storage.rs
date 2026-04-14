use std::path::{PathBuf};
use dirs;
use walkdir::WalkDir;

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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub last_wallpaper: Option<String>,
}

pub fn save_config(path: String) {
    let config = AppConfig { last_wallpaper: Some(path) };
    let config_path = config_file_path();
    if let Ok(json) = serde_json::to_string(&config) {
        let _ = std::fs::write(config_path, json);
    }
}

pub fn load_config() -> Option<String> {
    let config_path = config_file_path();
    if let Ok(json) = std::fs::read_to_string(config_path) {
        if let Ok(config) = serde_json::from_str::<AppConfig>(&json) {
            return config.last_wallpaper;
        }
    }
    None
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
