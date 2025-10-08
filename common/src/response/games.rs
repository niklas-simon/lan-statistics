use serde::{Deserialize, Serialize};

use crate::game::Game;

#[derive(Serialize, Deserialize, Clone)]
pub struct GamesResponse {
    pub games: Vec<Game>,
    pub hash: u64
}