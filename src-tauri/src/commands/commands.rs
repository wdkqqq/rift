use crate::discord::rpc::DiscordRpcService;
use crate::music::library::MusicLibrary;
use crate::music::playback::{PlaybackService, PlaybackState};
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
