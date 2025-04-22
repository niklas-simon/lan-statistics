use std::{fs::File, io::Write, path::PathBuf, sync::Mutex};

use config_file::FromConfigFile;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub id: String,
    pub remote: String,
    pub name: Option<String>,
    pub autostart: bool
}

pub fn default_config() -> Config {
    Config {
        id: Uuid::new_v4().to_string(),
        remote: String::from("https://lan.pein-gera.de"),
        name: None,
        autostart: true
    }
}

static CONFIG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn set_config_path(path: PathBuf) -> Result<(), String> {
    let mut config_path = CONFIG_PATH.lock().map_err(|e| e.to_string())?;

    *config_path = Some(path);

    Ok(())
}

pub fn get_config_path() -> Result<PathBuf, String> {
    let config_path = CONFIG_PATH.lock().map_err(|e| e.to_string())?;
    let path = match config_path.clone() {
        Some(p) => Ok(p),
        None => Err(String::from("config path unspecified"))
    }?;

    Ok(path)
}

pub fn get_config() -> Result<Config, String> {
    Config::from_config_file(get_config_path()?).map_err(|e| e.to_string())
}

pub fn set_config(config: &Config) -> Result<(), String> {
    let path = get_config_path()?;

    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    let content = toml::to_string_pretty(config).map_err(|e| e.to_string())?;

    file.write(content.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn create_default_config() -> Result<Config, String> {
    let default: Config = default_config();

    set_config(&default)?;

    Ok(default)
}

pub fn get_or_create_config() -> Result<Config, String> {
    get_config().or_else(|e| {
        eprintln!("error getting config: {e}");
        create_default_config()
    })
}