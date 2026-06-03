mod components;
mod services;

use components::{tray, window};
use services::{
    commands, shortcut,
    storage::{ensure_storage_initialized, widgets_dir},
}; 

use notify::{RecursiveMode, Watcher};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{Emitter, Manager};

//
// Run the application
// - Init Tauri and modules
// - Setup event handlers
// - Setup watcher for widgets directory
//
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    //
    // Initialize Tauri Instance
    //
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_wallpaper::init());

    // Setup modular features
    let builder = shortcut::setup_shortcut(builder);

    builder
        .setup(|app| {
            
            // Initialize storage
            ensure_storage_initialized();

            // Initialize modular features
            shortcut::init_shortcut(app)?;
            tray::init_tray(app)?;

            let app_handle = app.handle().clone();

            //
            // Widgets Watcher with debounce
            //
            let last_update = Arc::new(Mutex::new(Instant::now()));
            let last_update_clone = last_update.clone();
            let app_handle_clone = app_handle.clone();

            let mut watcher =
                notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
                    Ok(event) => {
                        if event.kind.is_modify()
                            || event.kind.is_create()
                            || event.kind.is_remove()
                        {
                            let mut last = last_update_clone.lock().unwrap();
                            let now = Instant::now();
                            if now.duration_since(*last).as_millis() >= 300 {
                                *last = now;
                                drop(last); // Release lock before emitting
                                let _ = app_handle_clone.emit("update-widgets", ());
                            }
                        }
                    }
                    Err(e) => println!("watch error: {:?}", e),
                })
                .expect("Failed to create watcher");

            watcher
                .watch(
                    std::path::Path::new(&widgets_dir()),
                    RecursiveMode::Recursive,
                )
                .expect("Failed to watch widgets directory");

            app.manage(std::sync::Mutex::new(watcher));

            // Setup monitors and windows
            window::setup_monitors(app)?;

            // Background thread to monitor window visibility (maximize/fullscreen) and pause wallpapers
            let app_handle_clone2 = app_handle.clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    check_and_update_visibility(&app_handle_clone2);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_wallpapers,
            commands::get_default_wallpaper,
            commands::get_widgets,
            commands::refresh_app,
            commands::get_monitors,
            commands::get_monitor_wallpaper,
            commands::set_monitor_wallpaper,
            commands::get_monitor_widgets,
            commands::set_monitor_widgets,
            commands::get_active_setup,
            commands::set_active_setup,
            commands::get_custom_mode,
            commands::set_custom_mode,
            commands::get_setups
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "windows")]
fn check_and_update_visibility<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    use windows_sys::Win32::Foundation::{HWND, RECT, BOOL, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowRect, IsZoomed, GetClassNameW,
        EnumWindows, IsWindowVisible,
    };
    use windows_sys::Win32::Graphics::Gdi::{
        MonitorFromWindow, GetMonitorInfoW, MONITORINFO, MONITOR_DEFAULTTONEAREST,
        MonitorFromPoint
    };
    use windows_sys::Win32::Foundation::POINT;

    let fg_hwnd = unsafe { GetForegroundWindow() };
    if fg_hwnd == 0 {
        return;
    }

    let mut class_name = [0u16; 256];
    let len = unsafe { GetClassNameW(fg_hwnd, class_name.as_mut_ptr(), class_name.len() as i32) };
    if len > 0 {
        let name = String::from_utf16_lossy(&class_name[..len as usize]);
        if name == "Progman" || name == "WorkerW" {
            let monitors = app.available_monitors().unwrap_or_default();
            for i in 0..monitors.len() {
                let _ = app.emit(&format!("update-play-state-{}", i), true);
            }
            return;
        }
    }

    let monitors = app.available_monitors().unwrap_or_default();
    if monitors.is_empty() {
        return;
    }

    let fg_hmonitor = unsafe { MonitorFromWindow(fg_hwnd, MONITOR_DEFAULTTONEAREST) };

    let is_maximized = unsafe { IsZoomed(fg_hwnd) } != 0;
    
    let mut is_fullscreen = false;
    let mut rect: RECT = unsafe { std::mem::zeroed() };
    if unsafe { GetWindowRect(fg_hwnd, &mut rect) } != 0 {
        let mut monitor_info: MONITORINFO = unsafe { std::mem::zeroed() };
        monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if unsafe { GetMonitorInfoW(fg_hmonitor, &mut monitor_info as *mut _ as *mut _) } != 0 {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            let mon_width = monitor_info.rcMonitor.right - monitor_info.rcMonitor.left;
            let mon_height = monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top;
            
            if width >= mon_width && height >= mon_height {
                is_fullscreen = true;
            }
        }
    }

    let should_pause_fg_monitor = is_maximized || is_fullscreen;

    struct EnumState {
        maximized_monitors: Vec<isize>,
    }
    
    let mut state = EnumState { maximized_monitors: Vec::new() };
    
    unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let state = &mut *(lparam as *mut EnumState);
        if IsWindowVisible(hwnd) != 0 && IsZoomed(hwnd) != 0 {
            let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            if hmonitor != 0 {
                state.maximized_monitors.push(hmonitor);
            }
        }
        1
    }

    unsafe {
        EnumWindows(Some(enum_windows_callback), &mut state as *mut _ as LPARAM);
    }
    
    for (i, m) in monitors.iter().enumerate() {
        let pos = m.position();
        let size = m.size();
        let center_x = pos.x + (size.width as i32) / 2;
        let center_y = pos.y + (size.height as i32) / 2;
        
        let hmonitor = unsafe {
            MonitorFromPoint(
                POINT { x: center_x, y: center_y },
                MONITOR_DEFAULTTONEAREST
            )
        };
        
        let is_fg_monitor = hmonitor == fg_hmonitor;
        let has_maximized = state.maximized_monitors.contains(&hmonitor);
        
        let should_pause = (is_fg_monitor && should_pause_fg_monitor) || has_maximized;
        
        let _ = app.emit(&format!("update-play-state-{}", i), !should_pause);
    }
}

#[cfg(not(target_os = "windows"))]
fn check_and_update_visibility<R: tauri::Runtime>(_app: &tauri::AppHandle<R>) {
}

