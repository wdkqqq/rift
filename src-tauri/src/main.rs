// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod discord;
mod models;
mod music;

use commands::commands::*;
use discord::rpc::DiscordRpcService;
use music::library::MusicLibrary;
use music::playback::PlaybackService;

use tauri_plugin_fs::init;

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
    let playback_service = PlaybackService::start();
    let discord_rpc_service = DiscordRpcService::start();

    tauri::Builder::default()
        .manage(music_library)
        .manage(playback_service)
        .manage(discord_rpc_service)
        .plugin(init())
        .invoke_handler(tauri::generate_handler![
            search_music,
            get_music_stats,
            reindex_music,
            playback_load_and_play,
            playback_play,
            playback_pause,
            playback_seek,
            playback_set_volume,
            playback_get_state
        ])
        .run(tauri::generate_context!())
        .expect("Error while running application");
}
