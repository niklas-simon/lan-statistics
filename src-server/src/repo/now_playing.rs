use std::{collections::HashMap, sync::{LazyLock, Mutex}};
use chrono::{DateTime, Local, TimeDelta};
use common::{game::Game, response::now_playing::NowPlayingResponse};

#[derive(Clone)]
struct NowPlayingInfo {
    timestamp: DateTime<Local>,
    games: Vec<Game>
}

static STORE: LazyLock<Mutex<HashMap<String, NowPlayingInfo>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static LAST_UPDATE: LazyLock<Mutex<DateTime<Local>>> = LazyLock::new(|| Mutex::new(Local::now()));

pub fn get_list() -> Result<NowPlayingResponse, String> {
    let mut store_lock = STORE.lock()
        .map_err(|e| String::from("now_playing: error getting store lock: ") + &e.to_string())?;
    let mut players_per_game: HashMap<String, u16> = HashMap::new();
    let mut all_games: Vec<Game> = vec![];
    let mut expired: Vec<String> = vec![];

    store_lock.keys().for_each(|player| {
        let info = store_lock.get(player).unwrap();

        if info.timestamp < Local::now() - TimeDelta::minutes(1) {
            expired.push(player.clone());
            return;
        }

        info.games.iter().for_each(|game| {
            all_games.push(game.clone());

            *players_per_game.entry(game.name.clone()).or_insert(0) += 1;
        });
    });

    expired.iter().for_each(|player| {
        store_lock.remove(player);
    });

    let mut party: Option<Game> = None;
    let mut most_players: u16 = 0;

    players_per_game.keys().for_each(|game| {
        let value = *players_per_game.get(game).unwrap();

        if value > most_players {
            party = all_games.iter().find(|g| g.name == *game).map(|g| g.clone());
            most_players = value;
        }
    });

    return Ok(NowPlayingResponse {
        players: store_lock.iter().map(|(player, info)| (player.clone(), info.games.clone())).collect(),
        party: party
    })
}

pub fn update(player: String, games: Vec<Game>) -> Result<(), String> {
    let mut store_lock = STORE.lock()
        .map_err(|e| String::from("now_playing: error getting store lock: ") + &e.to_string())?;

    let info = NowPlayingInfo {
        timestamp: Local::now(),
        games: games
    };

    if let Some(entry) = store_lock.get_mut(&player) {
        *entry = info;
    } else {
        store_lock.insert(player, info);
    }

    let mut update_lock = LAST_UPDATE.lock()
        .map_err(|e| String::from("now_playing: error getting update lock: ") + &e.to_string())?;

    *update_lock = Local::now();

    Ok(())
}