use crate::music::library::MusicLibrary;
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
