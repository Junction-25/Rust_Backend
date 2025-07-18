use crate::db::Repository;
use crate::models::*;
use crate::utils::scoring::calculate_distance_km;
use anyhow::Result;
use std::sync::Arc;

#[derive(Clone)]
pub struct ComparisonService {
    repository: Arc<Repository>,
}

impl ComparisonService {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { repository }
    }

    pub async fn compare_properties(
        &self,
        property1_id: i32,
        property2_id: i32,
    ) -> Result<PropertyComparison> {
        // Get both properties
        let property1 = self.repository.get_property_by_id(property1_id).await?
            .ok_or_else(|| anyhow::anyhow!("First property not found"))?;
        
        let property2 = self.repository.get_property_by_id(property2_id).await?
            .ok_or_else(|| anyhow::anyhow!("Second property not found"))?;

        // Calculate comparison metrics
        let comparison_metrics = self.calculate_comparison_metrics(&property1, &property2);

        Ok(PropertyComparison {
            property1,
            property2,
            comparison_metrics,
        })
    }

    fn calculate_comparison_metrics(&self, property1: &Property, property2: &Property) -> ComparisonMetrics {
        // Price comparison
        let price_difference = property2.price - property1.price;
        let price_difference_percentage = if property1.price > 0.0 {
            (price_difference as f64 / property1.price as f64) * 100.0
        } else {
            0.0
        };

        // Area comparison
        let area_difference = property2.area_sqm - property1.area_sqm;
        let area_difference_percentage = if property1.area_sqm > 0 {
            (area_difference as f64 / property1.area_sqm as f64) * 100.0
        } else {
            0.0
        };

        // Location distance
        let location_distance_km = calculate_distance_km(
            property1.location.lat, 
            property1.location.lon, 
            property2.location.lat, 
            property2.location.lon
        );

        // Feature similarity
        let feature_similarity_score = self.calculate_feature_similarity(property1, property2);

        // Overall similarity (weighted average of different factors)
        let overall_similarity_score = self.calculate_overall_similarity(
            property1,
            property2,
            feature_similarity_score,
            location_distance_km,
        );

        ComparisonMetrics {
            price_difference,
            price_difference_percentage,
            area_difference,
            area_difference_percentage,
            location_distance_km,
            overall_similarity_score,
        }
    }

    fn calculate_feature_similarity(&self, property1: &Property, property2: &Property) -> f64 {
        // Compare property types and number of rooms
        let mut similarity_score = 0.0;
        let mut factors = 0;

        // Property type similarity
        if property1.property_type == property2.property_type {
            similarity_score += 1.0;
        }
        factors += 1;

        // Room count similarity (normalized difference)
        let room_diff = (property1.number_of_rooms - property2.number_of_rooms).abs();
        let room_similarity = if room_diff <= 1 {
            1.0 - (room_diff as f64 * 0.2) // Small penalty for 1 room difference
        } else {
            (0.0_f64).max(1.0 - (room_diff as f64 * 0.3)) // Larger penalty for bigger differences
        };
        similarity_score += room_similarity;
        factors += 1;

        similarity_score / factors as f64
    }

    fn calculate_overall_similarity(
        &self,
        property1: &Property,
        property2: &Property,
        feature_similarity: f64,
        distance_km: f64,
    ) -> f64 {
        // Property type similarity
        let type_similarity = if property1.property_type == property2.property_type {
            1.0
        } else {
            0.0
        };

        // Price similarity (closer prices = higher similarity)
        let price_diff_ratio = if property1.price > 0.0 {
            (property1.price - property2.price).abs() as f64 / property1.price.max(property2.price) as f64
        } else {
            0.0
        };
        let price_similarity = (1.0 - price_diff_ratio).max(0.0);

        // Area similarity
        let area_diff_ratio = if property1.area_sqm > 0 {
            (property1.area_sqm - property2.area_sqm).abs() as f64 / property1.area_sqm.max(property2.area_sqm) as f64
        } else {
            0.0
        };
        let area_similarity = (1.0 - area_diff_ratio).max(0.0);

        // Room similarity
        let room_diff = (property1.number_of_rooms - property2.number_of_rooms).abs();
        let room_similarity = if room_diff == 0 {
            1.0
        } else if room_diff == 1 {
            0.8
        } else if room_diff == 2 {
            0.6
        } else {
            0.2
        };

        // Location similarity (closer = more similar)
        let location_similarity = if distance_km <= 1.0 {
            1.0
        } else if distance_km <= 5.0 {
            1.0 - (distance_km - 1.0) / 4.0 * 0.3
        } else if distance_km <= 20.0 {
            0.7 - (distance_km - 5.0) / 15.0 * 0.5
        } else {
            0.2
        };

        // Weighted average
        const TYPE_WEIGHT: f64 = 0.2;
        const PRICE_WEIGHT: f64 = 0.25;
        const AREA_WEIGHT: f64 = 0.15;
        const ROOM_WEIGHT: f64 = 0.15;
        const LOCATION_WEIGHT: f64 = 0.15;
        const FEATURE_WEIGHT: f64 = 0.1;

        type_similarity * TYPE_WEIGHT
            + price_similarity * PRICE_WEIGHT
            + area_similarity * AREA_WEIGHT
            + room_similarity * ROOM_WEIGHT
            + location_similarity * LOCATION_WEIGHT
            + feature_similarity * FEATURE_WEIGHT
    }
}
