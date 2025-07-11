use std::{collections::HashSet, fs::File, io::BufReader, sync::LazyLock};

use log::{info, warn};
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tauri::{AppHandle, Emitter};

use crate::app::APP_HANDLE;

#[derive(Serialize, Clone)]
struct Process {
    name: String,
    exe: Option<String>,
    cmd: Vec<String>,
    cwd: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Eq, Hash)]
struct Game {
    name: String,
    label: String
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
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

fn update_now_playing(processes: &Vec<Process>, app_handle: &AppHandle) {
    let Ok(ref whitelist) = *GAMES else {
        warn!("update_now_playing: could not get whitelist: {}", (*GAMES).as_ref().err().unwrap_or(&String::from("unknown error")));
        return;
    };

    let open_games: HashSet<&Game> = processes.iter()
        .filter_map(|p| whitelist.iter().find(|g| g.name == p.name))
        .collect();

    if let Err(error) = app_handle.emit("now_playing", &open_games) {
        warn!("update_now_playing: Error occured while emitting event: {}", error.to_string())
    } else {
        info!("update_now_playing: currently playing {} games", open_games.len())
    }
}

pub fn poll() {
    let Ok(guard) = APP_HANDLE.lock() else {
        warn!("processes: Could not get lock on AppHandle");
        return;
    };

    let Some(ref app_handle) = *guard else {
        warn!("processes: Could not get AppHandle");
        return;
    };

    let processes = get_processes();

    if let Err(error) = app_handle.emit("processes", &processes) {
        warn!("processes: Error occured while emitting event: {}", error.to_string())
    } else {
        info!("processes: found and sent info about {} processes", processes.len())
    }

    update_now_playing(&processes, app_handle);
}