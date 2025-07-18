use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::{contact::Contact, property::Property};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub contact: Contact,
    pub property: Property,
    pub score: f64,
    pub explanation: RecommendationExplanation,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationExplanation {
    pub overall_score: f64,
    pub budget_match: BudgetMatch,
    pub location_match: LocationMatch,
    pub property_type_match: bool,
    pub size_match: SizeMatch,
    pub feature_match: FeatureMatch,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetMatch {
    pub is_within_budget: bool,
    pub budget_utilization: f64, // Percentage of budget used
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationMatch {
    pub distance_km: f64,
    pub is_preferred_location: bool,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeMatch {
    pub rooms_match: bool,
    pub area_match: bool,
    pub rooms_score: f64,
    pub area_score: f64,
    pub overall_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureMatch {
    pub required_features_met: bool,
    pub preferred_features_count: i32,
    pub total_preferred_features: i32,
    pub score: f64,
    pub missing_required_features: Vec<String>,
    pub matched_preferred_features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendationRequest {
    pub property_id: Uuid,
    pub limit: Option<usize>,
    pub min_score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkRecommendationRequest {
    pub limit_per_property: Option<usize>,
    pub min_score: Option<f64>,
    pub property_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkRecommendationResponse {
    pub recommendations: Vec<PropertyRecommendations>,
    pub total_properties: usize,
    pub total_recommendations: usize,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyRecommendations {
    pub property_id: Uuid,
    pub property_title: String,
    pub recommendations: Vec<Recommendation>,
    pub recommendation_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendationResponse {
    pub recommendations: Vec<Recommendation>,
    pub total_count: usize,
    pub processing_time_ms: u64,
}
