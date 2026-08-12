use crate::config::AppConfig;
use crate::coordinates::{airmass, alt_az_from_j2000, angular_separation_deg, precess_j2000};
use crate::error::AppResult;
use crate::models::{AstronomySample, Recommendation, SkyBrightness, WeatherSeries};
use crate::scoring;
use crate::weather;
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
struct Dso {
    name: String,
    kind: String,
    ra_deg: f64,
    dec_deg: f64,
    constellation: String,
    vmag: Option<f64>,
    bmag: Option<f64>,
    surface: Option<f64>,
    major: Option<f64>,
    messier: String,
    common: String,
}

fn opt_num(s: &str) -> Option<f64> {
    s.trim().parse().ok()
}

fn parse_dso(line: &str) -> Option<Dso> {
    let p: Vec<&str> = line.split('\t').collect();
    if p.len() < 12 || p[0] == "name" {
        return None;
    }
    Some(Dso {
        name: p[0].to_string(),
        kind: p[1].to_string(),
        ra_deg: p[2].parse().ok()?,
        dec_deg: p[3].parse().ok()?,
        constellation: p[4].to_string(),
        vmag: opt_num(p[5]),
        bmag: opt_num(p[6]),
        surface: opt_num(p[7]),
        major: opt_num(p[8]),
        messier: p[10].to_string(),
        common: p[11].to_string(),
    })
}

fn kind_name(kind: &str) -> &'static str {
    match kind {
        "G" | "GPair" | "GTrpl" | "GGroup" => "galaxy",
        "OCl" => "open cluster",
        "GCl" => "globular cluster",
        "PN" => "planetary nebula",
        "HII" | "EmN" | "Neb" | "RfN" | "Cl+N" => "nebula",
        "DrkN" => "dark nebula",
        "SNR" => "supernova remnant",
        "**" => "double star",
        "*Ass" => "stellar association",
        _ => "deep-sky object",
    }
}

fn display_name(d: &Dso) -> String {
    let common = d.common.split(',').next().unwrap_or("").trim();
    if !d.messier.trim().is_empty() {
        let m = d.messier.trim().trim_start_matches('0');
        if common.is_empty() {
            format!("M{m} ({})", d.name)
        } else {
            format!("M{m} — {common}")
        }
    } else if !common.is_empty() {
        format!("{} — {common}", d.name)
    } else {
        d.name.clone()
    }
}

fn equipment_for(mag: Option<f64>, major: Option<f64>, cfg: &AppConfig) -> Option<String> {
    let m = mag.unwrap_or(11.5);
    if m <= 6.0 && major.unwrap_or(0.0) >= 5.0 {
        return Some("naked eye".to_string());
    }
    let bino_limit = if cfg.options.binocular_aperture_mm > 0.0 {
        2.0 + 5.0 * cfg.options.binocular_aperture_mm.log10()
    } else {
        0.0
    };
    if cfg.options.binocular_aperture_mm > 0.0 && m <= bino_limit - 0.5 {
        return Some(format!(
            "{} mm binoculars",
            cfg.options.binocular_aperture_mm.round()
        ));
    }
    let scope_limit = if cfg.options.telescope_aperture_mm > 0.0 {
        2.0 + 5.0 * cfg.options.telescope_aperture_mm.log10()
    } else {
        0.0
    };
    if cfg.options.telescope_aperture_mm > 0.0 && m <= scope_limit + 0.5 {
        return Some(format!(
            "{} mm telescope",
            cfg.options.telescope_aperture_mm.round()
        ));
    }
    None
}

fn target_moon_penalty(
    d: &Dso,
    sample: &AstronomySample,
    t: DateTime<Utc>,
    ra_date: f64,
    dec_date: f64,
) -> (f64, Option<f64>) {
    let Some(moon) = sample.bodies.get("Moon") else {
        return (0.2, None);
    };
    let sep = angular_separation_deg(ra_date, dec_date, moon.ra_hours * 15.0, moon.dec_deg);
    let illum = moon.illuminated_fraction.unwrap_or(0.5).clamp(0.0, 1.0);
    let alt = if moon.altitude_deg <= -5.0 {
        0.0
    } else {
        ((moon.altitude_deg + 5.0) / 60.0).clamp(0.0, 1.0)
    };
    let proximity = if sep >= 120.0 {
        0.08
    } else if sep >= 60.0 {
        0.08 + (120.0 - sep) / 60.0 * 0.22
    } else if sep >= 20.0 {
        0.30 + (60.0 - sep) / 40.0 * 0.50
    } else {
        0.80 + (20.0 - sep) / 20.0 * 0.20
    };
    let sensitivity = match d.kind.as_str() {
        "G" | "GPair" | "GTrpl" | "GGroup" | "DrkN" => 1.0,
        "HII" | "EmN" | "Neb" | "RfN" | "SNR" => 0.9,
        "OCl" | "GCl" => 0.55,
        "PN" => 0.5,
        "**" => 0.2,
        _ => 0.7,
    };
    let _ = t;
    (
        (illum.powf(0.65) * alt * proximity * sensitivity).clamp(0.0, 0.95),
        Some(sep),
    )
}

