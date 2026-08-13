#[path = "../../astronomy_observer/src/astro.rs"]
mod astro;
#[path = "../../astronomy_observer/src/aurora.rs"]
mod aurora;
#[path = "../../astronomy_observer/src/comets.rs"]
mod comets;
#[path = "../../astronomy_observer/src/config.rs"]
mod config;
#[path = "../../astronomy_observer/src/coordinates.rs"]
mod coordinates;
#[path = "../../astronomy_observer/src/engine.rs"]
mod engine;
#[path = "../../astronomy_observer/src/error.rs"]
mod error;
#[path = "../../astronomy_observer/src/light_pollution.rs"]
mod light_pollution;
#[path = "../../astronomy_observer/src/meteors.rs"]
mod meteors;
#[path = "../../astronomy_observer/src/models.rs"]
mod models;
#[path = "../../astronomy_observer/src/satellites.rs"]
mod satellites;
#[path = "../../astronomy_observer/src/scoring.rs"]
mod scoring;
#[path = "../../astronomy_observer/src/targets.rs"]
mod targets;
#[path = "../../astronomy_observer/src/weather.rs"]
mod weather;

mod ha;
mod server;
mod session;
mod ui;

fn main() {
    if let Err(error) = server::run() {
        eprintln!("Astronomy Observer Web stopped with an error: {error}");
        std::process::exit(1);
    }
}
