use std::{fs::File, io::{Read, Write}};

use log::warn;
use serde::{Deserialize, Serialize};

use crate::api::connect;

#[derive(Deserialize, Serialize, Clone)]
pub struct Settings {
    pub autostart: bool,
    pub advanced: bool,
    pub remote_url: String,
    pub remote_db: String
}

pub fn default_config() -> Settings {
    Settings {
        autostart: true,
        advanced: false,
        remote_url: String::from("https://lan.pein-gera.de"),
        remote_db: String::from("lan-manager")
    }
}

pub const CONFIG_PATH: &str = "config.toml";

fn get_config() -> Result<Settings, String> {
    let mut config_file = File::open(CONFIG_PATH)
        .map_err(|e| e.to_string())?;
    let mut config_str = String::new();
    
    config_file.read_to_string(&mut config_str)
        .map_err(|e| e.to_string())?;

    toml::from_str::<Settings>(&mut config_str)
        .map_err(|e| e.to_string())
}

pub fn get_config_or_default() -> Settings {
    match get_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("config: error reading config: {}", e);
            default_config()
        }
    }
}

pub async fn set_config(config: &Settings) -> Result<(), String> {
    let old_config = get_config_or_default();
    let new_config = config.clone();

    let mut file = File::create(CONFIG_PATH).map_err(|e| e.to_string())?;
    let content = toml::to_string_pretty(&new_config).map_err(|e| e.to_string())?;

    file.write(content.as_bytes()).map_err(|e| e.to_string())?;

    if old_config.remote_db != new_config.remote_db || old_config.remote_url != new_config.remote_url {
        if let Err(err) = connect().await {
            return Err(err);
        }
    }

    Ok(())
}