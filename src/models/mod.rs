pub mod property;
pub mod contact;
pub mod recommendation;

pub use property::*;
pub use contact::*;
pub use recommendation::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RecommendationQuery {
    pub limit: Option<usize>,
    pub min_score: Option<f64>,
    pub budget_weight: Option<f64>,
    pub location_weight: Option<f64>,
    pub property_type_weight: Option<f64>,
    pub size_weight: Option<f64>,
}

impl RecommendationQuery {
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