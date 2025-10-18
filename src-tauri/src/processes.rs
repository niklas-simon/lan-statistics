use std::{collections::HashSet, env, fs, sync::LazyLock};

use chrono::{DateTime, Local, TimeDelta};
use common::{game::Game, response::now_playing::NowPlayingResponse};
use log::{info, warn};
use notify_rust::{Notification, Timeout};
use regex::Regex;
use reqwest::{header, Client};
use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tokio::sync::Mutex;

use crate::api::{get_games, put_now_playing, send_event};
const GRACE_PERIOD: u16 = 4;

#[derive(Serialize, Clone)]
struct Process {
    name: String,
    exe: Option<String>,
    cmd: Vec<String>,
    cwd: Option<String>
}

pub struct ProcessContext {
    pub last_response: Option<NowPlayingResponse>,
    pub last_put: DateTime<Local>,
    pub notification_ctx: Option<NotificationContext>
}

pub struct NotificationContext {
    active_game: Game,
    missed: u16,
    notified: bool
}

impl NotificationContext {
    pub fn new(game: &Game) -> NotificationContext {
        NotificationContext {
            active_game: game.clone(),
            missed: 1,
            notified: false
        }
    }

    pub fn switch_game(&mut self, game: &Game) {
        self.active_game = game.clone();
        self.missed = 1;
        self.notified = false;
    }

    pub fn inc(&mut self) {
        self.missed += 1;
    }

    pub fn notify(&mut self) {
        self.notified = true;
    }
}

pub static CTX: LazyLock<Mutex<ProcessContext>> = LazyLock::new(|| Mutex::new(ProcessContext {
    last_put: Local::now() - TimeDelta::days(300),
    last_response: None,
    notification_ctx: None
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

async fn get_icon(game: &Game) -> Option<String> {
    let tmp_dir = env::temp_dir();
    let icon_url = format!("http://localhost/api/v1/games/{}/icon", game.name);
    let res = Client::default().get(icon_url).send().await.ok()?;
    let filename = res.headers().get(header::CONTENT_DISPOSITION)
        .and_then(|d| d.to_str().ok())
        .and_then(|d| Regex::new("filename=\"(?<name>.*)\"").unwrap().captures(d))
        .and_then(|c| c.name("name"))
        .map(|n| n.as_str().to_owned())?;
    let bytes = res.bytes().await.ok()?;
    let path = tmp_dir.join(filename);

    fs::write(&path, bytes).ok()?;

    path.to_str().map(str::to_owned)
}

pub async fn poll() -> Result<(), String> {
    let processes = get_processes();

    let games = get_games().await?;

    let open_games: HashSet<String> = processes.into_iter()
        .map(|p| p.name)
        .filter(|p| games.games.iter().any(|g| &g.name == p))
        .collect();
    let last_put = CTX.lock().await.last_put;
    let res = put_now_playing(open_games.clone(), last_put).await?;

    let mut ctx_lock = CTX.lock().await;
    let mut others_playing = match (res, &ctx_lock.last_response) {
        (Some(others_playing), _) => {
            ctx_lock.last_put = Local::now();
            ctx_lock.last_response = Some(others_playing.clone());

            others_playing
        },
        (None, Some(last)) => last.clone(),
        (None, None) => return Ok(())
    };

    others_playing.active.sort_by(|a, b| {
        if a.players.len() != b.players.len() {
            return b.players.len().cmp(&a.players.len());
        }

        a.game.label.cmp(&b.game.label)
    });

    send_event("others_playing", &others_playing).await?;

    let Some(most_played) = others_playing.active.first() else {
        return Ok(());
    };

    info!("most_played: {}", most_played.game.label);

    // is majority playing game
    if most_played.players.len() < others_playing.online / 2 {
        info!("there is no majority");
        return Ok(());
    }

    // are you playing game
    if open_games.contains(&most_played.game.name) {
        info!("you are playing the most played game");
        return Ok(());
    }

    let Some(notification_ctx) = &mut ctx_lock.notification_ctx else {
        info!("created notification context");
        ctx_lock.notification_ctx = Some(NotificationContext::new(&most_played.game));

        return Ok(());
    };

    // is it still same game
    if notification_ctx.active_game != most_played.game {
        info!("the game has changed");
        notification_ctx.switch_game(&most_played.game);

        return Ok(());
    }

    // have you been notified
    if notification_ctx.notified {
        info!("already notified");
        return Ok(());
    }

    // are you over the grace period
    if notification_ctx.missed < GRACE_PERIOD {
        info!("grace period not reached");
        notification_ctx.inc();

        return Ok(());
    }

    info!("notifying");
    notification_ctx.notify();

    drop(ctx_lock);

    let icon = get_icon(&most_played.game).await;

    info!("icon at: {icon:?}");

    if let Err(e) = Notification::new()
        .summary("LAN Manager")
        .body(format!("Die Mehrheit spielt {}. Du nicht. Schande!", most_played.game.label).as_str())
        .timeout(Timeout::Milliseconds(30_000))
        .image_path(icon.unwrap_or(String::new()).as_str())
        .show() {
        warn!("failed to show notification: {e}");
    }

    Ok(())
}