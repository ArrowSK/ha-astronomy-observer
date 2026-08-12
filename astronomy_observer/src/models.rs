use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation_m: f64,
    pub label: String,
    pub timezone: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HourlyWeather {
    pub time: DateTime<Utc>,
    pub temperature_c: Option<f64>,
    pub relative_humidity_pct: Option<f64>,
    pub dew_point_c: Option<f64>,
    pub cloud_total_pct: Option<f64>,
    pub cloud_low_pct: Option<f64>,
    pub cloud_mid_pct: Option<f64>,
    pub cloud_high_pct: Option<f64>,
    pub visibility_km: Option<f64>,
    pub precipitation_probability_pct: Option<f64>,
    pub wind_speed_kmh: Option<f64>,
    pub wind_gust_kmh: Option<f64>,
    pub wind_500hpa_kmh: Option<f64>,
    pub wind_200hpa_kmh: Option<f64>,
    pub temperature_850hpa_c: Option<f64>,
    pub temperature_500hpa_c: Option<f64>,
    pub aerosol_optical_depth: Option<f64>,
    pub dust_ug_m3: Option<f64>,
    pub pm25_ug_m3: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherSeries {
    pub source: String,
    pub retrieved_at: DateTime<Utc>,
    pub stale: bool,
    pub hours: Vec<HourlyWeather>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BodyPosition {
    pub ra_hours: f64,
    pub dec_deg: f64,
    pub azimuth_deg: f64,
    pub altitude_deg: f64,
    pub magnitude: Option<f64>,
    pub illuminated_fraction: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AstronomySample {
    pub time: DateTime<Utc>,
    pub earth_ecliptic_au: [f64; 3],
    pub bodies: HashMap<String, BodyPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkyBrightness {
    pub sqm_mag_arcsec2: Option<f64>,
    pub artificial_mcd_m2: Option<f64>,
    pub source: String,
    pub nearest_distance_km: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarkSite {
    pub latitude: f64,
    pub longitude: f64,
    pub distance_km: f64,
    pub artificial_mcd_m2: f64,
    pub estimated_sqm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConditionScore {
    pub overall: f64,
    pub cloud: f64,
    pub transparency: f64,
    pub seeing_estimate: f64,
    pub darkness: f64,
    pub moon_interference: f64,
    pub wind: f64,
    pub dew: f64,
    pub deep_sky: f64,
    pub planetary: f64,
    pub imaging: f64,
    pub confidence: f64,
    pub dew_margin_c: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowContext {
    pub time: Option<DateTime<Utc>>,
    pub temperature_c: Option<f64>,
    pub relative_humidity_pct: Option<f64>,
    pub dew_point_c: Option<f64>,
    pub cloud_total_pct: Option<f64>,
    pub cloud_low_pct: Option<f64>,
    pub cloud_mid_pct: Option<f64>,
    pub cloud_high_pct: Option<f64>,
    pub visibility_km: Option<f64>,
    pub precipitation_probability_pct: Option<f64>,
    pub wind_speed_kmh: Option<f64>,
    pub wind_gust_kmh: Option<f64>,
    pub wind_500hpa_kmh: Option<f64>,
    pub wind_200hpa_kmh: Option<f64>,
    pub aerosol_optical_depth: Option<f64>,
    pub dust_ug_m3: Option<f64>,
    pub pm25_ug_m3: Option<f64>,
    pub sun_altitude_deg: Option<f64>,
    pub moon_altitude_deg: Option<f64>,
    pub moon_azimuth_deg: Option<f64>,
    pub moon_illumination_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub name: String,
    pub category: String,
    pub score: f64,
    pub best_time: DateTime<Utc>,
    pub altitude_deg: f64,
    pub azimuth_deg: f64,
    pub magnitude: Option<f64>,
    pub moon_separation_deg: Option<f64>,
    pub equipment: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NightOutlook {
    pub date: String,
    pub score: f64,
    pub best_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuroraStatus {
    pub probability_pct: Option<f64>,
    pub forecast_time: Option<String>,
    pub source: String,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub generated_at: DateTime<Utc>,
    pub location: Location,
    pub conditions: ConditionScore,
    pub best_window_start: Option<DateTime<Utc>>,
    pub best_window_end: Option<DateTime<Utc>>,
    pub best_window_context: WindowContext,
    pub sky_brightness: SkyBrightness,
    pub nearby_dark_site: Option<DarkSite>,
    pub weather_source: String,
    pub weather_stale: bool,
    pub recommendations: Vec<Recommendation>,
    pub outlook: Vec<NightOutlook>,
    pub aurora: AuroraStatus,
    pub source_status: HashMap<String, String>,
}