fn light_pollution_target_factor(d: &Dso, sky: &SkyBrightness) -> f64 {
    let Some(sqm) = sky.sqm_mag_arcsec2 else {
        return 0.85;
    };
    let base = ((sqm - 17.0) / 4.7).clamp(0.1, 1.0);
    let sensitivity = match d.kind.as_str() {
        "G" | "GPair" | "GTrpl" | "GGroup" | "DrkN" => 1.0,
        "HII" | "EmN" | "Neb" | "RfN" => 0.85,
        "OCl" | "GCl" => 0.45,
        "PN" => 0.4,
        "**" => 0.15,
        _ => 0.6,
    };
    (1.0 - sensitivity * (1.0 - base) * 0.75).clamp(0.2, 1.0)
}

pub fn deep_sky(
    cfg: &AppConfig,
    catalog: &Path,
    weather_series: &WeatherSeries,
    samples: &[AstronomySample],
    sky: &SkyBrightness,
    now: DateTime<Utc>,
    lat: f64,
    lon: f64,
) -> AppResult<Vec<Recommendation>> {
    let file = File::open(catalog)?;
    let mut best: Vec<Recommendation> = Vec::new();
    let end = now + chrono::Duration::hours(cfg.options.observing_window_hours as i64);
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        let Some(d) = parse_dso(&line) else { continue };
        let mag = d.vmag.or(d.bmag);
        let Some(equipment) = equipment_for(mag, d.major, cfg) else {
            continue;
        };
        let mut winner: Option<Recommendation> = None;
        for sample in samples
            .iter()
            .filter(|s| s.time >= now - chrono::Duration::minutes(20) && s.time <= end)
        {
            let Some(w) = weather::nearest(weather_series, sample.time) else {
                continue;
            };
            let sun = sample
                .bodies
                .get("Sun")
                .map(|x| x.altitude_deg)
                .unwrap_or(90.0);
            if sun > -6.0 {
                continue;
            }
            let (alt, az) = alt_az_from_j2000(d.ra_deg, d.dec_deg, lat, lon, sample.time);
            let horizon = cfg
                .horizon
                .altitude_at(az)
                .max(cfg.options.minimum_target_altitude);
            if alt < horizon {
                continue;
            }
            let cond = scoring::score_hour(w, sample, sky);
            let air = (1.0 / airmass(alt).max(1.0).powf(0.65)).clamp(0.15, 1.0);
            let (ra_date, dec_date) = precess_j2000(d.ra_deg, d.dec_deg, sample.time);
            let (moon_pen, sep) = target_moon_penalty(&d, sample, sample.time, ra_date, dec_date);
            let lp = light_pollution_target_factor(&d, sky);
            let magnitude_factor = mag
                .map(|m| (1.08 - (m - 5.0).max(0.0) / 18.0).clamp(0.45, 1.05))
                .unwrap_or(0.82);
            let surface_factor = d
                .surface
                .map(|sb| {
                    if sb <= 21.5 {
                        1.0
                    } else {
                        (1.0 - (sb - 21.5) / 12.0).clamp(0.45, 1.0)
                    }
                })
                .unwrap_or(0.9);
            let interest = if !d.messier.is_empty() {
                1.06
            } else if !d.common.is_empty() {
                1.03
            } else {
                1.0
            };
            let score = (cond.deep_sky / 100.0
                * air
                * (1.0 - moon_pen)
                * lp
                * magnitude_factor
                * surface_factor
                * interest
                * 100.0)
                .clamp(0.0, 100.0);
            let note = format!(
                "{} in {}; airmass {:.2}{}",
                kind_name(&d.kind),
                d.constellation,
                airmass(alt),
                d.surface
                    .map(|x| format!(", surface brightness {:.1}", x))
                    .unwrap_or_default()
            );
            let rec = Recommendation {
                name: display_name(&d),
                category: "deep sky".to_string(),
                score,
                best_time: sample.time,
                altitude_deg: alt,
                azimuth_deg: az,
                magnitude: mag,
                moon_separation_deg: sep,
                equipment: equipment.clone(),
                note,
            };
            if winner.as_ref().map(|x| score > x.score).unwrap_or(true) {
                winner = Some(rec);
            }
        }
        if let Some(r) = winner {
            best.push(r);
        }
    }
    best.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    best.truncate(20);
    Ok(best)
}

