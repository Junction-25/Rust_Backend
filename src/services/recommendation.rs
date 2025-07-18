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
        property_id: i32,
        limit: Option<usize>,
        min_score: Option<f64>,
        top_k: Option<usize>,
        top_percentile: Option<f64>,
        score_threshold_percentile: Option<f64>,
    ) -> Result<RecommendationResponse> {
        let start_time = std::time::Instant::now();
        
        // Check cache first
        let cache_key = format!(
            "property_{}_{:?}_{:?}_{:?}_{:?}_{:?}", 
            property_id, limit, min_score, top_k, top_percentile, score_threshold_percentile
        );
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
        let mut all_recommendations: Vec<Recommendation> = contacts
            .par_iter()
            .map(|contact| self.calculate_recommendation(contact, &property))
            .collect();

        // Sort by score (highest first) first for percentile calculations
        all_recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Apply advanced filtering
        let filtered_recommendations = self.apply_advanced_filters(
            all_recommendations,
            min_score,
            top_k,
            top_percentile,
            score_threshold_percentile,
            limit,
        );

        let final_recommendations = filtered_recommendations;

        // Cache the results
        self.cache.insert(cache_key, final_recommendations.clone()).await;

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(RecommendationResponse {
            total_count: final_recommendations.len(),
            recommendations: final_recommendations,
            processing_time_ms: processing_time,
        })
    }

    pub async fn get_recommendations_for_contact(
        &self,
        contact_id: i32,
        limit: Option<usize>,
        min_score: Option<f64>,
        top_k: Option<usize>,
        top_percentile: Option<f64>,
        score_threshold_percentile: Option<f64>,
    ) -> Result<RecommendationResponse> {
        let start_time = std::time::Instant::now();
        
        // Check cache first
        let cache_key = format!(
            "contact_{}_{:?}_{:?}_{:?}_{:?}_{:?}", 
            contact_id, limit, min_score, top_k, top_percentile, score_threshold_percentile
        );
        if let Some(cached_recommendations) = self.cache.get(&cache_key).await {
            return Ok(RecommendationResponse {
                recommendations: cached_recommendations.clone(),
                total_count: cached_recommendations.len(),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            });
        }

        // Get contact and properties
        let contact = self.repository.get_contact_by_id(contact_id).await?
            .ok_or_else(|| anyhow::anyhow!("Contact not found"))?;
        
        let properties = self.repository.get_all_active_properties().await?;

        // Calculate recommendations in parallel
        let mut all_recommendations: Vec<Recommendation> = properties
            .par_iter()
            .map(|property| self.calculate_recommendation(&contact, property))
            .collect();

        // Sort by score (highest first) first for percentile calculations
        all_recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Apply advanced filtering
        let final_recommendations = self.apply_advanced_filters(
            all_recommendations,
            min_score,
            top_k,
            top_percentile,
            score_threshold_percentile,
            limit,
        );

        // Cache the results
        self.cache.insert(cache_key, final_recommendations.clone()).await;

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(RecommendationResponse {
            total_count: final_recommendations.len(),
            recommendations: final_recommendations,
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
            let mut result = Vec::new();
            for &id in property_ids {
                if let Some(property) = self.repository.get_property_by_id(id).await? {
                    result.push(property);
                }
            }
            result
        } else {
            self.repository.get_all_active_properties().await?
        };

        let contacts = self.repository.get_all_active_contacts().await?;

        // Process in parallel
        let property_recommendations: Vec<PropertyRecommendations> = properties
            .par_iter()
            .map(|property| {
                let mut all_recommendations: Vec<Recommendation> = contacts
                    .par_iter()
                    .map(|contact| self.calculate_recommendation(contact, property))
                    .collect();

                // Sort by score (highest first) first for percentile calculations
                all_recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

                // Apply advanced filtering
                let filtered_recommendations = self.apply_advanced_filters(
                    all_recommendations,
                    request.min_score,
                    request.top_k,
                    request.top_percentile,
                    request.score_threshold_percentile,
                    request.limit_per_property,
                );

                PropertyRecommendations {
                    property_id: property.id,
                    property_address: property.address.clone(),
                    recommendation_count: filtered_recommendations.len(),
                    recommendations: filtered_recommendations,
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

    fn apply_advanced_filters(
        &self,
        mut recommendations: Vec<Recommendation>,
        min_score: Option<f64>,
        top_k: Option<usize>,
        top_percentile: Option<f64>,
        score_threshold_percentile: Option<f64>,
        limit: Option<usize>,
    ) -> Vec<Recommendation> {
        // Step 1: Apply minimum score filter
        if let Some(min_score) = min_score {
            recommendations.retain(|r| r.score >= min_score);
        }

        // Step 2: Apply score threshold percentile filter
        if let Some(percentile) = score_threshold_percentile {
            if !recommendations.is_empty() {
                let threshold_index = ((1.0 - percentile) * recommendations.len() as f64).floor() as usize;
                if threshold_index < recommendations.len() {
                    let threshold_score = recommendations[threshold_index].score;
                    recommendations.retain(|r| r.score >= threshold_score);
                }
            }
        }

        // Step 3: Apply top percentile filter
        if let Some(percentile) = top_percentile {
            if !recommendations.is_empty() {
                let keep_count = (percentile * recommendations.len() as f64).ceil() as usize;
                recommendations.truncate(keep_count.min(recommendations.len()));
            }
        }

        // Step 4: Apply top K filter
        if let Some(k) = top_k {
            recommendations.truncate(k);
        }

        // Step 5: Apply final limit (for backward compatibility)
        if let Some(limit) = limit {
            recommendations.truncate(limit);
        }

        recommendations
    }

    fn calculate_recommendation(&self, contact: &Contact, property: &Property) -> Recommendation {
        // Calculate individual scores
        let budget_score = calculate_budget_score(property.price, contact.min_budget, contact.max_budget);
        let location_score = calculate_location_score(property, contact);
        let property_type_score = calculate_property_type_score(property, contact);
        let size_score = calculate_size_score(property, contact);

        // Calculate overall score
        let overall_score = calculate_overall_score(
            budget_score,
            location_score,
            property_type_score,
            size_score,
        );

        // Calculate closest distance to preferred locations
        let min_distance = if !contact.preferred_locations.is_empty() {
            contact.preferred_locations.iter()
                .map(|loc| calculate_distance_km(
                    property.location.lat, 
                    property.location.lon, 
                    loc.lat, 
                    loc.lon
                ))
                .fold(f64::INFINITY, f64::min)
        } else {
            0.0
        };

        // Generate explanation reasons
        let mut reasons = Vec::new();
        
        if budget_score > 0.8 {
            reasons.push("Excellent budget match".to_string());
        } else if budget_score > 0.6 {
            reasons.push("Good budget fit".to_string());
        } else if budget_score < 0.3 {
            reasons.push("Budget concerns".to_string());
        }

        if location_score > 0.8 {
            reasons.push("Perfect location match".to_string());
        } else if location_score > 0.6 {
            reasons.push("Good location proximity".to_string());
        } else if location_score < 0.3 {
            reasons.push("Location may be distant".to_string());
        }

        if property_type_score == 1.0 {
            reasons.push("Preferred property type".to_string());
        } else if property_type_score == 0.0 {
            reasons.push("Different property type".to_string());
        }

        if size_score > 0.8 {
            reasons.push("Ideal size requirements".to_string());
        } else if size_score < 0.3 {
            reasons.push("Size concerns".to_string());
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
                    is_within_budget: property.price >= contact.min_budget && property.price <= contact.max_budget,
                    budget_utilization: if contact.max_budget > contact.min_budget {
                        (property.price - contact.min_budget) / (contact.max_budget - contact.min_budget)
                    } else {
                        1.0
                    },
                    score: budget_score,
                },
                location_match: LocationMatch {
                    distance_km: min_distance,
                    is_preferred_location: min_distance <= 15.0, // Within 15km is considered preferred
                    score: location_score,
                },
                property_type_match: contact.property_types.contains(&property.property_type),
                size_match: SizeMatch {
                    rooms_match: property.number_of_rooms >= contact.min_rooms,
                    area_match: property.area_sqm >= contact.min_area_sqm && property.area_sqm <= contact.max_area_sqm,
                    score: size_score,
                },
                reasons,
            },
            created_at: Utc::now(),
        }
    }
}
