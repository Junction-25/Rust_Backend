use actix_web::{web, HttpResponse, Result, http::header};
use crate::services::{QuoteService, RecommendationService};
use crate::services::quote::{QuoteRequest, ComparisonQuoteRequest};
use crate::api::recommendations::ErrorResponse;
use serde::Deserialize;

pub async fn generate_quote(
    request: web::Json<QuoteRequest>,
    service: web::Data<QuoteService>,
) -> Result<HttpResponse> {
    match service.generate_property_quote(request.into_inner()).await {
        Ok(response) => {
            // Return PDF as download
            Ok(HttpResponse::Ok()
                .content_type("application/pdf")
                .insert_header((
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"quote_{}.pdf\"", response.quote_id),
                ))
                .body(response.pdf_data))
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
        Ok(pdf_data) => {
            Ok(HttpResponse::Ok()
                .content_type("application/pdf")
                .insert_header((
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"property_comparison.pdf\"",
                ))
                .body(pdf_data))
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
    _recommendation_service: web::Data<RecommendationService>,
    _quote_service: web::Data<QuoteService>,
) -> Result<HttpResponse> {
    // TODO: Implement property-to-contact recommendations
    // For now, return a simple response
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Recommendation quote generation not yet implemented for new schema",
        "property_id": query.property_id
    })))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/quotes")
            .route("/generate", web::post().to(generate_quote))
            .route("/comparison", web::post().to(generate_comparison_quote))
            .route("/recommendations", web::get().to(generate_recommendation_quote))
    );
}
