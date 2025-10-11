use std::{collections::HashSet, sync::LazyLock};

use chrono::{DateTime, Local, TimeDelta};
use common::{response::{games::GamesResponse, now_playing::NowPlayingResponse}};
use log::{info, warn};
use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tauri::{Emitter};
use tokio::sync::Mutex;

use crate::{api::{get_games, put_now_playing}, app::APP_HANDLE};

#[derive(Serialize, Clone)]
struct Process {
    name: String,
    exe: Option<String>,
    cmd: Vec<String>,
    cwd: Option<String>
}

pub struct ProcessContext {
    pub last_response: Option<NowPlayingResponse>,
    pub last_put: DateTime<Local>
}

static GAMES: LazyLock<Mutex<Option<GamesResponse>>> = LazyLock::new(|| Mutex::new(None));

pub static CTX: LazyLock<Mutex<ProcessContext>> = LazyLock::new(|| Mutex::new(ProcessContext {
    last_put: Local::now() - TimeDelta::days(300),
    last_response: None
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
        .filter_map(|p| p.name().to_str().map(|name| Process {
                name: String::from(name),
                exe: p.exe()
                    .and_then(|e| e.to_str()).map(String::from),
                cmd: p.cmd().iter()
                    .filter_map(|s| s.to_str())
                    .map(String::from)
                    .collect(),
                cwd: p.cwd()
                    .and_then(|e| e.to_str()).map(String::from)
            })
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

pub async fn poll() {
    info!("poll: started");
    let processes = get_processes();

    let mut games_lock = GAMES.lock().await;
    let games = match games_lock.as_ref() {
        Some(g) => g,
        None => {
            match get_games().await {
                Ok(g) => {
                    *games_lock = Some(g);
                    
                    games_lock.as_ref().unwrap()
                },
                Err(e) => {
                    warn!("poll: could not get whitelist: {e}");
                    return;
                }
            }
        }
    };

    let open_games: HashSet<String> = processes.into_iter()
        .map(|p| p.name)
        .filter(|p| games.games.iter().any(|g| &g.name == p))
        .collect();

    if let Err(error) = send_event("now_playing", &open_games) {
        warn!("poll: Error occured while emitting event: {error}")
    } else {
        info!("poll: currently playing {} games", open_games.len())
    }

    let ctx_lock = CTX.lock().await;
    let last_put = ctx_lock.last_put;

    drop(ctx_lock);

    let res = put_now_playing(open_games, last_put).await;

    match res {
        Ok(Some(others_playing)) => {
            let mut ctx_lock = CTX.lock().await;

            ctx_lock.last_put = Local::now();
            ctx_lock.last_response = Some(others_playing);

            info!("poll: got new info");

            if let Err(e) = send_event("others_playing", &ctx_lock.last_response) {
                warn!("poll: Error while emitting event: {e}");
            }
        },
        Ok(None) => info!("poll: no new info received"),
        Err(err) => warn!("poll: Error while put_now_playing: {err}")
    }
}