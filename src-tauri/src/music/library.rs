use super::scanner::index_music_files;
use crate::models::models::Song;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug)]
pub struct MusicLibrary {
    pub library: Mutex<HashMap<String, Song>>,
}

impl MusicLibrary {
    pub fn new() -> Self {
        let music_library = index_music_files();
        Self {
            library: Mutex::new(music_library),
        }
    }

    pub fn reindex(&self) -> usize {
        let new_library = index_music_files();
        let mut library = self.library.lock().unwrap();
        *library = new_library;
        library.len()
    }

    pub fn get_stats(&self) -> usize {
        let library = self.library.lock().unwrap();
        library.len()
    }

    pub fn search(&self, query: &str) -> Vec<Song> {
        let library = self.library.lock().unwrap();
        let query_lower = query.trim().to_lowercase();

        let mut results: Vec<Song> = library
            .values()
            .filter(|song| {
                song.title.to_lowercase().contains(&query_lower)
                    || song.subtitle.to_lowercase().contains(&query_lower)
                    || song.album.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();

        results.sort_by(|a, b| {
            let score = |song: &Song| -> usize {
                if song.title.to_lowercase().contains(&query_lower) {
                    return 0;
                }
                if song.subtitle.to_lowercase().contains(&query_lower) {
                    return 1;
                }
                if song.album.to_lowercase().contains(&query_lower) {
                    return 2;
                }
                3
            };

            score(a)
                .cmp(&score(b))
                .then_with(|| a.subtitle.cmp(&b.subtitle))
                .then_with(|| a.album.cmp(&b.album))
                .then_with(|| a.title.cmp(&b.title))
        });

        results
    }

    pub fn by_path(&self, path: &str) -> Option<Song> {
        let library = self.library.lock().unwrap();
        library.get(path).cloned()
    }
}
