use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub recommendation: RecommendationConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecommendationConfig {
    pub threshold: f64,
    pub max_recommendations: usize,
    pub cache_ttl_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    pub ttl_seconds: u64,
    pub max_capacity: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://username:password@localhost/real_estate_db".to_string());

        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap_or(8080);

        let recommendation_threshold = env::var("RECOMMENDATION_THRESHOLD")
            .unwrap_or_else(|_| "0.3".to_string())
            .parse()
            .unwrap_or(0.3);

        let max_recommendations = env::var("MAX_RECOMMENDATIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);

        let cache_ttl_seconds = env::var("CACHE_TTL_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .unwrap_or(3600);

        let cache_max_capacity = env::var("CACHE_MAX_CAPACITY")
            .unwrap_or_else(|_| "10000".to_string())
            .parse()
            .unwrap_or(10000);

        Ok(Config {
            database: DatabaseConfig {
                url: database_url,
                max_connections: 10,
                min_connections: 1,
            },
            server: ServerConfig {
                host: server_host,
                port: server_port,
            },
            recommendation: RecommendationConfig {
                threshold: recommendation_threshold,
                max_recommendations,
                cache_ttl_seconds,
            },
            cache: CacheConfig {
                ttl_seconds: cache_ttl_seconds,
                max_capacity: cache_max_capacity,
            },
        })
    }
}
