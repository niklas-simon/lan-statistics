use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Eq, Hash)]
pub struct Game {
    pub name: String,
    pub label: String
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}