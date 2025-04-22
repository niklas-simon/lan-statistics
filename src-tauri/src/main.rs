// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env::{args, current_dir, current_exe}, fs::File, io::{Read, Write}, path::{Path, PathBuf}, process::{exit, Command}
};

use config::set_config_path;
use log::{LevelFilter, info, warn, error};
use metrics::metrics_loop;
use sysinfo::{get_current_pid, Pid, ProcessesToUpdate, System};
mod app;
mod config;
mod metrics;

const USAGE: &'static str = "\
usage: lan-tracker.exe [-s|--service] [-c|--config path] [-h|--help]\n\
    -s|--service: start background task\n\
    -c|--config: specify which config file to use\n\
    -h|--help:   print this usage information";

fn main() {
    current_exe().map_err(|e| e.to_string())
        .and_then(|exe| match exe.parent() {
            Some(p) => Ok(p.to_path_buf()),
            None => Err(String::from("could not get parent folder of current_exe"))
        })
        .and_then(|path|
            simple_logging::log_to_file(path.join("latest.log"), LevelFilter::Info).map_err(|e| e.to_string())
        ).unwrap_or_else(|e| {
            error!("error initializing logger: {e}");
            exit(1);
        });
    
    
    let mut is_service = false;
    let mut config_path_buf = current_dir()
        .and_then(|dir| Ok(dir.join(Path::new("config.toml"))))
        .unwrap_or_else(|e| {
            error!("error getting config path: {}", e.to_string());
            exit(1);
        });

    let mut arguments = args();
    while let Some(arg) = arguments.next() {
        if arg == "-s" || arg == "--service" {
            is_service = true;
        }
        if arg == "-c" || arg == "--config" {
            config_path_buf = current_dir()
                .and_then(|dir| 
                    Ok(dir.join(Path::new(
                        &arguments.next().unwrap_or_else(|| {
                            error!("missing parameter after option: -c");
                            info!("{USAGE}");
                            exit(1);
                        })
                    ).to_path_buf()))
                ).unwrap_or_else(|e| {
                    error!("error getting path from argument: {}", e.to_string());
                    exit(1);
                });
        }
        if arg == "-h" || arg == "--help" {
            info!("{}", USAGE);
            exit(0);
        }
    }

    set_config_path(&config_path_buf).unwrap_or_else(|e| {
        error!("error setting config path: {e}");
        exit(1);
    });

    let mut sys = System::new();
    let lock_path = config_path_buf.parent()
        .map(|p| p.join(Path::new("service.lock")))
        .unwrap_or(PathBuf::from("service.lock"));
    let service_pid = File::open(&lock_path).map_err(|e| e.to_string())
        .and_then(|mut f| {
            let mut out = String::new();
            f.read_to_string(&mut out).map_err(|e| e.to_string())?;
            Ok(out)
        })
        .and_then(|pid| pid.parse::<usize>().map_err(|e| e.to_string()))
        .map(|pid| Pid::from(pid))
        .and_then(|pid| {
            sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
            match sys.process(pid) {
                Some(_) => Ok(pid),
                None => Err(format!("process with pid {pid} not found"))
            }
        });

    if is_service {
        match service_pid {
            Ok(pid) => {
                warn!("service is already running with pid {pid}");
                exit(0);
            },
            Err(e) => {
                warn!("error reading service.lock file: {e}");
                match File::create(&lock_path).map_err(|e| e.to_string())
                    .and_then(|f|
                        Ok((f, get_current_pid().map_err(|e| e.to_string())?))
                    ).and_then(|(mut f, pid)| f.write(pid.to_string().as_bytes()).map_err(|e| e.to_string())) {
                    Ok(_) => (),
                    Err(e) => error!("error writing lock file: {e}")
                }
            }
        }

        match metrics_loop() {
            Ok(()) => error!("error: metrics loop stopped unexpectedly"),
            Err(s) => error!("error in metrics loop: {s}")
        }
    } else {
        match service_pid {
            Ok(pid) => warn!("service is already running with pid {pid}"),
            Err(e) => {
                warn!("error reading service.lock file: {e}");
                match current_exe()
                .and_then(|exe| 
                    Command::new(exe)
                        .args(["-s", "-c", config_path_buf.to_str().unwrap_or("config.toml")])
                        .spawn()
                ) {
                    Ok(_) => (),
                    Err(e) => error!("error spawning service: {e}")
                }
            }
        }

        app::run();
    }
}
