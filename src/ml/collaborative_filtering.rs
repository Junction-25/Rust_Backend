use crate::models::{Contact, Property, Recommendation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPropertyMatrix {
    pub users: Vec<i32>,                          // Contact IDs
    pub properties: Vec<i32>,                     // Property IDs
    pub ratings: Vec<Vec<f64>>,                   // User-Property interaction matrix
    pub confidence: Vec<Vec<f64>>,                // Confidence scores for implicit feedback
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLRecommendation {
    pub property_id: i32,
    pub contact_id: i32,
    pub ml_score: f64,
    pub traditional_score: f64,
    pub hybrid_score: f64,
    pub prediction_confidence: f64,
    pub recommendation_type: RecommendationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    ContentBased,
    CollaborativeFiltering,
    Hybrid,
    ColdStart,
}

#[derive(Clone)]
pub struct CollaborativeFilteringEngine {
    user_item_matrix: Option<UserPropertyMatrix>,
    user_features: HashMap<i32, Vec<f64>>,
    item_features: HashMap<i32, Vec<f64>>,
    similarity_cache: HashMap<(i32, i32), f64>,
}

impl CollaborativeFilteringEngine {
    pub fn new() -> Self {
        Self {
            user_item_matrix: None,
            user_features: HashMap::new(),
            item_features: HashMap::new(),
            similarity_cache: HashMap::new(),
        }
    }

    /// Build user-item matrix from historical recommendations/interactions
    pub fn build_matrix_from_recommendations(&mut self, recommendations: &[Recommendation]) -> anyhow::Result<()> {
        let mut user_ids: Vec<i32> = recommendations.iter().map(|r| r.contact.id).collect();
        user_ids.sort_unstable();
        user_ids.dedup();

        let mut property_ids: Vec<i32> = recommendations.iter().map(|r| r.property.id).collect();
        property_ids.sort_unstable();
        property_ids.dedup();

        let n_users = user_ids.len();
        let n_items = property_ids.len();

        let mut ratings = vec![vec![0.0; n_items]; n_users];
        let mut confidence = vec![vec![0.0; n_items]; n_users];

        // Create lookup maps for efficient indexing
        let user_idx_map: HashMap<i32, usize> = user_ids.iter().enumerate().map(|(i, &id)| (id, i)).collect();
        let item_idx_map: HashMap<i32, usize> = property_ids.iter().enumerate().map(|(i, &id)| (id, i)).collect();

        // Fill the matrix with recommendation scores as implicit feedback
        for rec in recommendations {
            if let (Some(&user_idx), Some(&item_idx)) = (
                user_idx_map.get(&rec.contact.id),
                item_idx_map.get(&rec.property.id)
            ) {
                ratings[user_idx][item_idx] = rec.score;
                // Higher scores get higher confidence
                confidence[user_idx][item_idx] = rec.score * rec.score; // Quadratic confidence
            }
        }

        self.user_item_matrix = Some(UserPropertyMatrix {
            users: user_ids,
            properties: property_ids,
            ratings,
            confidence,
        });

        Ok(())
    }

    /// Extract user features based on preferences and behavior
    pub fn extract_user_features(&mut self, contacts: &[Contact]) {
        for contact in contacts {
            let mut features = Vec::new();
            
            // Budget features (normalized)
            let budget_mid = (contact.min_budget + contact.max_budget) / 2.0;
            let budget_range = contact.max_budget - contact.min_budget;
            features.push(budget_mid / 50_000_000.0); // Normalize by 50M DZD
            features.push(budget_range / 50_000_000.0);
            
            // Area preferences
            let area_mid = (contact.min_area_sqm + contact.max_area_sqm) as f64 / 2.0;
            let area_range = (contact.max_area_sqm - contact.min_area_sqm) as f64;
            features.push(area_mid / 200.0); // Normalize by 200 sqm
            features.push(area_range / 200.0);
            
            // Room requirements
            features.push(contact.min_rooms as f64 / 5.0); // Normalize by 5 rooms
            
            // Property type preferences (one-hot encoding)
            let property_types = &["apartment", "house", "land", "office", "commercial"];
            for ptype in property_types {
                features.push(if contact.property_types.contains(&ptype.to_string()) { 1.0 } else { 0.0 });
            }
            
            // Location diversity (number of preferred locations)
            features.push(contact.preferred_locations.len() as f64 / 5.0); // Normalize by 5 locations
            
            self.user_features.insert(contact.id, features);
        }
    }

    /// Extract property features
    pub fn extract_item_features(&mut self, properties: &[Property]) {
        for property in properties {
            let mut features = Vec::new();
            
            // Price feature (normalized)
            features.push(property.price / 50_000_000.0); // Normalize by 50M DZD
            
            // Area feature
            features.push(property.area_sqm as f64 / 200.0); // Normalize by 200 sqm
            
            // Room count
            features.push(property.number_of_rooms as f64 / 5.0); // Normalize by 5 rooms
            
            // Property type (one-hot encoding)
            let property_types = &["apartment", "house", "land", "office", "commercial"];
            for ptype in property_types {
                features.push(if property.property_type == *ptype { 1.0 } else { 0.0 });
            }
            
            // Location features (normalized coordinates)
            features.push((property.location.lat + 90.0) / 180.0); // Normalize latitude
            features.push((property.location.lon + 180.0) / 360.0); // Normalize longitude
            
            self.item_features.insert(property.id, features);
        }
    }

    /// Calculate user-user similarity using cosine similarity
    pub fn calculate_user_similarity(&mut self, user1_id: i32, user2_id: i32) -> f64 {
        if let Some(cached) = self.similarity_cache.get(&(user1_id.min(user2_id), user1_id.max(user2_id))) {
            return *cached;
        }

        let similarity = if let (Some(features1), Some(features2)) = (
            self.user_features.get(&user1_id),
            self.user_features.get(&user2_id)
        ) {
            self.cosine_similarity(features1, features2)
        } else {
            0.0
        };

        self.similarity_cache.insert((user1_id.min(user2_id), user1_id.max(user2_id)), similarity);
        similarity
    }

    /// Calculate item-item similarity
    pub fn calculate_item_similarity(&self, item1_id: i32, item2_id: i32) -> f64 {
        if let (Some(features1), Some(features2)) = (
            self.item_features.get(&item1_id),
            self.item_features.get(&item2_id)
        ) {
            self.cosine_similarity(features1, features2)
        } else {
            0.0
        }
    }

    /// Cosine similarity calculation
    fn cosine_similarity(&self, vec1: &[f64], vec2: &[f64]) -> f64 {
        if vec1.len() != vec2.len() {
            return 0.0;
        }

        let dot_product: f64 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f64 = vec1.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm2: f64 = vec2.iter().map(|x| x * x).sum::<f64>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1 * norm2)
        }
    }

    /// Predict rating using collaborative filtering
    pub fn predict_user_item_rating(&mut self, user_id: i32, item_id: i32, k_neighbors: usize) -> (f64, f64) {
        if let Some(ref matrix) = self.user_item_matrix {
            // Find user and item indices
            let user_idx = matrix.users.iter().position(|&id| id == user_id);
            let item_idx = matrix.properties.iter().position(|&id| id == item_id);

            if let (Some(user_idx), Some(item_idx)) = (user_idx, item_idx) {
                // User-based collaborative filtering
                let mut similarities_and_ratings = Vec::new();

                for (other_user_idx, &other_user_id) in matrix.users.iter().enumerate() {
                    if other_user_idx != user_idx && matrix.ratings[other_user_idx][item_idx] > 0.0 {
                        // Use cached similarity or calculate basic similarity without caching
                        let similarity = if let Some(&cached_sim) = self.similarity_cache.get(&(user_id.min(other_user_id), user_id.max(other_user_id))) {
                            cached_sim
                        } else {
                            // Basic similarity without caching during prediction
                            if let (Some(features1), Some(features2)) = (
                                self.user_features.get(&user_id),
                                self.user_features.get(&other_user_id)
                            ) {
                                self.cosine_similarity(features1, features2)
                            } else {
                                0.0
                            }
                        };
                        
                        if similarity > 0.1 { // Minimum similarity threshold
                            similarities_and_ratings.push((
                                similarity,
                                matrix.ratings[other_user_idx][item_idx],
                                matrix.confidence[other_user_idx][item_idx]
                            ));
                        }
                    }
                }

                // Sort by similarity and take top k
                similarities_and_ratings.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
                similarities_and_ratings.truncate(k_neighbors);

                if !similarities_and_ratings.is_empty() {
                    let weighted_sum: f64 = similarities_and_ratings
                        .iter()
                        .map(|(sim, rating, conf)| sim * rating * conf)
                        .sum();
                    let weight_sum: f64 = similarities_and_ratings
                        .iter()
                        .map(|(sim, _, conf)| sim * conf)
                        .sum();

                    if weight_sum > 0.0 {
                        let prediction = weighted_sum / weight_sum;
                        let confidence = weight_sum / similarities_and_ratings.len() as f64;
                        return (prediction, confidence);
                    }
                }
            }
        }

        // Fallback to content-based prediction
        self.content_based_prediction(user_id, item_id)
    }

    /// Content-based prediction fallback
    fn content_based_prediction(&self, user_id: i32, item_id: i32) -> (f64, f64) {
        if let (Some(user_features), Some(item_features)) = (
            self.user_features.get(&user_id),
            self.item_features.get(&item_id)
        ) {
            // Calculate compatibility based on feature similarity
            let similarity = self.cosine_similarity(user_features, item_features);
            (similarity * 0.7, 0.3) // Lower confidence for content-based
        } else {
            (0.5, 0.1) // Very low confidence default
        }
    }

    /// Generate ML-enhanced recommendations
    pub fn generate_ml_recommendations(
        &mut self,
        user_id: i32,
        candidate_items: &[i32],
        traditional_scores: &HashMap<i32, f64>,
        k_neighbors: usize,
    ) -> Vec<MLRecommendation> {
        let mut ml_recommendations = Vec::new();

        for &item_id in candidate_items {
            let (ml_score, confidence) = self.predict_user_item_rating(user_id, item_id, k_neighbors);
            let traditional_score = traditional_scores.get(&item_id).copied().unwrap_or(0.0);
            
            // Hybrid scoring: combine ML and traditional scores
            let hybrid_score = if confidence > 0.5 {
                0.6 * ml_score + 0.4 * traditional_score // Trust ML more if confident
            } else {
                0.3 * ml_score + 0.7 * traditional_score // Trust traditional more if uncertain
            };

            let recommendation_type = if confidence > 0.7 {
                RecommendationType::CollaborativeFiltering
            } else if confidence > 0.3 {
                RecommendationType::Hybrid
            } else {
                RecommendationType::ContentBased
            };

            ml_recommendations.push(MLRecommendation {
                property_id: item_id,
                contact_id: user_id,
                ml_score,
                traditional_score,
                hybrid_score,
                prediction_confidence: confidence,
                recommendation_type,
            });
        }

        // Sort by hybrid score
        ml_recommendations.sort_by(|a, b| b.hybrid_score.partial_cmp(&a.hybrid_score).unwrap());
        ml_recommendations
    }

    /// Update the model with new feedback
    pub fn update_with_feedback(&mut self, user_id: i32, item_id: i32, rating: f64, confidence: f64) -> anyhow::Result<()> {
        if let Some(ref mut matrix) = self.user_item_matrix {
            if let (Some(user_idx), Some(item_idx)) = (
                matrix.users.iter().position(|&id| id == user_id),
                matrix.properties.iter().position(|&id| id == item_id)
            ) {
                // Update rating with exponential moving average
                let alpha = 0.3; // Learning rate
                let current_rating = matrix.ratings[user_idx][item_idx];
                matrix.ratings[user_idx][item_idx] = alpha * rating + (1.0 - alpha) * current_rating;
                matrix.confidence[user_idx][item_idx] = confidence;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let engine = CollaborativeFilteringEngine::new();
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        assert!((engine.cosine_similarity(&vec1, &vec2) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_feature_extraction() {
        let mut engine = CollaborativeFilteringEngine::new();
        let contacts = vec![]; // Add test contacts
        engine.extract_user_features(&contacts);
        // Add assertions
    }
}
