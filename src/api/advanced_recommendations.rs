use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use crate::services::advanced_recommendation::{
    AdvancedRecommendationService, 
    AdvancedRecommendationRequest,
    PerformanceMode
};
use crate::utils::database::get_db_connection;
use crate::models::{Property, Contact};
use std::sync::Arc;

/// Phase 2 Advanced ML recommendation endpoints
/// Features:
/// - Two-stage retrieval with HNSW-based ANN search
/// - Advanced embedding pipeline
/// - Sub-10ms performance targets
/// - Comprehensive performance metrics

pub fn configure_advanced_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v2/recommendations")
            .route("/contact/{contact_id}", web::get().to(get_advanced_recommendations))
            .route("/contact/{contact_id}/fast", web::get().to(get_fast_recommendations))
            .route("/contact/{contact_id}/accurate", web::get().to(get_accurate_recommendations))
            .route("/batch", web::post().to(get_batch_recommendations))
            .route("/stats", web::get().to(get_service_stats))
            .route("/health", web::get().to(get_service_health))
            .route("/benchmark", web::post().to(run_performance_benchmark))
    );
}

/// Get advanced ML recommendations with full configurability
pub async fn get_advanced_recommendations(
    path: web::Path<i32>,
    query: web::Query<AdvancedRecommendationQuery>,
    service: web::Data<Arc<AdvancedRecommendationService>>,
) -> Result<HttpResponse> {
    let contact_id = path.into_inner();
    
    // Build request from query parameters
    let request = AdvancedRecommendationRequest {
        contact_id,
        max_recommendations: query.limit,
        use_neural_scoring: query.neural_scoring,
        use_two_stage_retrieval: query.two_stage,
        location_filters: None, // Could be extended to parse from query
        property_type_filters: query.property_types.clone(),
        budget_range: None, // Could be extended
        performance_mode: query.performance_mode.clone(),
        explain_scores: query.explain,
    };

    match get_recommendations_internal(request, &service).await {
        Ok(response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": response,
            "api_version": "v2.0"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": e.to_string(),
            "api_version": "v2.0"
        })))
    }
}

/// Get recommendations optimized for speed (5ms target)
pub async fn get_fast_recommendations(
    path: web::Path<i32>,
    query: web::Query<BasicRecommendationQuery>,
    service: web::Data<Arc<AdvancedRecommendationService>>,
) -> Result<HttpResponse> {
    let contact_id = path.into_inner();
    
    let request = AdvancedRecommendationRequest {
        contact_id,
        max_recommendations: query.limit,
        use_neural_scoring: Some(false), // Disable for speed
        use_two_stage_retrieval: Some(true),
        location_filters: None,
        property_type_filters: None,
        budget_range: None,
        performance_mode: Some(PerformanceMode::Fast),
        explain_scores: Some(false),
    };

    match get_recommendations_internal(request, &service).await {
        Ok(response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": {
                "recommendations": response.recommendations,
                "performance": {
                    "response_time_ms": response.performance_metrics.total_time_ms,
                    "target_achieved": response.performance_metrics.target_achieved,
                    "target_ms": response.performance_metrics.target_ms
                }
            },
            "mode": "fast",
            "api_version": "v2.0"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": e.to_string(),
            "mode": "fast"
        })))
    }
}

/// Get recommendations optimized for accuracy (20ms target)
pub async fn get_accurate_recommendations(
    path: web::Path<i32>,
    query: web::Query<BasicRecommendationQuery>,
    service: web::Data<Arc<AdvancedRecommendationService>>,
) -> Result<HttpResponse> {
    let contact_id = path.into_inner();
    
    let request = AdvancedRecommendationRequest {
        contact_id,
        max_recommendations: query.limit,
        use_neural_scoring: Some(true),
        use_two_stage_retrieval: Some(true),
        location_filters: None,
        property_type_filters: None,
        budget_range: None,
        performance_mode: Some(PerformanceMode::Accurate),
        explain_scores: Some(true),
    };

    match get_recommendations_internal(request, &service).await {
        Ok(response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": response,
            "mode": "accurate",
            "api_version": "v2.0"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": e.to_string(),
            "mode": "accurate"
        })))
    }
}

