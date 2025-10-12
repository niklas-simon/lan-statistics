use std::{collections::HashSet, sync::LazyLock};

use reqwest::{StatusCode, Client};
use chrono::{DateTime, Local};
use common::{response::{games::GamesResponse, now_playing::{NowPlayingEntry, NowPlayingResponse, Player}}};
use serde::Serialize;
use tauri::Emitter;
use tokio::sync::Mutex;

use crate::{app::APP_HANDLE, config::get_or_create_config};

static GAMES: LazyLock<Mutex<Option<GamesResponse>>> = LazyLock::new(|| Mutex::new(None));

pub async fn put_now_playing(games: HashSet<String>, last_update: DateTime<Local>) -> Result<Option<NowPlayingResponse>, String> {
    let config = get_or_create_config(false)?;
    let client = Client::default();
    let body = NowPlayingEntry {
        games: games.into_iter().collect(),
        player: Player {
            id: config.id,
            name: config.name.unwrap_or("unknown".to_string())
        }
    };

    let res = client.put(config.remote + "/api/v1/now-playing")
        .query(&[("last_update", last_update.format("%Y-%m-%dT%H:%M:%S").to_string())])
        .bearer_auth(config.password.unwrap_or("".to_string()))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Keine Verbindung zum Server\n{e}"))?;

    match res.status() {
        StatusCode::OK => res.json::<NowPlayingResponse>().await
            .map(Some)
            .map_err(|e| format!("Ungültige Antwort vom Server\n{e}")),
        StatusCode::NOT_MODIFIED => Ok(None),
        code => {
            let body = res.text().await.unwrap_or("".to_string());

            if code == 401 {
                Err(format!("Passwort inkorrekt\n{code}: {body}"))
            } else if code == 400 {
                Err(format!("Ungültige Anfrage zum Server\n{code}: {body}"))
            } else {
                Err(format!("Unbekannter Fehler\n{code}: {body}"))
            }
        }
    }
}

pub async fn get_games() -> Result<GamesResponse, String> {
    if let Some(games) = GAMES.lock().await.as_ref() {
        return Ok(games.clone());
    }

    let config = get_or_create_config(false)?;
    let client = Client::default();
    let games = client.get(config.remote + "/api/v1/games")
        .send().await
        .map_err(|e| format!("Keine Verbindung zum Server\n{e}"))?
        .json::<GamesResponse>().await
        .map_err(|e| format!("Ungültige Antwort vom Server\n{e}"))?;

    *GAMES.lock().await = Some(games.clone());

    Ok(games)
}

pub async fn send_event<T: Serialize>(event: &str, obj: &T) -> Result<(), String> {
    let app_lock_opt = APP_HANDLE.lock().await;
    let Some(app_lock) = app_lock_opt.as_ref() else {
        return Err("app handle not set".to_string())
    };

    app_lock.emit(event, obj).map_err(|e| format!("could not emit event: {e}"))
}