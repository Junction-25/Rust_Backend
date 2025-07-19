mod config;
mod models;
mod db;
mod services;
mod api;
mod utils;
mod ml;

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use actix::Actor;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init();

    // Load configuration
    let config = config::Config::from_env().expect("Failed to load configuration");
    
    // Setup database connection
    let database_pool = PgPool::connect(&config.database.url)
        .await
        .expect("Failed to connect to database");

    // Run migrations (you'll need to add these)
    sqlx::migrate!("./migrations")
        .run(&database_pool)
        .await
        .expect("Failed to run migrations");

    // Setup repository
    let repository = Arc::new(db::Repository::new(database_pool));

    // Setup services
    let recommendation_service = services::RecommendationService::new(
        repository.clone(),
        Duration::from_secs(config.recommendation.cache_ttl_seconds),
        config.cache.max_capacity,
    );
    
    let comparison_service = services::ComparisonService::new(repository.clone());
    let quote_service = services::QuoteService::new(repository.clone());
    
    // Initialize AI recommendation service
    let ai_service = services::AIRecommendationService::new(
        repository.clone(),
        Arc::new(recommendation_service.clone()),
    );

    // Initialize WebSocket manager
    let ws_manager = services::realtime::WebSocketManager::default().start();
    
    // Initialize real-time notification service
    let notification_service = services::realtime::RealtimeNotificationService::new(ws_manager.clone());
    
    // Start background notifications
    notification_service.start_background_notifications();

    let server_host = config.server.host.clone();
    let server_port = config.server.port;

    log::info!("Starting server at http://{}:{}", server_host, server_port);
    log::info!("WebSocket endpoint available at ws://{}:{}/ws", server_host, server_port);

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(recommendation_service.clone()))
            .app_data(web::Data::new(comparison_service.clone()))
            .app_data(web::Data::new(quote_service.clone()))
            .app_data(web::Data::new(ai_service.clone()))
            .app_data(web::Data::new(ws_manager.clone()))
            .app_data(web::Data::new(notification_service.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .configure(api::configure_routes)
            .configure(services::realtime::configure_websocket_routes)
            .route("/health", web::get().to(health_check))
    })
    .bind(format!("{}:{}", server_host, server_port))?
    .run()
    .await
}

async fn health_check() -> actix_web::Result<actix_web::HttpResponse> {
    Ok(actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    })))
}
