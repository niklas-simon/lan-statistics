use std::{collections::{BTreeMap, HashMap}, net::{IpAddr, Ipv4Addr}, sync::{Arc, LazyLock, Mutex}, vec};

use base64::{prelude::BASE64_STANDARD, Engine};
use log::{error, warn};
use serde::Serialize;
use spacetimedb_sdk::{Identity, Table};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::{api::{connect, creds_store, CTX}, app, config::{self, Settings}, module_bindings::{set_userinfo, GameTableAccess, RemoteTables, UserInfo, UserTableAccess}};

pub static APP_HANDLE: LazyLock<Arc<Mutex<Option<AppHandle>>>> = LazyLock::new(|| Arc::new(Mutex::new(None)));

#[tauri::command]
async fn set_config(app_handle: AppHandle, config: Settings) -> Result<(), String> {
    let old_config = config::get_config_or_default();

    if old_config.autostart != config.autostart {
        if config.autostart {
            app_handle.autolaunch().enable().map_err(|e| e.to_string())?;
        } else {
            app_handle.autolaunch().disable().map_err(|e| e.to_string())?;
        }
    }

    config::set_config(&config).await?;
    Ok(())
}

#[tauri::command]
async fn get_config() -> Result<Settings, String> {
    Ok(config::get_config_or_default())
}

fn get_games_map(db: &RemoteTables) -> HashMap::<String, Vec<String>> {
    let mut users = HashMap::<Identity, String>::new();
    let mut games_by_player = HashMap::<String, Vec<String>>::new();

    for user in db.user().iter() {
        users.insert(user.identity, user.name);
    }

    for game in db.game().iter() {
        let Some(user) = users.get(&game.user) else {
            continue;
        };
        let entry = games_by_player.entry(user.clone()).or_insert(vec![]);

        entry.push(game.name);
    }

    games_by_player
}

pub fn send_games_update(db: &RemoteTables) {
    let games_by_player = get_games_map(db);

    if let Err(err) = app::send_event("games", &games_by_player) {
        warn!("error sending games event: {}", err);
    }
}

#[tauri::command]
async fn get_games() -> HashMap<String, Vec<String>> {
    let ctx_lock = CTX.lock().await;
    if let Some(ref conn) = *ctx_lock {
        get_games_map(&conn.db)
    } else {
        HashMap::new()
    }
}

#[tauri::command]
async fn get_connection() -> bool {
    let ctx_lock = CTX.lock().await;
    ctx_lock.is_some()
}

#[tauri::command]
async fn retry_connection() -> Result<(), String> {
    connect().await
}

fn get_claims(token: String) -> Result<BTreeMap<String, String>, String> {
    let parts = token.split(".").collect::<Vec<&str>>();

    if parts.len() != 3 {
        return Err(String::from("bad token: could not be split into 3 parts by '.'"))
    }

    let json_str = BASE64_STANDARD.decode(parts[2])
        .map_err(|e| e.to_string())?;

    serde_json::from_slice::<BTreeMap<String, String>>(&json_str)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn login(token: String) -> Result<(), String> {
    println!("login: {}", token);
    let store = creds_store();

    store.save(&token)
        .map_err(|e| e.to_string())?;

    connect().await?;

    let claims = get_claims(token)?;
    let Some(email) = claims.get("email").cloned() else { return Err(String::from("claim not found: email")) };
    let Some(picture) = claims.get("picture").cloned() else { return Err(String::from("claim not found: picture")) };
    let name = claims.get("given_name").cloned()
        .or(claims.get("name").cloned())
        .unwrap_or(email.clone());

    let networks = sysinfo::Networks::new_with_refreshed_list();
    let mut addrs: Vec<(Ipv4Addr, u64)> = vec![];

    for (name, network) in networks.iter() {
        if !name.contains("Ethernet") && !name.contains("WiFi") {
            continue;
        }

        for ip in network.ip_networks().iter() {
            let IpAddr::V4(ipv4) = ip.addr else {
                continue;
            };

            if ipv4.is_private() {
                addrs.push((ipv4, network.total_packets_received()));
            }
        }
    }

    let ip = addrs.iter().max_by_key(|a| a.1).map(|a| a.0.to_string());

    let userinfo = UserInfo {
        email,
        ip: ip.unwrap_or("unknown".to_string()),
        name,
        picture
    };

    if let Some(ref conn) = *CTX.lock().await {
        conn.reducers.set_userinfo(userinfo)
            .map_err(|e| e.to_string())?;
    } else {
        warn!("app: could not get spacetime connection");
    }

    Ok(())
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
            let config_autostart = config::get_config_or_default().autostart;
            let current_autostart = app.autolaunch().is_enabled().unwrap_or(false);
            if current_autostart != config_autostart {
                let res = if config_autostart {
                        app.autolaunch().enable()
                } else {
                    app.autolaunch().disable()
                };
                if let Err(err) = res {
                    warn!("app: error configurating autostart: {}", err.to_string());
                }
            }

            let Ok(mut app_handle) = APP_HANDLE.lock() else {
                error!("app: Error getting lock on AppHandle");
                return Ok(());
            };

            *app_handle = Some(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            set_config,
            get_games,
            get_connection,
            retry_connection,
            login
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn send_event<T: Serialize>(event: &str, obj: &T) -> Result<(), String> {
    let Ok(guard) = APP_HANDLE.lock() else {
        return Err(String::from("send_event: Could not get lock on AppHandle"));
    };

    let Some(ref app_handle) = *guard else {
        return Err(String::from("send_event: Could not get AppHandle"));
    };

    app_handle.emit(event, obj).map_err(|e| e.to_string())
}