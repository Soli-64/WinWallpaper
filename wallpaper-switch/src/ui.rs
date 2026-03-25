use eframe::egui::{
    self, 
    CentralPanel, 
    ScrollArea, 
    TextureHandle, 
    Color32
};

use image::imageops::FilterType;
use std::path::Path;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SetLayeredWindowAttributes,
    FindWindowW,
    GetWindowLongPtrW,
    SetWindowLongPtrW,
    GWL_EXSTYLE,
    WS_EX_LAYERED,
};
use super::services::{
    hotkey::toggle_window_state,
    storage::{
        thumb_dir,
        list_files_recursive
    },
    request::send_to_tauri_server
};

const LWA_ALPHA: u32 = 0x00000002;

static MOUSE_PASSTHROUGH_STATE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static WINDOW_HWND: Lazy<Mutex<Option<isize>>> = Lazy::new(|| Mutex::new(None));

#[derive(Clone)]
#[allow(dead_code)]
struct CarouselItem {
    tex: TextureHandle,
    size: egui::Vec2,
    is_video: bool,
}

fn load_image(
    ctx: &egui::Context,
    path: impl AsRef<Path>,
    display_size: egui::Vec2,
) -> anyhow::Result<CarouselItem> {
    
    let bytes = std::fs::read(path.as_ref())?;
    
    let mut img = image::load_from_memory(&bytes)?.to_rgba8();
    
    let max_side = 2048;
    let (orig_w, orig_h) = img.dimensions();

    if orig_w as usize > max_side || orig_h as usize > max_side {
        let ratio = orig_h as f32 / orig_w as f32;
        let new_w = 320 as u32;
        let new_h = (320.0*ratio).round() as u32;

        img = image::imageops::resize(&img, new_w, new_h, FilterType::Nearest);
    }
    
    let color_image = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_flat_samples().as_slice());
    
    let tex = ctx.load_texture(
        path.as_ref()
            .file_name()
            .unwrap_or_default()
            .to_string_lossy(),
        color_image,
        egui::TextureOptions::default(),
    );
    Ok(CarouselItem {
        tex,
        size: display_size,
        is_video: false,
    })
}

fn load_video_image(
    ctx: &egui::Context,
    path: impl AsRef<Path>,
    display_size: egui::Vec2,
) -> anyhow::Result<CarouselItem> {
    // For videos, we load the thumbnail PNG that was extracted
    let bytes = std::fs::read(path.as_ref())?;
    
    let mut img = image::load_from_memory(&bytes)?.to_rgba8();
    
    let max_side = 2048;
    let (orig_w, orig_h) = img.dimensions();

    if orig_w as usize > max_side || orig_h as usize > max_side {
        let ratio = orig_h as f32 / orig_w as f32;
        let new_w = 320 as u32;
        let new_h = (320.0*ratio).round() as u32;

        img = image::imageops::resize(&img, new_w, new_h, FilterType::Nearest);
    }
    
    let color_image = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_flat_samples().as_slice());
    
    let tex = ctx.load_texture(
        path.as_ref()
            .file_name()
            .unwrap_or_default()
            .to_string_lossy(),
        color_image,
        egui::TextureOptions::default(),
    );
    Ok(CarouselItem {
        tex,
        size: display_size,
        is_video: true,
    })
}

pub fn get_mouse_passthrough_state() -> bool {
    MOUSE_PASSTHROUGH_STATE.lock().unwrap().clone()
}

pub fn set_mouse_passthrough_state(state: bool) {
    *MOUSE_PASSTHROUGH_STATE.lock().unwrap() = state;
}

pub fn set_window_alpha(alpha: u8) {
    if let Some(hwnd) = *WINDOW_HWND.lock().unwrap() {
        unsafe {
            let ex = GetWindowLongPtrW(hwnd as HWND, GWL_EXSTYLE) as isize;
            if (ex as u32 & WS_EX_LAYERED) == 0 {
                SetWindowLongPtrW(hwnd as HWND, GWL_EXSTYLE, (ex | WS_EX_LAYERED as isize) as isize);
            }
            SetLayeredWindowAttributes(hwnd as HWND, 0, alpha, LWA_ALPHA);
        }
    }
}

pub fn store_window_hwnd(hwnd: isize) {
    *WINDOW_HWND.lock().unwrap() = Some(hwnd);
}

