use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use serde::{Deserialize, Serialize};
use crate::models::{Property, Contact, Recommendation};
use crate::models::recommendation::{RecommendationExplanation, BudgetMatch, LocationMatch, SizeMatch};
use crate::utils::feature_store::{FeatureStore, ContactFeatures};
use crate::utils::scoring::calculate_neural_enhanced_score;

/// Two-stage retrieval system: Fast ANN retrieval + precise re-ranking
/// Stage 1: HNSW-based approximate nearest neighbor search for candidate generation
/// Stage 2: Neural-enhanced precise scoring and ranking of top candidates
#[derive(Clone)]
pub struct TwoStageRetrievalEngine {
    feature_store: Arc<FeatureStore>,
    ann_index: Arc<RwLock<ANNIndex>>,
    config: RetrievalConfig,
    stats: Arc<RwLock<RetrievalStats>>,
}

#[derive(Debug, Clone)]
pub struct RetrievalConfig {
    pub stage1_candidates: usize,    // Number of candidates from ANN search
    pub stage2_top_k: usize,         // Final number of recommendations
    pub ann_ef_construction: usize,  // HNSW construction parameter
    pub ann_ef_search: usize,        // HNSW search parameter
    pub ann_max_connections: usize,  // HNSW M parameter
    pub similarity_threshold: f32,   // Minimum similarity for candidates
    pub rerank_enabled: bool,        // Enable stage 2 re-ranking
    pub use_location_filtering: bool, // Pre-filter by location
    pub max_distance_km: f64,        // Maximum distance for location filtering
}

#[derive(Debug, Clone)]
pub struct RetrievalStats {
    pub total_searches: u64,
    pub stage1_avg_time_ms: f64,
    pub stage2_avg_time_ms: f64,
    pub total_avg_time_ms: f64,
    pub cache_hit_rate: f64,
    pub index_size: usize,
    pub last_index_rebuild: Instant,
}

/// Hierarchical Navigable Small World (HNSW) index for fast similarity search
pub struct ANNIndex {
    /// Property ID to embedding mapping
    embeddings: HashMap<i32, Vec<f32>>,
    /// HNSW graph structure: property_id -> Vec<(neighbor_id, distance)>
    graph: HashMap<i32, Vec<(i32, f32)>>,
    /// Multi-level structure: level -> property_ids
    levels: Vec<HashSet<i32>>,
    /// Entry point for search
    entry_point: Option<i32>,
    /// Index configuration
    config: ANNIndexConfig,
    /// Index metadata
    metadata: ANNIndexMetadata,
}

#[derive(Debug, Clone)]
pub struct ANNIndexConfig {
    pub embedding_dim: usize,
    pub max_connections: usize,    // M parameter
    pub max_connections_0: usize,  // M parameter for layer 0
    pub ef_construction: usize,    // ef parameter during construction
    pub level_multiplier: f64,     // ml parameter
}

#[derive(Debug, Clone)]
pub struct ANNIndexMetadata {
    pub total_properties: usize,
    pub total_connections: usize,
    pub avg_connections_per_node: f64,
    pub max_level: usize,
    pub build_time_ms: u64,
    pub memory_usage_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub contact_id: i32,
    pub stage1_candidates: Vec<CandidateProperty>,
    pub stage2_recommendations: Vec<Recommendation>,
    pub stage1_time_ms: f64,
    pub stage2_time_ms: f64,
    pub total_time_ms: f64,
    pub cache_hits: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateProperty {
    pub property_id: i32,
    pub similarity_score: f32,
    pub distance: f32,
    pub source: CandidateSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CandidateSource {
    ANN,          // From ANN search
    LocationFilter, // From location-based filtering  
    Cache,        // From similarity cache
    Fallback,     // From fallback search
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            stage1_candidates: 100,
            stage2_top_k: 20,
            ann_ef_construction: 200,
            ann_ef_search: 50,
            ann_max_connections: 16,
            similarity_threshold: 0.1,
            rerank_enabled: true,
            use_location_filtering: true,
            max_distance_km: 50.0,
        }
    }
}

impl Default for ANNIndexConfig {
    fn default() -> Self {
        Self {
            embedding_dim: 128,
            max_connections: 16,
            max_connections_0: 32,
            ef_construction: 200,
            level_multiplier: 1.0 / (2.0_f64).ln(),
        }
    }
}

