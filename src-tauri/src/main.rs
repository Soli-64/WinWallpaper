// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "--disable-audio-output --disable-background-networking --mute-audio --disk-cache-size=1 --media-cache-size=1");
    win_wallpaper_lib::run()
}
