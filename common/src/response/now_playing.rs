use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::game::Game;

#[derive(Serialize, Deserialize, Clone, Eq)]
pub struct Player {
    pub id: String,
    pub name: String
}

impl Hash for Player {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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