impl TwoStageRetrievalEngine {
    pub fn new(feature_store: Arc<FeatureStore>, config: RetrievalConfig) -> Self {
        let ann_config = ANNIndexConfig::default();
        let ann_index = ANNIndex::new(ann_config);
        
        Self {
            feature_store,
            ann_index: Arc::new(RwLock::new(ann_index)),
            config,
            stats: Arc::new(RwLock::new(RetrievalStats {
                total_searches: 0,
                stage1_avg_time_ms: 0.0,
                stage2_avg_time_ms: 0.0,
                total_avg_time_ms: 0.0,
                cache_hit_rate: 0.0,
                index_size: 0,
                last_index_rebuild: Instant::now(),
            })),
        }
    }

    /// Build or rebuild the ANN index from feature store
    pub async fn build_index(&self) -> anyhow::Result<()> {
        let start = Instant::now();
        
        // Get all property embeddings from feature store
        let embeddings = self.feature_store.get_all_property_embeddings();
        
        if embeddings.is_empty() {
            return Err(anyhow::anyhow!("No property embeddings found in feature store"));
        }
        
        // Build HNSW index
        let mut index = self.ann_index.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on ANN index"))?;
        
        index.build_from_embeddings(embeddings)?;
        
        // Update stats
        if let Ok(mut stats) = self.stats.write() {
            stats.index_size = index.metadata.total_properties;
            stats.last_index_rebuild = Instant::now();
        }
        
        let build_time = start.elapsed();
        log::info!("ANN index built in {:.2}ms with {} properties", 
                  build_time.as_millis(), index.metadata.total_properties);
        
        Ok(())
    }

    /// Two-stage retrieval: ANN candidates + neural re-ranking
    pub async fn retrieve_recommendations(
        &self,
        contact_id: i32,
        properties: &[Property],
    ) -> anyhow::Result<RetrievalResult> {
        let start_total = Instant::now();
        let mut cache_hits = 0;

        // Get contact features
        let contact_features = self.feature_store.get_contact_features(contact_id)
            .ok_or_else(|| anyhow::anyhow!("Contact features not found for ID: {}", contact_id))?;

        // === STAGE 1: Fast ANN Candidate Generation ===
        let start_stage1 = Instant::now();
        
        let mut candidates = Vec::new();
        
        // 1.1 Location-based pre-filtering (if enabled)
        if self.config.use_location_filtering {
            let location_candidates = self.location_based_filtering(&contact_features, properties)?;
            candidates.extend(location_candidates);
        }
        
        // 1.2 ANN search for similar embeddings
        let ann_candidates = self.ann_search(&contact_features).await?;
        candidates.extend(ann_candidates);
        
        // 1.3 Check similarity cache for cached scores
        for property in properties.iter().take(self.config.stage1_candidates) {
            if let Some(cached_score) = self.feature_store.get_cached_similarity(contact_id, property.id) {
                candidates.push(CandidateProperty {
                    property_id: property.id,
                    similarity_score: cached_score,
                    distance: 1.0 - cached_score,
                    source: CandidateSource::Cache,
                });
                cache_hits += 1;
            }
        }
        
        // Deduplicate and sort candidates
        candidates = self.deduplicate_candidates(candidates);
        candidates.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        candidates.truncate(self.config.stage1_candidates);
        
        let stage1_time = start_stage1.elapsed();

        // === STAGE 2: Neural-Enhanced Re-ranking ===
        let start_stage2 = Instant::now();
        
        let recommendations = if self.config.rerank_enabled {
            self.neural_reranking(contact_id, &candidates, properties).await?
        } else {
            // Convert candidates directly to recommendations
            self.candidates_to_recommendations(&candidates, properties)?
        };
        
        let stage2_time = start_stage2.elapsed();
        let total_time = start_total.elapsed();

        // Update stats
        self.update_stats(stage1_time, stage2_time, total_time, cache_hits).await?;

        Ok(RetrievalResult {
            contact_id,
            stage1_candidates: candidates,
            stage2_recommendations: recommendations,
            stage1_time_ms: stage1_time.as_millis() as f64,
            stage2_time_ms: stage2_time.as_millis() as f64,
            total_time_ms: total_time.as_millis() as f64,
            cache_hits,
        })
    }

