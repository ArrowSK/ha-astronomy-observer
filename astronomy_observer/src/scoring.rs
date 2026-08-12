use crate::astro;
use crate::models::{
    AstronomySample, ConditionScore, HourlyWeather, NightOutlook, SkyBrightness, WeatherSeries,
};
use chrono::{DateTime, Duration, Timelike, Utc};
use chrono_tz::Tz;
use std::collections::BTreeMap;

fn clamp100(x: f64) -> f64 {
    x.clamp(0.0, 100.0)
}
fn f01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

fn geometric(parts: &[(f64, f64)]) -> f64 {
    let mut sum_w = 0.0;
    let mut log_sum = 0.0;
    for &(value, weight) in parts {
        if weight <= 0.0 {
            continue;
        }
        sum_w += weight;
        if value <= 0.0 {
            return 0.0;
        }
        log_sum += weight * value.max(0.001).ln();
    }
    if sum_w == 0.0 {
        0.0
    } else {
        (log_sum / sum_w).exp()
    }
}

fn cloud_factor(w: &HourlyWeather) -> (f64, f64) {
    let total = w.cloud_total_pct.or_else(|| {
        let vals = [w.cloud_low_pct, w.cloud_mid_pct, w.cloud_high_pct];
        let mut m: f64 = 0.0;
        let mut any = false;
        for x in vals.into_iter().flatten() {
            m = m.max(x);
            any = true;
        }
        any.then_some(m)
    });
    let Some(total) = total else {
        return (0.55, 0.25);
    };
    let clear = f01(1.0 - total / 100.0).powf(1.45);
    let cirrus = w
        .cloud_high_pct
        .map(|h| f01(1.0 - h / 100.0).powf(0.25))
        .unwrap_or(0.9);
    let low = w
        .cloud_low_pct
        .map(|h| f01(1.0 - h / 100.0).powf(0.15))
        .unwrap_or(0.95);
    (
        f01(clear * cirrus * low),
        if w.cloud_high_pct.is_some() && w.cloud_low_pct.is_some() {
            1.0
        } else {
            0.75
        },
    )
}

fn dew_factor(w: &HourlyWeather) -> (f64, Option<f64>, f64) {
    let margin = match (w.temperature_c, w.dew_point_c) {
        (Some(t), Some(d)) => Some(t - d),
        _ => None,
    };
    let factor = margin
        .map(|m| {
            if m >= 6.0 {
                1.0
            } else if m >= 3.0 {
                0.7 + (m - 3.0) * 0.1
            } else if m >= 1.0 {
                0.35 + (m - 1.0) * 0.175
            } else {
                0.15 + m.max(0.0) * 0.2
            }
        })
        .unwrap_or_else(|| {
            w.relative_humidity_pct
                .map(|rh| f01((105.0 - rh) / 35.0))
                .unwrap_or(0.6)
        });
    (
        f01(factor),
        margin,
        if margin.is_some() {
            1.0
        } else if w.relative_humidity_pct.is_some() {
            0.65
        } else {
            0.2
        },
    )
}

fn transparency_factor(w: &HourlyWeather, dew: f64) -> (f64, f64) {
    let mut parts = Vec::new();
    let mut quality: f64 = 0.0;
    let mut qweight: f64 = 0.0;
    if let Some(v) = w.visibility_km {
        parts.push((f01((v - 3.0) / 27.0), 0.35));
        quality += 0.35;
        qweight += 0.35;
    }
    if let Some(aod) = w.aerosol_optical_depth {
        parts.push(((-3.0 * aod.max(0.0)).exp().clamp(0.05, 1.0), 0.35));
        quality += 0.35;
        qweight += 0.35;
    }
    if let Some(rh) = w.relative_humidity_pct {
        parts.push((f01((105.0 - rh) / 35.0), 0.2));
        quality += 0.2;
        qweight += 0.2;
    }
    if let Some(pm) = w.pm25_ug_m3 {
        parts.push(((-pm.max(0.0) / 45.0).exp().clamp(0.1, 1.0), 0.1));
        quality += 0.1;
        qweight += 0.1;
    }
    parts.push((dew, 0.15));
    let weighted = if parts.is_empty() {
        0.55
    } else {
        let sumw: f64 = parts.iter().map(|x| x.1).sum();
        parts.iter().map(|x| x.0 * x.1).sum::<f64>() / sumw
    };
    let confidence = if qweight > 0.0 {
        (quality / 1.0).min(1.0)
    } else {
        0.2
    };
    (f01(weighted), confidence)
}

