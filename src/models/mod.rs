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
}