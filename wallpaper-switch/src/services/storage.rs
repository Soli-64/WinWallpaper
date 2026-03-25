use walkdir::{DirEntry, WalkDir};
use std::path::{PathBuf,Path};


fn data_dir() -> PathBuf {
    dirs::data_dir()
    .expect("Can't find user data directory")
}

fn doc_dir() -> PathBuf {
    dirs::document_dir()
    .expect("Can't find user document directory")
}

// App storage directory in AppData/Roaming

pub fn app_dir() -> PathBuf {
    let path = data_dir()
    .join("com.lsoapps.winwallpaper");
    path.exists().then(|| path)
    .expect("Can't find app data directory")
}

pub fn thumb_dir() -> PathBuf {
    let path = app_dir()
    .join("thumbnails");
    path.exists().then(|| path)
    .expect("Can't find thumbnails directory")
}

// App config and wallpapers directory in Documents

pub fn config_dir() -> PathBuf {
    let path = doc_dir()
    .join("win-wallpaper");
    path.exists().then(|| path)
    .expect("Can't find config directory")
}

pub fn wallpapers_dir() -> PathBuf {
    let path = config_dir()
    .join("wallpapers");
    path.exists().then(|| path)
    .expect("Can't find wallpapers directory")
}

pub fn list_files_recursive(
    root: impl AsRef<Path>,
    max_depth: Option<usize>,
    extensions: Option<&[&str]>,
) -> Result<Vec<PathBuf>, walkdir::Error> {
    let mut walker = WalkDir::new(root).follow_links(false);
    if let Some(d) = max_depth {
        walker = walker.max_depth(d);
    }
    
    let mut files = Vec::new();
    for entry in walker {
        let entry: DirEntry = entry?; 
        let path = entry.path();

        
        if !path.is_file() {
            continue;
        }
        
        if let Some(exts) = extensions {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !exts.iter().any(|&e| e.eq_ignore_ascii_case(ext)) {
                    continue;
                }
            } else {
                continue; 
            }
        }

        files.push(path.to_path_buf());
    }

    Ok(files)
}

fn create_app_dir() -> () {
    let app_dir = app_dir();
    std::fs::create_dir(&app_dir).expect("Can't create data dir com.lsoapps.winwallpaper");
}

fn create_thumb_dir() -> () {
    let thumb_dir = thumb_dir();
    std::fs::create_dir(&thumb_dir).expect("Can't create thumbnails dir");
}

fn create_config() {

    let config_dir = config_dir();
    let wallpapers_dir = wallpapers_dir();
    let config_file = config_dir.join("config.json");

    std::fs::create_dir(&config_dir).expect("Can't create config dir win-wallpaper");

    std::fs::create_dir(&wallpapers_dir).expect("Can't create wallpapers dir");

    std::fs::File::create(&config_file).expect("Can't create config file");

}

fn ensure_config() {
    
    let config_dir = config_dir();

    if !config_dir.exists() {
        create_config();
    }
    
    let wp_dir = wallpapers_dir();

    if !wp_dir.exists() {
        std::fs::create_dir(&wp_dir).expect("Can't create wallpapers dir");
    }

}

fn ensure_storage() {
    let app_dir = app_dir();
    
    if !app_dir.exists() {
        create_app_dir();
        create_thumb_dir();
    }
}

pub fn ensure_storage_initialized() {
    ensure_storage();
    ensure_config();
}