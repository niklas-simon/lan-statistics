use std::collections::HashSet;

use chrono::{DateTime, Local};
use common::{game::Game, response::now_playing::NowPlayingResponse};

use crate::config::get_or_create_config;

pub async fn put_now_playing(games: &HashSet<&Game>) -> Result<(), String> {
    let config = get_or_create_config(false)?;
    let client = reqwest::Client::new();

    let res = client.put(config.remote + "/api/v1/now-playing/" + &config.id)
        .json(games)
        .send()
        .await.map_err(|e| e.to_string())?;

    match res.status() {
        reqwest::StatusCode::NO_CONTENT => Ok(()),
        code => Err(code.to_string() + &String::from(": ") + &res.text().await.map_err(|e| e.to_string())?)
    }
}

pub async fn get_now_playing(last_update: DateTime<Local>) -> Result<Option<NowPlayingResponse>, String> {
    let config = get_or_create_config(false)?;
    let client = reqwest::Client::new();

    let res = client.get(config.remote + "/api/v1/now-playing")
        .query(&("last_update", last_update.format("%Y-%m-%dT%H:%M:%S").to_string()))
        .send()
        .await.map_err(|e| e.to_string())?;

    match res.status() {
        reqwest::StatusCode::OK => res.json::<NowPlayingResponse>().await
            .map(|r| Some(r))
            .map_err(|e| e.to_string()),
        reqwest::StatusCode::NOT_MODIFIED => Ok(None),
        code => Err(code.to_string() + &String::from(": ") + &res.text().await.map_err(|e| e.to_string())?)
    }
}