fn seeing_factor(w: &HourlyWeather) -> (f64, f64) {
    let mut score: f64 = 1.0;
    let mut known: f64 = 0.0;
    if let Some(v) = w.wind_200hpa_kmh {
        score *= if v <= 35.0 {
            1.0
        } else if v <= 70.0 {
            1.0 - (v - 35.0) / 100.0
        } else {
            0.65 - (v - 70.0) / 200.0
        }
        .clamp(0.25, 1.0);
        known += 0.45;
    }
    if let Some(v) = w.wind_500hpa_kmh {
        score *= if v <= 25.0 {
            1.0
        } else if v <= 60.0 {
            1.0 - (v - 25.0) / 120.0
        } else {
            0.7 - (v - 60.0) / 220.0
        }
        .clamp(0.3, 1.0);
        known += 0.35;
    }
    if let Some(v) = w.wind_speed_kmh {
        let surface = if v < 2.0 {
            0.82
        } else if v <= 15.0 {
            1.0
        } else if v <= 30.0 {
            1.0 - (v - 15.0) / 45.0
        } else {
            0.67 - (v - 30.0) / 90.0
        };
        score *= surface.clamp(0.3, 1.0);
        known += 0.2;
    }
    if known == 0.0 {
        (0.55, 0.15)
    } else {
        (
            score.powf(1.0 / known.max(0.1)).clamp(0.1, 1.0),
            known.min(1.0),
        )
    }
}

fn wind_factor(w: &HourlyWeather) -> (f64, f64) {
    match w.wind_speed_kmh {
        Some(v) if v <= 8.0 => (1.0, 1.0),
        Some(v) if v <= 20.0 => (1.0 - (v - 8.0) / 30.0, 1.0),
        Some(v) if v <= 40.0 => (0.6 - (v - 20.0) / 50.0, 1.0),
        Some(_) => (0.2, 1.0),
        None => (0.6, 0.2),
    }
}

fn darkness_factor(sun_alt: f64, sky: &SkyBrightness) -> (f64, f64) {
    let astro_dark = if sun_alt >= -6.0 {
        0.0
    } else if sun_alt >= -12.0 {
        (sun_alt + 6.0).abs() / 6.0 * 0.35
    } else if sun_alt >= -18.0 {
        0.35 + (sun_alt + 12.0).abs() / 6.0 * 0.65
    } else {
        1.0
    };
    let lp = sky
        .sqm_mag_arcsec2
        .map(|sqm| f01((sqm - 17.0) / 4.7).max(0.12));
    (
        astro_dark * lp.map(|x| 0.35 + 0.65 * x).unwrap_or(0.75),
        if lp.is_some() { 1.0 } else { 0.45 },
    )
}

fn moon_interference(sample: &AstronomySample) -> (f64, f64) {
    let Some(moon) = sample.bodies.get("Moon") else {
        return (0.25, 0.2);
    };
    let illum = moon.illuminated_fraction.unwrap_or(0.5).clamp(0.0, 1.0);
    let alt_factor = if moon.altitude_deg <= -5.0 {
        0.0
    } else {
        ((moon.altitude_deg + 5.0) / 55.0).clamp(0.0, 1.0).sqrt()
    };
    (
        f01(illum.powf(0.7) * alt_factor),
        if moon.illuminated_fraction.is_some() {
            1.0
        } else {
            0.65
        },
    )
}

