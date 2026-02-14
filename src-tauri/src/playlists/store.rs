use crate::config::config::get_config_path;
use rusqlite::{params, Connection};
use std::sync::Mutex;

const FAVORITES_SLUG: &str = "favorites-tracks";
const FAVORITES_NAME: &str = "Favorite songs";

#[derive(Debug)]
pub struct PlaylistStore {
    connection: Mutex<Connection>,
}

impl PlaylistStore {
    pub fn new() -> Self {
        let db_path = get_config_path().join("playlists.db");
        if let Err(error) = std::fs::create_dir_all(get_config_path()) {
            panic!(
                "Failed to create config directory for playlists DB: {}",
                error
            );
        }

        let connection = Connection::open(db_path).expect("Failed to open playlists DB");
        let store = Self {
            connection: Mutex::new(connection),
        };

        store.init_schema();
        store.ensure_default_playlists();
        store
    }

    fn init_schema(&self) {
        let connection = self.connection.lock().unwrap();
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 CREATE TABLE IF NOT EXISTS playlists (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    slug TEXT NOT NULL UNIQUE,
                    name TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                 );
                 CREATE TABLE IF NOT EXISTS playlist_tracks (
                    playlist_id INTEGER NOT NULL,
                    track_path TEXT NOT NULL,
                    added_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY (playlist_id, track_path),
                    FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
                 );
                 CREATE INDEX IF NOT EXISTS idx_playlist_tracks_path ON playlist_tracks(track_path);",
            )
            .expect("Failed to initialize playlists schema");
    }

    fn ensure_default_playlists(&self) {
        let connection = self.connection.lock().unwrap();
        connection
            .execute(
                "INSERT OR IGNORE INTO playlists (slug, name) VALUES (?1, ?2)",
                params![FAVORITES_SLUG, FAVORITES_NAME],
            )
            .expect("Failed to ensure default playlists");
    }

    pub fn get_all(&self) -> Result<Vec<PlaylistRecord>, String> {
        let connection = self.connection.lock().unwrap();
        let mut statement = connection
            .prepare("SELECT slug, name FROM playlists ORDER BY name COLLATE NOCASE ASC")
            .map_err(|error| error.to_string())?;

        let rows = statement
            .query_map([], |row| {
                Ok(PlaylistRecord {
                    slug: row.get(0)?,
                    name: row.get(1)?,
                })
            })
            .map_err(|error| error.to_string())?;

        let mut playlists = Vec::new();
        for row in rows {
            playlists.push(row.map_err(|error| error.to_string())?);
        }
        Ok(playlists)
    }

    pub fn get_track_paths(&self, playlist_slug: &str) -> Result<Vec<String>, String> {
        let connection = self.connection.lock().unwrap();
        let mut statement = connection
            .prepare(
                "SELECT pt.track_path
                 FROM playlist_tracks pt
                 JOIN playlists p ON p.id = pt.playlist_id
                 WHERE p.slug = ?1
                 ORDER BY pt.added_at DESC",
            )
            .map_err(|error| error.to_string())?;

        let rows = statement
            .query_map(params![playlist_slug], |row| row.get(0))
            .map_err(|error| error.to_string())?;

        let mut paths = Vec::new();
        for row in rows {
            paths.push(row.map_err(|error| error.to_string())?);
        }
        Ok(paths)
    }

    pub fn add_track(&self, playlist_slug: &str, track_path: &str) -> Result<bool, String> {
        let connection = self.connection.lock().unwrap();

        let playlist_id: i64 = connection
            .query_row(
                "SELECT id FROM playlists WHERE slug = ?1",
                params![playlist_slug],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;

        let inserted = connection
            .execute(
                "INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_path) VALUES (?1, ?2)",
                params![playlist_id, track_path],
            )
            .map_err(|error| error.to_string())?;

        Ok(inserted > 0)
    }

    pub fn remove_track(&self, playlist_slug: &str, track_path: &str) -> Result<bool, String> {
        let connection = self.connection.lock().unwrap();

        let deleted = connection
            .execute(
                "DELETE FROM playlist_tracks
                 WHERE playlist_id = (SELECT id FROM playlists WHERE slug = ?1)
                 AND track_path = ?2",
                params![playlist_slug, track_path],
            )
            .map_err(|error| error.to_string())?;

        Ok(deleted > 0)
    }

    pub fn playlist_slugs_for_track(&self, track_path: &str) -> Result<Vec<String>, String> {
        let connection = self.connection.lock().unwrap();
        let mut statement = connection
            .prepare(
                "SELECT p.slug
                 FROM playlist_tracks pt
                 JOIN playlists p ON p.id = pt.playlist_id
                 WHERE pt.track_path = ?1",
            )
            .map_err(|error| error.to_string())?;

        let rows = statement
            .query_map(params![track_path], |row| row.get(0))
            .map_err(|error| error.to_string())?;

        let mut slugs = Vec::new();
        for row in rows {
            slugs.push(row.map_err(|error| error.to_string())?);
        }
        Ok(slugs)
    }
}

#[derive(Debug, Clone)]
pub struct PlaylistRecord {
    pub slug: String,
    pub name: String,
}
