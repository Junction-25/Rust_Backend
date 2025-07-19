use std::collections::HashMap;
use serde::{Deserialize, Serialize};

const EMBEDDING_SIZE: usize = 32;
const DEFAULT_EMBEDDING: [f32; EMBEDDING_SIZE] = [0.1; EMBEDDING_SIZE];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralBinner {
    bins: HashMap<String, Vec<f64>>,
    embeddings: HashMap<String, Vec<f32>>,
}

impl NeuralBinner {
    pub fn new() -> Self {
        let mut binner = Self {
            bins: HashMap::new(),
            embeddings: HashMap::new(),
        };
        
        // Initialize with default quantile bins for key features
        binner.initialize_default_bins();
        binner
    }

    fn initialize_default_bins(&mut self) {
        // Price bins (in thousands)
        let price_bins = vec![0.0, 150_000.0, 250_000.0, 350_000.0, 500_000.0, 750_000.0, 1_000_000.0, f64::INFINITY];
        let price_embeddings = Self::generate_feature_embeddings(price_bins.len() - 1, 0.1);
        self.bins.insert("price".to_string(), price_bins);
        self.embeddings.insert("price".to_string(), price_embeddings);

        // Area bins (square meters)
        let area_bins = vec![0.0, 50.0, 75.0, 100.0, 150.0, 200.0, 300.0, f64::INFINITY];
        let area_embeddings = Self::generate_feature_embeddings(area_bins.len() - 1, 0.2);
        self.bins.insert("area".to_string(), area_bins);
        self.embeddings.insert("area".to_string(), area_embeddings);

        // Room bins
        let room_bins = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, f64::INFINITY];
        let room_embeddings = Self::generate_feature_embeddings(room_bins.len() - 1, 0.3);
        self.bins.insert("rooms".to_string(), room_bins);
        self.embeddings.insert("rooms".to_string(), room_embeddings);

        // Budget bins (for contacts)
        let budget_bins = vec![0.0, 200_000.0, 300_000.0, 400_000.0, 600_000.0, 800_000.0, 1_200_000.0, f64::INFINITY];
        let budget_embeddings = Self::generate_feature_embeddings(budget_bins.len() - 1, 0.4);
        self.bins.insert("budget".to_string(), budget_bins);
        self.embeddings.insert("budget".to_string(), budget_embeddings);
    }

    fn generate_feature_embeddings(num_bins: usize, base_seed: f32) -> Vec<f32> {
        let mut embeddings = Vec::with_capacity(num_bins * EMBEDDING_SIZE);
        
        for bin_idx in 0..num_bins {
            let mut bin_embedding = Vec::with_capacity(EMBEDDING_SIZE);
            
            // Generate pseudo-random but deterministic embeddings
            for dim in 0..EMBEDDING_SIZE {
                let seed = base_seed + (bin_idx as f32 * 0.1) + (dim as f32 * 0.01);
                let value = (seed.sin() * 10000.0).fract().abs();
                // Normalize to [-1, 1] range
                bin_embedding.push((value - 0.5) * 2.0);
            }
            
            // Normalize the embedding vector
            let norm: f32 = bin_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for val in &mut bin_embedding {
                    *val /= norm;
                }
            }
            
            embeddings.extend(bin_embedding);
        }
        
        embeddings
    }

    pub fn get_embedding(&self, feature: &str, value: f64) -> Vec<f32> {
        match self.bins.get(feature) {
            Some(bins) => {
                let bin_idx = bins.partition_point(|&x| x <= value).saturating_sub(1).min(bins.len() - 2);
                let embeddings = self.embeddings.get(feature).unwrap();
                let start_idx = bin_idx * EMBEDDING_SIZE;
                let end_idx = start_idx + EMBEDDING_SIZE;
                
                if end_idx <= embeddings.len() {
                    embeddings[start_idx..end_idx].to_vec()
                } else {
                    DEFAULT_EMBEDDING.to_vec()
                }
            },
            None => DEFAULT_EMBEDDING.to_vec(),
        }
    }

    pub fn get_bin_index(&self, feature: &str, value: f64) -> usize {
        match self.bins.get(feature) {
            Some(bins) => bins.partition_point(|&x| x <= value).saturating_sub(1).min(bins.len() - 2),
            None => 0,
        }
    }

    pub fn get_feature_vector(&self, features: &HashMap<String, f64>) -> Vec<f32> {
        let mut feature_vector = Vec::new();
        
        // Combine embeddings for all features
        for feature_name in ["price", "area", "rooms", "budget"] {
            if let Some(&value) = features.get(feature_name) {
                let embedding = self.get_embedding(feature_name, value);
                feature_vector.extend(embedding);
            } else {
                feature_vector.extend(DEFAULT_EMBEDDING.iter().copied());
            }
        }
        
        feature_vector
    }
}

