use std::path::PathBuf;

use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::config::{self, get_config_path, Config};

#[tauri::command]
fn set_config(app_handle: AppHandle, config: Config) -> Result<(), String> {
    let old_config = config::get_or_create_config()?;

    if old_config.autostart != config.autostart {
        if config.autostart {
            app_handle.autolaunch().enable().map_err(|e| e.to_string())?;
        } else {
            app_handle.autolaunch().disable().map_err(|e| e.to_string())?;
        }
    }

    config::set_config(&config)?;
    Ok(())
}

#[tauri::command]
fn get_config() -> Result<Config, String> {
    config::get_or_create_config()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let static_path = Box::<PathBuf>::from(get_config_path().unwrap_or(PathBuf::from("config.toml")));
    let config_arg = Box::leak::<'static>(static_path).to_str().unwrap_or("config.toml");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            app.get_webview_window("main")
                .and_then(|w| w.set_focus().ok())
                .unwrap_or_else(|| eprintln!("no frontend window found"))
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["-s", "-c", config_arg]),
        ))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_autostart = config::get_or_create_config()
                .and_then(|c| Ok(c.autostart))
                .unwrap_or(false);
            let current_autostart = app.autolaunch().is_enabled().unwrap_or(false);
            if current_autostart != config_autostart {
                if config_autostart {
                    app.autolaunch().enable().map_err(|e| e.to_string())?;
                } else {
                    app.autolaunch().disable().map_err(|e| e.to_string())?;
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_config,
            get_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
