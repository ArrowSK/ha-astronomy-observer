use crate::error::{err, AppResult};
use chrono::{DateTime, Datelike, Timelike, Utc};
use std::f64::consts::PI;

pub fn deg_to_rad(x: f64) -> f64 {
    x * PI / 180.0
}
pub fn rad_to_deg(x: f64) -> f64 {
    x * 180.0 / PI
}
pub fn norm_deg(x: f64) -> f64 {
    ((x % 360.0) + 360.0) % 360.0
}

pub fn julian_date(t: DateTime<Utc>) -> f64 {
    let mut y = t.year();
    let mut m = t.month() as i32;
    let d = t.day() as f64
        + (t.hour() as f64
            + (t.minute() as f64 + (t.second() as f64 + t.nanosecond() as f64 / 1e9) / 60.0)
                / 60.0)
            / 24.0;
    if m <= 2 {
        y -= 1;
        m += 12;
    }
    let a = (y as f64 / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (y as f64 + 4716.0)).floor() + (30.6001 * (m as f64 + 1.0)).floor() + d + b - 1524.5
}

pub fn gmst_deg(t: DateTime<Utc>) -> f64 {
    let jd = julian_date(t);
    let tt = (jd - 2451545.0) / 36525.0;
    norm_deg(
        280.46061837 + 360.98564736629 * (jd - 2451545.0) + 0.000387933 * tt * tt
            - tt * tt * tt / 38710000.0,
    )
}

pub fn precess_j2000(ra_deg: f64, dec_deg: f64, t: DateTime<Utc>) -> (f64, f64) {
    let centuries = (julian_date(t) - 2451545.0) / 36525.0;
    let zeta = deg_to_rad(
        (2306.2181 * centuries + 0.30188 * centuries.powi(2) + 0.017998 * centuries.powi(3))
            / 3600.0,
    );
    let z = deg_to_rad(
        (2306.2181 * centuries + 1.09468 * centuries.powi(2) + 0.018203 * centuries.powi(3))
            / 3600.0,
    );
    let theta = deg_to_rad(
        (2004.3109 * centuries - 0.42665 * centuries.powi(2) - 0.041833 * centuries.powi(3))
            / 3600.0,
    );
    let ra = deg_to_rad(ra_deg);
    let dec = deg_to_rad(dec_deg);
    let a = dec.cos() * (ra + zeta).sin();
    let b = theta.cos() * dec.cos() * (ra + zeta).cos() - theta.sin() * dec.sin();
    let c = theta.sin() * dec.cos() * (ra + zeta).cos() + theta.cos() * dec.sin();
    (
        norm_deg(rad_to_deg(a.atan2(b)) + rad_to_deg(z)),
        rad_to_deg(c.clamp(-1.0, 1.0).asin()),
    )
}

pub fn alt_az_from_j2000(
    ra_deg: f64,
    dec_deg: f64,
    lat_deg: f64,
    lon_deg: f64,
    t: DateTime<Utc>,
) -> (f64, f64) {
    let (ra_date, dec_date) = precess_j2000(ra_deg, dec_deg, t);
    alt_az_of_date(ra_date, dec_date, lat_deg, lon_deg, t)
}

pub fn alt_az_of_date(
    ra_deg: f64,
    dec_deg: f64,
    lat_deg: f64,
    lon_deg: f64,
    t: DateTime<Utc>,
) -> (f64, f64) {
    let lat = deg_to_rad(lat_deg);
    let dec = deg_to_rad(dec_deg);
    let ha = deg_to_rad(norm_deg(gmst_deg(t) + lon_deg - ra_deg));
    let sin_alt = dec.sin() * lat.sin() + dec.cos() * lat.cos() * ha.cos();
    let alt = sin_alt.clamp(-1.0, 1.0).asin();
    let y = -ha.sin() * dec.cos();
    let x = dec.sin() * lat.cos() - dec.cos() * lat.sin() * ha.cos();
    let az = norm_deg(rad_to_deg(y.atan2(x)));
    (rad_to_deg(alt), az)
}

pub fn angular_separation_deg(ra1_deg: f64, dec1_deg: f64, ra2_deg: f64, dec2_deg: f64) -> f64 {
    let r1 = deg_to_rad(ra1_deg);
    let d1 = deg_to_rad(dec1_deg);
    let r2 = deg_to_rad(ra2_deg);
    let d2 = deg_to_rad(dec2_deg);
    let cos_sep = d1.sin() * d2.sin() + d1.cos() * d2.cos() * (r1 - r2).cos();
    rad_to_deg(cos_sep.clamp(-1.0, 1.0).acos())
}

pub fn airmass(altitude_deg: f64) -> f64 {
    if altitude_deg <= -5.0 {
        return 99.0;
    }
    let s = deg_to_rad(altitude_deg.max(0.0)).sin();
    1.0 / (s + 0.50572 * (altitude_deg.max(0.0) + 6.07995).powf(-1.6364))
}

pub fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0088;
    let dlat = deg_to_rad(lat2 - lat1);
    let dlon = deg_to_rad(lon2 - lon1);
    let a = (dlat / 2.0).sin().powi(2)
        + deg_to_rad(lat1).cos() * deg_to_rad(lat2).cos() * (dlon / 2.0).sin().powi(2);
    2.0 * r * a.sqrt().asin()
}

#[derive(Debug, Clone)]
pub struct HorizonMask {
    points: Vec<(f64, f64)>,
}
impl HorizonMask {
    pub fn parse(text: &str) -> AppResult<Self> {
        let mut points = Vec::new();
        for part in text.split(',').filter(|p| !p.trim().is_empty()) {
            let mut it = part.trim().split(':');
            let az: f64 = it
                .next()
                .ok_or_else(|| err("invalid horizon mask"))?
                .trim()
                .parse()?;
            let alt: f64 = it
                .next()
                .ok_or_else(|| err("invalid horizon mask"))?
                .trim()
                .parse()?;
            if it.next().is_some() || !(0.0..=360.0).contains(&az) || !(-10.0..=90.0).contains(&alt)
            {
                return Err(err(format!("invalid horizon point: {part}")));
            }
            points.push((norm_deg(az), alt));
        }
        if points.is_empty() {
            points.push((0.0, 0.0));
        }
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        points.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-9);
        Ok(Self { points })
    }
    pub fn altitude_at(&self, azimuth_deg: f64) -> f64 {
        if self.points.len() == 1 {
            return self.points[0].1;
        }
        let az = norm_deg(azimuth_deg);
        for i in 0..self.points.len() {
            let (a1, h1) = self.points[i];
            let (mut a2, h2) = self.points[(i + 1) % self.points.len()];
            let mut x = az;
            if i == self.points.len() - 1 {
                a2 += 360.0;
                if x < a1 {
                    x += 360.0;
                }
            }
            if x >= a1 && x <= a2 {
                let f = if (a2 - a1).abs() < 1e-9 {
                    0.0
                } else {
                    (x - a1) / (a2 - a1)
                };
                return h1 + f * (h2 - h1);
            }
        }
        self.points[0].1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    #[test]
    fn horizon_wraps() {
        let h = HorizonMask::parse("0:10,90:20,180:10,270:0").unwrap();
        assert!((h.altitude_at(45.0) - 15.0).abs() < 1e-8);
        assert!((h.altitude_at(315.0) - 5.0).abs() < 1e-8);
    }
    #[test]
    fn gmst_known_range() {
        let t = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        assert!((gmst_deg(t) - 280.46061837).abs() < 0.001);
    }
}
