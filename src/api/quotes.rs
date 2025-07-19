use actix_web::{web, HttpResponse, Result};
use crate::services::QuoteService;
use crate::services::quote::{QuoteRequest, ComparisonQuoteRequest};
use crate::api::recommendations::ErrorResponse;
use serde::Deserialize;
use chrono;

pub async fn generate_quote(
    request: web::Json<QuoteRequest>,
    service: web::Data<QuoteService>,
) -> Result<HttpResponse> {
    match service.generate_property_quote(request.into_inner()).await {
        Ok(response) => {
            // Return JSON response instead of PDF
            Ok(HttpResponse::Ok().json(response))
        },
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to generate quote".to_string(),
            message: e.to_string(),
        })),
    }
}

pub async fn generate_comparison_quote(
    request: web::Json<ComparisonQuoteRequest>,
    service: web::Data<QuoteService>,
) -> Result<HttpResponse> {
    match service.generate_comparison_quote(request.into_inner()).await {
        Ok(response) => {
            // Return JSON response instead of PDF
            Ok(HttpResponse::Ok().json(response))
        },
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to generate comparison quote".to_string(),
            message: e.to_string(),
        })),
    }
}

#[derive(Deserialize)]
pub struct RecommendationQuoteQuery {
    pub property_id: i32,
}

pub async fn generate_recommendation_quote(
    query: web::Query<RecommendationQuoteQuery>,
    _quote_service: web::Data<QuoteService>,
) -> Result<HttpResponse> {
    // For now, return a JSON template response
    let response = serde_json::json!({
        "property_id": query.property_id,
        "message": "Recommendation quote in JSON format",
        "recommendations": [],
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "status": "Template response - integrate with recommendation service"
    });

    Ok(HttpResponse::Ok().json(response))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/quotes")
            .route("/generate", web::post().to(generate_quote))
            .route("/comparison", web::post().to(generate_comparison_quote))
            // .route("/recommendations", web::get().to(generate_recommendation_quote))
    );
}
