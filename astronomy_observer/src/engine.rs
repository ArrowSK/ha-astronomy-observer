use crate::config::AppConfig;
use crate::error::{err, AppResult};
use crate::ha::HaClient;
use crate::models::{AstronomySample, HourlyWeather, Recommendation, Snapshot, WindowContext};
use crate::{
    astro, aurora, comets, light_pollution, meteors, satellites, scoring, targets, weather,
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use std::collections::HashMap;
use std::path::Path;

fn timeline(now: DateTime<Utc>, days: usize) -> Vec<DateTime<Utc>> {
    let start = Utc
        .timestamp_opt((now.timestamp() / 1800) * 1800, 0)
        .single()
        .unwrap_or(now);
    let end = now + Duration::days(days as i64);
    let mut time = start;
    let mut out = Vec::new();
    while time <= end {
        out.push(time);
        time += Duration::minutes(30);
    }
    out
}

fn aurora_recommendation(
    snapshot_score: f64,
    aurora: &crate::models::AuroraStatus,
    samples: &[AstronomySample],
    now: DateTime<Utc>,
) -> Option<Recommendation> {
    let probability = aurora.probability_pct?;
    if probability < 2.0 {
        return None;
    }
    let sample = samples
        .iter()
        .filter(|sample| sample.time >= now)
        .find(|sample| {
            sample
                .bodies
                .get("Sun")
                .map(|sun| sun.altitude_deg < -8.0)
                .unwrap_or(false)
        })?;
    let score = (probability * 0.7 + snapshot_score * 0.3).clamp(0.0, 100.0);
    Some(Recommendation {
        name: "Aurora".to_string(),
        category: "aurora".to_string(),
        score,
        best_time: sample.time,
        altitude_deg: 0.0,
        azimuth_deg: 0.0,
        magnitude: None,
        moon_separation_deg: None,
        equipment: "naked eye or camera".to_string(),
        note: format!("NOAA OVATION local probability {:.0}%", probability),
    })
}

fn window_context(
    time: Option<DateTime<Utc>>,
    weather_series: &crate::models::WeatherSeries,
    samples: &[AstronomySample],
) -> WindowContext {
    let Some(time) = time else {
        return WindowContext::default();
    };
    let weather: Option<&HourlyWeather> = weather::nearest(weather_series, time);
    let sample = astro::sample_nearest(samples, time);
    let sun = sample.and_then(|sample| sample.bodies.get("Sun"));
    let moon = sample.and_then(|sample| sample.bodies.get("Moon"));

    WindowContext {
        time: Some(time),
        temperature_c: weather.and_then(|value| value.temperature_c),
        relative_humidity_pct: weather.and_then(|value| value.relative_humidity_pct),
        dew_point_c: weather.and_then(|value| value.dew_point_c),
        cloud_total_pct: weather.and_then(|value| value.cloud_total_pct),
        cloud_low_pct: weather.and_then(|value| value.cloud_low_pct),
        cloud_mid_pct: weather.and_then(|value| value.cloud_mid_pct),
        cloud_high_pct: weather.and_then(|value| value.cloud_high_pct),
        visibility_km: weather.and_then(|value| value.visibility_km),
        precipitation_probability_pct: weather
            .and_then(|value| value.precipitation_probability_pct),
        wind_speed_kmh: weather.and_then(|value| value.wind_speed_kmh),
        wind_gust_kmh: weather.and_then(|value| value.wind_gust_kmh),
        wind_500hpa_kmh: weather.and_then(|value| value.wind_500hpa_kmh),
        wind_200hpa_kmh: weather.and_then(|value| value.wind_200hpa_kmh),
        aerosol_optical_depth: weather.and_then(|value| value.aerosol_optical_depth),
        dust_ug_m3: weather.and_then(|value| value.dust_ug_m3),
        pm25_ug_m3: weather.and_then(|value| value.pm25_ug_m3),
        sun_altitude_deg: sun.map(|value| value.altitude_deg),
        moon_altitude_deg: moon.map(|value| value.altitude_deg),
        moon_azimuth_deg: moon.map(|value| value.azimuth_deg),
        moon_illumination_pct: moon
            .and_then(|value| value.illuminated_fraction)
            .map(|value| value * 100.0),
    }
}

pub fn refresh(cfg: &AppConfig, ha: &HaClient) -> AppResult<Snapshot> {
    let now = Utc::now();
    let location = ha.location(&cfg.options.primary_person)?;
    let weather_series = weather::fetch(
        &location,
        cfg.options.forecast_days,
        cfg.options.external_location_precision,
        &cfg.data_dir,
    )?;

    let sqm_entity_value = if cfg.options.sqm_entity.trim().is_empty() {
        None
    } else {
        ha.numeric_state(&cfg.options.sqm_entity)
            .map(|value| (cfg.options.sqm_entity.as_str(), value))
    };
    let light_pollution_path = cfg.config_dir.join(&cfg.options.light_pollution_file);
    let (sky_brightness, nearby_dark_site) = light_pollution::lookup(
        &location,
        cfg.options.sqm_override,
        sqm_entity_value,
        &light_pollution_path,
        cfg.options.nearby_dark_site_radius_km,
    );

    let times = timeline(now, cfg.options.forecast_days);
    let samples = astro::calculate(&location, &times)?;
    if samples.is_empty() {
        return Err(err("astronomy engine returned no samples"));
    }

    let hourly = scoring::hourly_scores(&weather_series, &samples, &sky_brightness);
    let (best_window_start, best_window_end, conditions) =
        match scoring::best_window(&hourly, &samples, now, cfg.options.observing_window_hours) {
            Some((start, end, conditions)) => (Some(start), Some(end), conditions),
            None => {
                let conditions = hourly
                    .first()
                    .map(|entry| entry.1.clone())
                    .unwrap_or_default();
                (None, None, conditions)
            }
        };
    let outlook = scoring::outlook(
        &hourly,
        &samples,
        &location.timezone,
        cfg.options.forecast_days,
    );
    let best_window_context = window_context(best_window_start, &weather_series, &samples);

    let catalog = Path::new("/usr/share/astronomy-observer/catalog.tsv");
    let meteor_showers = Path::new("/usr/share/astronomy-observer/meteor_showers.csv");
    let mut candidates = Vec::new();
    candidates.extend(targets::deep_sky(
        cfg,
        catalog,
        &weather_series,
        &samples,
        &sky_brightness,
        now,
        location.latitude,
        location.longitude,
    )?);
    candidates.extend(targets::solar_system(
        cfg,
        &weather_series,
        &samples,
        &sky_brightness,
        now,
    ));
    if let Some(milky_way) = targets::milky_way(
        cfg,
        &weather_series,
        &samples,
        &sky_brightness,
        now,
        location.latitude,
        location.longitude,
    ) {
        candidates.push(milky_way);
    }
    candidates.extend(meteors::recommendations(
        cfg,
        meteor_showers,
        &weather_series,
        &samples,
        &sky_brightness,
        now,
        location.latitude,
        location.longitude,
    ));

    let (comet_recommendations, comet_source) = comets::recommendations(
        cfg,
        location.latitude,
        location.longitude,
        &weather_series,
        &samples,
        &sky_brightness,
        now,
        &cfg.data_dir,
    );
    candidates.extend(comet_recommendations);

    let (satellite_recommendations, satellite_source) = satellites::recommendations(
        cfg,
        &location,
        &weather_series,
        &samples,
        &sky_brightness,
        now,
        &cfg.data_dir,
    );
    candidates.extend(satellite_recommendations);

    let aurora = aurora::fetch(&location, &cfg.data_dir, cfg.options.enable_aurora);
    if let Some(recommendation) = aurora_recommendation(conditions.overall, &aurora, &samples, now)
    {
        candidates.push(recommendation);
    }
    let recommendations = targets::select_diverse(candidates, 10);

    let mut source_status = HashMap::new();
    source_status.insert("location".to_string(), location.source.clone());
    source_status.insert("weather".to_string(), weather_series.source.clone());
    source_status.insert(
        "astronomy".to_string(),
        "Astronomy Engine, local".to_string(),
    );
    source_status.insert(
        "deep_sky".to_string(),
        "OpenNGC pinned build catalogue".to_string(),
    );
    source_status.insert("light_pollution".to_string(), sky_brightness.source.clone());
    source_status.insert("comets".to_string(), comet_source);
    source_status.insert("satellites".to_string(), satellite_source);
    source_status.insert("aurora".to_string(), aurora.source.clone());

    Ok(Snapshot {
        generated_at: now,
        location,
        conditions,
        best_window_start,
        best_window_end,
        best_window_context,
        sky_brightness,
        nearby_dark_site,
        weather_source: weather_series.source.clone(),
        weather_stale: weather_series.stale,
        recommendations,
        outlook,
        aurora,
        source_status,
    })
}
