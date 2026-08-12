use crate::error::{err, AppResult};
use crate::models::{HourlyWeather, Location, WeatherSeries};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use ureq::Agent;

const USER_AGENT: &str =
    "AstronomyObserver/0.1 (+https://github.com/ArrowSK/ha-astronomy-observer)";

fn agent() -> Agent {
    let config = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(20)))
        .build();
    config.into()
}

fn rounded(value: f64, precision: u32) -> String {
    format!("{:.*}", precision as usize, value)
}

fn get_json(agent: &Agent, url: &str, user_agent: &str) -> AppResult<Value> {
    Ok(agent
        .get(url)
        .header("User-Agent", user_agent)
        .call()?
        .body_mut()
        .read_json()?)
}

fn parse_open_meteo_time(s: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M")
        .ok()
        .map(|x| x.and_utc())
        .or_else(|| {
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|x| x.with_timezone(&Utc))
        })
}

fn array_value(hourly: &Value, key: &str, index: usize) -> Option<f64> {
    hourly.get(key)?.as_array()?.get(index)?.as_f64()
}

fn fetch_air_quality(
    agent: &Agent,
    lat: &str,
    lon: &str,
    days: usize,
) -> AppResult<AirQualityByTime> {
    let url = format!(
        "https://air-quality-api.open-meteo.com/v1/air-quality?latitude={lat}&longitude={lon}&hourly=aerosol_optical_depth,dust,pm2_5&timezone=UTC&forecast_days={days}"
    );
    let value = get_json(agent, &url, USER_AGENT)?;
    let hourly = value
        .get("hourly")
        .ok_or_else(|| err("Open-Meteo air-quality response has no hourly data"))?;
    let times = hourly
        .get("time")
        .and_then(Value::as_array)
        .ok_or_else(|| err("Open-Meteo air-quality response has no times"))?;
    let mut map = HashMap::new();
    for (i, raw) in times.iter().enumerate() {
        let Some(t) = raw.as_str().and_then(parse_open_meteo_time) else {
            continue;
        };
        map.insert(
            t.timestamp(),
            (
                array_value(hourly, "aerosol_optical_depth", i),
                array_value(hourly, "dust", i),
                array_value(hourly, "pm2_5", i),
            ),
        );
    }
    Ok(map)
}

fn fetch_open_meteo(location: &Location, days: usize, precision: u32) -> AppResult<WeatherSeries> {
    let a = agent();
    let lat = rounded(location.latitude, precision);
    let lon = rounded(location.longitude, precision);
    let variables = concat!(
        "temperature_2m,relative_humidity_2m,dew_point_2m,cloud_cover,",
        "cloud_cover_low,cloud_cover_mid,cloud_cover_high,visibility,",
        "precipitation_probability,wind_speed_10m,wind_gusts_10m,",
        "wind_speed_500hPa,wind_speed_200hPa,temperature_850hPa,temperature_500hPa"
    );
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&hourly={variables}&timezone=UTC&forecast_days={days}&wind_speed_unit=kmh"
    );
    let value = get_json(&a, &url, USER_AGENT)?;
    let hourly = value
        .get("hourly")
        .ok_or_else(|| err("Open-Meteo response has no hourly data"))?;
    let times = hourly
        .get("time")
        .and_then(Value::as_array)
        .ok_or_else(|| err("Open-Meteo response has no times"))?;
    let air = fetch_air_quality(&a, &lat, &lon, days).unwrap_or_default();
    let mut hours = Vec::with_capacity(times.len());
    for (i, raw) in times.iter().enumerate() {
        let Some(time) = raw.as_str().and_then(parse_open_meteo_time) else {
            continue;
        };
        let (aod, dust, pm25) = air
            .get(&time.timestamp())
            .copied()
            .unwrap_or((None, None, None));
        hours.push(HourlyWeather {
            time,
            temperature_c: array_value(hourly, "temperature_2m", i),
            relative_humidity_pct: array_value(hourly, "relative_humidity_2m", i),
            dew_point_c: array_value(hourly, "dew_point_2m", i),
            cloud_total_pct: array_value(hourly, "cloud_cover", i),
            cloud_low_pct: array_value(hourly, "cloud_cover_low", i),
            cloud_mid_pct: array_value(hourly, "cloud_cover_mid", i),
            cloud_high_pct: array_value(hourly, "cloud_cover_high", i),
            visibility_km: array_value(hourly, "visibility", i).map(|x| x / 1000.0),
            precipitation_probability_pct: array_value(hourly, "precipitation_probability", i),
            wind_speed_kmh: array_value(hourly, "wind_speed_10m", i),
            wind_gust_kmh: array_value(hourly, "wind_gusts_10m", i),
            wind_500hpa_kmh: array_value(hourly, "wind_speed_500hPa", i),
            wind_200hpa_kmh: array_value(hourly, "wind_speed_200hPa", i),
            temperature_850hpa_c: array_value(hourly, "temperature_850hPa", i),
            temperature_500hpa_c: array_value(hourly, "temperature_500hPa", i),
            aerosol_optical_depth: aod,
            dust_ug_m3: dust,
            pm25_ug_m3: pm25,
        });
    }
    if hours.len() < 12 {
        return Err(err("Open-Meteo returned too few hourly records"));
    }
    Ok(WeatherSeries {
        source: "Open-Meteo".to_string(),
        retrieved_at: Utc::now(),
        stale: false,
        hours,
    })
}

