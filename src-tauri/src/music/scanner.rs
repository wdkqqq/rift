use super::metadata::read_audio_metadata;
use crate::models::models::Song;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

pub fn get_music_directory() -> PathBuf {
    dirs::audio_dir()
        .or_else(|| dirs::home_dir().map(|p| p.join("Music")))
        .unwrap_or_else(|| {
            #[cfg(target_os = "windows")]
            {
                PathBuf::from("C:\\Users\\Public\\Music")
            }
            #[cfg(target_os = "macos")]
            {
                PathBuf::from("/Users/Shared/Music")
            }
            #[cfg(target_os = "linux")]
            {
                PathBuf::from("/var/lib/flatpak/exports/share/music")
            }
        })
}

pub fn index_music_files() -> HashMap<String, Song> {
    let music_dir = get_music_directory();
    let mut music_library = HashMap::new();

    println!("Indexing music from: {:?}", music_dir);

    if !music_dir.exists() {
        eprintln!("Music directory does not exist: {:?}", music_dir);
        return music_library;
    }

    index_directory(&music_dir, &mut music_library);

    if let Err(e) = cleanup_unused_covers(&music_library) {
        eprintln!("Error cleaning up unused covers: {}", e);
    }

    println!("Indexed {} songs", music_library.len());
    music_library
}

fn index_directory(dir: &PathBuf, music_library: &mut HashMap<String, Song>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                index_directory(&path, music_library);
            } else {
                if let Some(extension) = path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    if matches!(
                        ext.as_str(),
                        "mp3" | "flac" | "alac" | "wav" | "m4a" | "ogg" | "aac"
                    ) {
                        if let Some(song) = read_audio_metadata(&path) {
                            music_library.insert(path.to_string_lossy().to_string(), song);
                        }
                    }
                }
            }
        }
    }
}

fn cleanup_unused_covers(music_library: &HashMap<String, Song>) -> std::io::Result<()> {
    let cache_dir = match dirs::cache_dir() {
        Some(mut cache) => {
            cache.push("me.wdkq.rift");
            cache.push("covers");
            cache
        }
        None => PathBuf::from("./.cover_cache"),
    };

    if !cache_dir.exists() {
        return Ok(());
    }

    let mut used_covers = HashSet::new();

    for song in music_library.values() {
        if !song.cover.is_empty() {
            used_covers.insert(song.cover.clone());
        }
    }

    let entries = fs::read_dir(&cache_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy().to_string();

                if !used_covers.contains(&file_name_str) {
                    fs::remove_file(&path)?;
                }
            }
        }
    }

    Ok(())
}
