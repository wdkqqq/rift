use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub launch_at_startup: bool,
    pub volume_normalization: bool,
    pub autoplay: bool,
    pub crossfade: bool,
    pub gapless_playback: bool,
    pub normalize_by_album: bool,
    pub discord_rpc: bool,
    pub online_requests: bool,
    pub automatic_updates: bool,
    pub dark_theme: bool,
    pub native_decorations: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            launch_at_startup: false,
            volume_normalization: false,
            autoplay: true,
            crossfade: false,
            gapless_playback: true,
            normalize_by_album: false,
            discord_rpc: true,
            online_requests: true,
            automatic_updates: true,
            dark_theme: true,
            native_decorations: false,
        }
    }
}

pub fn get_config_path() -> PathBuf {
    let mut config_dir = dirs::config_dir()
        .or_else(|| dirs::data_local_dir())
        .or_else(|| dirs::home_dir().map(|p| p.join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));

    config_dir.push("Rift");
    config_dir
}

pub fn load_config() -> Config {
    let config_path = get_config_path().join("config.json");

    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => config,
                Err(_) => {
                    eprintln!("Error parsing config file, using default config");
                    Config::default()
                }
            },
            Err(_) => {
                eprintln!("Error reading config file, using default config");
                Config::default()
            }
        }
    } else {
        let default_config = Config::default();
        save_config(&default_config);
        default_config
    }
}

pub fn save_config(config: &Config) {
    let config_dir = get_config_path();
    let config_path = config_dir.join("config.json");

    if let Err(e) = fs::create_dir_all(&config_dir) {
        eprintln!("Error creating config directory: {}", e);
        return;
    }

    match serde_json::to_string_pretty(config) {
        Ok(json) => {
            if let Err(e) = fs::write(&config_path, json) {
                eprintln!("Error saving config file: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Error serializing config: {}", e);
        }
    }
}
