use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    pub title: String,
    pub subtitle: String,
    pub album: String,
    pub added_at: i64,
    pub duration: String,
    pub cover: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Playlist {
    pub slug: String,
    pub name: String,
}
