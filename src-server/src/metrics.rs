use std::{collections::{HashMap, HashSet}, sync::LazyLock};

use actix_web::{error::ErrorInternalServerError, web, HttpResponse, Responder, Result, Scope};
use chrono::{DateTime, Duration, Local};
use common::{game::Game, response::now_playing::Player};
use prometheus::{Counter, CounterVec, Gauge, GaugeVec, Opts, Registry, TextEncoder};
use tokio::sync::Mutex;

struct MetricsFamily {
    lan_game_seconds_total: Counter,
    lan_game_active: Gauge
}

impl MetricsFamily {
    fn with(player: &Player, game: &Game) -> MetricsFamily {
        MetricsFamily {
            lan_game_seconds_total: CTX.lan_game_seconds_total_vec.with_label_values(&[&player.id, &game.name, &player.name, &game.label, &game.icon]),
            lan_game_active: CTX.lan_game_active_vec.with_label_values(&[&player.id, &game.name, &player.name, &game.label, &game.icon])
        }
    }
}

struct PlayerMetricsContext {
    last_seen: DateTime<Local>,
    map: HashMap<Game, MetricsFamily>
}

struct MetricsContext {
    registry: Registry,
    lan_game_seconds_total_vec: CounterVec,
    lan_game_active_vec: GaugeVec,
    map: Mutex<HashMap<Player, PlayerMetricsContext>>
}

static CTX: LazyLock<MetricsContext> = LazyLock::new(|| {
    let lan_game_seconds_total_opts = Opts::new("lan_game_seconds_total", "counts the times a game has been recorded for target player with an approximate resolution of 5s");
    let lan_game_active_opts = Opts::new("lan_game_active", "gauge displaying current active game per player");
    let lan_game_seconds_total_vec = CounterVec::new(lan_game_seconds_total_opts, &["player", "game", "player_name", "game_label", "game_icon"]).expect("failed to create CounterVec lan_game_seconds_total");
    let lan_game_active_vec = GaugeVec::new(lan_game_active_opts, &["player", "game", "player_name", "game_label", "game_icon"]).expect("failed to create CounterVec lan_game_played_seconds");
    let registry = Registry::new();

    registry.register(Box::new(lan_game_seconds_total_vec.clone()))
        .expect("failed to register lan_game_seconds_total");
    registry.register(Box::new(lan_game_active_vec.clone()))
        .expect("failed to register lan_game_active");

    MetricsContext { 
        registry, 
        lan_game_seconds_total_vec, 
        lan_game_active_vec, 
        map: HashMap::new().into()
    }
});

pub async fn scrape() -> Result<impl Responder> {
    let mut buffer = String::new();
    let encoder = TextEncoder::new();
    let metric_families = CTX.registry.gather();

    encoder.encode_utf8(&metric_families, &mut buffer)
        .map_err(|e| ErrorInternalServerError(format!("failed to encode metrics: {e}")))?;

    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(buffer))
}

pub async fn record_played_games(player: Player, games: Vec<Game>) {
    let mut counter_lock = CTX.map.lock().await;
    let by_player = counter_lock
        .entry(player.clone())
        .or_insert(PlayerMetricsContext {
            last_seen: Local::now() - Duration::seconds(5),
            map: HashMap::new()
        });

    let mut expired: HashSet<Game> = by_player.map.keys().cloned().collect();
    
    for game in games {
        let by_game = by_player.map
            .entry(game.clone())
            .or_insert(MetricsFamily::with(&player, &game));
        let now = Local::now();
        let duration = now - by_player.last_seen;

        by_game.lan_game_seconds_total.inc_by(duration.as_seconds_f64());
        by_game.lan_game_active.set(1.0);
        by_player.last_seen = now;

        expired.remove(&game);
    }

    for game in expired {
        let Some(by_game) = by_player.map.get(&game) else {
            continue;
        };

        by_game.lan_game_active.set(0.0);
    }
}

pub async fn record_expired_player(player: &String) {
    let counter_lock = CTX.map.lock().await;
    let Some((_, by_player)) = counter_lock.iter().find(|(p, _)| &p.id == player) else {
        return;
    };

    for by_game in by_player.map.values() {
        by_game.lan_game_active.set(0.0);
    }
}

pub fn get_scope() -> Scope {
    web::scope("/metrics")
        .route("", web::get().to(scrape))
}