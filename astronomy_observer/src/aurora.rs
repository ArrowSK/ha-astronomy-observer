use crate::coordinates::haversine_km;
use crate::models::{AuroraStatus, Location};
use chrono::Utc;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::time::Duration;
use ureq::Agent;

const URL: &str = "https://services.swpc.noaa.gov/json/ovation_aurora_latest.json";
const USER_AGENT: &str = "AstronomyObserver/0.1 (+https://github.com/ArrowSK/ha-astronomy-observer)";
const MAX_CACHE_AGE_SECONDS: i64 = 3 * 3600;

#[derive(Deserialize)]
struct Ovation {
    #[serde(rename = "Forecast Time")]
    forecast_time: Option<String>,
    coordinates: Vec<[f64; 3]>,
}

fn agent() -> Agent {
    let config = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(25)))
        .build();
    config.into()
}

fn cache_age_seconds(stamp: &Path) -> Option<i64> {
    let timestamp = fs::read_to_string(stamp).ok()?.trim().parse::<i64>().ok()?;
    Some((Utc::now().timestamp() - timestamp).max(0))
}

pub fn fetch(location: &Location, data_dir: &Path, enabled: bool) -> AuroraStatus {
    if !enabled {
        return AuroraStatus {
            probability_pct: None,
            forecast_time: None,
            source: "disabled".to_string(),
            stale: false,
        };
    }

    let cache = data_dir.join("ovation_aurora_latest.json");
    let stamp = data_dir.join("ovation_aurora.timestamp");
    let (text, stale, source) = match agent()
        .get(URL)
        .header("User-Agent", USER_AGENT)
        .call()
        .and_then(|mut response| response.body_mut().read_to_string())
    {
        Ok(text) => {
            let _ = fs::write(&cache, &text);
            let _ = fs::write(&stamp, Utc::now().timestamp().to_string());
            (text, false, "NOAA SWPC OVATION".to_string())
        }
        Err(_) => {
            let age = cache_age_seconds(&stamp).unwrap_or(i64::MAX);
            if age > MAX_CACHE_AGE_SECONDS {
                return AuroraStatus {
                    probability_pct: None,
                    forecast_time: None,
                    source: "NOAA OVATION unavailable".to_string(),
                    stale: true,
                };
            }
            match fs::read_to_string(&cache) {
                Ok(text) => (text, true, "NOAA SWPC OVATION cache".to_string()),
                Err(_) => {
                    return AuroraStatus {
                        probability_pct: None,
                        forecast_time: None,
                        source: "NOAA OVATION unavailable".to_string(),
                        stale: true,
                    }
                }
            }
        }
    };

    let Ok(data) = serde_json::from_str::<Ovation>(&text) else {
        return AuroraStatus {
            probability_pct: None,
            forecast_time: None,
            source: "NOAA OVATION parse error".to_string(),
            stale,
        };
    };

    let mut best: Option<(f64, f64)> = None;
    for coordinate in data.coordinates {
        let mut lon = coordinate[0];
        if lon > 180.0 {
            lon -= 360.0;
        }
        let distance = haversine_km(location.latitude, location.longitude, coordinate[1], lon);
        if best.map(|current| distance < current.0).unwrap_or(true) {
            best = Some((distance, coordinate[2]));
        }
    }
    let probability = best
        .filter(|value| value.0 < 300.0)
        .map(|value| value.1.clamp(0.0, 100.0));

    AuroraStatus {
        probability_pct: probability,
        forecast_time: data.forecast_time,
        source,
        stale,
    }
}
