// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{process::exit, time::Duration};
use clokwerk::{AsyncScheduler, TimeUnits};
use log::error;

mod app;
mod config;
mod processes;
mod module_bindings;
mod api;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yml", Default::default())
        .unwrap_or_else(|e| {
            eprintln!("error configuring logger: {}", e);
            exit(1);
        });

    if let Err(err) = api::connect().await {
        error!("error: could not connect to SpacetimeDB: {}", err);
    }

    let mut scheduler = AsyncScheduler::new();

    scheduler.every(5.seconds()).run(async || {
        processes::poll().await;
    });

    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    app::run();
}