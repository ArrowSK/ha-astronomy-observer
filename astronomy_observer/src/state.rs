use crate::error::AppResult;
use crate::ha::HaClient;
use crate::models::{Recommendation, Snapshot};
use chrono::Timelike;
use chrono_tz::Tz;
use serde_json::{json, Value};

fn attrs(name: &str, icon: &str, unit: Option<&str>) -> Value {
    let mut value = json!({"friendly_name": name, "icon": icon});
    if let Some(unit) = unit {
        value["unit_of_measurement"] = json!(unit);
    }
    value
}

fn publish_one(ha: &HaClient, id: &str, state: Value, attributes: Value) {
    if let Err(error) = ha.set_state(id, state, attributes) {
        eprintln!("could not publish {id}: {error}");
    }
}

fn score(ha: &HaClient, id: &str, name: &str, value: f64) {
    publish_one(
        ha,
        id,
        json!(format!("{value:.0}")),
        attrs(name, "mdi:gauge", Some("%")),
    );
}

fn optional_number(
    ha: &HaClient,
    id: &str,
    name: &str,
    icon: &str,
    unit: Option<&str>,
    value: Option<f64>,
    decimals: usize,
) {
    let state = value
        .map(|number| json!(format!("{number:.decimals$}")))
        .unwrap_or_else(|| json!("unknown"));
    publish_one(ha, id, state, attrs(name, icon, unit));
}

fn top_category<'a>(snapshot: &'a Snapshot, category: &str) -> Option<&'a Recommendation> {
    snapshot
        .recommendations
        .iter()
        .find(|recommendation| recommendation.category == category)
}

fn recommendation_attributes(recommendation: &Recommendation) -> Value {
    json!({
        "score": recommendation.score,
        "category": recommendation.category,
        "best_time": recommendation.best_time,
        "altitude_deg": recommendation.altitude_deg,
        "azimuth_deg": recommendation.azimuth_deg,
        "magnitude": recommendation.magnitude,
        "moon_separation_deg": recommendation.moon_separation_deg,
        "equipment": recommendation.equipment,
        "note": recommendation.note,
    })
}

