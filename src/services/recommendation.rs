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

    pub async fn get_recommendations_for_contact(
        &self,
        contact_id: i32,
        limit: Option<usize>,
        min_score: Option<f64>,
    ) -> Result<RecommendationResponse> {
        let start_time = std::time::Instant::now();
        
        // Check cache first
        let cache_key = format!("contact_{}_{:?}_{:?}", contact_id, limit, min_score);
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
        let recommendations: Vec<Recommendation> = properties
            .par_iter()
            .filter_map(|property| {
                let recommendation = self.calculate_recommendation(&contact, property);
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

        // Get contacts (either specified ones or all active)
        let contacts = if let Some(contact_ids) = &request.contact_ids {
            let mut result = Vec::new();
            for &id in contact_ids {
                if let Some(contact) = self.repository.get_contact_by_id(id).await? {
                    result.push(contact);
                }
            }
            result
        } else {
            self.repository.get_all_active_contacts().await?
        };

        let properties = self.repository.get_all_active_properties().await?;

        // Process in parallel
        let contact_recommendations: Vec<ContactRecommendations> = contacts
            .par_iter()
            .map(|contact| {
                let mut recommendations: Vec<Recommendation> = properties
                    .par_iter()
                    .filter_map(|property| {
                        let recommendation = self.calculate_recommendation(contact, property);
                        if recommendation.score >= request.min_score.unwrap_or(0.0) {
                            Some(recommendation)
                        } else {
                            None
                        }
                    })
                    .collect();

                // Sort and limit
                recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
                if let Some(limit) = request.limit_per_contact {
                    recommendations.truncate(limit);
                }

                ContactRecommendations {
                    contact_id: contact.id,
                    contact_name: contact.name.clone(),
                    recommendation_count: recommendations.len(),
                    recommendations,
                }
            })
            .collect();

        let total_recommendations = contact_recommendations
            .iter()
            .map(|cr| cr.recommendation_count)
            .sum();

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(BulkRecommendationResponse {
            total_contacts: contact_recommendations.len(),
            total_recommendations,
            processing_time_ms: processing_time,
            recommendations: contact_recommendations,
        })
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