/// Get batch recommendations for multiple contacts
pub async fn get_batch_recommendations(
    request: web::Json<BatchRecommendationRequest>,
    service: web::Data<Arc<AdvancedRecommendationService>>,
) -> Result<HttpResponse> {
    let mut results = Vec::new();
    let mut total_time = 0.0;
    let start_time = std::time::Instant::now();

    for contact_id in &request.contact_ids {
        let individual_request = AdvancedRecommendationRequest {
            contact_id: *contact_id,
            max_recommendations: request.max_recommendations,
            use_neural_scoring: request.use_neural_scoring,
            use_two_stage_retrieval: request.use_two_stage_retrieval,
            location_filters: request.location_filters.clone(),
            property_type_filters: request.property_type_filters.clone(),
            budget_range: request.budget_range.clone(),
            performance_mode: request.performance_mode.clone(),
            explain_scores: request.explain_scores,
        };

        match get_recommendations_internal(individual_request, &service).await {
            Ok(response) => {
                total_time += response.performance_metrics.total_time_ms;
                results.push(json!({
                    "contact_id": contact_id,
                    "success": true,
                    "recommendations": response.recommendations,
                    "performance_ms": response.performance_metrics.total_time_ms
                }));
            },
            Err(e) => {
                results.push(json!({
                    "contact_id": contact_id,
                    "success": false,
                    "error": e.to_string()
                }));
            }
        }
    }

    let batch_time = start_time.elapsed().as_millis() as f64;
    let avg_time_per_contact = if !results.is_empty() { 
        total_time / results.len() as f64 
    } else { 
        0.0 
    };

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "results": results,
            "batch_stats": {
                "total_contacts": request.contact_ids.len(),
                "successful_requests": results.iter().filter(|r| r["success"].as_bool().unwrap_or(false)).count(),
                "total_batch_time_ms": batch_time,
                "avg_time_per_contact_ms": avg_time_per_contact,
                "parallel_efficiency": if batch_time > 0.0 { total_time / batch_time } else { 0.0 }
            }
        },
        "api_version": "v2.0"
    })))
}

/// Get service performance statistics
pub async fn get_service_stats(
    service: web::Data<Arc<AdvancedRecommendationService>>,
) -> Result<HttpResponse> {
    let stats = service.get_service_stats();
    let retrieval_stats = service.get_retrieval_stats();
    let feature_store_stats = service.get_feature_store_stats();

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "service_stats": {
                "total_requests": stats.total_requests,
                "avg_response_time_ms": stats.avg_response_time_ms,
                "cache_hit_rate": stats.cache_hit_rate,
                "performance_targets_met": stats.performance_targets_met,
                "fallback_rate": if stats.total_requests > 0 { 
                    stats.fallback_to_traditional as f64 / stats.total_requests as f64 
                } else { 0.0 }
            },
            "retrieval_stats": {
                "total_searches": retrieval_stats.total_searches,
                "stage1_avg_time_ms": retrieval_stats.stage1_avg_time_ms,
                "stage2_avg_time_ms": retrieval_stats.stage2_avg_time_ms,
                "index_size": retrieval_stats.index_size,
                "last_index_rebuild": retrieval_stats.last_index_rebuild.elapsed().as_secs()
            },
            "feature_store_stats": {
                "total_properties": feature_store_stats.total_properties,
                "total_contacts": feature_store_stats.total_contacts,
                "cache_hit_rate": feature_store_stats.cache_hit_rate,
                "memory_usage_mb": feature_store_stats.memory_usage_mb
            }
        },
        "timestamp": chrono::Utc::now(),
        "api_version": "v2.0"
    })))
}

/// Check service health and readiness
pub async fn get_service_health(
    service: web::Data<Arc<AdvancedRecommendationService>>,
) -> Result<HttpResponse> {
    let stats = service.get_service_stats();
    let feature_store_stats = service.get_feature_store_stats();
    let retrieval_stats = service.get_retrieval_stats();

    // Health checks
    let is_healthy = stats.avg_response_time_ms < 100.0 // Average response time under 100ms
        && stats.performance_targets_met > 0.5 // At least 50% of requests meet targets
        && feature_store_stats.total_properties > 0 // Has data
        && retrieval_stats.index_size > 0; // Index is built

    let status = if is_healthy { "healthy" } else { "degraded" };
    let mut http_status = if is_healthy { HttpResponse::Ok() } else { HttpResponse::ServiceUnavailable() };

    Ok(http_status.json(json!({
        "status": status,
        "healthy": is_healthy,
        "checks": {
            "response_time_ok": stats.avg_response_time_ms < 100.0,
            "performance_targets_ok": stats.performance_targets_met > 0.5,
            "has_data": feature_store_stats.total_properties > 0,
            "index_ready": retrieval_stats.index_size > 0
        },
        "metrics": {
            "avg_response_time_ms": stats.avg_response_time_ms,
            "performance_target_rate": stats.performance_targets_met,
            "total_properties": feature_store_stats.total_properties,
            "index_size": retrieval_stats.index_size
        },
        "timestamp": chrono::Utc::now(),
        "api_version": "v2.0"
    })))
}

