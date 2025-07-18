use actix_web::{web, HttpResponse, Result, http::header};
use crate::services::{QuoteService, RecommendationService};
use crate::services::quote::{QuoteRequest, ComparisonQuoteRequest};
use crate::api::recommendations::ErrorResponse;
use uuid::Uuid;
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
    pub property_id: Uuid,
}

pub async fn generate_recommendation_quote(
    query: web::Query<RecommendationQuoteQuery>,
    recommendation_service: web::Data<RecommendationService>,
    quote_service: web::Data<QuoteService>,
) -> Result<HttpResponse> {
    // First get recommendations
    let recommendations_result = recommendation_service.get_recommendations_for_property(
        query.property_id,
        Some(10), // Limit to top 10
        Some(0.3), // Minimum score of 0.3
    ).await;

    match recommendations_result {
        Ok(recommendations_response) => {
            // Then generate PDF quote
            match quote_service.generate_recommendation_quote(
                query.property_id,
                &recommendations_response.recommendations,
            ).await {
                Ok(pdf_data) => {
                    Ok(HttpResponse::Ok()
                        .content_type("application/pdf")
                        .insert_header((
                            header::CONTENT_DISPOSITION,
                            format!("attachment; filename=\"recommendations_{}.pdf\"", query.property_id),
                        ))
                        .body(pdf_data))
                },
                Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to generate recommendation quote".to_string(),
                    message: e.to_string(),
                })),
            }
        },
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to get recommendations".to_string(),
            message: e.to_string(),
        })),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/quotes")
            .route("/generate", web::post().to(generate_quote))
            .route("/comparison", web::post().to(generate_comparison_quote))
            .route("/recommendations", web::get().to(generate_recommendation_quote))
    );
}
