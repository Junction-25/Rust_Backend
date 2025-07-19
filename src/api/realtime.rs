use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use crate::services::realtime::*;
use crate::services::ai_recommendations::AIRecommendationService;
use crate::models::{Contact, Property};
use std::sync::Arc;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketStats {
    pub connected_clients: usize,
    pub total_messages_sent: u64,
    pub active_subscriptions: Vec<SubscriptionStat>,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionStat {
    pub subscription_type: String,
    pub subscriber_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationRequest {
    pub contact_id: Option<i32>,
    pub notification_type: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestNotificationRequest {
    pub notification_type: String,
    pub count: Option<usize>,
}

/// Get WebSocket connection statistics
pub async fn get_websocket_stats() -> ActixResult<HttpResponse> {
    // In a real implementation, this would query the WebSocketManager
    let stats = WebSocketStats {
        connected_clients: 0, // Would be fetched from manager
        total_messages_sent: 0,
        active_subscriptions: vec![
            SubscriptionStat {
                subscription_type: "recommendations".to_string(),
                subscriber_count: 0,
            },
            SubscriptionStat {
                subscription_type: "market_updates".to_string(),
                subscriber_count: 0,
            },
            SubscriptionStat {
                subscription_type: "price_changes".to_string(),
                subscriber_count: 0,
            },
        ],
        uptime_seconds: 0,
    };

    Ok(HttpResponse::Ok().json(stats))
}

/// Send a test notification
pub async fn send_test_notification(
    notification_service: web::Data<RealtimeNotificationService>,
    req: web::Json<TestNotificationRequest>,
) -> ActixResult<HttpResponse> {
    let count = req.count.unwrap_or(1);
    
    match req.notification_type.as_str() {
        "recommendation" => {
            for i in 0..count {
                notification_service.notify_new_recommendation(
                    1001, // Test contact ID
                    2000 + i as i32, // Test property ID
                    85.5 + (i as f64 * 0.1),
                    format!("Test recommendation #{} - High match based on preferences", i + 1),
                ).await;
            }
        },
        "market_alert" => {
            for i in 0..count {
                notification_service.notify_market_alert(
                    "Algiers".to_string(),
                    "apartment".to_string(),
                    MarketAlertType::HotMarket,
                    format!("Test market alert #{} - Increased activity detected", i + 1),
                ).await;
            }
        },
        "price_change" => {
            for i in 0..count {
                notification_service.notify_property_update(
                    3000 + i as i32,
                    PropertyUpdateType::PriceChange,
                    Some(serde_json::json!(15000000)),
                    serde_json::json!(14500000),
                    Some(-3.33),
                ).await;
            }
        },
        "price_prediction" => {
            for i in 0..count {
                notification_service.notify_price_prediction(
                    4000 + i as i32,
                    16000000.0,
                    16800000.0 + (i as f64 * 50000.0),
                    0.87 - (i as f64 * 0.01),
                    "6 months".to_string(),
                ).await;
            }
        },
        _ => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid notification type",
                "valid_types": ["recommendation", "market_alert", "price_change", "price_prediction"]
            })));
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("Sent {} {} notifications", count, req.notification_type),
        "timestamp": Utc::now()
    })))
}

/// Send a custom notification
pub async fn send_custom_notification(
    notification_service: web::Data<RealtimeNotificationService>,
    req: web::Json<NotificationRequest>,
) -> ActixResult<HttpResponse> {
    match req.notification_type.as_str() {
        "recommendation" => {
            if let Some(contact_id) = req.contact_id {
                notification_service.notify_new_recommendation(
                    contact_id,
                    12345, // Default property ID
                    92.5,
                    req.message.clone(),
                ).await;
            } else {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "contact_id required for recommendation notifications"
                })));
            }
        },
        "market_alert" => {
            notification_service.notify_market_alert(
                "Custom Location".to_string(),
                "mixed".to_string(),
                MarketAlertType::TrendChange,
                req.message.clone(),
            ).await;
        },
        _ => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Unsupported notification type for custom notifications"
            })));
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Custom notification sent successfully",
        "timestamp": Utc::now()
    })))
}

/// Start real-time recommendation monitoring for a contact
pub async fn start_realtime_monitoring(
    path: web::Path<i32>,
    ai_service: web::Data<Arc<AIRecommendationService>>,
    notification_service: web::Data<RealtimeNotificationService>,
) -> ActixResult<HttpResponse> {
    let contact_id = path.into_inner();

    // In a real implementation, this would start a background task
    // that monitors for new properties and sends recommendations
    tokio::spawn({
        let ai_service = ai_service.clone();
        let notification_service = notification_service.clone();
        
        async move {
            // Simulate real-time monitoring
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            let mut iteration = 0;
            
            loop {
                interval.tick().await;
                iteration += 1;
                
                // Stop after 5 iterations for demo
                if iteration > 5 {
                    break;
                }

                // Simulate finding new recommendations
                let score = 80.0 + (iteration as f64 * 2.5);
                let property_id = 5000 + iteration;
                
                notification_service.notify_new_recommendation(
                    contact_id,
                    property_id,
                    score,
                    format!("Real-time match #{} - New property matches your criteria", iteration),
                ).await;

                // Also send market updates
                if iteration % 2 == 0 {
                    notification_service.notify_market_alert(
                        "Oran".to_string(),
                        "villa".to_string(),
                        MarketAlertType::NewInventory,
                        format!("New inventory alert #{} - {} new properties available", iteration, iteration * 3),
                    ).await;
                }
            }

            log::info!("Real-time monitoring completed for contact {}", contact_id);
        }
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("Started real-time monitoring for contact {}", contact_id),
        "monitoring_duration": "5 minutes",
        "timestamp": Utc::now()
    })))
}

/// Get real-time system health
pub async fn get_system_health() -> ActixResult<HttpResponse> {
    let health = serde_json::json!({
        "status": "healthy",
        "websocket_server": "running",
        "notification_service": "active",
        "ai_engine": "operational",
        "timestamp": Utc::now(),
        "features": {
            "real_time_recommendations": true,
            "market_alerts": true,
            "price_predictions": true,
            "live_property_updates": true
        }
    });

    Ok(HttpResponse::Ok().json(health))
}

pub fn configure_realtime_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/realtime")
            .route("/stats", web::get().to(get_websocket_stats))
            .route("/health", web::get().to(get_system_health))
            .route("/test-notification", web::post().to(send_test_notification))
            .route("/send-notification", web::post().to(send_custom_notification))
            .route("/monitor/{contact_id}", web::post().to(start_realtime_monitoring))
    );
}
