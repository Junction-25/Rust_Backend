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

    // Initialize market trends and weight adjuster
    let market_trends = Arc::new(std::sync::RwLock::new(
        std::collections::HashMap::<String, ml::market_trends::MarketTrend>::new()
    ));
    
    // Create weight adjuster with default config
    let weight_adjuster = ml::weight_adjuster::WeightAdjuster::new(None);
    
    // In a real application, you would update market trends periodically
    // For example, using a background task or an API endpoint
    
    // Setup services
    let recommendation_service = services::RecommendationService::new(
        repository.clone(),
        Duration::from_secs(config.recommendation.cache_ttl_seconds),
        config.cache.max_capacity,
        Some(weight_adjuster),
    );
    
    let comparison_service = services::ComparisonService::new(repository.clone());
    let quote_service = services::QuoteService::new(repository.clone());
    
    // Initialize AI recommendation service
    let ai_service = services::AIRecommendationService::new(
        repository.clone(),
        Arc::new(recommendation_service.clone()),
    );

    // Initialize Phase 2 Advanced Recommendation Service
    let advanced_config = services::advanced_recommendation::AdvancedServiceConfig::default();
    let advanced_service = Arc::new(services::advanced_recommendation::AdvancedRecommendationService::new(
        recommendation_service.clone(),
        advanced_config,
    ).expect("Failed to initialize advanced recommendation service"));
    
    // Initialize the advanced service with sample data for testing
    tokio::spawn({
        let advanced_service = advanced_service.clone();
        async move {
            log::info!("Initializing advanced recommendation service with sample data...");
            
            // Create sample properties for testing
            let sample_properties = vec![
                models::Property {
                    id: 1,
                    address: "123 Main St, Downtown, New York".to_string(),
                    location: models::property::Location {
                        lat: 40.7589,
                        lon: -73.9851,
                    },
                    price: 450000.0,
                    area_sqm: 85,
                    property_type: "apartment".to_string(),
                    number_of_rooms: 2,
                },
                models::Property {
                    id: 2,
                    address: "456 Oak Ave, Suburbs, Brooklyn".to_string(),
                    location: models::property::Location {
                        lat: 40.6892,
                        lon: -73.9442,
                    },
                    price: 680000.0,
                    area_sqm: 120,
                    property_type: "house".to_string(),
                    number_of_rooms: 3,
                }
            ];
            
            // Create sample contacts
            let sample_contacts = vec![
                models::Contact {
                    id: 1,
                    name: "John Doe".to_string(),
                    preferred_locations: vec![
                        models::property::NamedLocation {
                            name: "New York".to_string(),
                            lat: 40.7589,
                            lon: -73.9851,
                        }
                    ],
                    min_budget: 300000.0,
                    max_budget: 500000.0,
                    min_area_sqm: 60,
                    max_area_sqm: 120,
                    property_types: vec!["apartment".to_string()],
                    min_rooms: 2,
                },
                models::Contact {
                    id: 2,
                    name: "Jane Smith".to_string(),
                    preferred_locations: vec![
                        models::property::NamedLocation {
                            name: "Brooklyn".to_string(),
                            lat: 40.6892,
                            lon: -73.9442,
                        }
                    ],
                    min_budget: 500000.0,
                    max_budget: 700000.0,
                    min_area_sqm: 100,
                    max_area_sqm: 150,
                    property_types: vec!["house".to_string()],
                    min_rooms: 3,
                }
            ];
            
            // Initialize with sample data
            if let Err(e) = advanced_service.initialize(&sample_properties, &sample_contacts).await {
                log::error!("Failed to initialize advanced service: {}", e);
            } else {
                log::info!("Advanced recommendation service initialized successfully with sample data");
            }
        }
    });

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
            .app_data(web::Data::new(advanced_service.clone()))
            .app_data(web::Data::new(ws_manager.clone()))
            .app_data(web::Data::new(notification_service.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .configure(api::configure_routes)
            .configure(api::advanced_recommendations::configure_advanced_routes)
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
