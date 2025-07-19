pub mod contacts;
pub mod properties;
pub mod recommendations;
pub mod advanced_recommendations;
pub mod comparisons;
pub mod quotes;
pub mod ai;
pub mod realtime;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    recommendations::configure_routes(cfg);
    comparisons::configure_routes(cfg);
    quotes::configure_routes(cfg);
    ai::configure_routes(cfg);
    realtime::configure_realtime_routes(cfg);
}
