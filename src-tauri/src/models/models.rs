use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    pub title: String,
    pub subtitle: String,
    pub album: String,
    pub track_number: Option<u32>,
    pub added_at: i64,
    pub duration: String,
    pub cover: String,
    pub path: String,
    pub genre: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Playlist {
    pub slug: String,
    pub name: String,
}
