use crate::error::{err, AppResult};
use crate::models::Location;
use serde_json::{json, Value};
use std::time::Duration;
use ureq::Agent;

const CORE_API: &str = "http://supervisor/core/api";

#[derive(Clone)]
pub struct HaClient {
    token: String,
    agent: Agent,
}

impl HaClient {
    pub fn new() -> AppResult<Self> {
        let token = std::env::var("SUPERVISOR_TOKEN")
            .map_err(|_| err("SUPERVISOR_TOKEN is not available"))?;
        let config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build();
        Ok(Self {
            token,
            agent: config.into(),
        })
    }

    fn auth(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn get_json(&self, path: &str) -> AppResult<Value> {
        let url = format!("{CORE_API}{path}");
        Ok(self
            .agent
            .get(&url)
            .header("Authorization", &self.auth())
            .header("Content-Type", "application/json")
            .call()?
            .body_mut()
            .read_json()?)
    }

    pub fn people(&self) -> AppResult<Vec<Value>> {
        let states = self.get_json("/states")?;
        let mut people = Vec::new();
        for state in states.as_array().cloned().unwrap_or_default() {
            let Some(entity_id) = state.get("entity_id").and_then(Value::as_str) else {
                continue;
            };
            if !entity_id.starts_with("person.") {
                continue;
            }
            let attributes = state.get("attributes").unwrap_or(&Value::Null);
            let name = attributes
                .get("friendly_name")
                .and_then(Value::as_str)
                .unwrap_or(entity_id);
            let has_location = attributes.get("latitude").and_then(Value::as_f64).is_some()
                && attributes
                    .get("longitude")
                    .and_then(Value::as_f64)
                    .is_some();
            people.push(json!({
                "entity_id": entity_id,
                "name": name,
                "has_location": has_location
            }));
        }
        people.sort_by(|a, b| {
            let left = a
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_ascii_lowercase();
            let right = b
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_ascii_lowercase();
            left.cmp(&right)
        });
        Ok(people)
    }

    pub fn location(&self, person: &str) -> AppResult<Location> {
        let cfg = self.get_json("/config")?;
        let home_lat = cfg
            .get("latitude")
            .and_then(Value::as_f64)
            .ok_or_else(|| err("Home Assistant config has no latitude"))?;
        let home_lon = cfg
            .get("longitude")
            .and_then(Value::as_f64)
            .ok_or_else(|| err("Home Assistant config has no longitude"))?;
        let elevation = cfg.get("elevation").and_then(Value::as_f64).unwrap_or(0.0);
        let timezone = cfg
            .get("time_zone")
            .and_then(Value::as_str)
            .unwrap_or("UTC")
            .to_string();

        if !person.trim().is_empty() {
            let entity = person.trim();
            if !entity.starts_with("person.") {
                return Err(err("primary_person must be a person.* entity"));
            }
            if let Ok(state) = self.get_json(&format!("/states/{entity}")) {
                let attrs = state.get("attributes").unwrap_or(&Value::Null);
                if let (Some(lat), Some(lon)) = (
                    attrs.get("latitude").and_then(Value::as_f64),
                    attrs.get("longitude").and_then(Value::as_f64),
                ) {
                    let label = attrs
                        .get("friendly_name")
                        .and_then(Value::as_str)
                        .unwrap_or(entity)
                        .to_string();
                    return Ok(Location {
                        latitude: lat,
                        longitude: lon,
                        elevation_m: elevation,
                        label,
                        timezone,
                        source: entity.to_string(),
                    });
                }
            }
        }

        Ok(Location {
            latitude: home_lat,
            longitude: home_lon,
            elevation_m: elevation,
            label: "Home".to_string(),
            timezone,
            source: "homeassistant_config".to_string(),
        })
    }

    pub fn numeric_state(&self, entity_id: &str) -> Option<f64> {
        if entity_id.trim().is_empty() {
            return None;
        }
        let value = self
            .get_json(&format!("/states/{}", entity_id.trim()))
            .ok()?;
        let state = value.get("state")?.as_str()?;
        let number = state.parse::<f64>().ok()?;
        number.is_finite().then_some(number)
    }

    pub fn set_state(&self, entity_id: &str, state: Value, attributes: Value) -> AppResult<()> {
        let url = format!("{CORE_API}/states/{entity_id}");
        let body = json!({"state": state, "attributes": attributes});
        self.agent
            .post(&url)
            .header("Authorization", &self.auth())
            .header("Content-Type", "application/json")
            .send_json(&body)?;
        Ok(())
    }

    pub fn fire_event(&self, event_type: &str, data: Value) -> AppResult<()> {
        let url = format!("{CORE_API}/events/{event_type}");
        self.agent
            .post(&url)
            .header("Authorization", &self.auth())
            .header("Content-Type", "application/json")
            .send_json(&data)?;
        Ok(())
    }
}
