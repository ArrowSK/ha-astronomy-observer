use crate::error::AppResult;
use crate::models::Location;

#[derive(Clone)]
pub struct HaClient {
    location: Location,
}

impl HaClient {
    pub fn for_location(location: &Location) -> Self {
        Self {
            location: location.clone(),
        }
    }

    pub fn location(&self, _person: &str) -> AppResult<Location> {
        let mut value = self.location.clone();
        value.source = "Android device/manual location, local".to_string();
        Ok(value)
    }

    pub fn numeric_state(&self, _entity_id: &str) -> Option<f64> {
        None
    }
}
