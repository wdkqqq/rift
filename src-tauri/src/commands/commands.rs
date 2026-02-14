use crate::config::config::{load_config, save_config, Config};
use crate::discord::rpc::DiscordRpcService;
use crate::music::library::MusicLibrary;
use crate::music::playback::{PlaybackService, PlaybackState};
use crate::playlists::store::PlaylistStore;
use tauri::State;

#[tauri::command]
pub fn search_music(query: String, state: State<MusicLibrary>) -> Vec<crate::models::models::Song> {
    state.search(&query)
}

#[tauri::command]
pub fn get_music_stats(state: State<MusicLibrary>) -> usize {
    state.get_stats()
}

#[tauri::command]
pub fn reindex_music(state: State<MusicLibrary>) -> usize {
    state.reindex()
}

#[tauri::command]
pub fn playback_load_and_play(
    path: String,
    playback: State<PlaybackService>,
    library: State<MusicLibrary>,
    rpc: State<DiscordRpcService>,
) -> Result<PlaybackState, String> {
    let playback_state = playback.load_and_play(path.clone())?;
    rpc.set_track(library.by_path(&path));
    rpc.sync_playback(
        playback_state.is_playing,
        playback_state.current_time,
        playback_state.duration,
    );
    Ok(playback_state)
}

#[tauri::command]
pub fn playback_play(
    playback: State<PlaybackService>,
    rpc: State<DiscordRpcService>,
) -> Result<PlaybackState, String> {
    let playback_state = playback.play()?;
    rpc.sync_playback(
        playback_state.is_playing,
        playback_state.current_time,
        playback_state.duration,
    );
    Ok(playback_state)
}

#[tauri::command]
pub fn playback_pause(
    playback: State<PlaybackService>,
    rpc: State<DiscordRpcService>,
) -> Result<PlaybackState, String> {
    let playback_state = playback.pause()?;
    rpc.sync_playback(
        playback_state.is_playing,
        playback_state.current_time,
        playback_state.duration,
    );
    Ok(playback_state)
}

#[tauri::command]
pub fn playback_seek(
    position_seconds: f64,
    state: State<PlaybackService>,
    rpc: State<DiscordRpcService>,
) -> Result<PlaybackState, String> {
    let playback_state = state.seek(position_seconds)?;
    rpc.sync_playback(
        playback_state.is_playing,
        playback_state.current_time,
        playback_state.duration,
    );
    Ok(playback_state)
}

#[tauri::command]
pub fn playback_set_volume(
    volume: f32,
    state: State<PlaybackService>,
) -> Result<PlaybackState, String> {
    state.set_volume(volume)
}

#[tauri::command]
pub fn playback_get_state(state: State<PlaybackService>) -> Result<PlaybackState, String> {
    state.get_state()
}

#[tauri::command]
pub fn get_app_config() -> Config {
    load_config()
}

#[tauri::command]
pub fn set_app_config(config: Config, rpc: State<DiscordRpcService>) -> Config {
    save_config(&config);
    rpc.set_enabled(config.discord_rpc);
    config
}

#[tauri::command]
pub fn get_playlists(
    playlists: State<PlaylistStore>,
) -> Result<Vec<crate::models::models::Playlist>, String> {
    let records = playlists.get_all()?;
    Ok(records
        .into_iter()
        .map(|playlist| crate::models::models::Playlist {
            slug: playlist.slug,
            name: playlist.name,
        })
        .collect())
}

#[tauri::command]
pub fn get_playlist_tracks(
    playlist_slug: String,
    playlists: State<PlaylistStore>,
    library: State<MusicLibrary>,
) -> Result<Vec<crate::models::models::Song>, String> {
    let paths = playlists.get_track_paths(&playlist_slug)?;
    let mut songs = Vec::new();

    for path in paths {
        if let Some(song) = library.by_path(&path) {
            songs.push(song);
        }
    }

    Ok(songs)
}

#[tauri::command]
pub fn add_track_to_playlist(
    playlist_slug: String,
    track_path: String,
    playlists: State<PlaylistStore>,
    library: State<MusicLibrary>,
) -> Result<bool, String> {
    if library.by_path(&track_path).is_none() {
        return Err("Track was not found in indexed library".to_string());
    }

    playlists.add_track(&playlist_slug, &track_path)
}

#[tauri::command]
pub fn remove_track_from_playlist(
    playlist_slug: String,
    track_path: String,
    playlists: State<PlaylistStore>,
) -> Result<bool, String> {
    playlists.remove_track(&playlist_slug, &track_path)
}

#[tauri::command]
pub fn get_track_playlist_memberships(
    track_path: String,
    playlists: State<PlaylistStore>,
) -> Result<Vec<String>, String> {
    playlists.playlist_slugs_for_track(&track_path)
}
