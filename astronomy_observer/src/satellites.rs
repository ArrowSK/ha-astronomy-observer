use crate::astro;
use crate::config::AppConfig;
use crate::coordinates::{deg_to_rad, gmst_deg, norm_deg, rad_to_deg};
use crate::models::{AstronomySample, Location, Recommendation, SkyBrightness, WeatherSeries};
use crate::{scoring, weather};
use chrono::{DateTime, Duration, Utc};
use sgp4::{Constants, Elements, MinutesSinceEpoch};
use std::fs;
use std::path::Path;
use std::time::Duration as StdDuration;
use ureq::Agent;

const CELESTRAK_VISUAL: &str =
    "https://celestrak.org/NORAD/elements/gp.php?GROUP=visual&FORMAT=json";
const EARTH_RADIUS_KM: f64 = 6378.137;
const USER_AGENT: &str =
    "AstronomyObserver/0.1 (+https://github.com/ArrowSK/ha-astronomy-observer)";
const NORMAL_CACHE_SECONDS: i64 = 12 * 3600;
const MAX_STALE_CACHE_SECONDS: i64 = 48 * 3600;

fn agent() -> Agent {
    let c = Agent::config_builder()
        .timeout_global(Some(StdDuration::from_secs(25)))
        .build();
    c.into()
}

fn fetch_elements(data_dir: &Path) -> Result<(Vec<Elements>, String), String> {
    let cache = data_dir.join("celestrak_visual.json");
    let stamp = data_dir.join("celestrak_visual.timestamp");
    let timestamp = fs::read_to_string(&stamp)
        .ok()
        .and_then(|value| value.trim().parse::<i64>().ok());
    let age = timestamp
        .map(|value| (Utc::now().timestamp() - value).max(0))
        .unwrap_or(i64::MAX);

    let (text, source) = if age < NORMAL_CACHE_SECONDS {
        (
            fs::read_to_string(&cache).map_err(|error| error.to_string())?,
            "CelesTrak visual GP data cache".to_string(),
        )
    } else {
        match agent()
            .get(CELESTRAK_VISUAL)
            .header("User-Agent", USER_AGENT)
            .call()
            .and_then(|mut response| response.body_mut().read_to_string())
        {
            Ok(text) => {
                let _ = fs::write(&cache, &text);
                let _ = fs::write(&stamp, Utc::now().timestamp().to_string());
                (text, "CelesTrak visual GP data".to_string())
            }
            Err(error) => {
                if age > MAX_STALE_CACHE_SECONDS {
                    return Err(format!(
                        "CelesTrak unavailable and cached elements are too old: {error}"
                    ));
                }
                (
                    fs::read_to_string(&cache)
                        .map_err(|_| format!("CelesTrak unavailable: {error}"))?,
                    format!("CelesTrak cached GP data ({} h old)", age / 3600),
                )
            }
        }
    };
    let elements = serde_json::from_str::<Vec<Elements>>(&text)
        .map_err(|error| format!("cannot parse CelesTrak data: {error}"))?;
    Ok((elements, source))
}

fn observer_ecef(location: &Location) -> [f64; 3] {
    let lat = deg_to_rad(location.latitude);
    let lon = deg_to_rad(location.longitude);
    let h = location.elevation_m / 1000.0;
    let f = 1.0 / 298.257223563;
    let e2 = f * (2.0 - f);
    let n = EARTH_RADIUS_KM / (1.0 - e2 * lat.sin().powi(2)).sqrt();
    [
        (n + h) * lat.cos() * lon.cos(),
        (n + h) * lat.cos() * lon.sin(),
        (n * (1.0 - e2) + h) * lat.sin(),
    ]
}

fn topocentric(
    position_teme: [f64; 3],
    location: &Location,
    time: DateTime<Utc>,
) -> (f64, f64, f64) {
    let th = deg_to_rad(gmst_deg(time));
    let c = th.cos();
    let s = th.sin();
    let ecef = [
        c * position_teme[0] + s * position_teme[1],
        -s * position_teme[0] + c * position_teme[1],
        position_teme[2],
    ];
    let obs = observer_ecef(location);
    let d = [ecef[0] - obs[0], ecef[1] - obs[1], ecef[2] - obs[2]];
    let lat = deg_to_rad(location.latitude);
    let lon = deg_to_rad(location.longitude);
    let east = -lon.sin() * d[0] + lon.cos() * d[1];
    let north = -lat.sin() * lon.cos() * d[0] - lat.sin() * lon.sin() * d[1] + lat.cos() * d[2];
    let up = lat.cos() * lon.cos() * d[0] + lat.cos() * lon.sin() * d[1] + lat.sin() * d[2];
    let range = (east * east + north * north + up * up).sqrt();
    let alt = rad_to_deg(up.atan2((east * east + north * north).sqrt()));
    let az = norm_deg(rad_to_deg(east.atan2(north)));
    (alt, az, range)
}

