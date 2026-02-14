use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    pub title: String,
    pub subtitle: String,
    pub album: String,
    pub duration: String,
    pub cover: String,
    pub path: String,
}