    /// Location-based candidate filtering
    fn location_based_filtering(
        &self,
        contact_features: &ContactFeatures,
        properties: &[Property],
    ) -> anyhow::Result<Vec<CandidateProperty>> {
        let mut candidates = Vec::new();
        
        for property in properties {
            for location_pref in &contact_features.location_preferences {
                let distance_km = self.calculate_distance(
                    location_pref.lat, location_pref.lon,
                    property.location.lat as f32, property.location.lon as f32,
                );
                
                if distance_km <= location_pref.radius_km && distance_km <= self.config.max_distance_km as f32 {
                    let similarity = 1.0 - (distance_km / location_pref.radius_km) * location_pref.weight;
                    
                    candidates.push(CandidateProperty {
                        property_id: property.id,
                        similarity_score: similarity.max(0.0),
                        distance: distance_km,
                        source: CandidateSource::LocationFilter,
                    });
                }
            }
        }
        
        Ok(candidates)
    }

    /// ANN search using HNSW index
    async fn ann_search(&self, contact_features: &ContactFeatures) -> anyhow::Result<Vec<CandidateProperty>> {
        let index = self.ann_index.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on ANN index"))?;
        
        let candidates = index.search(
            &contact_features.embedding,
            self.config.stage1_candidates,
            self.config.ann_ef_search,
        )?;
        
        Ok(candidates.into_iter()
            .filter(|c| c.similarity_score >= self.config.similarity_threshold)
            .map(|mut c| {
                c.source = CandidateSource::ANN;
                c
            })
            .collect())
    }

    /// Neural-enhanced re-ranking of candidates
    async fn neural_reranking(
        &self,
        contact_id: i32,
        candidates: &[CandidateProperty],
        properties: &[Property],
    ) -> anyhow::Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();
        
        // Get contact for detailed scoring
        // For now, create a dummy contact - in real implementation, fetch from DB
        let contact = Contact {
            id: contact_id,
            name: format!("Contact {}", contact_id),
            preferred_locations: vec![],
            min_budget: 0.0,
            max_budget: f64::MAX,
            min_area_sqm: 0,
            max_area_sqm: i32::MAX,
            property_types: vec!["apartment".to_string()], // Default
            min_rooms: 0,
        };
        
        for candidate in candidates.iter().take(self.config.stage2_top_k) {
            if let Some(property) = properties.iter().find(|p| p.id == candidate.property_id) {
                // For now, use a simplified scoring approach
                // In production, this would use the full neural enhanced scoring
                let score = candidate.similarity_score as f64;
                
                // Cache the similarity score
                let _ = self.feature_store.cache_similarity(contact_id, property.id, score as f32);
                
                // Create explanation with available fields from scoring module
                let explanation = RecommendationExplanation {
                    overall_score: score,
                    budget_match: BudgetMatch {
                        is_within_budget: true,
                        budget_utilization: 0.5,
                        score: 0.5,
                    },
                    location_match: LocationMatch {
                        distance_km: 0.0,
                        is_preferred_location: true,
                        score: score,
                    },
                    property_type_match: true,
                    size_match: SizeMatch {
                        rooms_match: true,
                        area_match: true,
                        score: 0.5,
                    },
                    reasons: vec!["Neural enhanced similarity match".to_string()],
                };
                
                recommendations.push(Recommendation {
                    contact: contact.clone(),
                    property: property.clone(),
                    score,
                    explanation,
                    created_at: chrono::Utc::now(),
                });
            }
        }
        
        // Sort by score descending
        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        recommendations.truncate(self.config.stage2_top_k);
        
