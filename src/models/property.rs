use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub property_type: PropertyType,
    pub price: i64, // Price in cents to avoid floating point issues
    pub location: Location,
    pub area_sqm: i32,
    pub rooms: i32,
    pub bathrooms: i32,
    pub features: Vec<String>,
    pub images: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PropertyType {
    Apartment,
    House,
    Condo,
    Townhouse,
    Villa,
    Studio,
    Commercial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub address: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub postal_code: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyFilter {
    pub property_type: Option<PropertyType>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub min_area: Option<i32>,
    pub max_area: Option<i32>,
    pub min_rooms: Option<i32>,
    pub max_rooms: Option<i32>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub features: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyComparison {
    pub property1: Property,
    pub property2: Property,
    pub comparison_metrics: ComparisonMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonMetrics {
    pub price_difference: i64,
    pub price_difference_percentage: f64,
    pub area_difference: i32,
    pub area_difference_percentage: f64,
    pub location_distance_km: f64,
    pub feature_similarity_score: f64,
    pub overall_similarity_score: f64,
}