pub fn solar_system(
    cfg: &AppConfig,
    weather_series: &WeatherSeries,
    samples: &[AstronomySample],
    sky: &SkyBrightness,
    now: DateTime<Utc>,
) -> Vec<Recommendation> {
    let end = now + chrono::Duration::hours(cfg.options.observing_window_hours as i64);
    let bodies = [
        "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune", "Pluto",
    ];
    let mut out = Vec::new();
    for body in bodies {
        let mut winner: Option<Recommendation> = None;
        for sample in samples
            .iter()
            .filter(|s| s.time >= now - chrono::Duration::minutes(20) && s.time <= end)
        {
            let Some(pos) = sample.bodies.get(body) else {
                continue;
            };
            let horizon = cfg
                .horizon
                .altitude_at(pos.azimuth_deg)
                .max(cfg.options.minimum_target_altitude);
            if pos.altitude_deg < horizon {
                continue;
            }
            let sun = sample
                .bodies
                .get("Sun")
                .map(|x| x.altitude_deg)
                .unwrap_or(90.0);
            if sun > -4.0 && body != "Moon" {
                continue;
            }
            let Some(w) = weather::nearest(weather_series, sample.time) else {
                continue;
            };
            let c = scoring::score_hour(w, sample, sky);
            let alt = (1.0 / airmass(pos.altitude_deg).max(1.0).powf(0.55)).clamp(0.2, 1.0);
            let (equipment, visibility) = match body {
                "Moon" => ("naked eye / binoculars / telescope".to_string(), 1.0),
                "Venus" | "Mars" | "Jupiter" | "Saturn" => (
                    if cfg.options.telescope_aperture_mm > 0.0 {
                        format!("{} mm telescope", cfg.options.telescope_aperture_mm.round())
                    } else {
                        "naked eye".to_string()
                    },
                    1.0,
                ),
                "Mercury" => ("binoculars or telescope".to_string(), 0.9),
                "Uranus" => (
                    "binoculars or telescope".to_string(),
                    if cfg.options.telescope_aperture_mm >= 70.0
                        || cfg.options.binocular_aperture_mm >= 40.0
                    {
                        1.0
                    } else {
                        0.0
                    },
                ),
                "Neptune" => (
                    "telescope".to_string(),
                    if cfg.options.telescope_aperture_mm >= 80.0 {
                        1.0
                    } else {
                        0.0
                    },
                ),
                "Pluto" => (
                    "large telescope".to_string(),
                    if cfg.options.telescope_aperture_mm >= 250.0 {
                        1.0
                    } else {
                        0.0
                    },
                ),
                _ => ("telescope".to_string(), 1.0),
            };
            if visibility == 0.0 {
                continue;
            }
            let base = if body == "Moon" {
                c.planetary / 100.0 * (1.0 - c.moon_interference / 100.0 * 0.15)
            } else {
                c.planetary / 100.0
            };
            let score = (base * alt * visibility * 100.0).clamp(0.0, 100.0);
            let note = if body == "Moon" {
                format!(
                    "{:.0}% illuminated",
                    pos.illuminated_fraction.unwrap_or(0.0) * 100.0
                )
            } else {
                format!("apparent magnitude {:.1}", pos.magnitude.unwrap_or(99.0))
            };
            let rec = Recommendation {
                name: body.to_string(),
                category: if body == "Moon" { "Moon" } else { "planet" }.to_string(),
                score,
                best_time: sample.time,
                altitude_deg: pos.altitude_deg,
                azimuth_deg: pos.azimuth_deg,
                magnitude: pos.magnitude,
                moon_separation_deg: None,
                equipment,
                note,
            };
            if winner.as_ref().map(|x| score > x.score).unwrap_or(true) {
                winner = Some(rec);
            }
        }
        if let Some(r) = winner {
            out.push(r);
        }
    }
    out
}

