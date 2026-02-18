use crate::config::config::{load_config, save_config, Config};
use crate::discord::rpc::DiscordRpcService;
use crate::music::history::{ContinueListeningItem, ListeningHistoryStore, ListeningSource};
use crate::music::library::MusicLibrary;
use crate::music::playback::{PlaybackService, PlaybackState};
use crate::playlists::store::PlaylistStore;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

const HOME_SECTION_LIMIT: usize = 24;

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
    source: Option<ListeningSource>,
    playback: State<PlaybackService>,
    library: State<MusicLibrary>,
    rpc: State<DiscordRpcService>,
    history: State<ListeningHistoryStore>,
) -> Result<PlaybackState, String> {
    let playback_state = playback.load_and_play(path.clone())?;
    if let Err(error) = history.record_play(&path, source) {
        eprintln!("Failed to persist listening history: {error}");
    }
    rpc.set_track(library.by_path(&path));
    rpc.sync_playback(
        playback_state.is_playing,
        playback_state.current_time,
        playback_state.duration,
    );
    Ok(playback_state)
}

#[derive(Serialize)]
pub struct HomeInsights {
    pub continue_listening_paths: Vec<String>,
    pub continue_listening_items: Vec<ContinueListeningItem>,
    pub most_played_week_paths: Vec<String>,
}

#[tauri::command]
pub fn get_home_insights(history: State<ListeningHistoryStore>) -> HomeInsights {
    HomeInsights {
        continue_listening_paths: history.recent_paths(HOME_SECTION_LIMIT),
        continue_listening_items: history.recent_items(HOME_SECTION_LIMIT),
        most_played_week_paths: history.most_played_week_paths(HOME_SECTION_LIMIT),
    }
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
pub fn playback_toggle_mute(state: State<PlaybackService>) -> Result<PlaybackState, String> {
    state.toggle_mute()
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
pub fn set_onboarding_played(played: bool) -> Config {
    let mut config = load_config();
    config.onboarding_played = played;
    save_config(&config);
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

#[derive(Serialize)]
pub struct ArtistImage {
    pub name: String,
    pub image_filename: Option<String>,
}

#[derive(Deserialize)]
struct DeezerArtistSearchResponse {
    data: Vec<DeezerArtistItem>,
}

#[derive(Deserialize)]
struct DeezerArtistItem {
    picture_medium: Option<String>,
    picture_big: Option<String>,
    picture: Option<String>,
}

fn artist_images_dir() -> Result<PathBuf, String> {
    let dir = match dirs::cache_dir() {
        Some(mut cache) => {
            cache.push("me.wdkq.rift");
            cache.push("artists");
            cache
        }
        None => PathBuf::from(".artist_cache"),
    };
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

fn artist_key(name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(name.trim().to_lowercase().as_bytes());
    format!("{:x}", hasher.finalize())
}

fn cached_artist_image_filename(dir: &Path, key: &str) -> Option<Option<String>> {
    let image_filename = format!("{}.img", key);
    let missing_filename = format!("{}.missing", key);

    if dir.join(&image_filename).exists() {
        return Some(Some(image_filename));
    }

    if dir.join(&missing_filename).exists() {
        return Some(None);
    }

    None
}

fn resolve_deezer_artist_image(
    client: &Client,
    artist_name: &str,
) -> Result<Option<String>, String> {
    let response = client
        .get("https://api.deezer.com/search/artist")
        .query(&[("q", artist_name)])
        .send()
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let parsed: DeezerArtistSearchResponse = response.json().map_err(|error| error.to_string())?;

    Ok(parsed
        .data
        .into_iter()
        .find_map(|item| item.picture_medium.or(item.picture_big).or(item.picture)))
}

fn ensure_artist_image_cached(
    client: &Client,
    dir: &Path,
    artist_name: &str,
) -> Result<Option<String>, String> {
    let key = artist_key(artist_name);
    if let Some(cached) = cached_artist_image_filename(dir, &key) {
        return Ok(cached);
    }

    let image_url = resolve_deezer_artist_image(client, artist_name)?;
    if let Some(url) = image_url {
        let bytes = client
            .get(url)
            .send()
            .map_err(|error| error.to_string())?
            .bytes()
            .map_err(|error| error.to_string())?;
        let image_filename = format!("{}.img", key);
        fs::write(dir.join(&image_filename), &bytes).map_err(|error| error.to_string())?;
        return Ok(Some(image_filename));
    }

    fs::write(dir.join(format!("{}.missing", key)), b"").map_err(|error| error.to_string())?;
    Ok(None)
}

#[tauri::command]
pub fn get_artist_images(artist_names: Vec<String>) -> Result<Vec<ArtistImage>, String> {
    let config = load_config();
    let dir = artist_images_dir()?;
    let client = Client::builder()
        .user_agent("Rift/1.0")
        .build()
        .map_err(|error| error.to_string())?;
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for raw_name in artist_names {
        let name = raw_name.trim().to_string();
        if name.is_empty() || !seen.insert(name.clone()) {
            continue;
        }

        let key = artist_key(&name);
        let cached = cached_artist_image_filename(&dir, &key);
        let image_filename = if let Some(value) = cached {
            value
        } else if config.online_requests {
            ensure_artist_image_cached(&client, &dir, &name)?
        } else {
            None
        };

        result.push(ArtistImage {
            name,
            image_filename,
        });
    }

    Ok(result)
}

#[tauri::command]
pub fn get_random_track(
    library: State<MusicLibrary>,
) -> Result<Option<crate::models::models::Song>, String> {
    let lib = library.library.lock().map_err(|e| e.to_string())?;
    if lib.is_empty() {
        return Ok(None);
    }

    use rand::seq::IteratorRandom;
    let random_song = lib.values().choose(&mut rand::thread_rng()).cloned();
    Ok(random_song)
}
