use serde::{Deserialize, Serialize};
use crate::models::property::NamedLocation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: i32,
    pub name: String,
    pub preferred_locations: Vec<NamedLocation>,
    pub min_budget: f64,
    pub max_budget: f64,
    pub min_area_sqm: i32,
    pub max_area_sqm: i32,
    pub property_types: Vec<String>,
    pub min_rooms: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactPreferences {
    pub min_budget: f64,
    pub max_budget: f64,
    pub preferred_locations: Vec<NamedLocation>,
    pub property_types: Vec<String>,
    pub min_area_sqm: i32,
    pub max_area_sqm: i32,
    pub min_rooms: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactFilter {
    pub min_budget: Option<f64>,
    pub max_budget: Option<f64>,
    pub property_type: Option<String>,
    pub min_rooms: Option<i32>,
    pub min_area_sqm: Option<i32>,
    pub max_area_sqm: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateContactRequest {
    pub name: String,
    pub preferences: ContactPreferences,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateContactRequest {
    pub name: Option<String>,
    pub preferences: Option<ContactPreferences>,
}

impl Contact {
    pub fn get_all(_connection: &mut crate::utils::database::Connection) -> anyhow::Result<Vec<Contact>> {
        // For now, return empty vector - in real implementation, query from database
        Ok(Vec::new())
    }
}
