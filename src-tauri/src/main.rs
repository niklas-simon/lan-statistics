// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env::{args, current_dir, current_exe}, path::Path, process::id
};

use config::set_config_path;
use metrics::metrics_loop;
use sysinfo::{ProcessesToUpdate, System};
mod app;
mod config;
mod metrics;

const USAGE: &'static str = "\
usage: lan-tracker.exe [-s|--silent] [-c|--config path] [-h|--help]\n\
    -s|--silent: only start background task\n\
    -c|--config: specify which config file to use\n\
    -h|--help:   print this usage information";

#[tokio::main]
async fn main() {
    let mut other_instance = false;

    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    if sys.processes().iter().any(|(pid, info)|
        pid.as_u32() != id() &&
        info.exe().and_then(|proc|
            Some(current_exe().ok()
                .and_then(|curr| Some(curr == proc))
                .unwrap_or(false)
            )
        ).unwrap_or(false)
    ) {
        other_instance = true;
        println!("another instance is already running");
    }

    let mut is_silent = false;
    let mut config_path_buf = current_dir()
        .and_then(|dir| Ok(dir.join(Path::new("config.toml"))))
        .expect("default config dir could not be constructed");

    let mut arguments = args();
    while let Some(arg) = arguments.next() {
        if arg == "-s" || arg == "--silent" {
            is_silent = true;
        }
        if arg == "-c" || arg == "--config" {
            config_path_buf = current_dir()
                .and_then(|dir| 
                    Ok(dir.join(Path::new(
                        &arguments.next()
                        .expect(USAGE)
                    ).to_path_buf()))
                ).expect("could not construct path from the given argument");
        }
        if arg == "-h" || arg == "--help" {
            println!("{}", USAGE);
        }
    }

    set_config_path(config_path_buf).expect("error setting config path");
    let mut metrics_task = None;

    if !other_instance {
        metrics_task = Some(tokio::task::spawn_blocking(move || {
            match metrics_loop() {
                Ok(()) => eprintln!("error: metrics loop stopped unexpectedly"),
                Err(s) => eprintln!("error in metrics loop: {s}")
            }
        }));
    }

    if !is_silent {
        app::run();
    }

    match metrics_task {
        Some(t) => t.await.unwrap_or_else(|e|
            eprintln!("error with metrics task: {}", e.to_string())
        ),
        None => ()
    }
}
