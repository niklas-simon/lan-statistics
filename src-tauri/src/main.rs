// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{process::exit, time::Duration};

use clokwerk::{Scheduler, TimeUnits};

mod app;
mod config;
mod processes;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yml", Default::default())
        .unwrap_or_else(|e| {
            eprintln!("error configuring logger: {}", e);
            exit(1);
        });

    let mut scheduler = Scheduler::new();

    scheduler.every(5.seconds()).run(|| {
        processes::poll();
    });

    tokio::spawn(async move {
        loop {
            scheduler.run_pending();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    app::run();
}
