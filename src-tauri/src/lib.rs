mod storage;
mod thumbnail;

use tauri::{Manager, Position, Size, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_wallpaper::{WallpaperExt, AttachRequest};
use storage::{wallpapers_dir, list_files_recursive, ensure_storage_initialized, save_config, load_config};
use thumbnail::ThumbnailManager;

#[tauri::command]
fn set_wallpaper_config(path: String) {
    save_config(path);
}

#[tauri::command]
fn get_default_wallpaper() -> String {
    if let Some(path) = load_config() {
        if std::path::Path::new(&path).exists() {
            return path;
        }
    }

    // Fallback: first available wallpaper
    let w_dir = wallpapers_dir();
    let files = list_files_recursive(w_dir, 1, Some(&["jpg", "jpeg", "png", "mp4", "webm"]));
    if let Some(first) = files.first() {
        return first.to_string_lossy().to_string();
    }

    "".to_string()
}

#[derive(serde::Serialize)]
pub struct WallpaperItem {
    name: String,
    path: String,
    thumb_path: String,
    is_video: bool,
}

#[tauri::command]
fn get_wallpapers() -> Vec<WallpaperItem> {
    ensure_storage_initialized();
    
    let mut items = Vec::new();
    let extensions = ["png", "jpg", "jpeg", "webp", "mp4", "webm", "mov"]; 
    let paths = list_files_recursive(wallpapers_dir(), 1, Some(&extensions));
    
    let thumb_manager = ThumbnailManager::new();

    for path in paths {
        let is_video = match path.extension() {
            Some(ext) => ["mp4", "webm", "mov"].contains(&ext.to_string_lossy().to_lowercase().as_str()),
            None => false,
        };

        let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let thumb_path = match thumb_manager.create_thumbnail(&path, is_video) {
            Ok(p) => p.to_string_lossy().into_owned(),
            Err(e) => {
                eprintln!("Failed to create thumbnail for {:?}: {}", path, e);
                // Fallback to original path if thumbnail fails (might not display well but won't crash)
                path.to_string_lossy().into_owned()
            }
        };

        items.push(WallpaperItem {
            name,
            path: path.to_string_lossy().into_owned(),
            thumb_path,
            is_video,
        });
    }

    items
}

use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_wallpaper::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, shortcut, event| {
                println!("Shortcut event received: {:?} for {}", event.state(), shortcut.to_string());
                if event.state() == ShortcutState::Pressed {
                    let shortcut_str = shortcut.to_string().to_lowercase();
                    if shortcut_str == "alt+w" || shortcut_str == "alt+keyw" {
                        println!("Alt+W detected! Toggling switch-bar...");
                        if let Some(window) = app.get_webview_window("switch-bar") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            println!("Current visibility: {}", is_visible);
                            if is_visible {
                                window.hide().unwrap();
                                window.set_ignore_cursor_events(true).unwrap();
                                println!("Hidden and ignoring cursor events.");
                            } else {
                                window.show().unwrap();
                                window.set_ignore_cursor_events(false).unwrap();
                                window.set_focus().unwrap();
                                println!("Shown and accepting cursor events.");
                            }
                        } else {
                            println!("Error: 'switch-bar' window not found!");
                        }
                    }
                }
            })
            .build())
        .setup(|app| {
            ensure_storage_initialized();

            // Register Shortcut
            match app.global_shortcut().register("alt+w") {
                Ok(_) => println!("Successfully registered Alt+W shortcut"),
                Err(e) => println!("Failed to register Alt+W shortcut: {}", e),
            }

            let monitors = app.available_monitors().unwrap();
            for (i, monitor) in monitors.iter().enumerate() {
                let label = format!("wallpaper-{}", i);
                println!("Creating window {} for monitor: {}x{} @ ({},{})", 
                    label, monitor.size().width, monitor.size().height, monitor.position().x, monitor.position().y);

                let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
                    .title("Animated Wallpaper")
                    .decorations(false)
                    .transparent(true)
                    .resizable(false)
                    .visible(false)      
                    .fullscreen(true)              
                    .build()?;

                let pos = monitor.position();
                let size = monitor.size();

                window.set_position(Position::Physical(*pos))?;
                window.set_size(Size::Physical(*size))?;
                window.show()?;
                
                // Attach as wallpaper
                app.handle().wallpaper().attach(AttachRequest::new(&label))?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_wallpapers, set_wallpaper_config, get_default_wallpaper])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
