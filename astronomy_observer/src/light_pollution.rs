use crate::coordinates::haversine_km;
use crate::models::{DarkSite, Location, SkyBrightness};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

// Falchi et al. use 174 µcd/m² as the reference natural zenith luminance.
const NATURAL_MCD_M2: f64 = 0.174;

pub fn artificial_to_sqm(artificial_mcd_m2: f64) -> f64 {
    let total = NATURAL_MCD_M2 + artificial_mcd_m2.max(0.0);
    22.0 - 2.5 * (total / NATURAL_MCD_M2).log10()
}

fn parse_row(line: &str) -> Option<(f64, f64, f64)> {
    let mut parts = line.split(',').map(str::trim);
    let lat = parts.next()?.parse().ok()?;
    let lon = parts.next()?.parse().ok()?;
    let artificial = parts.next()?.parse().ok()?;
    Some((lat, lon, artificial))
}

pub fn lookup(
    location: &Location,
    sqm_override: f64,
    sqm_entity: Option<(&str, f64)>,
    file: &Path,
    dark_radius_km: f64,
) -> (SkyBrightness, Option<DarkSite>) {
    if sqm_override > 0.0 {
        return (
            SkyBrightness {
                sqm_mag_arcsec2: Some(sqm_override),
                artificial_mcd_m2: None,
                source: "configured SQM".to_string(),
                nearest_distance_km: None,
            },
            None,
        );
    }

    if let Some((entity_id, value)) = sqm_entity {
        if (15.0..=23.0).contains(&value) {
            return (
                SkyBrightness {
                    sqm_mag_arcsec2: Some(value),
                    artificial_mcd_m2: None,
                    source: format!("Home Assistant SQM sensor ({entity_id})"),
                    nearest_distance_km: None,
                },
                None,
            );
        }
    }

    let file = match File::open(file) {
        Ok(file) => file,
        Err(_) => {
            return (
                SkyBrightness {
                    source: "unknown".to_string(),
                    ..Default::default()
                },
                None,
            )
        }
    };

    let mut nearest: Option<(f64, f64)> = None;
    let mut darker: Option<DarkSite> = None;

    // The file is streamed rather than loaded into memory. A full local grid can therefore
    // be quite large without raising the steady-state RAM use of the app.
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        if line.trim().is_empty()
            || line.trim_start().starts_with('#')
            || line.to_ascii_lowercase().starts_with("latitude")
        {
            continue;
        }
        let Some((lat, lon, artificial)) = parse_row(&line) else {
            continue;
        };
        if !lat.is_finite() || !lon.is_finite() || !artificial.is_finite() || artificial < 0.0 {
            continue;
        }

        let distance = haversine_km(location.latitude, location.longitude, lat, lon);
        if nearest.map(|current| distance < current.0).unwrap_or(true) {
            nearest = Some((distance, artificial));
        }

        if dark_radius_km > 0.0 && distance <= dark_radius_km {
            let replace = darker
                .as_ref()
                .map(|current| {
                    artificial < current.artificial_mcd_m2 * 0.98
                        || ((artificial - current.artificial_mcd_m2).abs() < 1e-9
                            && distance < current.distance_km)
                })
                .unwrap_or(true);
            if replace {
                darker = Some(DarkSite {
                    latitude: lat,
                    longitude: lon,
                    distance_km: distance,
                    artificial_mcd_m2: artificial,
                    estimated_sqm: artificial_to_sqm(artificial),
                });
            }
        }
    }

    let sky = match nearest {
        Some((distance, artificial)) if distance <= 10.0 => SkyBrightness {
            sqm_mag_arcsec2: Some(artificial_to_sqm(artificial)),
            artificial_mcd_m2: Some(artificial),
            source: "imported Falchi atlas grid".to_string(),
            nearest_distance_km: Some(distance),
        },
        Some((distance, _)) => SkyBrightness {
            source: "light-pollution grid has no nearby point".to_string(),
            nearest_distance_km: Some(distance),
            ..Default::default()
        },
        None => SkyBrightness {
            source: "unknown".to_string(),
            ..Default::default()
        },
    };

    if let (Some(current), Some(site)) = (sky.artificial_mcd_m2, darker.as_ref()) {
        if site.artificial_mcd_m2 >= current * 0.9 {
            darker = None;
        }
    }

    (sky, darker)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn natural_reference_is_22() {
        assert!((artificial_to_sqm(0.0) - 22.0).abs() < 1e-9);
    }

    #[test]
    fn imported_grid_selects_nearest_and_darker_site() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("astronomy-observer-lp-{}.csv", std::process::id()));
        fs::write(
            &path,
            "latitude,longitude,artificial_mcd_m2\n47.500,19.000,2.0\n47.600,19.000,0.4\n",
        )
        .unwrap();
        let location = Location {
            latitude: 47.5,
            longitude: 19.0,
            elevation_m: 100.0,
            label: "test".into(),
            timezone: "UTC".into(),
            source: "test".into(),
        };
        let (sky, dark) = lookup(&location, 0.0, None, &path, 50.0);
        assert!(sky.sqm_mag_arcsec2.is_some());
        assert!(dark.is_some());
        let _ = fs::remove_file(path);
    }
}
