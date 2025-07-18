use actix_web::{web, HttpResponse, Result};
use crate::services::ComparisonService;
use crate::api::recommendations::ErrorResponse;
use uuid::Uuid;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ComparisonQuery {
    pub property1_id: Uuid,
    pub property2_id: Uuid,
}

pub async fn compare_properties(
    query: web::Query<ComparisonQuery>,
    service: web::Data<ComparisonService>,
) -> Result<HttpResponse> {
    match service.compare_properties(query.property1_id, query.property2_id).await {
        Ok(comparison) => Ok(HttpResponse::Ok().json(comparison)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to compare properties".to_string(),
            message: e.to_string(),
        })),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/comparisons")
            .route("/properties", web::get().to(compare_properties))
    );
}
