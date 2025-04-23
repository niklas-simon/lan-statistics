// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env::{args, current_exe, set_current_dir}, process::{exit, Command}
};

use log::{error, info, warn};
use metrics::metrics_loop;
use named_lock::NamedLock;
mod app;
mod config;
mod metrics;

const USAGE: &str = "\
usage: lan-tracker.exe [-s|--service] [-h|--help]\n\
    -s|--service: start background task\n\
    -h|--help:   print this usage information";

fn main() {
    current_exe().map_err(|e| e.to_string())
        .and_then(|p| match p.parent() {
            Some(p) => Ok(p.to_path_buf()),
            None => Err(String::from("could not get program folder"))
        })
        .and_then(|p| set_current_dir(p).map_err(|e| e.to_string()))
        .unwrap_or_else(|e| {
            error!("error setting cwd: {e}");
            exit(1);
        });

    log4rs::init_file("log4rs.yml", Default::default())
        .unwrap_or_else(|e| {
            eprintln!("error configuring logger: {}", e);
            exit(1);
        });

    let mut is_service = false;

    let arguments = args();
    for arg in arguments {
        if arg == "-s" || arg == "--service" {
            is_service = true;
        }
        if arg == "-h" || arg == "--help" {
            info!("{}", USAGE);
            exit(0);
        }
    }

    if is_service {
        let guard = NamedLock::create("lan-tracker")
            .and_then(|l| l.try_lock());

        match guard {
            Ok(_) => (),
            Err(named_lock::Error::WouldBlock) => {
                warn!("could not get lock. Another instance is already running.");
                exit(0);
            },
            Err(e) => {
                error!("{}", e.to_string());
                exit(1);
            }
        };

        info!("starting service");

        match metrics_loop() {
            Ok(()) => error!("error: metrics loop stopped unexpectedly"),
            Err(s) => error!("error in metrics loop: {s}")
        }
    } else {
        match current_exe()
            .and_then(|exe| 
                Command::new(exe)
                    .args(["-s"])
                    .spawn()
            ) {
                Ok(_) => (),
                Err(e) => error!("error spawning service: {e}")
            }

        app::run();
    }
}