pub fn publish(ha: &HaClient, snapshot: &Snapshot, threshold: f64) -> AppResult<()> {
    score(
        ha,
        "sensor.astronomy_observer_score",
        "Astronomy Observer Score",
        snapshot.conditions.overall,
    );
    score(
        ha,
        "sensor.astronomy_observer_deep_sky",
        "Astronomy Observer Deep Sky",
        snapshot.conditions.deep_sky,
    );
    score(
        ha,
        "sensor.astronomy_observer_planetary",
        "Astronomy Observer Planetary",
        snapshot.conditions.planetary,
    );
    score(
        ha,
        "sensor.astronomy_observer_imaging",
        "Astronomy Observer Imaging",
        snapshot.conditions.imaging,
    );
    score(
        ha,
        "sensor.astronomy_observer_cloud",
        "Astronomy Observer Clear Sky",
        snapshot.conditions.cloud,
    );
    score(
        ha,
        "sensor.astronomy_observer_transparency",
        "Astronomy Observer Transparency",
        snapshot.conditions.transparency,
    );
    score(
        ha,
        "sensor.astronomy_observer_seeing",
        "Astronomy Observer Estimated Seeing",
        snapshot.conditions.seeing_estimate,
    );
    score(
        ha,
        "sensor.astronomy_observer_darkness",
        "Astronomy Observer Darkness",
        snapshot.conditions.darkness,
    );
    score(
        ha,
        "sensor.astronomy_observer_moon_interference",
        "Astronomy Observer Moon Interference",
        snapshot.conditions.moon_interference,
    );
    score(
        ha,
        "sensor.astronomy_observer_wind_score",
        "Astronomy Observer Wind Score",
        snapshot.conditions.wind,
    );
    score(
        ha,
        "sensor.astronomy_observer_dew_score",
        "Astronomy Observer Dew Score",
        snapshot.conditions.dew,
    );
    score(
        ha,
        "sensor.astronomy_observer_confidence",
        "Astronomy Observer Confidence",
        snapshot.conditions.confidence,
    );

    let timezone: Tz = snapshot.location.timezone.parse().unwrap_or(chrono_tz::UTC);
    let window = match (snapshot.best_window_start, snapshot.best_window_end) {
        (Some(start), Some(end)) => format!(
            "{:02}:{:02}–{:02}:{:02}",
            start.with_timezone(&timezone).hour(),
            start.with_timezone(&timezone).minute(),
            end.with_timezone(&timezone).hour(),
            end.with_timezone(&timezone).minute()
        ),
        _ => "none".to_string(),
    };
    let mut window_attributes = attrs("Astronomy Observer Best Window", "mdi:clock-outline", None);
    window_attributes["start_utc"] = json!(snapshot.best_window_start);
    window_attributes["end_utc"] = json!(snapshot.best_window_end);
    window_attributes["timezone"] = json!(snapshot.location.timezone);
    publish_one(
        ha,
        "sensor.astronomy_observer_best_window",
        json!(window),
        window_attributes,
    );

    let context = &snapshot.best_window_context;
    optional_number(
        ha,
        "sensor.astronomy_observer_cloud_cover",
        "Astronomy Observer Cloud Cover",
        "mdi:weather-cloudy",
        Some("%"),
        context.cloud_total_pct,
        0,
    );
    let mut cloud_layers = attrs(
        "Astronomy Observer Cloud Layers",
        "mdi:cloud-outline",
        Some("%"),
    );
    cloud_layers["low_pct"] = json!(context.cloud_low_pct);
    cloud_layers["mid_pct"] = json!(context.cloud_mid_pct);
    cloud_layers["high_pct"] = json!(context.cloud_high_pct);
    publish_one(
        ha,
        "sensor.astronomy_observer_cloud_layers",
        context
            .cloud_high_pct
            .map(|value| json!(format!("{value:.0}")))
            .unwrap_or_else(|| json!("unknown")),
        cloud_layers,
    );
    optional_number(
        ha,
        "sensor.astronomy_observer_visibility",
        "Astronomy Observer Visibility",
        "mdi:eye-outline",
        Some("km"),
        context.visibility_km,
        1,
    );
    optional_number(
        ha,
        "sensor.astronomy_observer_aod",
        "Astronomy Observer Aerosol Optical Depth",
        "mdi:blur",
        None,
        context.aerosol_optical_depth,
        3,
    );
    optional_number(
        ha,
        "sensor.astronomy_observer_wind",
        "Astronomy Observer Wind",
        "mdi:weather-windy",
        Some("km/h"),
        context.wind_speed_kmh,
        1,
    );
    optional_number(
        ha,
        "sensor.astronomy_observer_jet_stream",
        "Astronomy Observer 200 hPa Wind",
        "mdi:weather-windy-variant",
        Some("km/h"),
        context.wind_200hpa_kmh,
        0,
    );
    optional_number(
        ha,
        "sensor.astronomy_observer_dew_margin",
        "Astronomy Observer Dew Margin",
        "mdi:water-thermometer-outline",
        Some("°C"),
        snapshot.conditions.dew_margin_c,
        1,
    );
    optional_number(
        ha,
        "sensor.astronomy_observer_moon_illumination",
        "Astronomy Observer Moon Illumination",
        "mdi:moon-waning-crescent",
        Some("%"),
        context.moon_illumination_pct,
        0,
    );
    optional_number(
        ha,
        "sensor.astronomy_observer_moon_altitude",
        "Astronomy Observer Moon Altitude",
        "mdi:angle-acute",
        Some("°"),
        context.moon_altitude_deg,
        1,
    );
    optional_number(
        ha,
        "sensor.astronomy_observer_sun_altitude",
        "Astronomy Observer Sun Altitude",
        "mdi:white-balance-sunny",
        Some("°"),
        context.sun_altitude_deg,
        1,
    );

    let mut sky = attrs(
        "Astronomy Observer Sky Brightness",
        "mdi:weather-night",
        Some("mag/arcsec²"),
    );
    sky["source"] = json!(snapshot.sky_brightness.source);
    sky["artificial_mcd_m2"] = json!(snapshot.sky_brightness.artificial_mcd_m2);
    sky["nearest_grid_distance_km"] = json!(snapshot.sky_brightness.nearest_distance_km);
    publish_one(
        ha,
        "sensor.astronomy_observer_sky_brightness",
        snapshot
            .sky_brightness
            .sqm_mag_arcsec2
            .map(|value| json!(format!("{value:.2}")))
            .unwrap_or_else(|| json!("unknown")),
        sky,
    );

    let mut top = attrs(
        "Astronomy Observer Top Target",
        "mdi:star-four-points",
        None,
    );
    top["top_10"] = serde_json::to_value(&snapshot.recommendations)?;
    top["location"] = json!(snapshot.location.label);
    top["generated_at"] = json!(snapshot.generated_at);
    publish_one(
        ha,
        "sensor.astronomy_observer_top_target",
        json!(snapshot
            .recommendations
            .first()
            .map(|recommendation| recommendation.name.as_str())
            .unwrap_or("none")),
        top,
    );

    for index in 0..10 {
        let entity_id = format!("sensor.astronomy_observer_target_{}", index + 1);
        let friendly_name = format!("Astronomy Observer Target {}", index + 1);
        if let Some(recommendation) = snapshot.recommendations.get(index) {
            let mut attributes = recommendation_attributes(recommendation);
            attributes["friendly_name"] = json!(friendly_name);
            attributes["icon"] = json!("mdi:star-outline");
            publish_one(ha, &entity_id, json!(recommendation.name), attributes);
        } else {
            publish_one(
                ha,
                &entity_id,
                json!("none"),
                attrs(&friendly_name, "mdi:star-outline", None),
            );
        }
    }

    for (category, id, name, icon) in [
        (
            "meteor shower",
            "sensor.astronomy_observer_meteor_shower",
            "Astronomy Observer Meteor Shower",
            "mdi:meteor",
        ),
        (
            "comet",
            "sensor.astronomy_observer_comet",
            "Astronomy Observer Comet",
            "mdi:creation",
        ),
        (
            "satellite",
            "sensor.astronomy_observer_satellite_pass",
            "Astronomy Observer Satellite Pass",
            "mdi:satellite-variant",
        ),
    ] {
        let recommendation = top_category(snapshot, category);
        let attributes = recommendation
            .map(recommendation_attributes)
            .unwrap_or_else(|| attrs(name, icon, None));
        publish_one(
            ha,
            id,
            json!(recommendation
                .map(|value| value.name.as_str())
                .unwrap_or("none")),
            attributes,
        );
    }

    let mut aurora = attrs("Astronomy Observer Aurora", "mdi:aurora", Some("%"));
    aurora["source"] = json!(snapshot.aurora.source);
    aurora["forecast_time"] = json!(snapshot.aurora.forecast_time);
    aurora["stale"] = json!(snapshot.aurora.stale);
    publish_one(
        ha,
        "sensor.astronomy_observer_aurora",
        snapshot
            .aurora
            .probability_pct
            .map(|value| json!(format!("{value:.0}")))
            .unwrap_or_else(|| json!("unknown")),
        aurora,
    );

    let next_good_night = snapshot
        .outlook
        .iter()
        .find(|night| night.score >= threshold);
    let mut next_attributes = attrs(
        "Astronomy Observer Next Good Night",
        "mdi:calendar-star",
        None,
    );
    next_attributes["outlook"] = serde_json::to_value(&snapshot.outlook)?;
    publish_one(
        ha,
        "sensor.astronomy_observer_next_good_night",
        json!(next_good_night
            .map(|night| night.date.as_str())
            .unwrap_or("none")),
        next_attributes,
    );

    let mut dark_site = attrs(
        "Astronomy Observer Nearby Dark Site",
        "mdi:map-marker-star",
        None,
    );
    let dark_site_state = if let Some(site) = &snapshot.nearby_dark_site {
        dark_site["latitude"] = json!(site.latitude);
        dark_site["longitude"] = json!(site.longitude);
        dark_site["distance_km"] = json!(site.distance_km);
        dark_site["estimated_sqm"] = json!(site.estimated_sqm);
        format!("{:.1} km", site.distance_km)
    } else {
        "none".to_string()
    };
    publish_one(
        ha,
        "sensor.astronomy_observer_nearby_dark_site",
        json!(dark_site_state),
        dark_site,
    );

    let mut source_status = attrs(
        "Astronomy Observer Source Status",
        "mdi:database-check-outline",
        None,
    );
    source_status["sources"] = serde_json::to_value(&snapshot.source_status)?;
    source_status["weather_stale"] = json!(snapshot.weather_stale);
    publish_one(
        ha,
        "sensor.astronomy_observer_source_status",
        json!(if snapshot.weather_stale {
            "degraded"
        } else {
            "ok"
        }),
        source_status,
    );
    publish_one(
        ha,
        "sensor.astronomy_observer_weather_source",
        json!(snapshot.weather_source),
        attrs(
            "Astronomy Observer Weather Source",
            "mdi:weather-partly-cloudy",
            None,
        ),
    );
    publish_one(
        ha,
        "sensor.astronomy_observer_last_update",
        json!(snapshot.generated_at.to_rfc3339()),
        attrs("Astronomy Observer Last Update", "mdi:update", None),
    );
    publish_one(
        ha,
        "binary_sensor.astronomy_observer_good_observing",
        json!(if snapshot.conditions.overall >= threshold {
            "on"
        } else {
            "off"
        }),
        json!({
            "friendly_name": "Astronomy Observer Good Observing",
            "icon": "mdi:telescope"
        }),
    );

    let _ = ha.fire_event(
        "astronomy_observer_updated",
        json!({
            "score": snapshot.conditions.overall,
            "top_target": snapshot.recommendations.first().map(|value| value.name.as_str()),
            "generated_at": snapshot.generated_at
        }),
    );
    Ok(())
}
