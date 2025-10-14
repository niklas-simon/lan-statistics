use std::time::Duration;

use actix_web::{App, HttpServer};
use clokwerk::{AsyncScheduler, TimeUnits};

use crate::repo::now_playing;

mod api;
mod repo;
mod config;
mod metrics;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut scheduler = AsyncScheduler::new();

    scheduler.every(5.seconds()).run(async || {
        now_playing::clean().await;
    });

    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    HttpServer::new(|| {
        App::new()
            .service(api::get_scope())
            .service(metrics::get_scope())
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}