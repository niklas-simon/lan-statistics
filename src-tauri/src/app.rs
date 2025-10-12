use std::sync::{Arc, LazyLock};

use common::{game::Game, response::now_playing::NowPlayingResponse};
use log::warn;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;
use tokio::sync::Mutex;
use crate::{api, config::{self, Settings}, processes};

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

#[tauri::command]
async fn get_now_playing() -> Option<NowPlayingResponse> {
    processes::CTX.lock().await.last_response.clone()
}

#[tauri::command]
async fn get_games() -> Result<Vec<Game>, String> {
    api::get_games().await.map(|g| g.games)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());
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

            let handle_clone = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                *APP_HANDLE.lock().await = Some(handle_clone);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_config, set_config, get_now_playing, get_games])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
