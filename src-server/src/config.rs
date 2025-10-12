use std::{env, sync::LazyLock, fmt::Write as _};

use sha2::{Sha256, Digest};

pub static PASSWORD: LazyLock<String> = LazyLock::new(|| {
    let password = env::var("PASSWORD")
        .expect("environment variable PASSWORD not set");
    let mut hasher = Sha256::new();
    hasher.update(password);
    let mut out = String::new();
    write!(out, "{:x}", hasher.finalize()).map_err(|e| e.to_string())
        .expect("could not hash PASSWORD");

    out
});

pub static GAMES_FILE: LazyLock<String> = LazyLock::new(|| {    
    env::var("GAMES_FILE").unwrap_or("games.json".to_string())
});

pub static ICONS_DIR: LazyLock<String> = LazyLock::new(|| {    
    env::var("ICONS_DIR").unwrap_or("icons".to_string())
});