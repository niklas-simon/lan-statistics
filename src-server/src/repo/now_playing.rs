use std::{collections::HashMap, sync::LazyLock};
use chrono::{DateTime, Local, TimeDelta};
use common::{response::now_playing::{NowPlayingEntry, NowPlayingResponse, PartyPlayingEntry}};
use tokio::sync::Mutex;

use crate::{metrics, repo::games::get_game};

struct NowPlayingInfo {
    timestamp: DateTime<Local>,
    entry: NowPlayingEntry
}

static STORE: LazyLock<Mutex<HashMap<String, NowPlayingInfo>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static LAST_UPDATE: LazyLock<Mutex<DateTime<Local>>> = LazyLock::new(|| Mutex::new(Local::now()));

pub async fn get_list() -> NowPlayingResponse {
    let mut all_games: HashMap<String, PartyPlayingEntry> = HashMap::new();
    let store_lock = STORE.lock().await;

    for info in store_lock.values() {
        info.entry.games.iter().for_each(|name| {
            let Some(game) = get_game(name) else {
                return;
            };

            let entry = all_games.entry(name.clone()).or_insert(PartyPlayingEntry {
                game: game.clone(),
                players: vec![]
            });

            entry.players.push(info.entry.player.clone());
        });
    }

    NowPlayingResponse {
        active: all_games.into_values().collect(),
        online: store_lock.len()
    }
}

pub async fn update(mut entry: NowPlayingEntry) {
    let mut store_lock = STORE.lock().await;

    entry.games.sort();

    let info = NowPlayingInfo {
        timestamp: Local::now(),
        entry: entry.clone()
    };

    let mut is_update = false;

    if let Some(item) = store_lock.get_mut(&info.entry.player.id) {
        if item.entry.player.id != info.entry.player.id || !item.entry.games.iter().eq(info.entry.games.iter()) {
            is_update = true;
        }
        
        *item = info;
    } else {
        store_lock.insert(info.entry.player.id.clone(), info);

        is_update = true;
    }

    drop(store_lock);

    if is_update {
        *LAST_UPDATE.lock().await = Local::now();
    }

    metrics::record_played_games(entry.player, entry.games.iter()
        .filter_map(|g| get_game(g).cloned())
        .collect()).await;
}

pub async fn clean() {
    let mut store_lock = STORE.lock().await;
    let mut is_update = false;
    let mut expired: Vec<String> = vec![];

    for (player, info) in store_lock.iter() {
        if info.timestamp < Local::now() - TimeDelta::seconds(30) {
            expired.push(player.clone());
            is_update = true;
            continue;
        }
    }

    for player in expired {
        store_lock.remove(&player);
        metrics::record_expired_player(&player).await;
    };

    drop(store_lock);

    if is_update {
        *LAST_UPDATE.lock().await = Local::now();
    }
}