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
    pub detailed_analysis: ComparisonAnalysis,
    pub recommendation: ComparisonRecommendation,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonAnalysis {
    pub price_analysis: PriceAnalysis,
    pub space_analysis: SpaceAnalysis,
    pub location_analysis: LocationAnalysis,
    pub feature_analysis: FeatureAnalysis,
    pub value_analysis: ValueAnalysis,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceAnalysis {
    pub cheaper_property: i32,
    pub price_savings: f64,
    pub affordability_rating: String,
    pub price_per_sqm_comparison: (f64, f64), // (property1_price_per_sqm, property2_price_per_sqm)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpaceAnalysis {
    pub larger_property: i32,
    pub space_advantage: i32,
    pub room_comparison: String,
    pub space_efficiency: (f64, f64), // (property1_efficiency, property2_efficiency)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationAnalysis {
    pub distance_between: f64,
    pub location_similarity: String,
    pub accessibility_notes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureAnalysis {
    pub property_type_match: bool,
    pub feature_advantages: Vec<String>,
    pub common_features: Vec<String>,
    pub unique_features: (Vec<String>, Vec<String>), // (property1_unique, property2_unique)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValueAnalysis {
    pub better_value_property: i32,
    pub value_score: (f64, f64), // (property1_value_score, property2_value_score)
    pub investment_potential: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonRecommendation {
    pub recommended_property: i32,
    pub confidence_score: f64,
    pub key_reasons: Vec<String>,
    pub considerations: Vec<String>,
    pub summary: String,
}
