use crate::config::AppConfig;
use crate::coordinates::HorizonMask;
use crate::engine;
use crate::error::{err, AppResult};
use crate::ha::HaClient;
use crate::models::Location;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct LocationInput {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub elevation_m: f64,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub timezone: String,
}

#[derive(Debug, Deserialize)]
pub struct SnapshotInput {
    pub location: LocationInput,
    #[serde(default = "default_minimum_altitude")]
    pub minimum_target_altitude: f64,
    #[serde(default = "default_horizon")]
    pub horizon_mask: String,
}

fn default_minimum_altitude() -> f64 {
    20.0
}

fn default_horizon() -> String {
    "0:0,90:0,180:0,270:0".to_string()
}

pub fn validate(input: &SnapshotInput) -> Result<(), String> {
    let location = &input.location;
    if !location.latitude.is_finite() || !(-90.0..=90.0).contains(&location.latitude) {
        return Err("latitude must be between -90 and 90".to_string());
    }
    if !location.longitude.is_finite() || !(-180.0..=180.0).contains(&location.longitude) {
        return Err("longitude must be between -180 and 180".to_string());
    }
    if !location.elevation_m.is_finite() || !(-500.0..=9000.0).contains(&location.elevation_m) {
        return Err("elevation must be between -500 and 9000 metres".to_string());
    }
    if location.label.chars().count() > 100 {
        return Err("location label is limited to 100 characters".to_string());
    }
    if !input.minimum_target_altitude.is_finite()
        || !(0.0..=60.0).contains(&input.minimum_target_altitude)
    {
        return Err("lowest useful altitude must be between 0 and 60 degrees".to_string());
    }
    let timezone = if location.timezone.trim().is_empty() {
        "UTC"
    } else {
        location.timezone.trim()
    };
    timezone
        .parse::<chrono_tz::Tz>()
        .map_err(|_| "time zone must be a valid IANA time-zone name".to_string())?;
    HorizonMask::parse(input.horizon_mask.trim())
        .map(|_| ())
        .map_err(|error| format!("directional horizon mask is invalid: {error}"))
}

pub fn calculate(input: SnapshotInput, data_dir: &Path, config_dir: &Path) -> AppResult<Value> {
    validate(&input).map_err(err)?;
    fs::create_dir_all(data_dir)?;
    fs::create_dir_all(config_dir)?;

    let options_path = data_dir.join("android-options.json");
    let mut cfg = AppConfig::load(&options_path, data_dir, config_dir)?;
    cfg.options.primary_person.clear();
    cfg.options.sqm_entity.clear();
    cfg.options.minimum_target_altitude = input.minimum_target_altitude;
    cfg.options.horizon_mask = input.horizon_mask.trim().to_string();
    cfg.horizon = HorizonMask::parse(&cfg.options.horizon_mask)?;

    let timezone = if input.location.timezone.trim().is_empty() {
        "UTC".to_string()
    } else {
        input.location.timezone.trim().to_string()
    };
    let location = Location {
        latitude: input.location.latitude,
        longitude: input.location.longitude,
        elevation_m: input.location.elevation_m,
        label: if input.location.label.trim().is_empty() {
            "Current location".to_string()
        } else {
            input.location.label.trim().to_string()
        },
        timezone,
        source: "Android device/manual location, local".to_string(),
    };
    let ha = HaClient::for_location(&location);
    Ok(serde_json::to_value(engine::refresh(&cfg, &ha)?)?)
}
