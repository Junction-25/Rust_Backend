use crate::models::{Contact, Property};
use crate::ml::weight_adjuster::{WeightAdjuster, Weights};

pub fn calculate_distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;
    
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lng = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lng / 2.0).sin().powi(2);
    
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    
    EARTH_RADIUS_KM * c
}

pub fn calculate_budget_score(property_price: f64, budget_min: f64, budget_max: f64) -> f64 {
    if property_price < budget_min {
        // Property is below minimum budget - might be suspiciously cheap
        let diff_ratio = (budget_min - property_price) / budget_min;
        (1.0 - diff_ratio * 0.5).max(0.1) // Penalize but don't eliminate
    } else if property_price <= budget_max {
        // Property is within budget - perfect match
        let budget_utilization = (property_price - budget_min) / (budget_max - budget_min);
        // Give higher score for properties that use 60-90% of budget
        if budget_utilization >= 0.6 && budget_utilization <= 0.9 {
            1.0
        } else if budget_utilization < 0.6 {
            0.8 + budget_utilization * 0.2
        } else {
            1.0 - (budget_utilization - 0.9) * 2.0
        }
    } else {
        // Property is over budget
        let over_budget_ratio = (property_price - budget_max) / budget_max;
        (1.0 - over_budget_ratio * 2.0).max(0.0) // Heavily penalize over-budget properties
    }
}

pub fn calculate_location_score(property: &Property, contact: &Contact) -> f64 {
    if contact.preferred_locations.is_empty() {
        return 0.5; // Neutral score if no location preference
    }

    let mut best_score: f64 = 0.0;
    
    for preferred_location in &contact.preferred_locations {
        let distance = calculate_distance_km(
            property.location.lat, 
            property.location.lon, 
            preferred_location.lat, 
            preferred_location.lon
        );
        
        // Distance-based score (closer is better)
        let distance_score = if distance <= 5.0 {
            1.0
        } else if distance <= 15.0 {
            1.0 - (distance - 5.0) / 10.0 * 0.5
        } else if distance <= 50.0 {
            0.5 - (distance - 15.0) / 35.0 * 0.4
        } else {
            0.1
        };

        best_score = best_score.max(distance_score);
    }

    best_score
}

pub fn calculate_property_type_score(property: &Property, contact: &Contact) -> f64 {
    if contact.property_types.is_empty() {
        return 0.5; // Neutral score if no type preference
    }

    if contact.property_types.contains(&property.property_type) {
        1.0
    } else {
        0.0
    }
}

pub fn calculate_size_score(property: &Property, contact: &Contact) -> f64 {
    let mut room_score = 1.0;
    let mut area_score = 1.0;

    // Room matching
    if property.number_of_rooms < contact.min_rooms {
        room_score = 0.1; // Too few rooms
    }

    // Area matching
    if property.area_sqm < contact.min_area_sqm {
        area_score = 0.1; // Too small
    } else if property.area_sqm > contact.max_area_sqm {
        let overage_ratio = (property.area_sqm - contact.max_area_sqm) as f64 / contact.max_area_sqm as f64;
        area_score = (1.0 - overage_ratio * 0.5).max(0.3); // Penalize but not too much for larger areas
    }

    // Combine room and area scores
    (room_score + area_score) / 2.0
}

pub fn calculate_overall_score(
    budget_score: f64,
    location_score: f64,
    property_type_score: f64,
    size_score: f64,
    weight_adjuster: Option<&WeightAdjuster>,
    location: Option<&str>,
    property_type: Option<&str>,
) -> f64 {
    // Use dynamic weights if adjuster and required parameters are provided
    let weights = if let (Some(adjuster), Some(loc), Some(prop_type)) = (weight_adjuster, location, property_type) {
        adjuster.get_adjusted_weights(loc, prop_type)
    } else {
        // Fallback to default weights if adjuster is not provided
        Weights::default()
    };

    budget_score * weights.budget
        + location_score * weights.location
        + property_type_score * weights.property_type
        + size_score * weights.size
}

/// Calculate score with dynamic weights based on market conditions
pub fn calculate_dynamic_score(
    property: &Property,
    contact: &Contact,
    weight_adjuster: &WeightAdjuster,
) -> f64 {
    let budget_score = calculate_budget_score(
        property.price,
        contact.budget_min.unwrap_or(0.0),
        contact.budget_max.unwrap_or(f64::MAX),
    );
    
    let location_score = calculate_location_score(property, contact);
    let property_type_score = calculate_property_type_score(property, contact);
    let size_score = calculate_size_score(property, contact);

    calculate_overall_score(
        budget_score,
        location_score,
        property_type_score,
        size_score,
        Some(weight_adjuster),
        property.location.as_deref(),
        property.property_type.as_deref(),
    )
}
