// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod discord;
mod models;
mod music;

use commands::commands::*;
use discord::rpc::discord_rpc;
use music::library::MusicLibrary;
use std::thread;

fn main() {
    #[cfg(target_os = "linux")]
    // Temporary Nvidia/Wayland workaround for Tauri webview crashes
    if std::path::Path::new("/proc/driver/nvidia/version").exists() {
        unsafe {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            std::env::set_var("GDK_BACKEND", "x11");
        }
    }

    let music_library = MusicLibrary::new();

    thread::spawn(|| {
        discord_rpc();
    });

    tauri::Builder::default()
        .manage(music_library)
        .invoke_handler(tauri::generate_handler![
            search_music,
            get_music_stats,
            reindex_music
        ])
        .run(tauri::generate_context!())
        .expect("Error while running application");
}