pub fn milky_way(
    cfg: &AppConfig,
    weather_series: &WeatherSeries,
    samples: &[AstronomySample],
    sky: &SkyBrightness,
    now: DateTime<Utc>,
    lat: f64,
    lon: f64,
) -> Option<Recommendation> {
    // Sagittarius A* is used as a stable reference point for the Galactic Centre.
    const RA_DEG_J2000: f64 = 266.41683;
    const DEC_DEG_J2000: f64 = -29.00781;
    let end = now + chrono::Duration::hours(cfg.options.observing_window_hours as i64);
    let mut winner: Option<Recommendation> = None;

    for sample in samples
        .iter()
        .filter(|sample| sample.time >= now - chrono::Duration::minutes(20) && sample.time <= end)
    {
        let sun_altitude = sample
            .bodies
            .get("Sun")
            .map(|sun| sun.altitude_deg)
            .unwrap_or(90.0);
        if sun_altitude > -12.0 {
            continue;
        }
        let (altitude, azimuth) =
            alt_az_from_j2000(RA_DEG_J2000, DEC_DEG_J2000, lat, lon, sample.time);
        let horizon = cfg
            .horizon
            .altitude_at(azimuth)
            .max(cfg.options.minimum_target_altitude);
        if altitude < horizon {
            continue;
        }
        let weather = weather::nearest(weather_series, sample.time)?;
        let condition = scoring::score_hour(weather, sample, sky);
        let altitude_factor = (1.0 / airmass(altitude).max(1.0).powf(0.7)).clamp(0.15, 1.0);
        let sky_factor = sky
            .sqm_mag_arcsec2
            .map(|sqm| ((sqm - 18.0) / 3.5).clamp(0.15, 1.0))
            .unwrap_or(0.7);
        let moon = sample.bodies.get("Moon");
        let (ra_date, dec_date) = precess_j2000(RA_DEG_J2000, DEC_DEG_J2000, sample.time);
        let moon_separation = moon.map(|moon| {
            angular_separation_deg(ra_date, dec_date, moon.ra_hours * 15.0, moon.dec_deg)
        });
        let moon_penalty = moon
            .zip(moon_separation)
            .map(|(moon, separation)| {
                let illumination = moon.illuminated_fraction.unwrap_or(0.5).clamp(0.0, 1.0);
                let altitude_term = if moon.altitude_deg <= -5.0 {
                    0.0
                } else {
                    ((moon.altitude_deg + 5.0) / 60.0).clamp(0.0, 1.0)
                };
                illumination.powf(0.65)
                    * altitude_term
                    * (1.0 - separation / 180.0).clamp(0.08, 1.0)
            })
            .unwrap_or(0.15);
        let score = (condition.deep_sky / 100.0
            * altitude_factor
            * sky_factor
            * (1.0 - moon_penalty * 0.85)
            * 100.0)
            .clamp(0.0, 100.0);
        let note = match sky.sqm_mag_arcsec2 {
            Some(sqm) => format!("Galactic Centre; sky brightness {:.2} mag/arcsec²", sqm),
            None => "Galactic Centre; local sky brightness is unknown".to_string(),
        };
        let recommendation = Recommendation {
            name: "Milky Way — Galactic Centre".to_string(),
            category: "Milky Way".to_string(),
            score,
            best_time: sample.time,
            altitude_deg: altitude,
            azimuth_deg: azimuth,
            magnitude: None,
            moon_separation_deg: moon_separation,
            equipment: "naked eye or wide-field camera".to_string(),
            note,
        };
        if winner
            .as_ref()
            .map(|current| score > current.score)
            .unwrap_or(true)
        {
            winner = Some(recommendation);
        }
    }

    winner
}

pub fn select_diverse(mut candidates: Vec<Recommendation>, limit: usize) -> Vec<Recommendation> {
    candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    let mut selected = Vec::new();
    let mut deferred = Vec::new();
    for r in candidates {
        let count = selected
            .iter()
            .filter(|x: &&Recommendation| x.category == r.category)
            .count();
        let cap = match r.category.as_str() {
            "deep sky" => 5,
            "planet" => 3,
            "comet" => 2,
            "satellite" => 1,
            "meteor shower" => 1,
            "Milky Way" => 1,
            _ => 2,
        };
        if count < cap && selected.len() < limit {
            selected.push(r)
        } else {
            deferred.push(r)
        }
    }
    if selected.len() < limit {
        for r in deferred {
            if selected.len() >= limit {
                break;
            }
            selected.push(r);
        }
    }
    selected.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    selected
}