fn detail_number(details: &Value, key: &str) -> Option<f64> {
    details.get(key).and_then(Value::as_f64)
}

fn fetch_met_norway(location: &Location, precision: u32) -> AppResult<WeatherSeries> {
    let a = agent();
    let lat = rounded(location.latitude, precision);
    let lon = rounded(location.longitude, precision);
    let url = format!("https://api.met.no/weatherapi/locationforecast/2.0/compact?lat={lat}&lon={lon}&altitude={:.0}", location.elevation_m);
    let value = get_json(&a, &url, USER_AGENT)?;
    let series = value
        .pointer("/properties/timeseries")
        .and_then(Value::as_array)
        .ok_or_else(|| err("MET Norway response has no timeseries"))?;
    let mut hours = Vec::with_capacity(series.len());
    for item in series {
        let Some(time) = item
            .get("time")
            .and_then(Value::as_str)
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|t| t.with_timezone(&Utc))
        else {
            continue;
        };
        let details = item
            .pointer("/data/instant/details")
            .unwrap_or(&Value::Null);
        let precip = item
            .pointer("/data/next_1_hours/details/probability_of_precipitation")
            .and_then(Value::as_f64);
        hours.push(HourlyWeather {
            time,
            temperature_c: detail_number(details, "air_temperature"),
            relative_humidity_pct: detail_number(details, "relative_humidity"),
            dew_point_c: detail_number(details, "dew_point_temperature"),
            cloud_total_pct: detail_number(details, "cloud_area_fraction"),
            cloud_low_pct: detail_number(details, "cloud_area_fraction_low"),
            cloud_mid_pct: detail_number(details, "cloud_area_fraction_medium"),
            cloud_high_pct: detail_number(details, "cloud_area_fraction_high"),
            visibility_km: None,
            precipitation_probability_pct: precip,
            wind_speed_kmh: detail_number(details, "wind_speed").map(|x| x * 3.6),
            wind_gust_kmh: detail_number(details, "wind_speed_of_gust").map(|x| x * 3.6),
            ..Default::default()
        });
    }
    if hours.len() < 12 {
        return Err(err("MET Norway returned too few hourly records"));
    }
    Ok(WeatherSeries {
        source: "MET Norway".to_string(),
        retrieved_at: Utc::now(),
        stale: false,
        hours,
    })
}

fn save_cache(path: &Path, series: &WeatherSeries) {
    if let Ok(text) = serde_json::to_string(series) {
        let tmp = path.with_extension("tmp");
        if fs::write(&tmp, text).is_ok() {
            let _ = fs::rename(tmp, path);
        }
    }
}

fn load_cache(path: &Path) -> Option<WeatherSeries> {
    let text = fs::read_to_string(path).ok()?;
    let mut series: WeatherSeries = serde_json::from_str(&text).ok()?;
    if Utc::now()
        .signed_duration_since(series.retrieved_at)
        .num_hours()
        > 12
    {
        return None;
    }
    series.stale = true;
    series.source = format!("{} cache", series.source);
    Some(series)
}

pub fn fetch(
    location: &Location,
    days: usize,
    precision: u32,
    data_dir: &Path,
) -> AppResult<WeatherSeries> {
    let cache = data_dir.join("weather_cache.json");
    match fetch_open_meteo(location, days, precision) {
        Ok(series) => {
            save_cache(&cache, &series);
            return Ok(series);
        }
        Err(e) => eprintln!("Open-Meteo unavailable: {e}"),
    }
    match fetch_met_norway(location, precision) {
        Ok(series) => {
            save_cache(&cache, &series);
            return Ok(series);
        }
        Err(e) => eprintln!("MET Norway unavailable: {e}"),
    }
    load_cache(&cache)
        .ok_or_else(|| err("weather providers failed and no recent cache is available"))
}

pub fn nearest(series: &WeatherSeries, time: DateTime<Utc>) -> Option<&HourlyWeather> {
    series
        .hours
        .iter()
        .min_by_key(|h| (h.time.timestamp() - time.timestamp()).abs())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_open_meteo_time_without_zone() {
        assert_eq!(
            parse_open_meteo_time("2026-08-12T15:00")
                .unwrap()
                .timestamp(),
            1786546800
        );
    }
}
