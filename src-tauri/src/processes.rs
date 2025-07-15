use std::{collections::HashSet, fs::File, io::BufReader, sync::{LazyLock, Mutex}};

use chrono::{DateTime, Local, TimeDelta};
use common::game::Game;
use log::{info, warn};
use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tauri::{Emitter};

use crate::{api::{get_now_playing, put_now_playing}, app::APP_HANDLE};

#[derive(Serialize, Clone)]
struct Process {
    name: String,
    exe: Option<String>,
    cmd: Vec<String>,
    cwd: Option<String>
}

pub struct ProcessContext {
    pub last_games: HashSet<&'static Game>,
    pub last_put: DateTime<Local>,
    pub last_get: DateTime<Local>
}

static GAMES: LazyLock<Result<Vec<Game>, String>> = LazyLock::new(|| File::open("games.json")
    .map_err(|e| e.to_string())
    .and_then(|f| serde_json::from_reader(BufReader::new(f))
        .map_err(|e| e.to_string())
    )
);

static CTX: LazyLock<Mutex<ProcessContext>> = LazyLock::new(|| Mutex::new(ProcessContext {
    last_put: Local::now() - TimeDelta::days(300),
    last_get: Local::now() - TimeDelta::days(300),
    last_games: HashSet::new()
}));

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

fn send_event<T: Serialize>(event: &str, obj: &T) -> Result<(), String> {
    let Ok(guard) = APP_HANDLE.lock() else {
        return Err(String::from("send_event: Could not get lock on AppHandle"));
    };

    let Some(ref app_handle) = *guard else {
        return Err(String::from("send_event: Could not get AppHandle"));
    };

    app_handle.emit(event, obj).map_err(|e| e.to_string())
}

fn is_update(open_games: &HashSet<&Game>) -> bool {
    let Ok(ctx_lock) = CTX.lock() else {
        warn!("poll: could not get lock for CTX");
        return true;
    };

    if ctx_lock.last_games.len() == open_games.len() 
        && open_games.iter().all(|g| ctx_lock.last_games.contains(g))
        && ctx_lock.last_put + TimeDelta::seconds(30) > Local::now() {
        return false;
    }

    true
}

fn update_ctx(open_games: HashSet<&'static Game>) {
    let Ok(mut ctx_lock) = CTX.lock() else {
        warn!("poll: could not get lock for CTX");
        return;
    };

    ctx_lock.last_put = Local::now();
    ctx_lock.last_games = open_games;

}

pub async fn poll() {
    info!("poll: started");
    let processes = get_processes();

    if let Err(error) = send_event("processes", &processes) {
        warn!("poll: Error occured while emitting event: {}", error.to_string())
    } else {
        info!("poll: found and sent info about {} processes", processes.len())
    }

    let Ok(ref whitelist) = *GAMES else {
        warn!("poll: could not get whitelist: {}", (*GAMES).as_ref().err().unwrap_or(&String::from("unknown error")));
        return;
    };

    let open_games: HashSet<&Game> = processes.iter()
        .filter_map(|p| whitelist.iter().find(|g| g.name == p.name))
        .collect();

    if !is_update(&open_games) {
        return;
    }

    if let Err(error) = send_event("now_playing", &open_games) {
        warn!("poll: Error occured while emitting event: {}", error.to_string())
    } else {
        info!("poll: currently playing {} games", open_games.len())
    }

    match put_now_playing(&open_games).await {
        Ok(()) => (),
        Err(err) => warn!("poll: Error while put_now_playing: {}", err)
    }

    update_ctx(open_games);
}

pub async fn update_others() {
    let Ok(ctx) = CTX.lock() else {
        warn!("poll: could not get lock for CTX");
        return;
    };

    get_now_playing(ctx.last_get).await;
}