use crate::coordinates::haversine_km;
use crate::models::{DarkSite, Location, SkyBrightness};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

// Falchi et al. use 174 µcd/m² as the reference natural zenith luminance.
const NATURAL_MCD_M2: f64 = 0.174;
const ATLAS_MAGIC: &[u8; 8] = b"AOATLS1\0";
const ATLAS_HEADER_BYTES: u64 = 64;
const ATLAS_NODATA: u16 = u16::MAX;
const BUNDLED_ATLAS_PATH: &str = "/usr/share/astronomy-observer/world_atlas_3min.bin";

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

fn csv_lookup(
    location: &Location,
    file: &Path,
    dark_radius_km: f64,
) -> (SkyBrightness, Option<DarkSite>) {
    let file = match File::open(file) {
        Ok(file) => file,
        Err(_) => return (SkyBrightness::default(), None),
    };

    let mut nearest: Option<(f64, f64)> = None;
    let mut darker: Option<DarkSite> = None;

    // The optional CSV is streamed rather than loaded into memory. It remains useful for
    // people who want to supply a higher-resolution local grid than the bundled atlas.
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
            source: "imported light-pollution grid".to_string(),
            nearest_distance_km: Some(distance),
        },
        Some((distance, _)) => SkyBrightness {
            source: "light-pollution grid has no nearby point".to_string(),
            nearest_distance_km: Some(distance),
            ..Default::default()
        },
        None => SkyBrightness::default(),
    };

    if let (Some(current), Some(site)) = (sky.artificial_mcd_m2, darker.as_ref()) {
        if site.artificial_mcd_m2 >= current * 0.9 {
            darker = None;
        }
    }

    (sky, darker)
}

#[derive(Debug)]
struct BinaryAtlas {
    file: File,
    width: usize,
    height: usize,
    west: f64,
    north: f64,
    cell_lon: f64,
    cell_lat: f64,
    log_scale: f64,
    radiance_floor: f64,
}

fn read_u32(bytes: &[u8], start: usize) -> u32 {
    u32::from_le_bytes(
        bytes[start..start + 4]
            .try_into()
            .expect("fixed atlas header field"),
    )
}

fn read_f64(bytes: &[u8], start: usize) -> f64 {
    f64::from_le_bytes(
        bytes[start..start + 8]
            .try_into()
            .expect("fixed atlas header field"),
    )
}