pub fn score_hour(
    w: &HourlyWeather,
    sample: &AstronomySample,
    sky: &SkyBrightness,
) -> ConditionScore {
    let (cloud, cq) = cloud_factor(w);
    let (dew, margin, dq) = dew_factor(w);
    let (trans, tq) = transparency_factor(w, dew);
    let (seeing, sq) = seeing_factor(w);
    let (wind, wq) = wind_factor(w);
    let sun_alt = sample
        .bodies
        .get("Sun")
        .map(|x| x.altitude_deg)
        .unwrap_or(0.0);
    let (dark, darkq) = darkness_factor(sun_alt, sky);
    let (moon_penalty, mq) = moon_interference(sample);
    let moon_good = 1.0 - moon_penalty;
    let overall = geometric(&[
        (cloud, 0.35),
        (trans, 0.22),
        (dark, 0.18),
        (moon_good.max(0.05), 0.10),
        (wind, 0.07),
        (dew, 0.08),
    ]);
    let deep = geometric(&[
        (cloud, 0.34),
        (trans, 0.27),
        (dark, 0.24),
        (moon_good.max(0.03), 0.10),
        (wind, 0.05),
    ]);
    let planetary = geometric(&[
        (cloud, 0.40),
        (seeing, 0.32),
        (trans.max(0.25), 0.10),
        (wind, 0.12),
        (dark.max(0.25), 0.06),
    ]);
    let imaging = geometric(&[
        (cloud, 0.40),
        (trans, 0.22),
        (seeing, 0.13),
        (dark, 0.16),
        (wind, 0.09),
    ]);
    let confidence =
        (cq * 0.20 + tq * 0.25 + sq * 0.15 + darkq * 0.15 + mq * 0.10 + wq * 0.075 + dq * 0.075)
            .clamp(0.0, 1.0);
    ConditionScore {
        overall: clamp100(overall * 100.0),
        cloud: clamp100(cloud * 100.0),
        transparency: clamp100(trans * 100.0),
        seeing_estimate: clamp100(seeing * 100.0),
        darkness: clamp100(dark * 100.0),
        moon_interference: clamp100(moon_penalty * 100.0),
        wind: clamp100(wind * 100.0),
        dew: clamp100(dew * 100.0),
        deep_sky: clamp100(deep * 100.0),
        planetary: clamp100(planetary * 100.0),
        imaging: clamp100(imaging * 100.0),
        confidence: clamp100(confidence * 100.0),
        dew_margin_c: margin,
    }
}

pub fn hourly_scores(
    weather: &WeatherSeries,
    samples: &[AstronomySample],
    sky: &SkyBrightness,
) -> Vec<(DateTime<Utc>, ConditionScore)> {
    weather
        .hours
        .iter()
        .filter_map(|w| {
            let s = astro::sample_nearest(samples, w.time)?;
            if (s.time.timestamp() - w.time.timestamp()).abs() > 2100 {
                return None;
            }
            Some((w.time, score_hour(w, s, sky)))
        })
        .collect()
}