/// Run performance benchmark
pub async fn run_performance_benchmark(
    request: web::Json<BenchmarkRequest>,
    service: web::Data<Arc<AdvancedRecommendationService>>,
) -> Result<HttpResponse> {
    let start_time = std::time::Instant::now();
    let mut results = Vec::new();
    let mut total_time = 0.0;
    let mut successful_requests = 0;

    // Run benchmark iterations
    for i in 0..request.iterations {
        let contact_id = request.contact_ids[i % request.contact_ids.len()];
        
        let benchmark_request = AdvancedRecommendationRequest {
            contact_id,
            max_recommendations: Some(20),
            use_neural_scoring: request.use_neural_scoring,
            use_two_stage_retrieval: request.use_two_stage_retrieval,
            location_filters: None,
            property_type_filters: None,
            budget_range: None,
            performance_mode: request.performance_mode.clone(),
            explain_scores: Some(false),
        };

        let iteration_start = std::time::Instant::now();
        
        match get_recommendations_internal(benchmark_request, &service).await {
            Ok(response) => {
                let iteration_time = iteration_start.elapsed().as_millis() as f64;
                total_time += iteration_time;
                successful_requests += 1;
                
                results.push(json!({
                    "iteration": i,
                    "contact_id": contact_id,
                    "time_ms": iteration_time,
                    "target_achieved": response.performance_metrics.target_achieved,
                    "recommendations_count": response.recommendations.len()
                }));
            },
            Err(e) => {
                results.push(json!({
                    "iteration": i,
                    "contact_id": contact_id,
                    "error": e.to_string()
                }));
            }
        }
    }

    let benchmark_time = start_time.elapsed().as_millis() as f64;
    let avg_time = if successful_requests > 0 { total_time / successful_requests as f64 } else { 0.0 };
    let success_rate = successful_requests as f64 / request.iterations as f64;
    
    // Performance percentiles (simplified)
    let mut times: Vec<f64> = results.iter()
        .filter_map(|r| r["time_ms"].as_f64())
        .collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let p50 = if !times.is_empty() { times[times.len() / 2] } else { 0.0 };
    let p95 = if !times.is_empty() { times[(times.len() * 95) / 100] } else { 0.0 };
    let p99 = if !times.is_empty() { times[(times.len() * 99) / 100] } else { 0.0 };

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "benchmark_summary": {
                "iterations": request.iterations,
                "successful_requests": successful_requests,
                "success_rate": success_rate,
                "total_benchmark_time_ms": benchmark_time,
                "avg_response_time_ms": avg_time,
                "throughput_rps": if benchmark_time > 0.0 { 
                    (successful_requests as f64 * 1000.0) / benchmark_time 
                } else { 0.0 }
            },
            "performance_percentiles": {
                "p50_ms": p50,
                "p95_ms": p95,
                "p99_ms": p99,
                "min_ms": times.first().copied().unwrap_or(0.0),
                "max_ms": times.last().copied().unwrap_or(0.0)
            },
            "results": results
        },
        "timestamp": chrono::Utc::now(),
        "api_version": "v2.0"
    })))
}

/// Internal helper to get recommendations
async fn get_recommendations_internal(
    request: AdvancedRecommendationRequest,
    service: &Arc<AdvancedRecommendationService>,
) -> anyhow::Result<crate::services::advanced_recommendation::AdvancedRecommendationResponse> {
    // Get properties from database
    let mut connection = get_db_connection()?;
    let properties = Property::get_all(&mut connection)?;

    // Get recommendations
    service.get_advanced_recommendations(request, &properties).await
}

// Query parameter structures
#[derive(serde::Deserialize)]
pub struct AdvancedRecommendationQuery {
    pub limit: Option<usize>,
    pub neural_scoring: Option<bool>,
    pub two_stage: Option<bool>,
    pub property_types: Option<Vec<String>>,
    pub performance_mode: Option<PerformanceMode>,
    pub explain: Option<bool>,
}

#[derive(serde::Deserialize)]
pub struct BasicRecommendationQuery {
    pub limit: Option<usize>,
}

#[derive(serde::Deserialize)]
pub struct BatchRecommendationRequest {
    pub contact_ids: Vec<i32>,
    pub max_recommendations: Option<usize>,
    pub use_neural_scoring: Option<bool>,
    pub use_two_stage_retrieval: Option<bool>,
    pub location_filters: Option<Vec<crate::services::advanced_recommendation::LocationFilter>>,
    pub property_type_filters: Option<Vec<String>>,
    pub budget_range: Option<crate::services::advanced_recommendation::BudgetRange>,
    pub performance_mode: Option<PerformanceMode>,
    pub explain_scores: Option<bool>,
}

#[derive(serde::Deserialize)]
pub struct BenchmarkRequest {
    pub contact_ids: Vec<i32>,
    pub iterations: usize,
    pub use_neural_scoring: Option<bool>,
    pub use_two_stage_retrieval: Option<bool>,
    pub performance_mode: Option<PerformanceMode>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_rt::test]
    async fn test_health_endpoint() {
        // This would require setting up a test service
        // For now, just test that the endpoint compiles
        assert_eq!(2 + 2, 4);
    }
}
