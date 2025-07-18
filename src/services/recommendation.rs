use crate::db::Repository;
use crate::models::*;
use crate::utils::scoring::*;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use rayon::prelude::*;
use moka::future::Cache;
use std::time::Duration;

#[derive(Clone)]
pub struct RecommendationService {
    repository: Arc<Repository>,
    cache: Cache<String, Vec<Recommendation>>,
}

impl RecommendationService {
    pub fn new(repository: Arc<Repository>, cache_ttl: Duration, cache_capacity: u64) -> Self {
        let cache = Cache::builder()
            .time_to_live(cache_ttl)
            .max_capacity(cache_capacity)
            .build();

        Self {
            repository,
            cache,
        }
    }

    pub async fn get_recommendations_for_property(
        &self,
        property_id: uuid::Uuid,
        limit: Option<usize>,
        min_score: Option<f64>,
    ) -> Result<RecommendationResponse> {
        let start_time = std::time::Instant::now();
        
        // Check cache first
        let cache_key = format!("property_{}_{:?}_{:?}", property_id, limit, min_score);
        if let Some(cached_recommendations) = self.cache.get(&cache_key).await {
            return Ok(RecommendationResponse {
                recommendations: cached_recommendations.clone(),
                total_count: cached_recommendations.len(),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            });
        }

        // Get property and contacts
        let property = self.repository.get_property_by_id(property_id).await?
            .ok_or_else(|| anyhow::anyhow!("Property not found"))?;
        
        let contacts = self.repository.get_all_active_contacts().await?;

        // Calculate recommendations in parallel
        let recommendations: Vec<Recommendation> = contacts
            .par_iter()
            .filter_map(|contact| {
                let recommendation = self.calculate_recommendation(&property, contact);
                if recommendation.score >= min_score.unwrap_or(0.0) {
                    Some(recommendation)
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (highest first) and limit results
        let mut sorted_recommendations = recommendations;
        sorted_recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        if let Some(limit) = limit {
            sorted_recommendations.truncate(limit);
        }

        // Cache the results
        self.cache.insert(cache_key, sorted_recommendations.clone()).await;

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(RecommendationResponse {
            total_count: sorted_recommendations.len(),
            recommendations: sorted_recommendations,
            processing_time_ms: processing_time,
        })
    }

    pub async fn get_bulk_recommendations(
        &self,
        request: BulkRecommendationRequest,
    ) -> Result<BulkRecommendationResponse> {
        let start_time = std::time::Instant::now();

        // Get properties (either specified ones or all active)
        let properties = if let Some(property_ids) = &request.property_ids {
            self.repository.get_properties_by_ids(property_ids).await?
        } else {
            self.repository.get_all_active_properties().await?
        };

        let contacts = self.repository.get_all_active_contacts().await?;

        // Process in parallel
        let property_recommendations: Vec<PropertyRecommendations> = properties
            .par_iter()
            .map(|property| {
                let mut recommendations: Vec<Recommendation> = contacts
                    .par_iter()
                    .filter_map(|contact| {
                        let recommendation = self.calculate_recommendation(property, contact);
                        if recommendation.score >= request.min_score.unwrap_or(0.0) {
                            Some(recommendation)
                        } else {
                            None
                        }
                    })
                    .collect();

                // Sort and limit
                recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
                if let Some(limit) = request.limit_per_property {
                    recommendations.truncate(limit);
                }

                PropertyRecommendations {
                    property_id: property.id,
                    property_title: property.title.clone(),
                    recommendation_count: recommendations.len(),
                    recommendations,
                }
            })
            .collect();

        let total_recommendations = property_recommendations
            .iter()
            .map(|pr| pr.recommendation_count)
            .sum();

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(BulkRecommendationResponse {
            total_properties: property_recommendations.len(),
            total_recommendations,
            processing_time_ms: processing_time,
            recommendations: property_recommendations,
        })
    }

    fn calculate_recommendation(&self, property: &Property, contact: &Contact) -> Recommendation {
        // Calculate individual scores
        let budget_score = calculate_budget_score(property.price, contact.budget_min, contact.budget_max);
        let location_score = calculate_location_score(property, contact);
        let property_type_score = calculate_property_type_score(property, contact);
        let size_score = calculate_size_score(property, contact);
        let (feature_score, required_features_met) = calculate_feature_score(property, contact);

        // If required features are not met, return low score
        if !required_features_met {
            return Recommendation {
                contact: contact.clone(),
                property: property.clone(),
                score: 0.0,
                explanation: RecommendationExplanation {
                    overall_score: 0.0,
                    budget_match: BudgetMatch {
                        is_within_budget: property.price >= contact.budget_min && property.price <= contact.budget_max,
                        budget_utilization: if contact.budget_max > contact.budget_min {
                            (property.price - contact.budget_min) as f64 / (contact.budget_max - contact.budget_min) as f64
                        } else {
                            1.0
                        },
                        score: budget_score,
                    },
                    location_match: LocationMatch {
                        distance_km: if !contact.preferred_locations.is_empty() {
                            contact.preferred_locations.iter()
                                .map(|loc| calculate_distance_km(&property.location, loc))
                                .fold(f64::INFINITY, f64::min)
                        } else {
                            0.0
                        },
                        is_preferred_location: contact.preferred_locations.iter()
                            .any(|loc| loc.city.to_lowercase() == property.location.city.to_lowercase()),
                        score: location_score,
                    },
                    property_type_match: contact.preferred_property_types.contains(&property.property_type),
                    size_match: SizeMatch {
                        rooms_match: contact.min_rooms.map_or(true, |min| property.rooms >= min) &&
                                   contact.max_rooms.map_or(true, |max| property.rooms <= max),
                        area_match: contact.min_area.map_or(true, |min| property.area_sqm >= min) &&
                                  contact.max_area.map_or(true, |max| property.area_sqm <= max),
                        rooms_score: size_score,
                        area_score: size_score,
                        overall_score: size_score,
                    },
                    feature_match: FeatureMatch {
                        required_features_met: false,
                        preferred_features_count: contact.preferred_features.iter()
                            .filter(|feature| property.features.contains(feature))
                            .count() as i32,
                        total_preferred_features: contact.preferred_features.len() as i32,
                        score: 0.0,
                        missing_required_features: contact.required_features.iter()
                            .filter(|feature| !property.features.contains(feature))
                            .cloned()
                            .collect(),
                        matched_preferred_features: contact.preferred_features.iter()
                            .filter(|feature| property.features.contains(feature))
                            .cloned()
                            .collect(),
                    },
                    reasons: vec!["Required features not met".to_string()],
                },
                created_at: Utc::now(),
            };
        }

        // Calculate overall score
        let overall_score = calculate_overall_score(
            budget_score,
            location_score,
            property_type_score,
            size_score,
            feature_score,
        );

        // Generate explanation reasons
        let mut reasons = Vec::new();
        
        if budget_score > 0.8 {
            reasons.push("Excellent budget match".to_string());
        } else if budget_score > 0.6 {
            reasons.push("Good budget fit".to_string());
        }

        if location_score > 0.8 {
            reasons.push("Perfect location match".to_string());
        } else if location_score > 0.6 {
            reasons.push("Good location proximity".to_string());
        }

        if property_type_score == 1.0 {
            reasons.push("Preferred property type".to_string());
        }

        if size_score > 0.8 {
            reasons.push("Ideal size requirements".to_string());
        }

        if feature_score > 0.8 {
            reasons.push("Excellent feature match".to_string());
        }

        if reasons.is_empty() {
            reasons.push("Meets basic criteria".to_string());
        }

        Recommendation {
            contact: contact.clone(),
            property: property.clone(),
            score: overall_score,
            explanation: RecommendationExplanation {
                overall_score,
                budget_match: BudgetMatch {
                    is_within_budget: property.price >= contact.budget_min && property.price <= contact.budget_max,
                    budget_utilization: if contact.budget_max > contact.budget_min {
                        (property.price - contact.budget_min) as f64 / (contact.budget_max - contact.budget_min) as f64
                    } else {
                        1.0
                    },
                    score: budget_score,
                },
                location_match: LocationMatch {
                    distance_km: if !contact.preferred_locations.is_empty() {
                        contact.preferred_locations.iter()
                            .map(|loc| calculate_distance_km(&property.location, loc))
                            .fold(f64::INFINITY, f64::min)
                    } else {
                        0.0
                    },
                    is_preferred_location: contact.preferred_locations.iter()
                        .any(|loc| loc.city.to_lowercase() == property.location.city.to_lowercase()),
                    score: location_score,
                },
                property_type_match: contact.preferred_property_types.contains(&property.property_type),
                size_match: SizeMatch {
                    rooms_match: contact.min_rooms.map_or(true, |min| property.rooms >= min) &&
                               contact.max_rooms.map_or(true, |max| property.rooms <= max),
                    area_match: contact.min_area.map_or(true, |min| property.area_sqm >= min) &&
                              contact.max_area.map_or(true, |max| property.area_sqm <= max),
                    rooms_score: size_score,
                    area_score: size_score,
                    overall_score: size_score,
                },
                feature_match: FeatureMatch {
                    required_features_met: true,
                    preferred_features_count: contact.preferred_features.iter()
                        .filter(|feature| property.features.contains(feature))
                        .count() as i32,
                    total_preferred_features: contact.preferred_features.len() as i32,
                    score: feature_score,
                    missing_required_features: Vec::new(),
                    matched_preferred_features: contact.preferred_features.iter()
                        .filter(|feature| property.features.contains(feature))
                        .cloned()
                        .collect(),
                },
                reasons,
            },
            created_at: Utc::now(),
        }
    }
}
