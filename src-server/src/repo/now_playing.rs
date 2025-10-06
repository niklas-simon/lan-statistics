use std::{collections::HashMap, sync::LazyLock};
use chrono::{DateTime, Local, TimeDelta};
use common::{game::Game, response::now_playing::{NowPlayingEntry, NowPlayingResponse}};
use tokio::sync::Mutex;

struct NowPlayingInfo {
    timestamp: DateTime<Local>,
    entry: NowPlayingEntry
}

static STORE: LazyLock<Mutex<HashMap<String, NowPlayingInfo>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static LAST_UPDATE: LazyLock<Mutex<DateTime<Local>>> = LazyLock::new(|| Mutex::new(Local::now()));

pub async fn get_list() -> NowPlayingResponse {
    let mut players_per_game: HashMap<String, u16> = HashMap::new();
    let mut all_games: Vec<Game> = vec![];
    let mut expired: Vec<String> = vec![];
    let mut store_lock = STORE.lock().await;

    for (player, info) in store_lock.iter() {
        if info.timestamp < Local::now() - TimeDelta::minutes(1) {
            expired.push(player.clone());
            continue;
        }

        info.entry.games.iter().for_each(|game| {
            all_games.push(game.clone());

            *players_per_game.entry(game.name.clone()).or_insert(0) += 1;
        });
    }

    for player in expired {
        store_lock.remove(&player);
    };

    let mut party = vec![];
    let mut most_players: u16 = 0;

    for (game_id, value) in players_per_game.into_iter() {
        let Some(game) = all_games.iter().find(|g| g.name == *game_id).cloned() else {
            continue;
        };

        if value > most_players {
            party = vec![game];
            most_players = value;
        } else if value == most_players {
            party.push(game);
        }
    };

    NowPlayingResponse {
        players: store_lock.values().map(|v| v.entry.clone()).collect(),
        party
    }
}

pub async fn update(entry: NowPlayingEntry) {
    let mut store_lock = STORE.lock().await;

    let info = NowPlayingInfo {
        timestamp: Local::now(),
        entry
    };

    let mut is_update = false;

    if let Some(item) = store_lock.get_mut(&info.entry.player.id) {
        if item.entry.player.id != info.entry.player.id || !item.entry.games.iter().eq(info.entry.games.iter()) {
            is_update = true;
        }
        
        *item = info;
    } else {
        store_lock.insert(info.entry.player.id.clone(), info);
    }

    if is_update {
        *LAST_UPDATE.lock().await = Local::now();
    }
}