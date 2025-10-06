use std::{collections::HashSet, fs::File, io::BufReader, sync::{LazyLock}};

use common::game::Game;
use log::{info, warn};
use serde::Serialize;
use spacetimedb_sdk::Table;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::{api::CTX, app, module_bindings::{set_games, GameTableAccess}};

#[derive(Serialize, Clone)]
struct Process {
    name: String,
    exe: Option<String>,
    cmd: Vec<String>,
    cwd: Option<String>
}

static GAMES: LazyLock<Result<Vec<Game>, String>> = LazyLock::new(|| File::open("games.json")
    .map_err(|e| e.to_string())
    .and_then(|f| serde_json::from_reader(BufReader::new(f))
        .map_err(|e| e.to_string())
    )
);

fn get_processes() -> Vec<Process> {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All, 
        true,
        ProcessRefreshKind::nothing()
            .with_exe(UpdateKind::Always)
            .with_cwd(UpdateKind::Always)
            .with_cmd(UpdateKind::Always)
    );

    system.processes()
        .values()
        .filter_map(|p| p.name().to_str()
            .and_then(|name| Some(Process {
                name: String::from(name),
                exe: p.exe()
                    .and_then(|e| e.to_str())
                    .and_then(|e| Some(String::from(e))),
                cmd: p.cmd().iter()
                    .filter_map(|s| s.to_str())
                    .map(|s| String::from(s))
                    .collect(),
                cwd: p.cwd()
                    .and_then(|e| e.to_str())
                    .and_then(|e| Some(String::from(e)))
            }))
        )
        .collect()
}

pub async fn poll() {
    info!("poll: started");
    let processes = get_processes();

    if let Err(error) = app::send_event("processes", &processes) {
        warn!("poll: Error occured while emitting event: {}", error.to_string())
    } else {
        info!("poll: found and sent info about {} processes", processes.len())
    }

    let Ok(ref whitelist) = *GAMES else {
        warn!("poll: could not get whitelist: {}", (*GAMES).as_ref().err().unwrap_or(&String::from("unknown error")));
        return;
    };

    let open_games: HashSet<String> = processes.iter()
        .filter_map(|p| whitelist.iter()
            .find(|g| g.name == p.name)
            .map(|g| g.label.clone()))
        .collect();

    let Some(ref ctx_lock) = *CTX.lock().await else {
        warn!("processes: no connection to spacetime");
        return;
    };

    if ctx_lock.db.game().count() == open_games.len() as u64 && !ctx_lock.db.game().iter().any(|g| !open_games.contains(&g.name)) {
        info!("games already up to date");
        return;
    }

    if let Err(err) = ctx_lock.reducers.set_games(open_games.iter().map(|g| g.clone()).collect()) {
        warn!("could not set games: {}", err.to_string());
        return;
    } else {
        info!("updated games to {}", open_games.into_iter().collect::<Vec<String>>().join(", "))
    }
}