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
        
        // Generate detailed analysis
        let detailed_analysis = self.generate_detailed_analysis(&property1, &property2, &comparison_metrics);
        
        // Generate recommendation
        let recommendation = self.generate_comparison_recommendation(&property1, &property2, &comparison_metrics);

        Ok(PropertyComparison {
            property1,
            property2,
            comparison_metrics,
            detailed_analysis,
            recommendation,
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

    fn generate_detailed_analysis(
        &self,
        property1: &Property,
        property2: &Property,
        metrics: &ComparisonMetrics,
    ) -> ComparisonAnalysis {
        let price_analysis = self.analyze_price_comparison(property1, property2);
        let space_analysis = self.analyze_space_comparison(property1, property2);
        let location_analysis = self.analyze_location_comparison(property1, property2, metrics.location_distance_km);
        let feature_analysis = self.analyze_feature_comparison(property1, property2);
        let value_analysis = self.analyze_value_comparison(property1, property2);

        ComparisonAnalysis {
            price_analysis,
            space_analysis,
            location_analysis,
            feature_analysis,
            value_analysis,
        }
    }

    fn analyze_price_comparison(&self, property1: &Property, property2: &Property) -> PriceAnalysis {
        let cheaper_property = if property1.price <= property2.price {
            property1.id
        } else {
            property2.id
        };

        let price_savings = (property1.price - property2.price).abs();
        
        let affordability_rating = if price_savings < 10000.0 {
            "Similar pricing".to_string()
        } else if price_savings < 50000.0 {
            "Moderate price difference".to_string()
        } else if price_savings < 100000.0 {
            "Significant price difference".to_string()
        } else {
            "Major price difference".to_string()
        };

        let property1_price_per_sqm = property1.price / property1.area_sqm as f64;
        let property2_price_per_sqm = property2.price / property2.area_sqm as f64;

        PriceAnalysis {
            cheaper_property,
            price_savings,
            affordability_rating,
            price_per_sqm_comparison: (property1_price_per_sqm, property2_price_per_sqm),
        }
    }

    fn analyze_space_comparison(&self, property1: &Property, property2: &Property) -> SpaceAnalysis {
        let larger_property = if property1.area_sqm >= property2.area_sqm {
            property1.id
        } else {
            property2.id
        };

        let space_advantage = (property1.area_sqm - property2.area_sqm).abs();

        let room_comparison = match property1.number_of_rooms.cmp(&property2.number_of_rooms) {
            std::cmp::Ordering::Equal => "Same number of rooms".to_string(),
            std::cmp::Ordering::Greater => format!("Property 1 has {} more room(s)", property1.number_of_rooms - property2.number_of_rooms),
            std::cmp::Ordering::Less => format!("Property 2 has {} more room(s)", property2.number_of_rooms - property1.number_of_rooms),
        };

        // Space efficiency: area per room
        let property1_efficiency = property1.area_sqm as f64 / property1.number_of_rooms as f64;
        let property2_efficiency = property2.area_sqm as f64 / property2.number_of_rooms as f64;

        SpaceAnalysis {
            larger_property,
            space_advantage,
            room_comparison,
            space_efficiency: (property1_efficiency, property2_efficiency),
        }
    }

    fn analyze_location_comparison(&self, _property1: &Property, _property2: &Property, distance_km: f64) -> LocationAnalysis {
        let location_similarity = if distance_km < 1.0 {
            "Very close locations".to_string()
        } else if distance_km < 5.0 {
            "Same neighborhood".to_string()
        } else if distance_km < 20.0 {
            "Same city area".to_string()
        } else {
            "Different areas".to_string()
        };

        let mut accessibility_notes = Vec::new();
        if distance_km < 10.0 {
            accessibility_notes.push("Both properties are in the same general area".to_string());
        } else {
            accessibility_notes.push("Properties are in different areas - consider commute times".to_string());
        }

        LocationAnalysis {
            distance_between: distance_km,
            location_similarity,
            accessibility_notes,
        }
    }

    fn analyze_feature_comparison(&self, property1: &Property, property2: &Property) -> FeatureAnalysis {
        let property_type_match = property1.property_type == property2.property_type;
        
        let mut feature_advantages = Vec::new();
        let mut common_features = Vec::new();
        
        // Property type comparison
        if property_type_match {
            common_features.push(format!("Both are {}", property1.property_type));
        } else {
            feature_advantages.push(format!("Property 1: {}, Property 2: {}", property1.property_type, property2.property_type));
        }

        // Room comparison advantages
        if property1.number_of_rooms > property2.number_of_rooms {
            feature_advantages.push(format!("Property 1 has more rooms ({} vs {})", property1.number_of_rooms, property2.number_of_rooms));
        } else if property2.number_of_rooms > property1.number_of_rooms {
            feature_advantages.push(format!("Property 2 has more rooms ({} vs {})", property2.number_of_rooms, property1.number_of_rooms));
        } else {
            common_features.push(format!("Both have {} rooms", property1.number_of_rooms));
        }

        // Space comparison advantages
        if property1.area_sqm > property2.area_sqm {
            feature_advantages.push(format!("Property 1 is larger ({} vs {} sqm)", property1.area_sqm, property2.area_sqm));
        } else if property2.area_sqm > property1.area_sqm {
            feature_advantages.push(format!("Property 2 is larger ({} vs {} sqm)", property2.area_sqm, property1.area_sqm));
        }

        // For unique features, we'll create some example analysis
        let property1_unique = vec![
            format!("Located at {}", property1.address),
            format!("Priced at ${:.0}", property1.price),
        ];
        
        let property2_unique = vec![
            format!("Located at {}", property2.address),
            format!("Priced at ${:.0}", property2.price),
        ];

        FeatureAnalysis {
            property_type_match,
            feature_advantages,
            common_features,
            unique_features: (property1_unique, property2_unique),
        }
    }

    fn analyze_value_comparison(&self, property1: &Property, property2: &Property) -> ValueAnalysis {
        // Calculate value scores based on price per square meter and features
        let property1_price_per_sqm = property1.price / property1.area_sqm as f64;
        let property2_price_per_sqm = property2.price / property2.area_sqm as f64;
        
        // Simple value scoring: lower price per sqm = higher value score
        let max_price_per_sqm = property1_price_per_sqm.max(property2_price_per_sqm);
        let property1_value_score = 1.0 - (property1_price_per_sqm / max_price_per_sqm - 0.5).abs();
        let property2_value_score = 1.0 - (property2_price_per_sqm / max_price_per_sqm - 0.5).abs();

        let better_value_property = if property1_value_score >= property2_value_score {
            property1.id
        } else {
            property2.id
        };

        let investment_potential = if (property1_value_score - property2_value_score).abs() < 0.1 {
            "Both properties offer similar investment value".to_string()
        } else if better_value_property == property1.id {
            "Property 1 offers better value for money".to_string()
        } else {
            "Property 2 offers better value for money".to_string()
        };

        ValueAnalysis {
            better_value_property,
            value_score: (property1_value_score, property2_value_score),
            investment_potential,
        }
    }

    fn generate_comparison_recommendation(
        &self,
        property1: &Property,
        property2: &Property,
        metrics: &ComparisonMetrics,
    ) -> ComparisonRecommendation {
        let mut key_reasons = Vec::new();
        let mut considerations = Vec::new();
        let mut confidence_score: f64 = 0.5; // Base confidence

        // Price-based reasoning
        if metrics.price_difference.abs() > 50000.0 {
            if property1.price < property2.price {
                key_reasons.push("Property 1 is significantly more affordable".to_string());
                confidence_score += 0.2;
            } else {
                key_reasons.push("Property 2 is significantly more affordable".to_string());
                confidence_score += 0.2;
            }
        }

        // Space-based reasoning
        if metrics.area_difference.abs() > 20 {
            if property1.area_sqm > property2.area_sqm {
                key_reasons.push("Property 1 offers more living space".to_string());
                confidence_score += 0.15;
            } else {
                key_reasons.push("Property 2 offers more living space".to_string());
                confidence_score += 0.15;
            }
        }

        // Location-based reasoning
        if metrics.location_distance_km > 20.0 {
            considerations.push("Properties are in different areas - consider your commute and lifestyle preferences".to_string());
        } else if metrics.location_distance_km < 5.0 {
            key_reasons.push("Both properties are in similar locations".to_string());
            confidence_score += 0.1;
        }

        // Value-based reasoning
        let property1_value = property1.area_sqm as f64 / property1.price;
        let property2_value = property2.area_sqm as f64 / property2.price;
        
        let recommended_property = if property1_value > property2_value * 1.1 {
            confidence_score += 0.15;
            key_reasons.push("Property 1 offers better value per dollar".to_string());
            property1.id
        } else if property2_value > property1_value * 1.1 {
            confidence_score += 0.15;
            key_reasons.push("Property 2 offers better value per dollar".to_string());
            property2.id
        } else {
            considerations.push("Both properties offer similar value - consider your personal preferences".to_string());
            if property1.price <= property2.price { property1.id } else { property2.id }
        };

        // Ensure confidence doesn't exceed 1.0
        confidence_score = confidence_score.min(1.0);

        let summary = if confidence_score > 0.8 {
            format!("Property {} is clearly the better choice based on multiple factors", if recommended_property == property1.id { 1 } else { 2 })
        } else if confidence_score > 0.6 {
            format!("Property {} appears to be the better option, but both have merit", if recommended_property == property1.id { 1 } else { 2 })
        } else {
            "Both properties have similar overall value - your personal preferences should guide the final decision".to_string()
        };

        ComparisonRecommendation {
            recommended_property,
            confidence_score,
            key_reasons,
            considerations,
            summary,
        }
    }
}
