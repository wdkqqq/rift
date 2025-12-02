use lofty::{file::AudioFile, file::TaggedFileExt, read_from_path, tag::Accessor};
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::models::models::Song;

fn extract_and_cache_cover(audio_path: &Path, cache_dir: &Path) -> io::Result<Option<String>> {
    let tagged_file = match read_from_path(audio_path) {
        Ok(file) => file,
        Err(_) => return Ok(None),
    };

    let tag = match tagged_file.primary_tag() {
        Some(primary) => primary,
        None => match tagged_file.first_tag() {
            Some(first) => first,
            None => return Ok(None),
        },
    };

    let picture = match tag.pictures().first() {
        Some(pic) => pic,
        None => return Ok(None),
    };

    let mut hasher = Sha256::new();
    hasher.update(picture.data());
    let hash = hasher.finalize();

    let hash_hex = hash
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>();

    let extension = match picture.mime_type() {
        Some(mime_type) => match mime_type.as_str() {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/bmp" => "bmp",
            "image/tiff" => "tiff",
            "image/webp" => "webp",
            _ => {
                let data = picture.data();
                if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                    "jpg"
                } else if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
                    "png"
                } else if data.starts_with(&[0x47, 0x49, 0x46]) {
                    "gif"
                } else if data.starts_with(&[0x42, 0x4D]) {
                    "bmp"
                } else {
                    "jpg"
                }
            }
        },
        None => {
            let data = picture.data();
            if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                "jpg"
            } else if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
                "png"
            } else if data.starts_with(&[0x47, 0x49, 0x46]) {
                "gif"
            } else if data.starts_with(&[0x42, 0x4D]) {
                "bmp"
            } else {
                "jpg"
            }
        }
    };

    let file_name = format!("{}.{}", hash_hex, extension);
    let cache_file_path = cache_dir.join(&file_name);

    if cache_file_path.exists() {
        return Ok(Some(cache_file_path.to_string_lossy().to_string()));
    }

    fs::create_dir_all(cache_dir)?;

    fs::write(&cache_file_path, picture.data())?;

    Ok(Some(cache_file_path.to_string_lossy().to_string()))
}

pub fn read_audio_metadata(path: &PathBuf) -> Option<Song> {
    let cache_dir = match dirs::cache_dir() {
        Some(mut cache) => {
            cache.push("rift");
            cache.push("covers");
            cache
        }
        None => {
            let mut fallback = PathBuf::from(".");
            fallback.push(".cover_cache");
            fallback
        }
    };

    let cover_path = match extract_and_cache_cover(path, &cache_dir) {
        Ok(Some(path)) => path,
        Ok(None) => String::new(),
        Err(e) => {
            eprintln!("Warning: Could not cache cover for {:?}: {}", path, e);
            String::new()
        }
    };

    match read_from_path(path) {
        Ok(tagged_file) => {
            let tag = match tagged_file.primary_tag() {
                Some(primary_tag) => primary_tag,
                None => tagged_file.first_tag()?,
            };

            let title = tag.title().map(|s| s.to_string()).unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown Title")
                    .to_string()
            });

            let artist = tag
                .artist()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown Artist".to_string());

            let duration = tagged_file.properties().duration().as_secs();
            let minutes = duration / 60;
            let seconds = duration % 60;
            let duration_str = format!("{}:{:02}", minutes, seconds);

            Some(Song {
                title,
                subtitle: artist,
                duration: duration_str,
                cover: cover_path,
                path: path.to_string_lossy().to_string(),
            })
        }
        Err(e) => {
            eprintln!("Error reading metadata for {:?}: {}", path, e);
            None
        }
    }
}

pub fn clear_cover_cache() -> io::Result<()> {
    let cache_dir = match dirs::cache_dir() {
        Some(mut cache) => {
            cache.push("rift");
            cache.push("covers");
            cache
        }
        None => PathBuf::from("./.cover_cache"),
    };

    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)?;
        fs::create_dir_all(&cache_dir)?;
    }

    Ok(())
}

pub fn get_cached_cover_path(audio_path: &PathBuf) -> Option<String> {
    let cache_dir = match dirs::cache_dir() {
        Some(mut cache) => {
            cache.push("rift");
            cache.push("covers");
            cache
        }
        None => PathBuf::from("./.cover_cache"),
    };

    let tagged_file = read_from_path(audio_path).ok()?;
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())?;
    let picture = tag.pictures().first()?;

    let mut hasher = Sha256::new();
    hasher.update(picture.data());
    let hash = hasher.finalize();

    let hash_hex = hash
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>();

    let extensions = ["jpg", "png", "gif", "bmp", "tiff", "webp"];

    for ext in extensions.iter() {
        let file_name = format!("{}.{}", hash_hex, ext);
        let cache_path = cache_dir.join(file_name);

        if cache_path.exists() {
            return Some(cache_path.to_string_lossy().to_string());
        }
    }

    None
}
