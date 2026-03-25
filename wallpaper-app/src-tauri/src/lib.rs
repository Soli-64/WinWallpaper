mod server;
use server::{start_server};
use tauri::{Manager, PhysicalPosition, Position, Size, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_wallpaper::{WallpaperExt, AttachRequest};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_wallpaper::init())
        .setup(|app| {

            let monitors = app.available_monitors().unwrap();
            for (i, monitor) in monitors.iter().enumerate() {
                println!("Monitor: {}x{} @ ({},{})", monitor.size().width, monitor.size().height, monitor.position().x, monitor.position().y);

                let label = format!("wallpaper-{}", i);

                let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
                    .title("Animated Wallpaper")
                    .decorations(false)
                    .transparent(true)
                    .resizable(false)
                    .visible(false)      
                    .fullscreen(true)              // on montre après positionnement
                    .build()?;

                let pos = monitor.position();
                let size = monitor.size();

                window.set_position(Position::Physical(*pos))?;
                window.set_size(Size::Physical(*size))?;
                window.show()?;
                app.handle().wallpaper().attach(AttachRequest::new(&label.as_str()))?;
            }

            let _handle = app.handle().clone();
            tauri::async_runtime::spawn(async {
                println!("Starting Wp server server...");
                server::start_server(_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
