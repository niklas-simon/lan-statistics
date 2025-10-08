use std::collections::HashSet;

use reqwest::{StatusCode, Client};
use chrono::{DateTime, Local};
use common::{game::Game, response::{games::GamesResponse, now_playing::{NowPlayingEntry, NowPlayingResponse, Player}}};

use crate::config::get_or_create_config;

pub async fn put_now_playing(games: HashSet<Game>, last_update: DateTime<Local>) -> Result<Option<NowPlayingResponse>, String> {
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
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("failed to send request: {e}"))?;

    match res.status() {
        StatusCode::OK => res.json::<NowPlayingResponse>().await
            .map(Some)
            .map_err(|e| format!("failed to send request: {e}")),
        StatusCode::NOT_MODIFIED => Ok(None),
        code => {
            let body = res.text().await
                .map_err(|e| format!("unexpected error code: {code}: {e}"))?;

            Err(format!("{code}: {body}"))
        }
    }
}

pub async fn get_games() -> Result<GamesResponse, String> {
    let config = get_or_create_config(false)?;
    let client = Client::default();

    client.get(config.remote + "/api/v1/games")
        .send().await
        .map_err(|e| format!("failed to send request: {e}"))?
        .json::<GamesResponse>().await
        .map_err(|e| format!("failed to parse response: {e}"))
}