use crate::config::config::load_config;
use crate::models::models::Song;
use chrono::Utc;
use discord_rich_presence::activity::ActivityType;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const DISCORD_CLIENT_ID: &str = "1443346664822673538";

#[derive(Debug, Clone, Default)]
struct RpcTrack {
    title: String,
    artist: String,
    album: String,
}

#[derive(Debug, Deserialize)]
struct ItunesSearchResponse {
    results: Vec<ItunesTrack>,
}

#[derive(Debug, Deserialize)]
struct ItunesTrack {
    #[serde(rename = "artworkUrl100")]
    artwork_url_100: Option<String>,
    #[serde(rename = "artworkUrl60")]
    artwork_url_60: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct RpcState {
    track: Option<RpcTrack>,
    is_playing: bool,
    current_time: f64,
    duration: f64,
    rpc_enabled: bool,
}

pub struct DiscordRpcService {
    state: Arc<Mutex<RpcState>>,
}

impl DiscordRpcService {
    pub fn start() -> Self {
        let config = load_config();
        let mut initial_state = RpcState::default();
        initial_state.rpc_enabled = config.discord_rpc;
        let state = Arc::new(Mutex::new(initial_state));

        let state_for_thread = Arc::clone(&state);
        thread::spawn(move || {
            run_rpc_loop(state_for_thread);
        });

        Self { state }
    }

    pub fn set_track(&self, song: Option<Song>) {
        if let Ok(mut state) = self.state.lock() {
            state.track = song.map(|song| RpcTrack {
                title: song.title,
                artist: song.subtitle,
                album: song.album,
            });
        }
    }

    pub fn sync_playback(&self, is_playing: bool, current_time: f64, duration: f64) {
        if let Ok(mut state) = self.state.lock() {
            state.is_playing = is_playing;
            state.current_time = current_time.max(0.0);
            state.duration = duration.max(0.0);
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.rpc_enabled = enabled;
        }
    }
}

fn run_rpc_loop(state: Arc<Mutex<RpcState>>) {
    let http_client = Client::builder()
        .timeout(Duration::from_secs(4))
        .build()
        .ok();
    let mut cover_cache: HashMap<String, Option<String>> = HashMap::new();
    let mut last_payload = String::new();
    let mut client: Option<DiscordIpcClient> = None;

    loop {
        thread::sleep(Duration::from_secs(2));

        let snapshot = match state.lock() {
            Ok(state) => state.clone(),
            Err(_) => continue,
        };

        if !snapshot.rpc_enabled {
            if let Some(active_client) = client.as_mut() {
                let _ = active_client.clear_activity();
                let _ = active_client.close();
            }
            client = None;
            last_payload.clear();
            continue;
        }

        if client.is_none() {
            let mut new_client = DiscordIpcClient::new(DISCORD_CLIENT_ID);
            if let Err(error) = new_client.connect() {
                eprintln!("Cannot connect to Discord: {}", error);
                continue;
            }
            client = Some(new_client);
        }

        let Some(active_client) = client.as_mut() else {
            continue;
        };

        let Some(track) = snapshot.track else {
            let _ = active_client.clear_activity();
            last_payload.clear();
            continue;
        };

        let payload_key = format!(
            "{}|{}|{}|{}|{}",
            track.title,
            track.artist,
            snapshot.is_playing,
            snapshot.current_time as i64,
            snapshot.duration as i64
        );
        if payload_key == last_payload {
            continue;
        }

        let cover_key = format!("{}|{}", track.artist, track.title);
        let cover_url = cover_cache
            .entry(cover_key)
            .or_insert_with(|| {
                http_client
                    .as_ref()
                    .and_then(|client| fetch_track_cover_url(client, &track))
            })
            .clone();
        let large_image = cover_url.as_deref().unwrap_or("logo");

        let mut activity = activity::Activity::new()
            .activity_type(ActivityType::Listening)
            .details(&track.title)
            .state(&track.artist)
            .assets(
                activity::Assets::new()
                    .large_image(large_image)
                    .large_text(&track.album),
            );

        if snapshot.is_playing && snapshot.duration > 0.0 {
            let now = Utc::now().timestamp();
            let start = now - snapshot.current_time.floor() as i64;
            let end = start + snapshot.duration.ceil() as i64;
            activity = activity.timestamps(activity::Timestamps::new().start(start).end(end));
        }

        if let Err(error) = active_client.set_activity(activity) {
            eprintln!("Cannot set activity: {}", error);
            let _ = active_client.close();
            client = None;
            continue;
        }

        last_payload = payload_key;
    }
}

fn fetch_track_cover_url(client: &Client, track: &RpcTrack) -> Option<String> {
    if track.title.trim().is_empty() {
        return None;
    }

    let query = if track.artist.trim().is_empty() {
        track.title.clone()
    } else {
        format!("{} {}", track.artist, track.title)
    };

    let response = client
        .get("https://itunes.apple.com/search")
        .query(&[
            ("term", query.as_str()),
            ("entity", "song"),
            ("limit", "1"),
            ("country", "US"),
        ])
        .send()
        .ok()?;

    let parsed: ItunesSearchResponse = response.json().ok()?;
    let item = parsed.results.first()?;
    let raw_url = item
        .artwork_url_100
        .as_ref()
        .or(item.artwork_url_60.as_ref())?;

    if !raw_url.starts_with("http://") && !raw_url.starts_with("https://") {
        return None;
    }

    Some(upscale_itunes_artwork(raw_url))
}

fn upscale_itunes_artwork(url: &str) -> String {
    url.replace("100x100bb", "600x600bb")
        .replace("100x100", "600x600")
        .replace("60x60bb", "600x600bb")
        .replace("60x60", "600x600")
}
