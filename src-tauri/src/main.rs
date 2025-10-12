// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{process::exit, time::Duration};

use clokwerk::{AsyncScheduler, TimeUnits};
use log::error;

use crate::api::send_event;

mod app;
mod config;
mod processes;
mod api;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yml", Default::default())
        .unwrap_or_else(|e| {
            eprintln!("error configuring logger: {e}");
            exit(1);
        });

    let mut scheduler = AsyncScheduler::new();

    scheduler.every(5.seconds()).run(async || {
        let res = processes::poll().await.err();

        if let Err(e) = send_event("poll_error", &res).await {
            error!("failed to send event: {e}");
        }
    });

    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    app::run();
}