fn find_window_by_title(title: &str) -> Option<isize> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let wide: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0)).collect();
    unsafe {
        let h = FindWindowW(std::ptr::null(), wide.as_ptr());
        if h.is_null() { None } else { Some(h as isize) }
    }
}

struct WallpaperApp {
    visible: bool,
    thumbs_path: Vec<std::path::PathBuf>,
    items: Vec<CarouselItem>
}

impl WallpaperApp {

    fn new(_cc: &eframe::CreationContext) -> Self {
        
        let items = WallpaperApp::load_items(&_cc.egui_ctx.clone());

        Self {
            items,
            thumbs_path: Vec::new(),
            visible: false
        }
    }

    fn load_items(
        _ctx: &egui::Context
    ) -> Vec<CarouselItem> {

        let mut items = Vec::new();

        // Load image thumbnails
        let image_paths = match list_files_recursive(thumb_dir(), Some(2), Some(&["png","jpg","jpeg"])) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error reading thumbs directory: {}", e);
                return Vec::new();   
            }
        };

        for path in image_paths {
            // Skip video thumbnail PNGs (those that correspond to video files)
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            if filename.starts_with("thumb_") {
                let original_name = filename.replace("thumb_", "").replace(".png", "");
                
                // Check if this is actually a video by looking for the video file
                let is_actually_video = {
                    let video_extensions = ["mp4", "avi", "webm", "mov", "mkv", "flv"];
                    let wallpaper_dir = super::services::storage::wallpapers_dir();
                    video_extensions.iter().any(|&ext| {
                        wallpaper_dir.join(format!("{}.{}", original_name, ext)).exists()
                    })
                };
                
                if is_actually_video {
                    continue; // Skip, will load as video below
                }
            }

            match load_image(
                _ctx,
                &path,
                egui::Vec2 { x: 320.0, y: 180.0 },
            ) {
                Ok(item) => {
                    items.push(item);
                }
                Err(e) => {
                    eprintln!("Error loading {:?} : {}", path, e);
                }
            }
        }

        // Load video thumbnails
        let video_extensions = ["mp4", "avi", "webm", "mov", "mkv", "flv"];
        let video_thumb_paths = match list_files_recursive(thumb_dir(), Some(2), Some(&["png"])) {
            Ok(v) => {
                v.into_iter()
                    .filter(|p| {
                        let filename = p.file_name().unwrap_or_default().to_string_lossy();
                        if !filename.starts_with("thumb_") {
                            return false;
                        }
                        let original_name = filename.replace("thumb_", "").replace(".png", "");
                        video_extensions.iter().any(|&ext| {
                            let wallpaper_dir = super::services::storage::wallpapers_dir();
                            wallpaper_dir.join(format!("{}.{}", original_name, ext)).exists()
                        })
                    })
                    .collect::<Vec<_>>()
            }
            Err(e) => {
                eprintln!("Error reading video thumbs: {}", e);
                Vec::new()
            }
        };

        for path in video_thumb_paths {
            match load_video_image(
                _ctx,
                &path,
                egui::Vec2 { x: 320.0, y: 180.0 },
            ) {
                Ok(item) => {
                    items.push(item);
                }
                Err(e) => {
                    eprintln!("Error loading video {:?} : {}", path, e);
                }
            }
        }

        items
    }

    fn ui(&mut self, ui: &mut egui::Ui) {

        ScrollArea::horizontal()
            .show(ui, |ui| {
                
                ui.horizontal(|ui| {
    
                    for item in &self.items {
                        let response = ui.add(
                            egui::Image::new(&item.tex)
                        ).interact(egui::Sense::click());

                        if item.is_video {
                            let painter = ui.painter();
                            let rect = response.rect;
                            let center = rect.center();
                            let play_size = 40.0;
                            
                            painter.rect_filled(
                                rect,
                                0.0,
                                Color32::from_black_alpha(100),
                            );
                        
                        }

                        response.clicked().then(|| {
                            println!("Clicked on thumbnail {}", item.tex.name());
                            let filename = item.tex.name();
                            let original_name = filename.replace("thumb_", "");
                            let file_path = super::services::storage::wallpapers_dir().join(&original_name);

                            std::thread::spawn(move || {
                                tokio::runtime::Runtime::new().unwrap().block_on(async {
                                    if file_path.exists() {
                                        if let Err(e) = send_to_tauri_server(&file_path).await {
                                            eprintln!("Error sending to Tauri server: {}", e);
                                        }
                                    } else {
                                        let mut found = false;
                                        for ext in ["mp4", "webm"] {
                                            let video_path = super::services::storage::wallpapers_dir().join(format!("{}.{}", original_name, ext));
                                            if video_path.exists() {
                                                if let Err(e) = send_to_tauri_server(video_path.to_str().unwrap_or_default()).await {
                                                    eprintln!("Error sending to Tauri server: {}", e);
                                                }
                                                found = true;
                                                break;
                                            }
                                        }
                                        if !found {
                                            eprintln!("No video file found for {}", original_name);
                                        }
                                    }
                                });
                            });
                        });
                        ui.add_space(12.0);
                    }
                
                });

            });
    }
}

