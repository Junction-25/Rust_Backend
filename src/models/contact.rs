use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::property::{PropertyType, Location};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub budget_min: i64, // In cents
    pub budget_max: i64, // In cents
    pub preferred_locations: Vec<Location>,
    pub preferred_property_types: Vec<PropertyType>,
    pub min_rooms: Option<i32>,
    pub max_rooms: Option<i32>,
    pub min_area: Option<i32>,
    pub max_area: Option<i32>,
    pub required_features: Vec<String>,
    pub preferred_features: Vec<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactPreferences {
    pub budget_min: i64,
    pub budget_max: i64,
    pub preferred_locations: Vec<Location>,
    pub preferred_property_types: Vec<PropertyType>,
    pub min_rooms: Option<i32>,
    pub max_rooms: Option<i32>,
    pub min_area: Option<i32>,
    pub max_area: Option<i32>,
    pub required_features: Vec<String>,
    pub preferred_features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactFilter {
    pub budget_min: Option<i64>,
    pub budget_max: Option<i64>,
    pub property_type: Option<PropertyType>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub min_rooms: Option<i32>,
    pub max_rooms: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateContactRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub preferences: ContactPreferences,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateContactRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub preferences: Option<ContactPreferences>,
    pub notes: Option<String>,
    pub is_active: Option<bool>,
}
