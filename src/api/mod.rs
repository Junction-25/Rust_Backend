pub mod recommendations;
pub mod comparisons;
pub mod quotes;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    recommendations::configure_routes(cfg);
    comparisons::configure_routes(cfg);
    quotes::configure_routes(cfg);
}
