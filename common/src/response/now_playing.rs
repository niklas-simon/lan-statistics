use serde::{Deserialize, Serialize};

use crate::game::Game;

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: String,
    pub name: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NowPlayingEntry {
    pub player: Player,
    pub games: Vec<String>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PartyPlayingEntry {
    pub game: Game,
    pub players: Vec<Player>
}

pub type NowPlayingResponse = Vec<PartyPlayingEntry>;