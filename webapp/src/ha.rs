use crate::error::AppResult;
use crate::models::Location;

#[derive(Clone)]
pub struct HaClient;

impl HaClient {
    pub fn location(&self, _person: &str) -> AppResult<Location> {
        todo!()
    }

    pub fn numeric_state(&self, _entity_id: &str) -> Option<f64> {
        None
    }
}
