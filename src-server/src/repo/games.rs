use std::{fs::File, io::Read, sync::LazyLock};

use common::game::Game;

use crate::config::GAMES_FILE;

static GAMES: LazyLock<Vec<Game>> = LazyLock::new(|| {
    let mut buf = String::new();
    
    File::open(GAMES_FILE.as_str())
        .expect("could not find games list")
        .read_to_string(&mut buf)
        .expect("could not read games list");

    serde_json::from_str::<Vec<Game>>(&buf)
        .expect("could not parse games list")
});

pub fn get_games() -> &'static Vec<Game> {
    &GAMES
}

pub fn get_game(name: &str) -> Option<&Game> {
    GAMES.iter().find(|g| g.name.as_str() == name)
}