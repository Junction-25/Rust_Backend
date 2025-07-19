use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub id: i32,
    pub address: String,
    pub location: Location,
    pub price: f64,
    pub area_sqm: i32,
    pub property_type: String,
    pub number_of_rooms: i32,
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
    Office,
    Land,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedLocation {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyFilter {
    pub property_type: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub min_area: Option<i32>,
    pub max_area: Option<i32>,
    pub min_rooms: Option<i32>,
    pub max_rooms: Option<i32>,
    pub address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyComparison {
    pub property1: Property,
    pub property2: Property,
    pub comparison_metrics: ComparisonMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonMetrics {
    pub price_difference: f64,
    pub price_difference_percentage: f64,
    pub area_difference: i32,
    pub area_difference_percentage: f64,
    pub location_distance_km: f64,
    pub overall_similarity_score: f64,
}

impl Property {
    pub fn get_all(_connection: &mut crate::utils::database::Connection) -> anyhow::Result<Vec<Property>> {
        // For now, return empty vector - in real implementation, query from database
        Ok(Vec::new())
    }
}
