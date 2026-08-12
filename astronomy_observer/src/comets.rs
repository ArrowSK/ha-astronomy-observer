use crate::astro;
use crate::config::AppConfig;
use crate::coordinates::{
    alt_az_from_j2000, angular_separation_deg, deg_to_rad, norm_deg, rad_to_deg,
};
use crate::models::{AstronomySample, Recommendation, SkyBrightness, WeatherSeries};
use crate::{scoring, weather};
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use std::fs;
use std::path::Path;
use std::time::Duration as StdDuration;
use ureq::Agent;

const PRIMARY: &str = "https://www.minorplanetcenter.net/iau/MPCORB/AllCometEls.txt";
const FALLBACK: &str = "https://www.minorplanetcenter.net/iau/Ephemerides/Comets/Soft00Cmt.txt";
const GAUSS_K: f64 = 0.01720209895;
const USER_AGENT: &str =
    "AstronomyObserver/0.1 (+https://github.com/ArrowSK/ha-astronomy-observer)";
const MAX_STALE_CACHE_SECONDS: i64 = 7 * 24 * 3600;

#[derive(Debug, Clone)]
struct Comet {
    name: String,
    peri: DateTime<Utc>,
    q: f64,
    e: f64,
    argp: f64,
    node: f64,
    inc: f64,
    h: Option<f64>,
    slope: Option<f64>,
}
fn field(line: &str, a: usize, b: usize) -> &str {
    line.get(a..b).unwrap_or("").trim()
}
fn f(line: &str, a: usize, b: usize) -> Option<f64> {
    field(line, a, b).parse().ok()
}
fn i(line: &str, a: usize, b: usize) -> Option<i32> {
    field(line, a, b).parse().ok()
}

fn parse_line(line: &str) -> Option<Comet> {
    if line.len() < 120 {
        return None;
    }
    let year = i(line, 14, 18)?;
    let month = i(line, 19, 21)? as u32;
    let day = f(line, 22, 29)?;
    let day_int = day.floor() as u32;
    let date = NaiveDate::from_ymd_opt(year, month, day_int)?;
    let secs = ((day - day.floor()) * 86400.0).round() as i64;
    let peri = Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0)?) + Duration::seconds(secs);
    let q = f(line, 30, 39)?;
    let e = f(line, 41, 49)?;
    let argp = f(line, 51, 59)?;
    let node = f(line, 61, 69)?;
    let inc = f(line, 71, 79)?;
    if q <= 0.0 || e < 0.0 {
        return None;
    }
    let name = field(line, 102, 158).to_string();
    if name.is_empty() {
        return None;
    }
    Some(Comet {
        name,
        peri,
        q,
        e,
        argp,
        node,
        inc,
        h: f(line, 91, 95),
        slope: f(line, 96, 100),
    })
}

fn agent() -> Agent {
    let c = Agent::config_builder()
        .timeout_global(Some(StdDuration::from_secs(25)))
        .build();
    c.into()
}
fn get(url: &str) -> Result<String, String> {
    agent()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .and_then(|mut r| r.body_mut().read_to_string())
        .map_err(|e| e.to_string())
}
fn load(data_dir: &Path) -> Result<(Vec<Comet>, String), String> {
    let cache = data_dir.join("mpc_comets.txt");
    let stamp = data_dir.join("mpc_comets.timestamp");
    let fresh = fs::read_to_string(&stamp)
        .ok()
        .and_then(|s| s.trim().parse::<i64>().ok())
        .map(|t| Utc::now().timestamp() - t < 12 * 3600)
        .unwrap_or(false);
    let (text, source) = if fresh {
        (
            fs::read_to_string(&cache).map_err(|e| e.to_string())?,
            "MPC cached elements".to_string(),
        )
    } else {
        match get(PRIMARY).or_else(|_| get(FALLBACK)) {
            Ok(s) => {
                let _ = fs::write(&cache, &s);
                let _ = fs::write(&stamp, Utc::now().timestamp().to_string());
                (s, "Minor Planet Center".to_string())
            }
            Err(e) => {
                let age = fs::read_to_string(&stamp)
                    .ok()
                    .and_then(|s| s.trim().parse::<i64>().ok())
                    .map(|t| (Utc::now().timestamp() - t).max(0))
                    .unwrap_or(i64::MAX);
                if age > MAX_STALE_CACHE_SECONDS {
                    return Err(format!(
                        "MPC comet elements unavailable and cache is too old: {e}"
                    ));
                }
                (
                    fs::read_to_string(&cache)
                        .map_err(|_| format!("MPC comet elements unavailable: {e}"))?,
                    format!("MPC cached elements ({} h old)", age / 3600),
                )
            }
        }
    };
    let comets: Vec<_> = text.lines().filter_map(parse_line).collect();
    if comets.is_empty() {
        return Err("MPC comet file contained no parseable elements".into());
    }
    Ok((comets, source))
}