        Ok(recommendations)
    }

    /// Convert candidates to basic recommendations (no re-ranking)
    fn candidates_to_recommendations(
        &self,
        candidates: &[CandidateProperty],
        properties: &[Property],
    ) -> anyhow::Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();
        
        for candidate in candidates.iter().take(self.config.stage2_top_k) {
            if let Some(property) = properties.iter().find(|p| p.id == candidate.property_id) {
                // Create basic recommendation with ANN similarity as score
                let recommendation = Recommendation {
                    contact: Contact {
                        id: 0,
                        name: "Unknown".to_string(),
                        preferred_locations: vec![],
                        min_budget: 0.0,
                        max_budget: f64::MAX,
                        min_area_sqm: 0,
                        max_area_sqm: i32::MAX,
                        property_types: vec![],
                        min_rooms: 0,
                    },
                    property: property.clone(),
                    score: candidate.similarity_score as f64,
                    explanation: RecommendationExplanation {
                        overall_score: candidate.similarity_score as f64,
                        budget_match: BudgetMatch {
                            is_within_budget: true,
                            budget_utilization: 0.5,
                            score: 0.5,
                        },
                        location_match: LocationMatch {
                            distance_km: candidate.distance as f64,
                            is_preferred_location: true,
                            score: candidate.similarity_score as f64,
                        },
                        property_type_match: true,
                        size_match: SizeMatch {
                            rooms_match: true,
                            area_match: true,
                            score: 0.5,
                        },
                        reasons: vec!["ANN similarity match".to_string()],
                    },
                    created_at: chrono::Utc::now(),
                };
                
                recommendations.push(recommendation);
            }
        }
        
        Ok(recommendations)
    }

    /// Deduplicate candidates by property_id, keeping highest similarity
    fn deduplicate_candidates(&self, candidates: Vec<CandidateProperty>) -> Vec<CandidateProperty> {
        let mut best_candidates: HashMap<i32, CandidateProperty> = HashMap::new();
        
        for candidate in candidates {
            let entry = best_candidates.entry(candidate.property_id);
            entry.and_modify(|existing| {
                if candidate.similarity_score > existing.similarity_score {
                    *existing = candidate.clone();
                }
            }).or_insert(candidate);
        }
        
        best_candidates.into_values().collect()
    }

    /// Calculate distance between two points (haversine formula)
    fn calculate_distance(&self, lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> f32 {
        let r = 6371.0; // Earth's radius in km
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();
        
        let a = (d_lat / 2.0).sin().powi(2) + 
                lat1.to_radians().cos() * lat2.to_radians().cos() * 
                (d_lon / 2.0).sin().powi(2);
        
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        r * c
    }

    /// Update retrieval statistics
    async fn update_stats(
        &self,
        stage1_time: std::time::Duration,
        stage2_time: std::time::Duration,
        total_time: std::time::Duration,
        cache_hits: usize,
    ) -> anyhow::Result<()> {
        if let Ok(mut stats) = self.stats.write() {
            stats.total_searches += 1;
            let n = stats.total_searches as f64;
            
            // Update running averages
            let stage1_ms = stage1_time.as_millis() as f64;
            let stage2_ms = stage2_time.as_millis() as f64;
            let total_ms = total_time.as_millis() as f64;
            
            stats.stage1_avg_time_ms = (stats.stage1_avg_time_ms * (n - 1.0) + stage1_ms) / n;
            stats.stage2_avg_time_ms = (stats.stage2_avg_time_ms * (n - 1.0) + stage2_ms) / n;
            stats.total_avg_time_ms = (stats.total_avg_time_ms * (n - 1.0) + total_ms) / n;
            
            // Update cache hit rate
            stats.cache_hit_rate = cache_hits as f64 / n;
        }
        
        Ok(())
    }

    /// Get retrieval statistics
    pub fn get_stats(&self) -> RetrievalStats {
        if let Ok(stats) = self.stats.read() {
            stats.clone()
        } else {
            RetrievalStats {
                total_searches: 0,
                stage1_avg_time_ms: 0.0,
                stage2_avg_time_ms: 0.0,
                total_avg_time_ms: 0.0,
                cache_hit_rate: 0.0,
                index_size: 0,
                last_index_rebuild: Instant::now(),
            }
        }
    }
}

impl ANNIndex {
    pub fn new(config: ANNIndexConfig) -> Self {
        Self {
            embeddings: HashMap::new(),
            graph: HashMap::new(),
            levels: Vec::new(),
            entry_point: None,
            config,
            metadata: ANNIndexMetadata {
                total_properties: 0,
                total_connections: 0,
                avg_connections_per_node: 0.0,
                max_level: 0,
                build_time_ms: 0,
                memory_usage_mb: 0.0,
            },
        }
    }

