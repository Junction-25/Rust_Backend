use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::{Property, Contact};

/// High-performance feature store for ML embeddings and computed features
/// Supports real-time updates, TTL-based expiration, and memory optimization
#[derive(Clone)]
pub struct FeatureStore {
    property_embeddings: Arc<RwLock<HashMap<i32, PropertyFeatures>>>,
    contact_embeddings: Arc<RwLock<HashMap<i32, ContactFeatures>>>,
    similarity_cache: Arc<RwLock<HashMap<String, CachedSimilarity>>>,
    metadata: Arc<RwLock<FeatureStoreMetadata>>,
    config: FeatureStoreConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyFeatures {
    pub property_id: i32,
    pub embedding: Vec<f32>,        // 128-dim dense embedding
    pub sparse_features: HashMap<String, f32>, // Sparse categorical features
    pub location_embedding: Vec<f32>, // 32-dim location embedding
    pub price_bin: u8,              // Neural price bin (0-6)
    pub area_bin: u8,               // Neural area bin (0-6) 
    pub room_bin: u8,               // Neural room bin (0-5)
    pub property_type_id: u8,       // Encoded property type
    pub location_cluster: u16,      // Geographic cluster ID
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactFeatures {
    pub contact_id: i32,
    pub embedding: Vec<f32>,        // 128-dim dense embedding
    pub preference_embedding: Vec<f32>, // 64-dim preference vector
    pub budget_range: (f32, f32),    // Normalized budget range [0,1]
    pub area_range: (f32, f32),      // Normalized area range [0,1]
    pub location_preferences: Vec<LocationPreference>,
    pub property_type_weights: HashMap<String, f32>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationPreference {
    pub location_id: u16,
    pub weight: f32,
    pub lat: f32,
    pub lon: f32,
    pub radius_km: f32,
}

#[derive(Debug, Clone)]
pub struct CachedSimilarity {
    pub contact_id: i32,
    pub property_id: i32,
    pub similarity_score: f32,
    pub computed_at: Instant,
    pub ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct FeatureStoreConfig {
    pub property_ttl: Duration,
    pub contact_ttl: Duration,
    pub similarity_ttl: Duration,
    pub max_property_features: usize,
    pub max_contact_features: usize,
    pub max_similarity_cache: usize,
    pub cleanup_interval: Duration,
    pub embedding_dim: usize,
}

/// Statistics and metadata about the feature store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStoreStats {
    pub total_properties: usize,
    pub total_contacts: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f64,
    pub memory_usage_mb: f64,
    pub last_cleanup: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct FeatureStoreMetadata {
    pub total_properties: usize,
    pub total_contacts: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub last_cleanup: Instant,
    pub memory_usage_mb: f64,
}

impl Default for FeatureStoreConfig {
    fn default() -> Self {
        Self {
            property_ttl: Duration::from_secs(3600), // 1 hour
            contact_ttl: Duration::from_secs(1800),  // 30 minutes
            similarity_ttl: Duration::from_secs(300), // 5 minutes
            max_property_features: 100_000,
            max_contact_features: 50_000,
            max_similarity_cache: 1_000_000,
            cleanup_interval: Duration::from_secs(300),
            embedding_dim: 128,
        }
    }
}

impl FeatureStore {
    pub fn new(config: FeatureStoreConfig) -> Self {
        Self {
            property_embeddings: Arc::new(RwLock::new(HashMap::new())),
            contact_embeddings: Arc::new(RwLock::new(HashMap::new())),
            similarity_cache: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(FeatureStoreMetadata {
                total_properties: 0,
                total_contacts: 0,
                cache_hits: 0,
                cache_misses: 0,
                last_cleanup: Instant::now(),
                memory_usage_mb: 0.0,
            })),
            config,
        }
    }

    /// Store property features with automatic cleanup
    pub fn store_property_features(&self, features: PropertyFeatures) -> anyhow::Result<()> {
        let mut store = self.property_embeddings.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;
        
        // Check capacity and cleanup if needed
        if store.len() >= self.config.max_property_features {
            self.cleanup_expired_properties(&mut store)?;
            
            // If still at capacity, remove oldest entries
            if store.len() >= self.config.max_property_features {
                let oldest_count = store.len() - self.config.max_property_features + 1000;
                let mut entries: Vec<_> = store.iter().map(|(k, v)| (*k, v.last_accessed)).collect();
                entries.sort_by_key(|(_, last_accessed)| *last_accessed);
                
                let keys_to_remove: Vec<i32> = entries.into_iter().take(oldest_count).map(|(k, _)| k).collect();
                for property_id in keys_to_remove {
                    store.remove(&property_id);
                }
            }
        }
        
        store.insert(features.property_id, features);
        
        // Update metadata
        if let Ok(mut meta) = self.metadata.write() {
            meta.total_properties = store.len();
        }
        
        Ok(())
    }

    /// Store contact features with automatic cleanup
    pub fn store_contact_features(&self, features: ContactFeatures) -> anyhow::Result<()> {
        let mut store = self.contact_embeddings.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;
        
        // Check capacity and cleanup if needed
        if store.len() >= self.config.max_contact_features {
            self.cleanup_expired_contacts(&mut store)?;
            
            // If still at capacity, remove oldest entries
            if store.len() >= self.config.max_contact_features {
                let oldest_count = store.len() - self.config.max_contact_features + 1000;
                let mut entries: Vec<_> = store.iter().map(|(k, v)| (*k, v.last_accessed)).collect();
                entries.sort_by_key(|(_, last_accessed)| *last_accessed);
                
                let keys_to_remove: Vec<i32> = entries.into_iter().take(oldest_count).map(|(k, _)| k).collect();
                for contact_id in keys_to_remove {
                    store.remove(&contact_id);
                }
            }
        }
        
        store.insert(features.contact_id, features);
        
        // Update metadata
        if let Ok(mut meta) = self.metadata.write() {
            meta.total_contacts = store.len();
        }
        
        Ok(())
    }

    /// Retrieve property features by ID
    pub fn get_property_features(&self, property_id: i32) -> Option<PropertyFeatures> {
        if let Ok(mut store) = self.property_embeddings.write() {
            if let Some(mut features) = store.get_mut(&property_id) {
                features.last_accessed = chrono::Utc::now();
                
                // Update cache hit stats
                if let Ok(mut meta) = self.metadata.write() {
                    meta.cache_hits += 1;
                }
                
                return Some(features.clone());
            }
        }
        
        // Update cache miss stats
        if let Ok(mut meta) = self.metadata.write() {
            meta.cache_misses += 1;
        }
        
        None
    }

    /// Retrieve contact features by ID
    pub fn get_contact_features(&self, contact_id: i32) -> Option<ContactFeatures> {
        if let Ok(mut store) = self.contact_embeddings.write() {
            if let Some(mut features) = store.get_mut(&contact_id) {
                features.last_accessed = chrono::Utc::now();
                
                // Update cache hit stats
                if let Ok(mut meta) = self.metadata.write() {
                    meta.cache_hits += 1;
                }
                
                return Some(features.clone());
            }
        }
        
        // Update cache miss stats  
        if let Ok(mut meta) = self.metadata.write() {
            meta.cache_misses += 1;
        }
        
        None
    }

    /// Store precomputed similarity score
    pub fn cache_similarity(&self, contact_id: i32, property_id: i32, similarity: f32) -> anyhow::Result<()> {
        let key = format!("{}_{}", contact_id, property_id);
        let cached = CachedSimilarity {
            contact_id,
            property_id,
            similarity_score: similarity,
            computed_at: Instant::now(),
            ttl: self.config.similarity_ttl,
        };
        
        let mut cache = self.similarity_cache.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;
        
        cache.insert(key, cached);
        Ok(())
    }

    /// Retrieve cached similarity score
    pub fn get_cached_similarity(&self, contact_id: i32, property_id: i32) -> Option<f32> {
        let key = format!("{}_{}", contact_id, property_id);
        
        if let Ok(cache) = self.similarity_cache.read() {
            if let Some(cached) = cache.get(&key) {
                // Check if not expired
                if cached.computed_at.elapsed() <= cached.ttl {
                    return Some(cached.similarity_score);
                }
            }
        }
        
        None
    }

    /// Bulk retrieve property features for ANN search
    pub fn get_property_features_batch(&self, property_ids: &[i32]) -> Vec<PropertyFeatures> {
        if let Ok(store) = self.property_embeddings.read() {
            property_ids.iter()
                .filter_map(|id| store.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all property IDs for indexing
    pub fn get_all_property_ids(&self) -> Vec<i32> {
        if let Ok(store) = self.property_embeddings.read() {
            store.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get all property embeddings for ANN index building
    pub fn get_all_property_embeddings(&self) -> Vec<(i32, Vec<f32>)> {
        if let Ok(store) = self.property_embeddings.read() {
            store.iter()
                .map(|(id, features)| (*id, features.embedding.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Cleanup expired entries
    pub fn cleanup_expired(&self) -> anyhow::Result<(usize, usize, usize)> {
        let now = Instant::now();
        let mut cleaned_properties = 0;
        let mut cleaned_contacts = 0;
        let mut cleaned_similarities = 0;

        // Cleanup properties
        if let Ok(mut store) = self.property_embeddings.write() {
            cleaned_properties = self.cleanup_expired_properties(&mut store)?;
        }

        // Cleanup contacts
        if let Ok(mut store) = self.contact_embeddings.write() {
            cleaned_contacts = self.cleanup_expired_contacts(&mut store)?;
        }

        // Cleanup similarity cache
        if let Ok(mut cache) = self.similarity_cache.write() {
            let before_size = cache.len();
            cache.retain(|_, cached| now.duration_since(cached.computed_at) <= cached.ttl);
            cleaned_similarities = before_size - cache.len();
        }

        // Update metadata
        if let Ok(mut meta) = self.metadata.write() {
            meta.last_cleanup = now;
        }

        Ok((cleaned_properties, cleaned_contacts, cleaned_similarities))
    }

    /// Internal: Cleanup expired property features
    fn cleanup_expired_properties(&self, store: &mut HashMap<i32, PropertyFeatures>) -> anyhow::Result<usize> {
        let now = chrono::Utc::now();
        let before_size = store.len();
        
        store.retain(|_, features| (now - features.created_at).num_seconds() <= self.config.property_ttl.as_secs() as i64);
        
        Ok(before_size - store.len())
    }

    /// Internal: Cleanup expired contact features
    fn cleanup_expired_contacts(&self, store: &mut HashMap<i32, ContactFeatures>) -> anyhow::Result<usize> {
        let now = chrono::Utc::now();
        let before_size = store.len();
        
        store.retain(|_, features| (now - features.created_at).num_seconds() <= self.config.contact_ttl.as_secs() as i64);
        
        Ok(before_size - store.len())
    }

    /// Get feature store statistics
    pub fn get_stats(&self) -> FeatureStoreStats {
        let property_count = if let Ok(store) = self.property_embeddings.read() {
            store.len()
        } else {
            0
        };
        
        let contact_count = if let Ok(store) = self.contact_embeddings.read() {
            store.len()
        } else {
            0
        };
        
        let (cache_hits, cache_misses) = if let Ok(meta) = self.metadata.read() {
            (meta.cache_hits, meta.cache_misses)
        } else {
            (0, 0)
        };
        
        let cache_hit_rate = if cache_hits + cache_misses > 0 {
            cache_hits as f64 / (cache_hits + cache_misses) as f64
        } else {
            0.0
        };
        
        FeatureStoreStats {
            total_properties: property_count,
            total_contacts: contact_count,
            cache_hits,
            cache_misses,
            cache_hit_rate,
            memory_usage_mb: self.estimate_memory_usage(),
            last_cleanup: chrono::Utc::now(),
        }
    }

    /// Calculate approximate memory usage
    pub fn estimate_memory_usage(&self) -> f64 {
        let mut total_bytes = 0usize;
        
        // Property embeddings
        if let Ok(store) = self.property_embeddings.read() {
            for features in store.values() {
                total_bytes += std::mem::size_of::<PropertyFeatures>();
                total_bytes += features.embedding.len() * std::mem::size_of::<f32>();
                total_bytes += features.sparse_features.len() * std::mem::size_of::<f32>();
                total_bytes += features.location_embedding.len() * std::mem::size_of::<f32>();
            }
        }
        
        // Contact embeddings
        if let Ok(store) = self.contact_embeddings.read() {
            for features in store.values() {
                total_bytes += std::mem::size_of::<ContactFeatures>();
                total_bytes += features.embedding.len() * std::mem::size_of::<f32>();
                total_bytes += features.preference_embedding.len() * std::mem::size_of::<f32>();
                total_bytes += features.location_preferences.len() * std::mem::size_of::<LocationPreference>();
            }
        }
        
        // Similarity cache
        if let Ok(cache) = self.similarity_cache.read() {
            total_bytes += cache.len() * (std::mem::size_of::<String>() + std::mem::size_of::<CachedSimilarity>());
        }
        
        total_bytes as f64 / (1024.0 * 1024.0) // Convert to MB
    }
}

impl Default for FeatureStore {
    fn default() -> Self {
        Self::new(FeatureStoreConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_feature_storage() {
        let store = FeatureStore::default();
        
        let features = PropertyFeatures {
            property_id: 1,
            embedding: vec![0.1, 0.2, 0.3],
            sparse_features: vec![1.0, 0.0, 1.0],
            location_embedding: vec![0.5, 0.6],
            price_bin: 3,
            area_bin: 2,
            room_bin: 1,
            property_type_id: 1,
            location_cluster: 100,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
        };
        
        assert!(store.store_property_features(features.clone()).is_ok());
        let retrieved = store.get_property_features(1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().property_id, 1);
    }

    #[test]
    fn test_similarity_caching() {
        let store = FeatureStore::default();
        
        assert!(store.cache_similarity(1, 100, 0.85).is_ok());
        let cached = store.get_cached_similarity(1, 100);
        assert!(cached.is_some());
        assert!((cached.unwrap() - 0.85).abs() < 0.001);
    }

    #[test]
    fn test_memory_estimation() {
        let store = FeatureStore::default();
        let memory_usage = store.estimate_memory_usage();
        assert!(memory_usage >= 0.0);
    }
}
