use actix_web::{web, HttpResponse, Result};
use crate::services::recommendation::RecommendationService;
use crate::models::*;

pub async fn get_property_recommendations(
    path: web::Path<i32>,
    query: web::Query<RecommendationQuery>,
    service: web::Data<RecommendationService>,
) -> Result<HttpResponse> {
    let property_id = path.into_inner();
    
    // Validate weights if provided
    if let Err(e) = query.validate_weights() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Invalid weights".to_string(),
            message: e,
        }));
    }
    
    let (budget_weight, location_weight, property_type_weight, size_weight) = query.get_weights();
    
    match service.get_recommendations_for_property(
        property_id, 
        query.limit, 
        query.min_score,
        query.top_k,
        query.top_percentile,
        query.score_threshold_percentile,
        budget_weight,
        location_weight,
        property_type_weight,
        size_weight,
    ).await {
        Ok(recommendations) => Ok(HttpResponse::Ok().json(recommendations)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to get recommendations".to_string(),
            message: e.to_string(),
        })),
    }
}

pub async fn get_contact_recommendations(
    path: web::Path<i32>,
    query: web::Query<RecommendationQuery>,
    service: web::Data<RecommendationService>,
) -> Result<HttpResponse> {
    let contact_id = path.into_inner();
    
    // Validate weights if provided
    if let Err(e) = query.validate_weights() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Invalid weights".to_string(),
            message: e,
        }));
    }
    
    let (budget_weight, location_weight, property_type_weight, size_weight) = query.get_weights();
    
    match service.get_recommendations_for_contact(
        contact_id, 
        query.limit, 
        query.min_score,
        query.top_k,
        query.top_percentile,
        query.score_threshold_percentile,
        budget_weight,
        location_weight,
        property_type_weight,
        size_weight,
    ).await {
        Ok(recommendations) => Ok(HttpResponse::Ok().json(recommendations)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to get recommendations".to_string(),
            message: e.to_string(),
        })),
    }
}

pub async fn get_bulk_recommendations(
    request: web::Json<BulkRecommendationRequest>,
    service: web::Data<RecommendationService>,
) -> Result<HttpResponse> {
    let req = request.into_inner();
    
    // Validate weights if provided
    if let Err(e) = req.validate_weights() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Invalid weights".to_string(),
            message: e,
        }));
    }
    
    match service.get_bulk_recommendations(req).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to get bulk recommendations".to_string(),
            message: e.to_string(),
        })),
    }
}

#[derive(serde::Deserialize)]
pub struct RecommendationQuery {
    pub limit: Option<usize>,
    pub min_score: Option<f64>,
    pub top_k: Option<usize>,
    pub top_percentile: Option<f64>, // Top X% of scores (e.g., 0.1 for top 10%)
    pub score_threshold_percentile: Option<f64>, // Only return scores above Xth percentile
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

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/recommendations")
            .route("/property/{property_id}", web::get().to(get_property_recommendations))
            .route("/contact/{contact_id}", web::get().to(get_contact_recommendations))
            .route("/bulk", web::post().to(get_bulk_recommendations))
    );
}
