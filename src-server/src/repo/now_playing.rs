use std::{collections::HashMap, sync::LazyLock};
use chrono::{DateTime, Local, TimeDelta};
use common::{response::now_playing::{NowPlayingEntry, NowPlayingResponse, PartyPlayingEntry}};
use tokio::sync::Mutex;

struct NowPlayingInfo {
    timestamp: DateTime<Local>,
    entry: NowPlayingEntry
}

static STORE: LazyLock<Mutex<HashMap<String, NowPlayingInfo>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static LAST_UPDATE: LazyLock<Mutex<DateTime<Local>>> = LazyLock::new(|| Mutex::new(Local::now()));

pub async fn get_list() -> NowPlayingResponse {
    let mut all_games: HashMap<String, PartyPlayingEntry> = HashMap::new();
    let mut expired: Vec<String> = vec![];
    let mut store_lock = STORE.lock().await;

    for (player, info) in store_lock.iter() {
        if info.timestamp < Local::now() - TimeDelta::minutes(1) {
            expired.push(player.clone());
            continue;
        }

        info.entry.games.iter().for_each(|game| {
            let entry = all_games.entry(game.name.clone()).or_insert(PartyPlayingEntry {
                game: game.clone(),
                players: vec![]
            });

            entry.players.push(info.entry.player.clone());
        });
    }

    for player in expired {
        store_lock.remove(&player);
    };

    all_games.into_values().collect()
}

pub async fn update(mut entry: NowPlayingEntry) {
    let mut store_lock = STORE.lock().await;

    entry.games.sort_by_key(|g| g.name.clone());

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

        is_update = true;
    }

    if is_update {
        *LAST_UPDATE.lock().await = Local::now();
    }
}