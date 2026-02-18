// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod discord;
mod models;
mod music;
mod playlists;

use commands::commands::*;
use discord::rpc::DiscordRpcService;
use music::history::ListeningHistoryStore;
use music::library::MusicLibrary;
use music::playback::PlaybackService;
use playlists::store::PlaylistStore;

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
    let playlist_store = PlaylistStore::new();
    let listening_history = ListeningHistoryStore::new();

    tauri::Builder::default()
        .manage(music_library)
        .manage(playback_service)
        .manage(discord_rpc_service)
        .manage(playlist_store)
        .manage(listening_history)
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
            playback_toggle_mute,
            playback_get_state,
            get_app_config,
            set_app_config,
            get_playlists,
            get_playlist_tracks,
            add_track_to_playlist,
            remove_track_from_playlist,
            get_track_playlist_memberships,
            get_artist_images,
            get_home_insights,
            get_random_track
        ])
        .run(tauri::generate_context!())
        .expect("Error while running application");
}