fn solve_kepler(comet: &Comet, days: f64) -> Option<([f64; 3], f64)> {
    let w = deg_to_rad(comet.argp);
    let o = deg_to_rad(comet.node);
    let inc = deg_to_rad(comet.inc);
    let (x, y) = if (comet.e - 1.0).abs() < 1e-5 {
        let scale = (2.0 * comet.q.powi(3)).sqrt() / GAUSS_K;
        let target = days / scale;
        let mut d = target;
        for _ in 0..30 {
            let fv = d + d.powi(3) / 3.0 - target;
            let df = 1.0 + d * d;
            let step = fv / df;
            d -= step;
            if step.abs() < 1e-12 {
                break;
            }
        }
        (comet.q * (1.0 - d * d), 2.0 * comet.q * d)
    } else if comet.e < 1.0 {
        let a = comet.q / (1.0 - comet.e);
        let m = GAUSS_K * days / a.powf(1.5);
        let mut ee = m;
        for _ in 0..30 {
            let step = (ee - comet.e * ee.sin() - m) / (1.0 - comet.e * ee.cos());
            ee -= step;
            if step.abs() < 1e-12 {
                break;
            }
        }
        (
            a * (ee.cos() - comet.e),
            a * (1.0 - comet.e * comet.e).sqrt() * ee.sin(),
        )
    } else {
        let a = comet.q / (comet.e - 1.0);
        let m = GAUSS_K * days / a.powf(1.5);
        let mut hh = (m / comet.e).asinh();
        for _ in 0..30 {
            let step = (comet.e * hh.sinh() - hh - m) / (comet.e * hh.cosh() - 1.0);
            hh -= step;
            if step.abs() < 1e-12 {
                break;
            }
        }
        (
            a * (comet.e - hh.cosh()),
            a * (comet.e * comet.e - 1.0).sqrt() * hh.sinh(),
        )
    };
    let cw = w.cos();
    let sw = w.sin();
    let co = o.cos();
    let so = o.sin();
    let ci = inc.cos();
    let si = inc.sin();
    let xx = (co * cw - so * sw * ci) * x + (-co * sw - so * cw * ci) * y;
    let yy = (so * cw + co * sw * ci) * x + (-so * sw + co * cw * ci) * y;
    let zz = (sw * si) * x + (cw * si) * y;
    let r = (xx * xx + yy * yy + zz * zz).sqrt();
    (r.is_finite() && r > 0.0).then_some(([xx, yy, zz], r))
}

fn geocentric(comet: &Comet, sample: &AstronomySample) -> Option<(f64, f64, f64, f64)> {
    let days = (sample.time - comet.peri).num_seconds() as f64 / 86400.0;
    let (helio, r) = solve_kepler(comet, days)?;
    let g = [
        helio[0] - sample.earth_ecliptic_au[0],
        helio[1] - sample.earth_ecliptic_au[1],
        helio[2] - sample.earth_ecliptic_au[2],
    ];
    let delta = (g.iter().map(|x| x * x).sum::<f64>()).sqrt();
    if delta <= 0.0 {
        return None;
    }
    let eps = deg_to_rad(23.439291111);
    let eq = [
        g[0],
        g[1] * eps.cos() - g[2] * eps.sin(),
        g[1] * eps.sin() + g[2] * eps.cos(),
    ];
    let ra = norm_deg(rad_to_deg(eq[1].atan2(eq[0])));
    let dec = rad_to_deg((eq[2] / delta).clamp(-1.0, 1.0).asin());
    Some((ra, dec, r, delta))
}

fn apparent_mag(c: &Comet, r: f64, delta: f64) -> Option<f64> {
    let h = c.h?;
    let k = c.slope.unwrap_or(10.0);
    Some(h + 5.0 * delta.log10() + k * r.log10())
}
fn equipment(mag: f64, cfg: &AppConfig) -> Option<String> {
    if mag <= 5.5 {
        return Some("naked eye".into());
    }
    let bl = if cfg.options.binocular_aperture_mm > 0.0 {
        2.0 + 5.0 * cfg.options.binocular_aperture_mm.log10()
    } else {
        0.0
    };
    if mag <= bl {
        return Some(format!(
            "{} mm binoculars",
            cfg.options.binocular_aperture_mm.round()
        ));
    }
    let tl = if cfg.options.telescope_aperture_mm > 0.0 {
        2.0 + 5.0 * cfg.options.telescope_aperture_mm.log10()
    } else {
        0.0
    };
    if mag <= tl + 0.5 {
        return Some(format!(
            "{} mm telescope",
            cfg.options.telescope_aperture_mm.round()
        ));
    }
    None
}

