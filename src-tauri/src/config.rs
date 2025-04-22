use std::{fs::File, io::Write, path::PathBuf, sync::Mutex, fmt::Write as _};

use config_file::FromConfigFile;
use log::warn;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub id: String,
    pub remote: String,
    pub name: Option<String>,
    pub autostart: bool,
    pub password: Option<String>
}

pub fn default_config() -> Config {
    Config {
        id: Uuid::new_v4().to_string(),
        remote: String::from("https://lan.pein-gera.de"),
        name: None,
        autostart: true,
        password: None
    }
}

const PASSWORD_PLACEHOLDER: &str = "(unchanged)";

static CONFIG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn set_config_path(path: &PathBuf) -> Result<(), String> {
    let mut config_path = CONFIG_PATH.lock().map_err(|e| e.to_string())?;

    *config_path = Some(path.clone());

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

pub fn get_config(censor: bool) -> Result<Config, String> {
    Config::from_config_file(get_config_path()?)
        .map(|mut c| {
            if censor {
                c.password = c.password
                    .and_then(|_| Some(String::from(PASSWORD_PLACEHOLDER)));
                c
            } else {
                c
            }
        })
        .map_err(|e| e.to_string())
}

pub fn set_config(config: &Config) -> Result<(), String> {
    let path = get_config_path()?;

    let password = config.password.clone().map(|p|
        if p == PASSWORD_PLACEHOLDER {
            Ok::<Option<String>, String>(get_config(false).ok()
                .and_then(|c| c.password))
        } else {
            let mut hasher = Sha256::new();
            hasher.update(p);
            let mut out = String::new();
            write!(out, "{:x}", hasher.finalize()).map_err(|e| e.to_string())?;
            Ok(Some(out))
        }
    ).unwrap_or(Ok(None))?;
    let mut new_config = config.clone();
    new_config.password = password;

    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    let content = toml::to_string_pretty(&new_config).map_err(|e| e.to_string())?;

    file.write(content.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn create_default_config() -> Result<Config, String> {
    let default: Config = default_config();

    set_config(&default)?;

    Ok(default)
}

pub fn get_or_create_config(censor: bool) -> Result<Config, String> {
    get_config(censor).or_else(|e| {
        warn!("error getting config: {e}");
        create_default_config()
    })
}