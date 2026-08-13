#[path = "../astro.rs"]
mod astro;
#[path = "../aurora.rs"]
mod aurora;
#[path = "../comets.rs"]
mod comets;
#[path = "../config.rs"]
mod config;
#[path = "../coordinates.rs"]
mod coordinates;
#[path = "../engine.rs"]
mod engine;
#[path = "../error.rs"]
mod error;
#[path = "../light_pollution.rs"]
mod light_pollution;
#[path = "../meteors.rs"]
mod meteors;
#[path = "../models.rs"]
mod models;
#[path = "../satellites.rs"]
mod satellites;
#[path = "../scoring.rs"]
mod scoring;
#[path = "../targets.rs"]
mod targets;
#[path = "../weather.rs"]
mod weather;

#[path = "../../../webapp/src/ha.rs"]
mod ha;
#[path = "../../../webapp/src/server.rs"]
mod server;
#[path = "../../../webapp/src/session.rs"]
mod session;
#[path = "../../../webapp/src/ui.rs"]
mod ui;

fn main() {
    if let Err(error) = server::run() {
        eprintln!("Astronomy Observer Web stopped with an error: {error}");
        std::process::exit(1);
    }
}