impl BinaryAtlas {
    fn open(path: &Path) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut header = [0_u8; ATLAS_HEADER_BYTES as usize];
        file.read_exact(&mut header)?;
        if &header[0..8] != ATLAS_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unrecognised light-pollution atlas format",
            ));
        }

        let width = read_u32(&header, 8) as usize;
        let height = read_u32(&header, 12) as usize;
        let west = read_f64(&header, 16);
        let north = read_f64(&header, 24);
        let cell_lon = read_f64(&header, 32);
        let cell_lat = read_f64(&header, 40);
        let log_scale = read_f64(&header, 48);
        let radiance_floor = read_f64(&header, 56);

        if width == 0
            || height == 0
            || width > 100_000
            || height > 100_000
            || !west.is_finite()
            || !north.is_finite()
            || !cell_lon.is_finite()
            || !cell_lat.is_finite()
            || !log_scale.is_finite()
            || !radiance_floor.is_finite()
            || cell_lon <= 0.0
            || cell_lat <= 0.0
            || log_scale <= 0.0
            || radiance_floor <= 0.0
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid light-pollution atlas header",
            ));
        }

        let expected_size = ATLAS_HEADER_BYTES + width as u64 * height as u64 * 2;
        if file.metadata()?.len() != expected_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "light-pollution atlas size does not match its header",
            ));
        }

        Ok(Self {
            file,
            width,
            height,
            west,
            north,
            cell_lon,
            cell_lat,
            log_scale,
            radiance_floor,
        })
    }

    fn south(&self) -> f64 {
        self.north - self.cell_lat * self.height as f64
    }

    fn longitude_span(&self) -> f64 {
        self.cell_lon * self.width as f64
    }

    fn normalise_longitude(&self, longitude: f64) -> f64 {
        let span = self.longitude_span();
        if span > 359.0 {
            self.west + (longitude - self.west).rem_euclid(span)
        } else {
            longitude
        }
    }

    fn cell_for(&self, latitude: f64, longitude: f64) -> Option<(usize, usize)> {
        if !latitude.is_finite() || !longitude.is_finite() {
            return None;
        }
        let south = self.south();
        if latitude > self.north || latitude < south {
            return None;
        }

        let longitude = self.normalise_longitude(longitude);
        let east = self.west + self.longitude_span();
        if longitude < self.west || longitude >= east {
            return None;
        }

        let row_float = ((self.north - latitude) / self.cell_lat).floor();
        let col_float = ((longitude - self.west) / self.cell_lon).floor();
        let row = (row_float as isize).clamp(0, self.height as isize - 1) as usize;
        let col = (col_float as isize).clamp(0, self.width as isize - 1) as usize;
        Some((row, col))
    }

    fn cell_centre(&self, row: usize, col: usize) -> (f64, f64) {
        let latitude = self.north - (row as f64 + 0.5) * self.cell_lat;
        let mut longitude = self.west + (col as f64 + 0.5) * self.cell_lon;
        if longitude > 180.0 {
            longitude -= 360.0;
        } else if longitude < -180.0 {
            longitude += 360.0;
        }
        (latitude, longitude)
    }

    fn decode(&self, code: u16) -> Option<f64> {
        if code == ATLAS_NODATA {
            return None;
        }
        Some(self.radiance_floor * (10.0_f64.powf(code as f64 / self.log_scale) - 1.0))
    }

    fn read_cell(&mut self, row: usize, col: usize) -> io::Result<Option<f64>> {
        let index = row as u64 * self.width as u64 + col as u64;
        self.file
            .seek(SeekFrom::Start(ATLAS_HEADER_BYTES + index * 2))?;
        let mut bytes = [0_u8; 2];
        self.file.read_exact(&mut bytes)?;
        Ok(self.decode(u16::from_le_bytes(bytes)))
    }

    fn read_row(&mut self, row: usize, buffer: &mut [u8]) -> io::Result<()> {
        let expected = self.width * 2;
        if buffer.len() != expected {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "incorrect atlas row buffer size",
            ));
        }
        let offset = ATLAS_HEADER_BYTES + row as u64 * self.width as u64 * 2;
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(buffer)
    }

    fn darker_site(
        &mut self,
        location: &Location,
        radius_km: f64,
        current_artificial: f64,
    ) -> io::Result<Option<DarkSite>> {
        if radius_km <= 0.0 {
            return Ok(None);
        }
        let Some((base_row, base_col)) = self.cell_for(location.latitude, location.longitude)
        else {
            return Ok(None);
        };

        let max_rows = ((radius_km / (111.32 * self.cell_lat)).ceil() as isize + 1)
            .min(self.height as isize - 1)
            .max(0);
        let longitude_km_per_degree = 111.32 * location.latitude.to_radians().cos().abs().max(0.05);
        let max_cols = ((radius_km / (longitude_km_per_degree * self.cell_lon)).ceil() as isize
            + 2)
        .min((self.width as isize - 1) / 2)
        .max(0);

        let start_row = (base_row as isize - max_rows).max(0) as usize;
        let end_row = (base_row as isize + max_rows).min(self.height as isize - 1) as usize;
        let mut row_bytes = vec![0_u8; self.width * 2];
        let mut darker: Option<DarkSite> = None;
        let wraps_longitude = self.longitude_span() > 359.0;

        for row in start_row..=end_row {
            self.read_row(row, &mut row_bytes)?;
            for delta in -max_cols..=max_cols {
                let raw_col = base_col as isize + delta;
                let col = if wraps_longitude {
                    raw_col.rem_euclid(self.width as isize) as usize
                } else if raw_col < 0 || raw_col >= self.width as isize {
                    continue;
                } else {
                    raw_col as usize
                };

                let byte_index = col * 2;
                let code = u16::from_le_bytes([row_bytes[byte_index], row_bytes[byte_index + 1]]);
                let Some(artificial) = self.decode(code) else {
                    continue;
                };
                let (latitude, longitude) = self.cell_centre(row, col);
                let distance =
                    haversine_km(location.latitude, location.longitude, latitude, longitude);
                if distance > radius_km {
                    continue;
                }

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
                        latitude,
                        longitude,
                        distance_km: distance,
                        artificial_mcd_m2: artificial,
                        estimated_sqm: artificial_to_sqm(artificial),
                    });
                }
            }
        }

        if darker
            .as_ref()
            .is_some_and(|site| site.artificial_mcd_m2 >= current_artificial * 0.9)
        {
            darker = None;
        }
        Ok(darker)
    }
}

