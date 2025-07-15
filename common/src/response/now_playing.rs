use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::game::Game;

#[derive(Serialize, Deserialize)]
pub struct NowPlayingResponse {
    pub players: HashMap<String, Vec<Game>>,
    pub party: Option<Game>
}