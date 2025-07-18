use serde::{Deserialize, Serialize};
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
    pub score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendationRequest {
    pub contact_id: i32,
    pub limit: Option<usize>,
    pub min_score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkRecommendationRequest {
    pub limit_per_property: Option<usize>,
    pub min_score: Option<f64>,
    pub property_ids: Option<Vec<i32>>,
    pub top_k: Option<usize>,
    pub top_percentile: Option<f64>,
    pub score_threshold_percentile: Option<f64>,
    pub budget_weight: Option<f64>,
    pub location_weight: Option<f64>,
    pub property_type_weight: Option<f64>,
    pub size_weight: Option<f64>,
}

impl BulkRecommendationRequest {
    pub fn get_weights(&self) -> (f64, f64, f64, f64) {
        let budget = self.budget_weight.unwrap_or(0.3);
        let location = self.location_weight.unwrap_or(0.25);
        let property_type = self.property_type_weight.unwrap_or(0.2);
        let size = self.size_weight.unwrap_or(0.25);
        
        (budget, location, property_type, size)
    }
    
    pub fn validate_weights(&self) -> Result<(), String> {
        if let (Some(b), Some(l), Some(p), Some(s)) = (
            self.budget_weight,
            self.location_weight,
            self.property_type_weight,
            self.size_weight,
        ) {
            let sum = b + l + p + s;
            if (sum - 1.0).abs() > 0.001 {
                return Err(format!("Weights must sum to 1.0, got {:.3}", sum));
            }
            if b < 0.0 || l < 0.0 || p < 0.0 || s < 0.0 {
                return Err("All weights must be non-negative".to_string());
            }
        }
        Ok(())
    }
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
    pub property_id: i32,
    pub property_address: String,
    pub recommendations: Vec<Recommendation>,
    pub recommendation_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactRecommendations {
    pub contact_id: i32,
    pub contact_name: String,
    pub recommendations: Vec<Recommendation>,
    pub recommendation_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendationResponse {
    pub recommendations: Vec<Recommendation>,
    pub total_count: usize,
    pub processing_time_ms: u64,
}
