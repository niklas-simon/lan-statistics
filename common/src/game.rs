use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Serialize, Deserialize, Clone, Eq)]
pub struct Game {
    pub name: String,
    pub label: String,
    pub icon: String
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for Game {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}