use crate::models::{Contact, Property, Location};

pub fn calculate_distance_km(loc1: &Location, loc2: &Location) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;
    
    let lat1_rad = loc1.latitude.to_radians();
    let lat2_rad = loc2.latitude.to_radians();
    let delta_lat = (loc2.latitude - loc1.latitude).to_radians();
    let delta_lng = (loc2.longitude - loc1.longitude).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lng / 2.0).sin().powi(2);
    
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    
    EARTH_RADIUS_KM * c
}

pub fn calculate_budget_score(property_price: i64, budget_min: i64, budget_max: i64) -> f64 {
    if property_price < budget_min {
        // Property is below minimum budget - might be suspiciously cheap
        let diff_ratio = (budget_min - property_price) as f64 / budget_min as f64;
        (1.0 - diff_ratio * 0.5).max(0.1) // Penalize but don't eliminate
    } else if property_price <= budget_max {
        // Property is within budget - perfect match
        let budget_utilization = (property_price - budget_min) as f64 / (budget_max - budget_min) as f64;
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
        let over_budget_ratio = (property_price - budget_max) as f64 / budget_max as f64;
        (1.0 - over_budget_ratio * 2.0).max(0.0) // Heavily penalize over-budget properties
    }
}

pub fn calculate_location_score(property: &Property, contact: &Contact) -> f64 {
    if contact.preferred_locations.is_empty() {
        return 0.5; // Neutral score if no location preference
    }

    let mut best_score: f64 = 0.0;
    
    for preferred_location in &contact.preferred_locations {
        let distance = calculate_distance_km(&property.location, preferred_location);
        
        // Exact city match gets bonus
        let city_match_bonus = if property.location.city.to_lowercase() == preferred_location.city.to_lowercase() {
            0.3
        } else {
            0.0
        };

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

        let total_score = (distance_score + city_match_bonus).min(1.0);
        best_score = best_score.max(total_score);
    }

    best_score
}

pub fn calculate_property_type_score(property: &Property, contact: &Contact) -> f64 {
    if contact.preferred_property_types.is_empty() {
        return 0.5; // Neutral score if no type preference
    }

    if contact.preferred_property_types.contains(&property.property_type) {
        1.0
    } else {
        0.0
    }
}

pub fn calculate_size_score(property: &Property, contact: &Contact) -> f64 {
    let mut room_score = 1.0;
    let mut area_score = 1.0;

    // Room matching
    if let (Some(min_rooms), Some(max_rooms)) = (contact.min_rooms, contact.max_rooms) {
        if property.rooms < min_rooms {
            room_score = 0.1; // Too few rooms
        } else if property.rooms > max_rooms {
            room_score = 0.3; // Too many rooms (less penalized)
        }
    } else if let Some(min_rooms) = contact.min_rooms {
        if property.rooms < min_rooms {
            room_score = 0.1;
        }
    } else if let Some(max_rooms) = contact.max_rooms {
        if property.rooms > max_rooms {
            room_score = 0.3;
        }
    }

    // Area matching
    if let (Some(min_area), Some(max_area)) = (contact.min_area, contact.max_area) {
        if property.area_sqm < min_area {
            area_score = 0.2; // Too small
        } else if property.area_sqm > max_area {
            area_score = 0.4; // Too large (less penalized)
        }
    } else if let Some(min_area) = contact.min_area {
        if property.area_sqm < min_area {
            area_score = 0.2;
        }
    } else if let Some(max_area) = contact.max_area {
        if property.area_sqm > max_area {
            area_score = 0.4;
        }
    }

    (room_score + area_score) / 2.0
}

pub fn calculate_feature_score(property: &Property, contact: &Contact) -> (f64, bool) {
    let required_features_met = contact.required_features.iter()
        .all(|feature| property.features.contains(feature));

    if !required_features_met {
        return (0.0, false); // If required features are not met, score is 0
    }

    if contact.preferred_features.is_empty() {
        return (1.0, true); // All required features met, no preferences
    }

    let matched_preferred = contact.preferred_features.iter()
        .filter(|feature| property.features.contains(feature))
        .count();

    let preference_score = matched_preferred as f64 / contact.preferred_features.len() as f64;
    
    (preference_score, true)
}

pub fn calculate_overall_score(
    budget_score: f64,
    location_score: f64,
    property_type_score: f64,
    size_score: f64,
    feature_score: f64,
) -> f64 {
    // Weighted scoring with different importance levels
    const BUDGET_WEIGHT: f64 = 0.3;
    const LOCATION_WEIGHT: f64 = 0.25;
    const PROPERTY_TYPE_WEIGHT: f64 = 0.2;
    const SIZE_WEIGHT: f64 = 0.15;
    const FEATURE_WEIGHT: f64 = 0.1;

    budget_score * BUDGET_WEIGHT
        + location_score * LOCATION_WEIGHT
        + property_type_score * PROPERTY_TYPE_WEIGHT
        + size_score * SIZE_WEIGHT
        + feature_score * FEATURE_WEIGHT
}