    /// Build HNSW index from embeddings
    pub fn build_from_embeddings(&mut self, embeddings: Vec<(i32, Vec<f32>)>) -> anyhow::Result<()> {
        let start = Instant::now();
        
        if embeddings.is_empty() {
            return Err(anyhow::anyhow!("No embeddings provided"));
        }
        
        self.embeddings.clear();
        self.graph.clear();
        self.levels.clear();
        
        // Store embeddings
        for (id, embedding) in embeddings {
            if embedding.len() != self.config.embedding_dim {
                return Err(anyhow::anyhow!("Embedding dimension mismatch: expected {}, got {}", 
                                         self.config.embedding_dim, embedding.len()));
            }
            self.embeddings.insert(id, embedding);
        }
        
        let property_ids: Vec<i32> = self.embeddings.keys().cloned().collect();
        
        if property_ids.is_empty() {
            return Ok(());
        }
        
        // Initialize levels - simplified HNSW construction
        // In a full implementation, you would use proper probabilistic level assignment
        let max_level = (property_ids.len() as f64).log2().ceil() as usize;
        self.levels = vec![HashSet::new(); max_level + 1];
        
        // Add all nodes to level 0
        for &id in &property_ids {
            self.levels[0].insert(id);
            self.graph.insert(id, Vec::new());
        }
        
        // Simple construction: connect each node to its k nearest neighbors
        for &node_id in &property_ids {
            let node_embedding = &self.embeddings[&node_id];
            
            // Find k nearest neighbors
            let mut neighbors: Vec<(i32, f32)> = property_ids.iter()
                .filter(|&&id| id != node_id)
                .map(|&id| {
                    let distance = self.cosine_distance(node_embedding, &self.embeddings[&id]);
                    (id, distance)
                })
                .collect();
            
            neighbors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            neighbors.truncate(self.config.max_connections_0);
            
            self.graph.insert(node_id, neighbors);
        }
        
        // Set entry point (first node for simplicity)
        self.entry_point = property_ids.first().cloned();
        
        // Update metadata
        let total_connections: usize = self.graph.values().map(|neighbors| neighbors.len()).sum();
        self.metadata = ANNIndexMetadata {
            total_properties: property_ids.len(),
            total_connections,
            avg_connections_per_node: total_connections as f64 / property_ids.len() as f64,
            max_level: max_level,
            build_time_ms: start.elapsed().as_millis() as u64,
            memory_usage_mb: self.estimate_memory_usage(),
        };
        
        Ok(())
    }

    /// Search for nearest neighbors using HNSW algorithm
    pub fn search(
        &self,
        query_embedding: &[f32],
        k: usize,
        ef: usize,
    ) -> anyhow::Result<Vec<CandidateProperty>> {
        if query_embedding.len() != self.config.embedding_dim {
            return Err(anyhow::anyhow!("Query embedding dimension mismatch"));
        }
        
        if self.embeddings.is_empty() || self.entry_point.is_none() {
            return Ok(Vec::new());
        }
        
        // Simplified HNSW search - in practice, you'd implement the full multi-level search
        let mut candidates: Vec<(i32, f32)> = Vec::new();
        
        // Calculate distances to all nodes (brute force for simplicity)
        for (&property_id, embedding) in &self.embeddings {
            let distance = self.cosine_distance(query_embedding, embedding);
            candidates.push((property_id, distance));
        }
        
        // Sort by distance and take top k
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        candidates.truncate(k);
        
        // Convert to CandidateProperty
        let result = candidates.into_iter()
            .map(|(property_id, distance)| CandidateProperty {
                property_id,
                similarity_score: 1.0 - distance, // Convert distance to similarity
                distance,
                source: CandidateSource::ANN,
            })
            .collect();
        
        Ok(result)
    }

    /// Calculate cosine distance between two embeddings
    fn cosine_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 1.0; // Maximum distance
        }
        
        1.0 - (dot_product / (norm_a * norm_b))
    }

    /// Estimate memory usage of the index
    fn estimate_memory_usage(&self) -> f64 {
        let mut total_bytes = 0usize;
        
        // Embeddings
        total_bytes += self.embeddings.len() * self.config.embedding_dim * std::mem::size_of::<f32>();
        
        // Graph connections  
        for neighbors in self.graph.values() {
            total_bytes += neighbors.len() * std::mem::size_of::<(i32, f32)>();
        }
        
        // Levels
        for level in &self.levels {
            total_bytes += level.len() * std::mem::size_of::<i32>();
        }
        
        total_bytes as f64 / (1024.0 * 1024.0) // Convert to MB
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ann_index_creation() {
        let config = ANNIndexConfig::default();
        let index = ANNIndex::new(config);
        assert_eq!(index.embeddings.len(), 0);
        assert_eq!(index.metadata.total_properties, 0);
    }

    #[test]
    fn test_cosine_distance() {
        let config = ANNIndexConfig::default();
        let index = ANNIndex::new(config);
        
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let distance = index.cosine_distance(&a, &b);
        assert!((distance - 1.0).abs() < 0.001); // Should be maximum distance (orthogonal)
        
        let c = vec![1.0, 0.0, 0.0];
        let distance_same = index.cosine_distance(&a, &c);
        assert!(distance_same.abs() < 0.001); // Should be minimum distance (identical)
    }
}
