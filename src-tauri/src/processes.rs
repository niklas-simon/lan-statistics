use std::{collections::HashSet, sync::LazyLock};

use chrono::{DateTime, Local, TimeDelta};
use common::response::now_playing::NowPlayingResponse;
use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tokio::sync::Mutex;

use crate::api::{get_games, put_now_playing, send_event};

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

pub async fn poll() -> Result<(), String> {
    let processes = get_processes();

    let games = get_games().await?;

    let open_games: HashSet<String> = processes.into_iter()
        .map(|p| p.name)
        .filter(|p| games.games.iter().any(|g| &g.name == p))
        .collect();
    let last_put = CTX.lock().await.last_put;
    let res = put_now_playing(open_games, last_put).await?;

    if let Some(others_playing) = res {
        let mut ctx_lock = CTX.lock().await;

        ctx_lock.last_put = Local::now();
        ctx_lock.last_response = Some(others_playing);

        send_event("others_playing", &ctx_lock.last_response).await?;
    }

    Ok(())
}