fn sunlit(position: [f64; 3], sample: &AstronomySample) -> bool {
    let Some(sun) = sample.bodies.get("Sun") else {
        return true;
    };
    let ra = deg_to_rad(sun.ra_hours * 15.0);
    let dec = deg_to_rad(sun.dec_deg);
    let u = [dec.cos() * ra.cos(), dec.cos() * ra.sin(), dec.sin()];
    let dot = position[0] * u[0] + position[1] * u[1] + position[2] * u[2];
    if dot >= 0.0 {
        return true;
    }
    let r2 = position.iter().map(|x| x * x).sum::<f64>();
    let perp2 = (r2 - dot * dot).max(0.0);
    perp2.sqrt() > EARTH_RADIUS_KM
}

pub fn recommendations(
    cfg: &AppConfig,
    location: &Location,
    weather_series: &WeatherSeries,
    samples: &[AstronomySample],
    sky: &SkyBrightness,
    now: DateTime<Utc>,
    data_dir: &Path,
) -> (Vec<Recommendation>, String) {
    if !cfg.options.enable_satellites {
        return (Vec::new(), "disabled".to_string());
    }
    let (elements, source) = match fetch_elements(data_dir) {
        Ok(x) => x,
        Err(e) => return (Vec::new(), e),
    };
    let end = now + Duration::hours(cfg.options.observing_window_hours as i64);
    let step = Duration::minutes(2);
    let mut out = Vec::new();
    for el in elements.iter().take(400) {
        let Ok(constants) = Constants::from_elements(el) else {
            continue;
        };
        let mut t = now;
        let mut winner: Option<Recommendation> = None;
        while t <= end {
            let minutes = (t.naive_utc() - el.datetime).num_milliseconds() as f64 / 60000.0;
            let Ok(pred) = constants.propagate(MinutesSinceEpoch(minutes)) else {
                t += step;
                continue;
            };
            let Some(sample) = astro::sample_nearest(samples, t) else {
                t += step;
                continue;
            };
            let sun_alt = sample
                .bodies
                .get("Sun")
                .map(|x| x.altitude_deg)
                .unwrap_or(90.0);
            if sun_alt > -4.0 || !sunlit(pred.position, sample) {
                t += step;
                continue;
            }
            let (alt, az, range) = topocentric(pred.position, location, t);
            let horizon = cfg.horizon.altitude_at(az).max(10.0);
            if alt < horizon {
                t += step;
                continue;
            }
            let Some(w) = weather::nearest(weather_series, t) else {
                t += step;
                continue;
            };
            let c = scoring::score_hour(w, sample, sky);
            let alt_factor = ((alt - horizon) / (90.0 - horizon)).clamp(0.0, 1.0).sqrt();
            let score = (0.70 * c.cloud + 30.0 * alt_factor).clamp(0.0, 100.0);
            let name = el
                .object_name
                .clone()
                .unwrap_or_else(|| format!("NORAD {}", el.norad_id));
            let rec = Recommendation {
                name,
                category: "satellite".to_string(),
                score,
                best_time: t,
                altitude_deg: alt,
                azimuth_deg: az,
                magnitude: None,
                moon_separation_deg: None,
                equipment: "naked eye or binoculars".to_string(),
                note: format!(
                    "CelesTrak visual group; range {:.0} km; predicted sunlit",
                    range
                ),
            };
            if winner.as_ref().map(|x| score > x.score).unwrap_or(true) {
                winner = Some(rec)
            }
            t += step;
        }
        if let Some(r) = winner {
            out.push(r)
        }
    }
    out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    out.truncate(5);
    (out, source)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zenith_geometry_is_finite() {
        let l = Location {
            latitude: 47.5,
            longitude: 19.0,
            elevation_m: 100.0,
            label: String::new(),
            timezone: "UTC".into(),
            source: String::new(),
        };
        let t = Utc::now();
        let (a, z, r) = topocentric([7000.0, 0.0, 0.0], &l, t);
        assert!(a.is_finite() && z.is_finite() && r > 0.0);
    }
}