pub fn best_window(
    scores: &[(DateTime<Utc>, ConditionScore)],
    samples: &[AstronomySample],
    now: DateTime<Utc>,
    hours_ahead: usize,
) -> Option<(DateTime<Utc>, DateTime<Utc>, ConditionScore)> {
    let end = now + Duration::hours(hours_ahead as i64);
    let candidates: Vec<_> = scores
        .iter()
        .filter(|(t, _)| *t >= now - Duration::minutes(30) && *t <= end)
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let mut best: Option<(f64, usize)> = None;
    for i in 0..candidates.len() {
        let t0 = candidates[i].0;
        let vals: Vec<_> = candidates
            .iter()
            .skip(i)
            .take_while(|(t, _)| *t <= t0 + Duration::hours(2))
            .collect();
        if vals.len() < 2 {
            continue;
        }
        let sun_dark = astro::sample_nearest(samples, t0)
            .and_then(|s| s.bodies.get("Sun"))
            .map(|s| s.altitude_deg < -6.0)
            .unwrap_or(false);
        if !sun_dark {
            continue;
        }
        let avg = vals.iter().map(|x| x.1.overall).sum::<f64>() / vals.len() as f64;
        if best.map(|b| avg > b.0).unwrap_or(true) {
            best = Some((avg, i));
        }
    }
    let (_, i) = best?;
    let start = candidates[i].0;
    let end = (start + Duration::hours(2)).min(end);
    let members: Vec<&ConditionScore> = candidates
        .iter()
        .filter(|(t, _)| *t >= start && *t <= end)
        .map(|x| &x.1)
        .collect();
    let avg_field = |f: fn(&ConditionScore) -> f64| {
        members.iter().map(|s| f(s)).sum::<f64>() / members.len() as f64
    };
    Some((
        start,
        end,
        ConditionScore {
            overall: avg_field(|s| s.overall),
            cloud: avg_field(|s| s.cloud),
            transparency: avg_field(|s| s.transparency),
            seeing_estimate: avg_field(|s| s.seeing_estimate),
            darkness: avg_field(|s| s.darkness),
            moon_interference: avg_field(|s| s.moon_interference),
            wind: avg_field(|s| s.wind),
            dew: avg_field(|s| s.dew),
            deep_sky: avg_field(|s| s.deep_sky),
            planetary: avg_field(|s| s.planetary),
            imaging: avg_field(|s| s.imaging),
            confidence: avg_field(|s| s.confidence),
            dew_margin_c: members
                .iter()
                .filter_map(|s| s.dew_margin_c)
                .reduce(f64::min),
        },
    ))
}

pub fn outlook(
    scores: &[(DateTime<Utc>, ConditionScore)],
    samples: &[AstronomySample],
    timezone: &str,
    days: usize,
) -> Vec<NightOutlook> {
    let tz: Tz = timezone.parse().unwrap_or(chrono_tz::UTC);
    let mut grouped: BTreeMap<String, Vec<(DateTime<Utc>, f64)>> = BTreeMap::new();
    for (t, s) in scores {
        let sun = astro::sample_nearest(samples, *t)
            .and_then(|a| a.bodies.get("Sun"))
            .map(|b| b.altitude_deg)
            .unwrap_or(90.0);
        if sun > -12.0 {
            continue;
        }
        let local = t.with_timezone(&tz);
        let date = if local.hour() < 12 {
            local.date_naive() - Duration::days(1)
        } else {
            local.date_naive()
        };
        grouped
            .entry(date.to_string())
            .or_default()
            .push((*t, s.overall));
    }
    grouped
        .into_iter()
        .take(days)
        .filter_map(|(date, vals)| {
            vals.into_iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .map(|(best_time, score)| NightOutlook {
                    date,
                    score,
                    best_time,
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AstronomySample, BodyPosition};
    use chrono::TimeZone;
    use std::collections::HashMap;
    #[test]
    fn overcast_is_bad() {
        let t = Utc.with_ymd_and_hms(2026, 8, 12, 22, 0, 0).unwrap();
        let w = HourlyWeather {
            time: t,
            temperature_c: Some(15.0),
            dew_point_c: Some(10.0),
            cloud_total_pct: Some(100.0),
            cloud_high_pct: Some(100.0),
            wind_speed_kmh: Some(5.0),
            ..Default::default()
        };
        let mut bodies = HashMap::new();
        bodies.insert(
            "Sun".to_string(),
            BodyPosition {
                altitude_deg: -25.0,
                ..Default::default()
            },
        );
        bodies.insert(
            "Moon".to_string(),
            BodyPosition {
                altitude_deg: -20.0,
                illuminated_fraction: Some(0.0),
                ..Default::default()
            },
        );
        let a = AstronomySample {
            time: t,
            bodies,
            ..Default::default()
        };
        let s = score_hour(
            &w,
            &a,
            &SkyBrightness {
                sqm_mag_arcsec2: Some(21.5),
                ..Default::default()
            },
        );
        assert!(s.overall < 5.0);
    }
}
