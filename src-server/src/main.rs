use std::time::Duration;

use actix_web::{App, HttpServer, web::Data};
use clokwerk::{AsyncScheduler, TimeUnits};

use crate::{api::SharedData, repo::now_playing};

mod api;
mod repo;
mod config;
mod metrics;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut scheduler = AsyncScheduler::new();
    let shared = SharedData::new();
    let shared_clone = shared.clone();

    scheduler.every(5.seconds()).run(move || {
        let shared_clone_inner = shared_clone.clone();

        async move {
            now_playing::clean(shared_clone_inner.clone()).await;
        }
    });

    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    HttpServer::new(move || {
        let shared = shared.clone();

        App::new()
            .app_data(Data::new(shared))
            .service(api::get_scope())
            .service(metrics::get_scope())
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}