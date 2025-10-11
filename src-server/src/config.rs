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