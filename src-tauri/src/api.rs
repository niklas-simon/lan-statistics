use std::{collections::HashSet, env, fs, sync::LazyLock, time::Duration};

use regex::Regex;
use reqwest::{Client, ClientBuilder, StatusCode, header};
use chrono::{DateTime, Local};
use common::{game::Game, response::{games::GamesResponse, now_playing::{NowPlayingEntry, NowPlayingResponse, Player}}};
use serde::Serialize;
use tauri::Emitter;
use tokio::sync::Mutex;

use crate::{app::APP_HANDLE, config::get_or_create_config};

static GAMES: LazyLock<Mutex<Option<GamesResponse>>> = LazyLock::new(|| Mutex::new(None));
static CLIENT: LazyLock<Client> = LazyLock::new(|| ClientBuilder::default()
    .timeout(Duration::from_secs(5))
    .build()
    .expect("failed to build client")
);

pub async fn put_now_playing(games: HashSet<String>, last_update: DateTime<Local>) -> Result<Option<NowPlayingResponse>, String> {
    let config = get_or_create_config(false)?;
    let body = NowPlayingEntry {
        games: games.into_iter().collect(),
        player: Player {
            id: config.id,
            name: config.name.unwrap_or("unknown".to_string())
        }
    };

    let res = CLIENT.put(config.remote + "/api/v1/now-playing")
        .query(&[("last_update", last_update.to_rfc3339())])
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
    let games = CLIENT.get(config.remote + "/api/v1/games")
        .send().await
        .map_err(|e| format!("Keine Verbindung zum Server\n{e}"))?
        .json::<GamesResponse>().await
        .map_err(|e| format!("Ungültige Antwort vom Server\n{e}"))?;

    *GAMES.lock().await = Some(games.clone());

    Ok(games)
}

pub async fn get_icon(game: &Game) -> Option<String> {
    let config = get_or_create_config(false).ok()?;
    let tmp_dir = env::temp_dir();
    let icon_url = format!("{}/api/v1/games/{}/icon", config.remote, game.name);
    let res = CLIENT.get(icon_url).send().await.ok()?;
    let filename = res.headers().get(header::CONTENT_DISPOSITION)
        .and_then(|d| d.to_str().ok())
        .and_then(|d| Regex::new("filename=\"(?<name>.*)\"").unwrap().captures(d))
        .and_then(|c| c.name("name"))
        .map(|n| n.as_str().to_owned())?;
    let bytes = res.bytes().await.ok()?;
    let path = tmp_dir.join(filename);

    fs::write(&path, bytes).ok()?;

    path.to_str().map(str::to_owned)
}

pub async fn send_event<T: Serialize>(event: &str, obj: &T) -> Result<(), String> {
    let app_lock_opt = APP_HANDLE.lock().await;
    let Some(app_lock) = app_lock_opt.as_ref() else {
        return Err("app handle not set".to_string())
    };

    app_lock.emit(event, obj).map_err(|e| format!("could not emit event: {e}"))
}