fn binary_lookup(
    location: &Location,
    file: &Path,
    dark_radius_km: f64,
) -> (SkyBrightness, Option<DarkSite>) {
    let mut atlas = match BinaryAtlas::open(file) {
        Ok(atlas) => atlas,
        Err(_) => return (SkyBrightness::default(), None),
    };
    let Some((row, col)) = atlas.cell_for(location.latitude, location.longitude) else {
        return (
            SkyBrightness {
                source: "bundled World Atlas has no coverage at this latitude".to_string(),
                ..Default::default()
            },
            None,
        );
    };
    let artificial = match atlas.read_cell(row, col) {
        Ok(Some(value)) => value,
        _ => {
            return (
                SkyBrightness {
                    source: "bundled World Atlas has no value at this location".to_string(),
                    ..Default::default()
                },
                None,
            )
        }
    };
    let (cell_latitude, cell_longitude) = atlas.cell_centre(row, col);
    let distance = haversine_km(
        location.latitude,
        location.longitude,
        cell_latitude,
        cell_longitude,
    );
    let darker = atlas
        .darker_site(location, dark_radius_km, artificial)
        .unwrap_or(None);

    (
        SkyBrightness {
            sqm_mag_arcsec2: Some(artificial_to_sqm(artificial)),
            artificial_mcd_m2: Some(artificial),
            source: "bundled World Atlas 2015 location estimate (~3 arcmin)".to_string(),
            nearest_distance_km: Some(distance),
        },
        darker,
    )
}

pub fn lookup(
    location: &Location,
    sqm_override: f64,
    sqm_entity: Option<(&str, f64)>,
    custom_file: &Path,
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

    let (custom_sky, custom_dark) = csv_lookup(location, custom_file, dark_radius_km);
    if custom_sky.sqm_mag_arcsec2.is_some() {
        return (custom_sky, custom_dark);
    }

    let (bundled_sky, bundled_dark) =
        binary_lookup(location, Path::new(BUNDLED_ATLAS_PATH), dark_radius_km);
    if bundled_sky.sqm_mag_arcsec2.is_some() {
        return (bundled_sky, bundled_dark);
    }

    if !custom_sky.source.is_empty() {
        return (custom_sky, custom_dark);
    }
    if !bundled_sky.source.is_empty() {
        return (bundled_sky, bundled_dark);
    }

    (
        SkyBrightness {
            source: "unknown".to_string(),
            ..Default::default()
        },
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn encode_for_test(value: f64, scale: f64, floor: f64) -> u16 {
        ((1.0 + value / floor).log10() * scale)
            .round()
            .clamp(0.0, (ATLAS_NODATA - 1) as f64) as u16
    }

    fn write_test_atlas(path: &Path) {
        let width = 4_u32;
        let height = 3_u32;
        let west = 0.0_f64;
        let north = 3.0_f64;
        let cell_lon = 1.0_f64;
        let cell_lat = 1.0_f64;
        let scale = 8000.0_f64;
        let floor = 0.0001_f64;
        let values = [2.0, 0.2, 3.0, 4.0, 2.5, 1.0, 3.5, 4.5, 3.0, 2.0, 4.0, 5.0];

        let mut file = File::create(path).unwrap();
        file.write_all(ATLAS_MAGIC).unwrap();
        file.write_all(&width.to_le_bytes()).unwrap();
        file.write_all(&height.to_le_bytes()).unwrap();
        for value in [west, north, cell_lon, cell_lat, scale, floor] {
            file.write_all(&value.to_le_bytes()).unwrap();
        }
        for value in values {
            file.write_all(&encode_for_test(value, scale, floor).to_le_bytes())
                .unwrap();
        }
    }

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
        let (sky, dark) = csv_lookup(&location, &path, 50.0);
        assert!(sky.sqm_mag_arcsec2.is_some());
        assert!(dark.is_some());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn binary_atlas_uses_location_and_finds_darker_cell() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "astronomy-observer-atlas-{}.bin",
            std::process::id()
        ));
        write_test_atlas(&path);
        let location = Location {
            latitude: 2.5,
            longitude: 0.5,
            elevation_m: 0.0,
            label: "test".into(),
            timezone: "UTC".into(),
            source: "test".into(),
        };
        let (sky, dark) = binary_lookup(&location, &path, 150.0);
        assert!(sky.sqm_mag_arcsec2.is_some());
        assert!(sky.artificial_mcd_m2.unwrap() > 1.9);
        let dark = dark.expect("expected a darker nearby cell");
        assert!(dark.artificial_mcd_m2 < 0.3);
        assert!(dark.distance_km < 150.0);
        let _ = fs::remove_file(path);
    }
}
