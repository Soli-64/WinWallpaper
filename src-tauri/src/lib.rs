mod storage;
mod thumbnail;

use storage::{
    ensure_storage_initialized, get_shortcut, list_files_recursive, load_config, save_config,
    wallpapers_dir, widgets_config_path, widgets_dir,
};
use tauri::{Manager, Position, Size, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_wallpaper::{AttachRequest, WallpaperExt};
use thumbnail::ThumbnailManager;
use notify::{Watcher, RecursiveMode};

#[tauri::command]
fn set_wallpaper_config(path: String) {
    save_config(path);
}

// 
// Get default wallpaper (from config or fallback)
// 
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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Widget {
    id: String,
    name: String,
    html_file: String,
    #[serde(default)]
    html_content: String,
}

// 
// Get list of widgets (from widgets.json, loads and parse html files content)
// 
#[tauri::command]
fn get_widgets() -> Result<Vec<Widget>, String> {
    let config_path = widgets_config_path();
    if !config_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let mut widgets: Vec<Widget> = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let w_dir = widgets_dir();
    for widget in &mut widgets {
        let html_path = w_dir.join(&widget.html_file);
        if html_path.exists() {
            widget.html_content = std::fs::read_to_string(html_path).unwrap_or_default();
        }
    }

    Ok(widgets)
}

// 
// Get list of wallpapers (recursive w/ limited depth)
// Checks media format, creates thumbnails if needed, and returns list of wallpapers
//
#[tauri::command]
fn get_wallpapers() -> Vec<WallpaperItem> {
    ensure_storage_initialized();

    let mut items = Vec::new();
    let extensions = ["png", "jpg", "jpeg", "webp", "mp4", "webm", "mov"];
    let paths = list_files_recursive(wallpapers_dir(), 1, Some(&extensions));

    let thumb_manager = ThumbnailManager::new();

    for path in paths {
        let is_video = match path.extension() {
            Some(ext) => {
                ["mp4", "webm", "mov"].contains(&ext.to_string_lossy().to_lowercase().as_str())
            }
            None => false,
        };

        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
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

use tauri_plugin_global_shortcut::Shortcut;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItem, PredefinedMenuItem};
use tauri::Emitter;

// 
// Cycle through wallpapers for next/previous wallpaper events 
// (only in tray menu, upcoming global shortcut for full keyboard UX)
//
fn cycle_wallpaper(app: &tauri::AppHandle, forward: bool) {
    let wallpapers = get_wallpapers();
    if wallpapers.is_empty() { return; }
    
    let current = get_default_wallpaper();
    let mut current_idx = wallpapers.iter().position(|w| w.path == current).unwrap_or(0);
    
    if forward {
        current_idx = (current_idx + 1) % wallpapers.len();
    } else {
        current_idx = if current_idx == 0 { wallpapers.len() - 1 } else { current_idx - 1 };
    }
    
    let next_wp = &wallpapers[current_idx];
    
    let _ = app.emit("update-wallpaper", next_wp.path.clone());
    save_config(next_wp.path.clone());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_wallpaper::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    println!(
                        "Shortcut event received: {:?} for {}",
                        event.state(),
                        shortcut.to_string()
                    );
                    if event.state() == ShortcutState::Pressed {
                        let active_shortcut = get_shortcut();
                        let shortcut_str = shortcut.to_string().to_lowercase();

                        // Match either "alt+w" or "alt+keyw" (Tauri v2 format)
                        let matches = shortcut_str == active_shortcut
                            || shortcut_str == active_shortcut.replace("+", "+key");

                        if matches {
                            println!(
                                "Shortcut {} detected! Toggling switch-bar...",
                                active_shortcut
                            );


                            // Toggle switch-bar visibility
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
                                    let _ = window.eval("window.focus();");
                                    println!("Shown and accepting cursor events.");
                                }
                            } else {
                                println!("Error: 'switch-bar' window not found!");
                            }
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            // Storage
            ensure_storage_initialized();

            let app_handle = app.handle().clone();

            //
            // Widgets Watcher
            // Seems to be problems some with setIntervals, might need a  
            // restart when modifying asynchonous widgets scripts
            //
            let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                match res {
                    Ok(event) => {
                        if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                            let _ = app_handle.emit("update-widgets", ());
                        }
                    },
                    Err(e) => println!("watch error: {:?}", e),
                }
            }).expect("Failed to create watcher");

            watcher.watch(std::path::Path::new(&widgets_dir()), RecursiveMode::Recursive)
                .expect("Failed to watch widgets directory");

            app.manage(std::sync::Mutex::new(watcher));

            //
            // Tray menu
            //
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Open Wallpaper Bar", true, None::<&str>)?;
            let next_i = MenuItem::with_id(app, "next", "Next Wallpaper", true, None::<&str>)?;
            let prev_i = MenuItem::with_id(app, "prev", "Previous Wallpaper", true, None::<&str>)?;
            let open_dir_i = MenuItem::with_id(app, "open_dir", "Open Wallpapers Folder", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;

            let menu = MenuBuilder::new(app)
                .items(&[&show_i, &next_i, &prev_i, &sep, &open_dir_i, &sep, &quit_i])
                .build()?;

            let tray_builder = if let Some(icon) = app.default_window_icon() {
                TrayIconBuilder::with_id("main").icon(icon.clone())
            } else {
                TrayIconBuilder::with_id("main")
            };

            let _tray = tray_builder
                .tooltip("WinWallpaper")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("switch-bar") {
                                window.show().unwrap();
                                window.set_ignore_cursor_events(false).unwrap();
                                window.set_focus().unwrap();
                                let _ = window.eval("window.focus();");
                            }
                        }
                        "next" => cycle_wallpaper(app, true),
                        "prev" => cycle_wallpaper(app, false),
                        "open_dir" => {
                            let _ = std::process::Command::new("explorer")
                                .arg(wallpapers_dir())
                                .spawn();
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, button_state, .. } = event {
                        if button == MouseButton::Left && button_state == MouseButtonState::Up {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("switch-bar") {
                                let is_visible = window.is_visible().unwrap_or(false);
                                if is_visible {
                                    window.hide().unwrap();
                                    window.set_ignore_cursor_events(true).unwrap();
                                } else {
                                    window.show().unwrap();
                                    window.set_ignore_cursor_events(false).unwrap();
                                    window.set_focus().unwrap();
                                    let _ = window.eval("window.focus();");
                                }
                            }
                        }
                    }
                })
                .build(app)?;

            // Register Shortcut from config
            let shortcut_to_reg = get_shortcut();

            // Convert stored shortcut string into the Shortcut type required by the API
            // Shortcut (HotKey) can be parsed from a &str using TryFrom/FromStr implementations
            let shortcut_wrapper = Shortcut::try_from(shortcut_to_reg.as_str())
                .expect("Invalid shortcut format stored in config");
            match app.global_shortcut().register(shortcut_wrapper) {
                Ok(_) => println!("Successfully registered {} shortcut", shortcut_to_reg),
                Err(e) => println!("Failed to register {} shortcut: {}", shortcut_to_reg, e),
            }

            //
            // Monitors handler
            // Create multiple windows to match every screen
            //
            let monitors = app.available_monitors().unwrap();

            for (i, monitor) in monitors.iter().enumerate() {
                
                println!(
                    "Monitor: {}",
                    monitor.name().expect("Monitor name not found").as_str()
                );

                let label = format!("wallpaper-{}", i);
                println!(
                    "Creating window {} for monitor: {}x{} @ ({},{})",
                    label,
                    monitor.size().width,
                    monitor.size().height,
                    monitor.position().x,
                    monitor.position().y
                );

                let window =
                    WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
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
                app.handle()
                    .wallpaper()
                    .attach(AttachRequest::new(&label))?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_wallpapers,
            set_wallpaper_config,
            get_default_wallpaper,
            get_widgets
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
