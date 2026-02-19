use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

const MAX_HISTORY_EVENTS: usize = 5000;
const WEEK_SECONDS: i64 = 7 * 24 * 60 * 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListeningSource {
    pub kind: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContinueListeningItem {
    Album {
        path: String,
    },
    Playlist {
        playlist_slug: String,
        playlist_name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ListeningEvent {
    path: String,
    played_at: i64,
    #[serde(default)]
    source: Option<ListeningSource>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ListeningHistoryData {
    events: Vec<ListeningEvent>,
}

pub struct ListeningHistoryStore {
    data: Mutex<ListeningHistoryData>,
    file_path: PathBuf,
}

impl ListeningHistoryStore {
    pub fn new() -> Self {
        let file_path = history_file_path();
        let data = load_history(&file_path).unwrap_or_default();
        Self {
            data: Mutex::new(data),
            file_path,
        }
    }

    pub fn record_play(&self, path: &str, source: Option<ListeningSource>) -> Result<(), String> {
        let clean_path = path.trim();
        if clean_path.is_empty() {
            return Ok(());
        }

        let mut data = self
            .data
            .lock()
            .map_err(|_| "Listening history mutex is poisoned".to_string())?;

        data.events.push(ListeningEvent {
            path: clean_path.to_string(),
            played_at: Utc::now().timestamp(),
            source: source.and_then(normalize_source),
        });

        if data.events.len() > MAX_HISTORY_EVENTS {
            let overflow = data.events.len() - MAX_HISTORY_EVENTS;
            data.events.drain(0..overflow);
        }

        persist_history(&self.file_path, &data)
    }

    pub fn recent_paths(&self, limit: usize) -> Vec<String> {
        if limit == 0 {
            return Vec::new();
        }

        let data = match self.data.lock() {
            Ok(data) => data,
            Err(_) => return Vec::new(),
        };

        let mut unique = HashSet::new();
        let mut result = Vec::with_capacity(limit);

        for event in data.events.iter().rev() {
            if unique.insert(event.path.clone()) {
                result.push(event.path.clone());
                if result.len() >= limit {
                    break;
                }
            }
        }

        result
    }

    pub fn recent_items(&self, limit: usize) -> Vec<ContinueListeningItem> {
        if limit == 0 {
            return Vec::new();
        }

        let data = match self.data.lock() {
            Ok(data) => data,
            Err(_) => return Vec::new(),
        };

        let mut unique = HashSet::new();
        let mut result = Vec::with_capacity(limit);

        for event in data.events.iter().rev() {
            match event.source.as_ref().map(|source| source.kind.as_str()) {
                Some("playlist") => {
                    let Some(slug) = event
                        .source
                        .as_ref()
                        .and_then(|source| source.id.as_deref())
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(|value| value.to_string())
                    else {
                        continue;
                    };

                    let unique_key = format!("playlist::{}", slug.to_lowercase());
                    if !unique.insert(unique_key) {
                        continue;
                    }

                    let name = event
                        .source
                        .as_ref()
                        .and_then(|source| source.name.as_ref())
                        .map(|value| value.trim())
                        .filter(|value| !value.is_empty())
                        .unwrap_or(&slug)
                        .to_string();

                    result.push(ContinueListeningItem::Playlist {
                        playlist_slug: slug,
                        playlist_name: name,
                    });
                }
                _ => {
                    let unique_key = format!("album::{}", event.path);
                    if !unique.insert(unique_key) {
                        continue;
                    }
                    result.push(ContinueListeningItem::Album {
                        path: event.path.clone(),
                    });
                }
            }

            if result.len() >= limit {
                break;
            }
        }

        result
    }

    pub fn most_played_week_paths(&self, limit: usize) -> Vec<String> {
        if limit == 0 {
            return Vec::new();
        }

        let data = match self.data.lock() {
            Ok(data) => data,
            Err(_) => return Vec::new(),
        };

        let week_cutoff = Utc::now().timestamp() - WEEK_SECONDS;
        let mut counts: HashMap<String, (usize, i64)> = HashMap::new();

        for event in &data.events {
            if event.played_at < week_cutoff {
                continue;
            }
            let entry = counts.entry(event.path.clone()).or_insert((0, 0));
            entry.0 += 1;
            if event.played_at > entry.1 {
                entry.1 = event.played_at;
            }
        }

        let mut ranked: Vec<(String, usize, i64)> = counts
            .into_iter()
            .map(|(path, (count, last_played_at))| (path, count, last_played_at))
            .collect();

        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.2.cmp(&a.2)));

        ranked
            .into_iter()
            .take(limit)
            .map(|(path, _, _)| path)
            .collect()
    }
}

fn normalize_source(source: ListeningSource) -> Option<ListeningSource> {
    let kind = match source.kind.trim().to_lowercase().as_str() {
        "album" => "album",
        "playlist" => "playlist",
        _ => "other",
    }
    .to_string();

    let id = source
        .id
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    let name = source
        .name
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

    if kind == "other" && id.is_none() && name.is_none() {
        return None;
    }

    Some(ListeningSource { kind, id, name })
}

fn history_file_path() -> PathBuf {
    let mut base = dirs::data_local_dir()
        .or_else(dirs::cache_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    base.push("me.wdkq.rift");
    base.push("listening_history.json");
    base
}

fn load_history(path: &PathBuf) -> Result<ListeningHistoryData, String> {
    if !path.exists() {
        return Ok(ListeningHistoryData::default());
    }

    let raw = fs::read(path).map_err(|error| error.to_string())?;
    serde_json::from_slice(&raw).map_err(|error| error.to_string())
}

fn persist_history(path: &PathBuf, data: &ListeningHistoryData) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let raw = serde_json::to_vec(data).map_err(|error| error.to_string())?;
    fs::write(path, raw).map_err(|error| error.to_string())
}
