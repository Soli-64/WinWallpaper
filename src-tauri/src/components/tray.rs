use crate::services::commands::{get_monitor_wallpaper, get_wallpapers, refresh_config};
use crate::services::storage::{set_monitor_wallpaper, wallpapers_dir};
use tauri::menu::{MenuBuilder, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};

//
// Cycle through wallpapers for next/previous wallpaper events
// (only in tray menu, upcoming global shortcut for full keyboard UX)
//

fn cycle_wallpaper_for_monitor<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    monitor_index: u32,
    forward: bool,
) {
    // block on async wallpapers fetch
    let wallpapers = tauri::async_runtime::block_on(get_wallpapers());
    if wallpapers.is_empty() {
        return;
    }

    let current = get_monitor_wallpaper(monitor_index);
    let mut current_idx = wallpapers
        .iter()
        .position(|w| w.path == current)
        .unwrap_or(0);

    if forward {
        current_idx = (current_idx + 1) % wallpapers.len();
    } else {
        current_idx = if current_idx == 0 {
            wallpapers.len() - 1
        } else {
            current_idx - 1
        };
    }

    let next_wp = &wallpapers[current_idx];

    let _ = app.emit(
        &format!("update-monitor-{}", monitor_index),
        next_wp.path.clone(),
    );
    set_monitor_wallpaper(monitor_index, next_wp.path.clone());
}

fn cycle_wallpaper<R: tauri::Runtime>(app: &tauri::AppHandle<R>, forward: bool) {
    cycle_wallpaper_for_monitor(app, 1, forward);
}

//
// Init tray menu
// - [ next wp, previous wp, open wallpapers folder, quit  ]
//

pub fn init_tray<R: tauri::Runtime>(app: &tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    // Menu items
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app, "show", "Open Wallpaper Bar", true, None::<&str>)?;
    let next_i = MenuItem::with_id(app, "next", "Next Wallpaper", true, None::<&str>)?;
    let prev_i = MenuItem::with_id(app, "prev", "Previous Wallpaper", true, None::<&str>)?;
    let refresh_i = MenuItem::with_id(app, "refresh", "Refresh Config", true, None::<&str>)?;
    let open_dir_i = MenuItem::with_id(
        app,
        "open_dir",
        "Open Wallpapers Folder",
        true,
        None::<&str>,
    )?;
    let sep = PredefinedMenuItem::separator(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[
            &show_i,
            &next_i,
            &prev_i,
            &sep,
            &refresh_i,
            &open_dir_i,
            &sep,
            &quit_i,
        ])
        .build()?;

    let tray_builder = if let Some(icon) = app.default_window_icon() {
        TrayIconBuilder::<R>::with_id("main").icon(icon.clone())
    } else {
        TrayIconBuilder::<R>::with_id("main")
    };

    // Tray icon events
    let _tray = tray_builder
        .tooltip("WinWallpaper")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
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
            "refresh" => refresh_config(app),
            "open_dir" => {
                let _ = std::process::Command::new("explorer")
                    .arg(wallpapers_dir())
                    .spawn();
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = event
            {
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

    Ok(())
}