pub fn recommendations(
    cfg: &AppConfig,
    lat: f64,
    lon: f64,
    weather_series: &WeatherSeries,
    samples: &[AstronomySample],
    sky: &SkyBrightness,
    now: DateTime<Utc>,
    data_dir: &Path,
) -> (Vec<Recommendation>, String) {
    if !cfg.options.enable_comets {
        return (Vec::new(), "disabled".into());
    }
    let (comets, source) = match load(data_dir) {
        Ok(v) => v,
        Err(e) => return (Vec::new(), e),
    };
    let end = now + Duration::hours(cfg.options.observing_window_hours as i64);
    let mut out = Vec::new();
    for cmt in comets {
        if (cmt.peri - now).num_days().abs() > 3650 {
            continue;
        }
        let mut winner: Option<Recommendation> = None;
        for sample in samples.iter().filter(|s| s.time >= now && s.time <= end) {
            let sun = sample
                .bodies
                .get("Sun")
                .map(|x| x.altitude_deg)
                .unwrap_or(90.0);
            if sun > -6.0 {
                continue;
            }
            let Some((ra, dec, r, delta)) = geocentric(&cmt, sample) else {
                continue;
            };
            if r > 15.0 || delta > 15.0 {
                continue;
            }
            let mag = apparent_mag(&cmt, r, delta).unwrap_or(15.0);
            if mag > 16.0 {
                continue;
            }
            let Some(eq) = equipment(mag, cfg) else {
                continue;
            };
            let (alt, az) = alt_az_from_j2000(ra, dec, lat, lon, sample.time);
            let horizon = cfg
                .horizon
                .altitude_at(az)
                .max(cfg.options.minimum_target_altitude);
            if alt < horizon {
                continue;
            }
            let Some(w) = weather::nearest(weather_series, sample.time) else {
                continue;
            };
            let cond = scoring::score_hour(w, sample, sky);
            let moon = sample.bodies.get("Moon");
            let sep = moon.map(|m| {
                angular_separation_deg(
                    crate::coordinates::precess_j2000(ra, dec, sample.time).0,
                    crate::coordinates::precess_j2000(ra, dec, sample.time).1,
                    m.ra_hours * 15.0,
                    m.dec_deg,
                )
            });
            let moon_pen = moon
                .zip(sep)
                .map(|(m, s)| {
                    m.illuminated_fraction.unwrap_or(0.5)
                        * if m.altitude_deg > 0.0 { 1.0 } else { 0.0 }
                        * (1.0 - s / 180.0).clamp(0.05, 1.0)
                })
                .unwrap_or(0.15);
            let altf = ((alt - horizon) / (90.0 - horizon)).clamp(0.0, 1.0).sqrt();
            let bright = (1.0 - (mag - 5.0).max(0.0) / 18.0).clamp(0.45, 1.0);
            let score = (cond.deep_sky / 100.0 * altf * (1.0 - moon_pen * 0.75) * bright * 100.0)
                .clamp(0.0, 100.0);
            let rec = Recommendation {
                name: cmt.name.clone(),
                category: "comet".into(),
                score,
                best_time: sample.time,
                altitude_deg: alt,
                azimuth_deg: az,
                magnitude: Some(mag),
                moon_separation_deg: sep,
                equipment: eq,
                note: format!(
                    "MPC elements; estimated total magnitude {:.1}; r {:.2} AU, Δ {:.2} AU",
                    mag, r, delta
                ),
            };
            if winner.as_ref().map(|x| score > x.score).unwrap_or(true) {
                winner = Some(rec)
            }
        }
        if let Some(r) = winner {
            out.push(r)
        }
    }
    out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    out.truncate(8);
    (out, source)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parabolic_perihelion() {
        let c = Comet {
            name: "x".into(),
            peri: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            q: 1.0,
            e: 1.0,
            argp: 0.0,
            node: 0.0,
            inc: 0.0,
            h: Some(5.0),
            slope: Some(10.0),
        };
        let (v, r) = solve_kepler(&c, 0.0).unwrap();
        assert!((r - 1.0).abs() < 1e-9);
        assert!((v[0] - 1.0).abs() < 1e-9);
    }
}
