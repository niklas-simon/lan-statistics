use std::sync::{Arc, LazyLock, Mutex};

use log::{error, warn};
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;
use crate::config::{self, Settings};

pub static APP_HANDLE: LazyLock<Arc<Mutex<Option<AppHandle>>>> = LazyLock::new(|| Arc::new(Mutex::new(None)));

#[tauri::command]
fn set_config(app_handle: AppHandle, config: Settings) -> Result<(), String> {
    let old_config = config::get_or_create_config(true)?;

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
fn get_config() -> Result<Settings, String> {
    config::get_or_create_config(true)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            app.get_webview_window("main")
                .and_then(|w| w.set_focus().ok())
                .unwrap_or_else(|| warn!("no frontend window found"))
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let config_autostart = config::get_or_create_config(true)
                .map(|c| c.autostart)
                .unwrap_or(false);
            let current_autostart = app.autolaunch().is_enabled().unwrap_or(false);
            if current_autostart != config_autostart {
                if config_autostart {
                    app.autolaunch().enable().map_err(|e| e.to_string())?;
                } else {
                    app.autolaunch().disable().map_err(|e| e.to_string())?;
                }
            }

            let Ok(mut app_handle) = APP_HANDLE.lock() else {
                error!("app: Error getting lock on AppHandle");
                return Ok(())
            };

            *app_handle = Some(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_config, set_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
