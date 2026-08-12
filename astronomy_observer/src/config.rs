use crate::coordinates::HorizonMask;
use crate::error::{err, AppResult};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Options {
    pub primary_person: String,
    pub refresh_minutes: u64,
    pub forecast_days: usize,
    pub observing_window_hours: usize,
    pub minimum_target_altitude: f64,
    pub good_observing_threshold: f64,
    pub telescope_aperture_mm: f64,
    pub binocular_aperture_mm: f64,
    pub horizon_mask: String,
    pub sqm_override: f64,
    pub sqm_entity: String,
    pub light_pollution_file: String,
    pub nearby_dark_site_radius_km: f64,
    pub enable_satellites: bool,
    pub enable_aurora: bool,
    pub enable_comets: bool,
    pub external_location_precision: u32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            primary_person: String::new(),
            refresh_minutes: 30,
            forecast_days: 7,
            observing_window_hours: 14,
            minimum_target_altitude: 20.0,
            good_observing_threshold: 70.0,
            telescope_aperture_mm: 200.0,
            binocular_aperture_mm: 50.0,
            horizon_mask: "0:0,45:0,90:0,135:0,180:0,225:0,270:0,315:0".to_string(),
            sqm_override: 0.0,
            sqm_entity: String::new(),
            light_pollution_file: "light_pollution.csv".to_string(),
            nearby_dark_site_radius_km: 75.0,
            enable_satellites: true,
            enable_aurora: true,
            enable_comets: true,
            external_location_precision: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub options: Options,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub horizon: HorizonMask,
}

impl AppConfig {
    pub fn load(options_path: &Path, data_dir: &Path, config_dir: &Path) -> AppResult<Self> {
        let options: Options = if options_path.exists() {
            serde_json::from_str(&fs::read_to_string(options_path)?)?
        } else {
            Options::default()
        };

        if !(10..=180).contains(&options.refresh_minutes) {
            return Err(err("refresh_minutes must be between 10 and 180"));
        }
        if !(2..=7).contains(&options.forecast_days) {
            return Err(err("forecast_days must be between 2 and 7"));
        }
        if !(8..=18).contains(&options.observing_window_hours) {
            return Err(err("observing_window_hours must be between 8 and 18"));
        }
        if !(0.0..=60.0).contains(&options.minimum_target_altitude) {
            return Err(err("minimum_target_altitude must be between 0 and 60"));
        }
        if !(1.0..=100.0).contains(&options.good_observing_threshold) {
            return Err(err("good_observing_threshold must be between 1 and 100"));
        }
        if options.sqm_override != 0.0 && !(15.0..=23.0).contains(&options.sqm_override) {
            return Err(err(
                "sqm_override must be 0 or between 15 and 23 mag/arcsec²",
            ));
        }
        if !options.sqm_entity.trim().is_empty() && !options.sqm_entity.starts_with("sensor.") {
            return Err(err("sqm_entity must be a sensor.* entity"));
        }
        if options.external_location_precision < 2 || options.external_location_precision > 5 {
            return Err(err("external_location_precision must be between 2 and 5"));
        }

        let horizon = HorizonMask::parse(&options.horizon_mask)?;
        Ok(Self {
            options,
            data_dir: data_dir.to_path_buf(),
            config_dir: config_dir.to_path_buf(),
            horizon,
        })
    }
}
