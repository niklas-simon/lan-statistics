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
    pub games: Vec<Game>
}

#[derive(Serialize, Deserialize)]
pub struct NowPlayingResponse {
    pub players: Vec<NowPlayingEntry>,
    pub party: Vec<Game>
}