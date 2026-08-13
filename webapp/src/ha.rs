use crate::error::AppResult;
use crate::models::Location;

#[derive(Clone)]
pub struct HaClient {
    latitude: f64,
    longitude: f64,
    elevation_m: f64,
    label: String,
    timezone: String,
}

impl HaClient {
    pub fn for_location(value: &Location) -> Self {
        Self {
            latitude: value.latitude,
            longitude: value.longitude,
            elevation_m: value.elevation_m,
            label: value.label.clone(),
            timezone: value.timezone.clone(),
        }
    }

    pub fn location(&self, _person: &str) -> AppResult<Location> {
        Ok(Location {
            latitude: self.latitude,
            longitude: self.longitude,
            elevation_m: self.elevation_m,
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            source: "web_request".to_string(),
        })
    }

    pub fn numeric_state(&self, _entity_id: &str) -> Option<f64> {
        None
    }
}
