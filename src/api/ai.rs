use actix_web::{web, HttpResponse, Result};
use crate::services::ai_recommendations::{AIRecommendationService, AIRecommendationRequest};
use crate::models::ErrorResponse;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AIRecommendationQuery {
    pub user_id: String,
    pub limit: Option<usize>,
    pub algorithm: Option<String>,
    pub personalization_level: Option<f64>,
    pub enable_ml_scoring: Option<bool>,
    pub enable_market_analysis: Option<bool>,
    pub enable_predictive_matching: Option<bool>,
    pub include_price_predictions: Option<bool>,
    pub min_confidence: Option<f64>,
}

/// Get AI-enhanced recommendations for a user
pub async fn get_ai_recommendations(
    query: web::Query<AIRecommendationQuery>,
    service: web::Data<AIRecommendationService>,
) -> Result<HttpResponse> {
    // For now, we'll use user_id as contact_id (convert string to int)
    let contact_id: i32 = query.user_id.parse().unwrap_or(1);
    
    let request = AIRecommendationRequest {
        contact_id,
        property_ids: None,
        enable_ml_scoring: query.enable_ml_scoring.unwrap_or(true),
        enable_market_analysis: query.enable_market_analysis.unwrap_or(true),
        enable_predictive_matching: query.enable_predictive_matching.unwrap_or(true),
        include_price_predictions: query.include_price_predictions.unwrap_or(true),
        min_confidence: query.min_confidence,
    };

    match service.get_ai_recommendations(request).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => {
            log::error!("AI recommendation error: {}", e);
            Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to get AI recommendations".to_string(),
                message: e.to_string(),
            }))
        }
    }
}

/// Initialize AI models
pub async fn initialize_ai_models(
    service: web::Data<AIRecommendationService>,
) -> Result<HttpResponse> {
    match service.initialize_models().await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "AI models initialized successfully"
        }))),
        Err(e) => {
            log::error!("AI initialization error: {}", e);
            Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to initialize AI models".to_string(),
                message: e.to_string(),
            }))
        }
    }
}

/// Get AI model statistics
pub async fn get_ai_model_stats(
    service: web::Data<AIRecommendationService>,
) -> Result<HttpResponse> {
    let stats = service.get_model_stats().await;
    Ok(HttpResponse::Ok().json(stats))
}

#[derive(Deserialize)]
pub struct FeedbackRequest {
    pub contact_id: i32,
    pub property_id: i32,
    pub feedback_type: String, // "view", "interest", "contact", etc.
    pub outcome: String,       // "positive", "negative", "neutral"
}

/// Update AI models with user feedback
pub async fn update_ai_with_feedback(
    request: web::Json<FeedbackRequest>,
    service: web::Data<AIRecommendationService>,
) -> Result<HttpResponse> {
    let req = request.into_inner();
    
    match service.update_with_feedback(
        req.contact_id,
        req.property_id,
        &req.feedback_type,
        &req.outcome,
    ).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "AI models updated with feedback"
        }))),
        Err(e) => {
            log::error!("AI feedback error: {}", e);
            Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to update AI models".to_string(),
                message: e.to_string(),
            }))
        }
    }
}

/// Get comprehensive market analysis
pub async fn get_market_analysis(
    service: web::Data<AIRecommendationService>,
) -> Result<HttpResponse> {
    let insights = service.generate_market_insights().await;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "market_insights": insights,
        "generated_at": chrono::Utc::now(),
        "model_version": "v1.0.0-hackathon"
    })))
}

/// Configure AI API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ai")
            .route("/recommendations", web::get().to(get_ai_recommendations))
            .route("/initialize", web::post().to(initialize_ai_models))
            .route("/stats", web::get().to(get_ai_model_stats))
            .route("/feedback", web::post().to(update_ai_with_feedback))
            .route("/market-analysis", web::get().to(get_market_analysis))
    );
}
