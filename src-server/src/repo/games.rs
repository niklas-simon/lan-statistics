use std::{fs::File, io::Read, sync::LazyLock};

use common::game::Game;

static GAMES: LazyLock<Vec<Game>> = LazyLock::new(|| {
    let mut buf = String::new();
    
    File::open("games.json")
        .expect("could not find games.json")
        .read_to_string(&mut buf)
        .expect("could not read games.json");

    serde_json::from_str::<Vec<Game>>(&buf)
        .expect("could not parse games.json")
});

pub fn get_games() -> &'static Vec<Game> {
    &GAMES
}

pub fn get_game(name: &str) -> Option<&Game> {
    GAMES.iter().find(|g| g.name.as_str() == name)
}