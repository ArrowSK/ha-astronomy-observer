mod astro;
mod aurora;
mod comets;
mod config;
mod coordinates;
mod engine;
mod error;
mod ha;
mod light_pollution;
mod meteors;
mod models;
mod satellites;
mod scoring;
mod state;
mod targets;
mod weather;
mod web;

use crate::config::AppConfig;
use crate::error::{err, AppResult};
use crate::ha::HaClient;
use crate::models::Snapshot;
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::flag;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, RwLock};
use std::time::{Duration, Instant};

fn arg_value(args: &[String], name: &str, default: &str) -> PathBuf {
    args.iter()
        .position(|item| item == name)
        .and_then(|index| args.get(index + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(default))
}

fn self_test() -> AppResult<()> {
    let horizon = coordinates::HorizonMask::parse("0:5,90:10,180:5,270:0")?;
    if (horizon.altitude_at(45.0) - 7.5).abs() > 0.01 {
        return Err(err("horizon interpolation self-test failed"));
    }
    if (light_pollution::artificial_to_sqm(0.0) - 22.0).abs() > 1e-9 {
        return Err(err("sky-brightness conversion self-test failed"));
    }
    println!("self-test passed");
    Ok(())
}

fn run() -> AppResult<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|item| item == "--self-test") {
        return self_test();
    }

    let options = arg_value(&args, "--options", "/data/options.json");
    let data_dir = arg_value(&args, "--data-dir", "/data");
    let config_dir = arg_value(&args, "--config-dir", "/config");
    std::fs::create_dir_all(&data_dir)?;

    AppConfig::load(&options, &data_dir, &config_dir)?;
    let ha = HaClient::new()?;
    let shared: Arc<RwLock<Option<Snapshot>>> = Arc::new(RwLock::new(None));
    let (refresh_tx, refresh_rx) = mpsc::channel();
    web::serve(
        shared.clone(),
        refresh_tx,
        data_dir.clone(),
        options.clone(),
        config_dir.clone(),
        ha.clone(),
    );

    let terminate = Arc::new(AtomicBool::new(false));
    flag::register(SIGTERM, terminate.clone())?;
    flag::register(SIGINT, terminate.clone())?;

    let mut next = Instant::now();

    while !terminate.load(Ordering::Relaxed) {
        if Instant::now() >= next {
            match AppConfig::load(&options, &data_dir, &config_dir) {
                Ok(cfg) => {
                    let interval = Duration::from_secs(cfg.options.refresh_minutes * 60);
                    match engine::refresh(&cfg, &ha) {
                        Ok(snapshot) => {
                            println!(
                                "refresh complete: score {:.0}, {} recommendations, weather {}",
                                snapshot.conditions.overall,
                                snapshot.recommendations.len(),
                                snapshot.weather_source
                            );
                            if let Err(error) =
                                state::publish(&ha, &snapshot, cfg.options.good_observing_threshold)
                            {
                                eprintln!("Home Assistant state publish error: {error}");
                            }
                            if let Ok(mut guard) = shared.write() {
                                *guard = Some(snapshot);
                            }
                        }
                        Err(error) => eprintln!("refresh failed: {error}"),
                    }
                    next = Instant::now() + interval;
                }
                Err(error) => {
                    eprintln!("configuration reload failed: {error}");
                    next = Instant::now() + Duration::from_secs(60);
                }
            }
        }

        let wait = next
            .saturating_duration_since(Instant::now())
            .min(Duration::from_secs(5));
        if refresh_rx.recv_timeout(wait).is_ok() {
            next = Instant::now();
        }
    }

    println!("Astronomy Observer stopped");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Astronomy Observer stopped with an error: {error}");
        std::process::exit(1);
    }
}
