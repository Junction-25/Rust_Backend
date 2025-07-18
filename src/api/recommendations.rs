use actix_web::{web, HttpResponse, Result};
use crate::services::RecommendationService;
use crate::models::*;
use uuid::Uuid;

pub async fn get_property_recommendations(
    path: web::Path<Uuid>,
    query: web::Query<RecommendationQuery>,
    service: web::Data<RecommendationService>,
) -> Result<HttpResponse> {
    let property_id = path.into_inner();
    
    match service.get_recommendations_for_property(
        property_id,
        query.limit,
        query.min_score,
    ).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
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
    match service.get_bulk_recommendations(request.into_inner()).await {
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
            .route("/bulk", web::post().to(get_bulk_recommendations))
    );
}