impl Default for NeuralBinner {
    fn default() -> Self {
        Self::new()
    }
}

// Location attention pooling utilities
#[derive(Debug, Clone)]
pub struct LocationAttentionPooler {
    distance_decay_factor: f64,
}

impl LocationAttentionPooler {
    pub fn new(distance_decay_factor: f64) -> Self {
        Self { distance_decay_factor }
    }

    pub fn calculate_attention_weights(&self, distances: &[f64]) -> Vec<f64> {
        let weights: Vec<f64> = distances
            .iter()
            .map(|&d| (-self.distance_decay_factor * d).exp())
            .collect();
        
        // Normalize weights to sum to 1
        let sum: f64 = weights.iter().sum();
        if sum > 0.0 {
            weights.iter().map(|w| w / sum).collect()
        } else {
            vec![1.0 / weights.len() as f64; weights.len()]
        }
    }

    pub fn pool_embeddings(&self, embeddings: &[Vec<f32>], weights: &[f64]) -> Vec<f32> {
        if embeddings.is_empty() || weights.is_empty() {
            return DEFAULT_EMBEDDING.to_vec();
        }

        let embedding_size = embeddings[0].len();
        let mut pooled = vec![0.0; embedding_size];

        for (embedding, &weight) in embeddings.iter().zip(weights.iter()) {
            for (i, &val) in embedding.iter().enumerate() {
                if i < pooled.len() {
                    pooled[i] += val * weight as f32;
                }
            }
        }

        pooled
    }
}

impl Default for LocationAttentionPooler {
    fn default() -> Self {
        Self::new(0.1) // Default decay factor
    }
}

// Utility functions for vector operations
pub fn dot_product(a: &[f32], b: &[f32]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| (x * y) as f64)
        .sum()
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot = dot_product(a, b);
    let norm_a: f64 = a.iter().map(|&x| (x * x) as f64).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|&x| (x * x) as f64).sum::<f64>().sqrt();
    
    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a * norm_b)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neural_binner_initialization() {
        let binner = NeuralBinner::new();
        assert!(binner.bins.contains_key("price"));
        assert!(binner.bins.contains_key("area"));
        assert!(binner.embeddings.contains_key("price"));
        assert!(binner.embeddings.contains_key("area"));
    }

    #[test]
    fn test_get_embedding() {
        let binner = NeuralBinner::new();
        let embedding = binner.get_embedding("price", 300_000.0);
        assert_eq!(embedding.len(), EMBEDDING_SIZE);
    }

    #[test]
    fn test_location_attention_pooler() {
        let pooler = LocationAttentionPooler::new(0.1);
        let distances = vec![1.0, 5.0, 10.0];
        let weights = pooler.calculate_attention_weights(&distances);
        
        // Weights should sum to approximately 1.0
        let sum: f64 = weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
        
        // Closer distances should have higher weights
        assert!(weights[0] > weights[1]);
        assert!(weights[1] > weights[2]);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
        
        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 1e-6);
    }
}