impl eframe::App for WallpaperApp {
    
    fn update(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        // List both image and video thumbnails
        let mut all_paths = list_files_recursive(thumb_dir(), Some(2), Some(&["png","jpg","jpeg"]))
            .expect("Failed to list thumbnail files");
        
        // Sort for consistency
        all_paths.sort();

        if all_paths != self.thumbs_path {
            self.thumbs_path = all_paths;
            self.items = WallpaperApp::load_items(_ctx);
        }

        static HWND_STORED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
        {
            let mut stored = HWND_STORED.lock().unwrap();
            if !*stored {
                *stored = true;
            }
        }
        
        let state = get_mouse_passthrough_state();
        if self.visible == state {
            if state {
                _ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(true));
                set_window_alpha(0); 
                self.visible = false;
            } else {
                _ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(false));
                set_window_alpha(255); 
                self.visible = true;
            }
        }

        if _ctx.input(|i| i.clone().consume_key(egui::Modifiers::ALT, egui::Key::W)) {
            set_mouse_passthrough_state(self.visible)
        }

        if self.visible {
            CentralPanel::default()
                .frame(egui::Frame::default().fill(Color32::from_rgb(20, 20, 20)))
                .show(_ctx, |ui| {
                    self.ui(ui);
                });
        } else {
            CentralPanel::default()
                .show(_ctx, |_ui| {});
        }  

    }
}

pub fn run_app() -> eframe::Result<()> {

    let dsip_sz = rdev::display_size().unwrap();

    let width: f32 = 1200.;
    let height: f32 = 200.;

    let x = ( dsip_sz.0 as f32 - width as f32) / 2.;
    let y = ( dsip_sz.1 as f32 - height as f32) / 2.;
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder { 
            title: None, 
            app_id: None, 
            position: Some(egui::Pos2 { x, y }),
            inner_size: Some(egui::Vec2::new(width, height)), 
            min_inner_size: None, 
            max_inner_size: None, 
            fullscreen: Some(false), 
            maximized: Some(false), 
            resizable: Some(false), 
            transparent: Some(true), 
            decorations: Some(false), 
            icon: None, 
            active: Some(true), 
            visible: Some(false), 
            fullsize_content_view: Some(false), 
            title_shown: Some(false), 
            titlebar_buttons_shown: Some(false), 
            titlebar_shown: Some(false), 
            drag_and_drop: Some(false),
            taskbar: Some(false), 
            close_button: Some(false), 
            minimize_button: Some(false), 
            maximize_button: Some(false), 
            window_level: Some(egui::viewport::WindowLevel::AlwaysOnTop), 
            mouse_passthrough: Some(true), 
            window_type: None,
            clamp_size_to_monitor_size: Some(false),
            movable_by_window_background: Some(false),
            has_shadow: Some(false)
        },
        ..Default::default()
    };

    eframe::run_native(
        "WallpaperManager-@lsoapps",
        native_options,
        Box::new(move |_cc| {
            let app = WallpaperApp::new(_cc);

            let ctx = _cc.egui_ctx.clone();

            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(500));
                if let Some(hwnd) = find_window_by_title("WallpaperManager-@lsoapps") {
                    store_window_hwnd(hwnd);
                }
                toggle_window_state(ctx.clone());
            });
            
            Ok(Box::new(app))
        }),
    )
}
