use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::models::{Property, Contact, Location};
use crate::utils::feature_store::{PropertyFeatures, ContactFeatures, LocationPreference};
use crate::utils::feature_engineering::{NeuralBinner, LocationAttentionPooler};

/// Advanced embedding pipeline for converting properties and contacts into dense vector representations
/// Combines multiple feature extraction techniques:
/// - Textual embeddings (description, amenities)
/// - Numerical feature encoding (price, area, rooms)
/// - Categorical embeddings (property type, location)
/// - Behavioral embeddings (user preferences, historical interactions)
pub struct EmbeddingPipeline {
    neural_binner: NeuralBinner,
    location_pooler: LocationAttentionPooler,
    text_encoder: TextEmbedder,
    categorical_enc        let budget_range = contact.max_budget - contact.min_budget;
        let budget_flexibility = if budget_range > 0.0 {
            1.0 - (budget_range / contact.max_budget).min(1.0)r: CategoricalEmbedder,
    numerical_normalizer: NumericalNormalizer,
    config: EmbeddingConfig,
}

#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub property_embedding_dim: usize,
    pub contact_embedding_dim: usize,
    pub text_embedding_dim: usize,
    pub categorical_embedding_dim: usize,
    pub location_embedding_dim: usize,
    pub use_text_features: bool,
    pub use_categorical_features: bool,
    pub use_location_features: bool,
    pub use_neural_binning: bool,
    pub normalize_embeddings: bool,
}

#[derive(Debug, Clone)]
pub struct TextEmbedder {
    /// Pre-computed embeddings for common words/phrases
    word_embeddings: HashMap<String, Vec<f32>>,
    /// TF-IDF vocabulary
    vocabulary: HashMap<String, usize>,
    /// IDF scores for vocabulary
    idf_scores: Vec<f32>,
    config: TextEmbedderConfig,
}

#[derive(Debug, Clone)]
pub struct TextEmbedderConfig {
    pub embedding_dim: usize,
    pub max_vocab_size: usize,
    pub min_word_freq: usize,
    pub use_tfidf: bool,
    pub use_ngrams: bool,
    pub ngram_size: usize,
}

#[derive(Debug, Clone)]
pub struct CategoricalEmbedder {
    /// Learned embeddings for each categorical feature
    property_type_embeddings: HashMap<String, Vec<f32>>,
    location_embeddings: HashMap<String, Vec<f32>>,
    amenity_embeddings: HashMap<String, Vec<f32>>,
    config: CategoricalEmbedderConfig,
}

#[derive(Debug, Clone)]
pub struct CategoricalEmbedderConfig {
    pub embedding_dim: usize,
    pub use_frequency_weighting: bool,
    pub min_category_freq: usize,
    pub unknown_token_init: f32,
}

#[derive(Debug, Clone)]
pub struct NumericalNormalizer {
    /// Statistics for each numerical feature
    feature_stats: HashMap<String, FeatureStats>,
    config: NormalizerConfig,
}

#[derive(Debug, Clone)]
pub struct FeatureStats {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub q25: f64,
    pub q75: f64,
}

#[derive(Debug, Clone)]
pub struct NormalizerConfig {
    pub normalization_method: NormalizationMethod,
    pub handle_outliers: bool,
    pub outlier_threshold: f64,
    pub clip_outliers: bool,
}

#[derive(Debug, Clone)]
pub enum NormalizationMethod {
    StandardScore,  // (x - mean) / std_dev
    MinMax,         // (x - min) / (max - min)
    Robust,         // (x - median) / (q75 - q25)
    Quantile,       // Map to quantile rank
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyEmbedding {
    pub property_id: i32,
    pub full_embedding: Vec<f32>,
    pub text_features: Vec<f32>,
    pub numerical_features: Vec<f32>,
    pub categorical_features: Vec<f32>,
    pub location_features: Vec<f32>,
    pub sparse_features: HashMap<String, f32>,
    pub metadata: EmbeddingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactEmbedding {
    pub contact_id: i32,
    pub preference_embedding: Vec<f32>,
    pub budget_features: Vec<f32>,
    pub location_features: Vec<f32>,
    pub behavioral_features: Vec<f32>,
    pub metadata: EmbeddingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub model_version: String,
    pub feature_hash: u64,
    pub dimensions: usize,
    pub sparsity: f32,
    pub norm: f32,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            property_embedding_dim: 128,
            contact_embedding_dim: 64,
            text_embedding_dim: 32,
            categorical_embedding_dim: 24,
            location_embedding_dim: 16,
            use_text_features: true,
            use_categorical_features: true,
            use_location_features: true,
            use_neural_binning: true,
            normalize_embeddings: true,
        }
    }
}

impl Default for TextEmbedderConfig {
    fn default() -> Self {
        Self {
            embedding_dim: 32,
            max_vocab_size: 10000,
            min_word_freq: 2,
            use_tfidf: true,
            use_ngrams: true,
            ngram_size: 2,
        }
    }
}

impl Default for CategoricalEmbedderConfig {
    fn default() -> Self {
        Self {
            embedding_dim: 8,
            use_frequency_weighting: true,
            min_category_freq: 5,
            unknown_token_init: 0.01,
        }
    }
}

impl Default for NormalizerConfig {
    fn default() -> Self {
        Self {
            normalization_method: NormalizationMethod::StandardScore,
            handle_outliers: true,
            outlier_threshold: 3.0,
            clip_outliers: true,
        }
    }
}

impl EmbeddingPipeline {
    pub fn new(
        neural_binner: NeuralBinner,
        location_pooler: LocationAttentionPooler,
        config: EmbeddingConfig,
    ) -> Self {
        let text_config = TextEmbedderConfig::default();
        let categorical_config = CategoricalEmbedderConfig::default();
        let normalizer_config = NormalizerConfig::default();
        
        Self {
            neural_binner,
            location_pooler,
            text_encoder: TextEmbedder::new(text_config),
            categorical_encoder: CategoricalEmbedder::new(categorical_config),
            numerical_normalizer: NumericalNormalizer::new(normalizer_config),
            config,
        }
    }

    /// Train the pipeline on a dataset of properties and contacts
    pub fn train(&mut self, properties: &[Property], contacts: &[Contact]) -> anyhow::Result<()> {
        log::info!("Training embedding pipeline with {} properties and {} contacts", 
                  properties.len(), contacts.len());

        // 1. Train text embedder on property descriptions and amenities
        if self.config.use_text_features {
            let mut text_corpus = Vec::new();
            // For now, use property type and address as text content
            for property in properties {
                text_corpus.push(property.property_type.clone());
                text_corpus.push(property.address.clone());
            }
            self.text_encoder.train(&text_corpus)?;
        }

        // 2. Train categorical embedder on property types and locations
        if self.config.use_categorical_features {
            self.categorical_encoder.train(properties)?;
        }

        // 3. Compute numerical feature statistics
        self.numerical_normalizer.compute_statistics(properties)?;

        log::info!("Embedding pipeline training completed");
        Ok(())
    }

    /// Generate property embedding
    pub fn encode_property(&self, property: &Property) -> anyhow::Result<PropertyEmbedding> {
        let mut full_embedding = Vec::new();
        let mut sparse_features = HashMap::new();

        // 1. Text features from property type and address
        let text_features = if self.config.use_text_features {
            let mut text_content = String::new();
            text_content.push_str(&property.property_type);
            text_content.push(' ');
            text_content.push_str(&property.address);
            self.text_encoder.encode(&text_content)?
        } else {
            vec![0.0; self.config.text_embedding_dim]
        };

        // 2. Numerical features (price, area, rooms)
        let numerical_features = self.encode_numerical_features(property)?;

        // 3. Categorical features (property type, location area)
        let categorical_features = if self.config.use_categorical_features {
            self.categorical_encoder.encode_property(property)?
        } else {
            vec![0.0; self.config.categorical_embedding_dim]
        };

        // 4. Location features using simple encoding
        let location_features = if self.config.use_location_features {
            vec![
                property.location.lat as f32 / 90.0,  // Normalize latitude
                property.location.lon as f32 / 180.0, // Normalize longitude
                // Add more location features as needed
            ]
        } else {
            vec![0.0; self.config.location_embedding_dim]
        };

        // 5. Neural binning features
        if self.config.use_neural_binning {
            let price_bin = self.neural_binner.get_bin_index("price", property.price);
            let area_bin = self.neural_binner.get_bin_index("area", property.area_sqm as f64);
            let room_bin = self.neural_binner.get_bin_index("rooms", property.number_of_rooms as f64);
            
            sparse_features.insert("price_bin".to_string(), price_bin as f32);
            sparse_features.insert("area_bin".to_string(), area_bin as f32);
            sparse_features.insert("room_bin".to_string(), room_bin as f32);
        }

        // Combine all features
        full_embedding.extend(text_features.clone());
        full_embedding.extend(numerical_features.clone());
        full_embedding.extend(categorical_features.clone());
        full_embedding.extend(location_features.clone());

        // Normalize if requested
        if self.config.normalize_embeddings {
            self.l2_normalize(&mut full_embedding)?;
        }

        // Compute metadata
        let feature_hash = self.compute_feature_hash(property);
        let sparsity = sparse_features.len() as f32 / (sparse_features.len() + full_embedding.len()) as f32;
        let norm = self.compute_l2_norm(&full_embedding);

        Ok(PropertyEmbedding {
            property_id: property.id,
            full_embedding,
            text_features,
            numerical_features,
            categorical_features,
            location_features,
            sparse_features,
            metadata: EmbeddingMetadata {
                created_at: chrono::Utc::now(),
                model_version: "1.0".to_string(),
                feature_hash,
                dimensions: self.config.property_embedding_dim,
                sparsity,
                norm,
            },
        })
    }

    /// Generate contact embedding
    pub fn encode_contact(&self, contact: &Contact) -> anyhow::Result<ContactEmbedding> {
        // 1. Budget preference features
        let budget_features = self.encode_budget_preferences(contact)?;

        // 2. Location preference features
        let location_features = self.encode_location_preferences(contact)?;

        // 3. Behavioral features (property type preferences, size preferences)
        let behavioral_features = self.encode_behavioral_features(contact)?;

        // Combine all features
        let mut preference_embedding = Vec::new();
        preference_embedding.extend(budget_features.clone());
        preference_embedding.extend(location_features.clone());
        preference_embedding.extend(behavioral_features.clone());

        // Normalize if requested
        if self.config.normalize_embeddings {
            self.l2_normalize(&mut preference_embedding)?;
        }

        // Compute metadata
        let feature_hash = self.compute_contact_feature_hash(contact);
        let norm = self.compute_l2_norm(&preference_embedding);

        Ok(ContactEmbedding {
            contact_id: contact.id,
            preference_embedding,
            budget_features,
            location_features,
            behavioral_features,
            metadata: EmbeddingMetadata {
                created_at: chrono::Utc::now(),
                model_version: "1.0".to_string(),
                feature_hash,
                dimensions: self.config.contact_embedding_dim,
                sparsity: 0.0, // Dense embeddings
                norm,
            },
        })
    }

    /// Encode numerical features for a property
    fn encode_numerical_features(&self, property: &Property) -> anyhow::Result<Vec<f32>> {
        let mut features = Vec::new();

        // Price feature
        let normalized_price = self.numerical_normalizer.normalize("price", property.price)?;
        features.push(normalized_price);

        // Area feature
        let normalized_area = self.numerical_normalizer.normalize("area_sqm", property.area_sqm as f64)?;
        features.push(normalized_area);

        // Rooms feature
        let normalized_rooms = self.numerical_normalizer.normalize("rooms", property.number_of_rooms as f64)?;
        features.push(normalized_rooms);

        // Additional derived features
        features.push((property.price / property.area_sqm as f64) as f32); // Price per sqm
        features.push((property.area_sqm as f32) / (property.number_of_rooms as f32 + 1.0)); // Area per room

        Ok(features)
    }

    /// Encode budget preferences for a contact
    fn encode_budget_preferences(&self, contact: &Contact) -> anyhow::Result<Vec<f32>> {
        let mut features = Vec::new();

        // Normalized budget range
        let normalized_min = self.numerical_normalizer.normalize("price", contact.min_budget)?;
        let normalized_max = self.numerical_normalizer.normalize("price", contact.max_budget)?;
        
        features.push(normalized_min);
        features.push(normalized_max);
        features.push((normalized_max - normalized_min) / 2.0); // Budget flexibility
        features.push((normalized_min + normalized_max) / 2.0); // Budget center

        Ok(features)
    }

    /// Encode location preferences for a contact
    fn encode_location_preferences(&self, contact: &Contact) -> anyhow::Result<Vec<f32>> {
        let mut features = vec![0.0; self.config.location_embedding_dim];

        if !contact.preferred_locations.is_empty() {
            // Use first preferred location for simplicity
            let first_location = &contact.preferred_locations[0];
            features[0] = first_location.lat as f32 / 90.0;  // Normalize latitude
            features[1] = first_location.lon as f32 / 180.0; // Normalize longitude
        }

        Ok(features)
    }

    /// Encode behavioral features for a contact
    fn encode_behavioral_features(&self, contact: &Contact) -> anyhow::Result<Vec<f32>> {
        let mut features = Vec::new();

        // Property type preferences (one-hot encoding)
        let property_types = ["apartment", "house", "condo", "townhouse", "studio"];
        for property_type in &property_types {
            let preference_score = if contact.property_types.contains(&property_type.to_string()) {
                1.0
            } else {
                0.0
            };
            features.push(preference_score);
        }

        // Size preferences
        let normalized_min_area = self.numerical_normalizer.normalize("area_sqm", contact.min_area_sqm as f64)?;
        let normalized_max_area = self.numerical_normalizer.normalize("area_sqm", contact.max_area_sqm as f64)?;
        let normalized_min_rooms = self.numerical_normalizer.normalize("rooms", contact.min_rooms as f64)?;

        features.push(normalized_min_area);
        features.push(normalized_max_area);
        features.push(normalized_min_rooms);

        // Size flexibility
        features.push((normalized_max_area - normalized_min_area) / 2.0);

        Ok(features)
    }

    /// L2 normalization
    fn l2_normalize(&self, embedding: &mut Vec<f32>) -> anyhow::Result<()> {
        let norm = self.compute_l2_norm(embedding);
        if norm > 0.0 {
            for value in embedding.iter_mut() {
                *value /= norm;
            }
        }
        Ok(())
    }

    /// Compute L2 norm
    fn compute_l2_norm(&self, embedding: &[f32]) -> f32 {
        embedding.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// Compute feature hash for cache invalidation
    fn compute_feature_hash(&self, property: &Property) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        property.id.hash(&mut hasher);
        property.price.to_bits().hash(&mut hasher);
        property.area_sqm.hash(&mut hasher);
        property.number_of_rooms.hash(&mut hasher);
        property.property_type.hash(&mut hasher);
        hasher.finish()
    }

    /// Compute feature hash for contact
    fn compute_contact_feature_hash(&self, contact: &Contact) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        contact.id.hash(&mut hasher);
        contact.min_budget.to_bits().hash(&mut hasher);
        contact.max_budget.to_bits().hash(&mut hasher);
        contact.min_area_sqm.hash(&mut hasher);
        contact.max_area_sqm.hash(&mut hasher);
        hasher.finish()
    }
}

impl TextEmbedder {
    pub fn new(config: TextEmbedderConfig) -> Self {
        Self {
            word_embeddings: HashMap::new(),
            vocabulary: HashMap::new(),
            idf_scores: Vec::new(),
            config,
        }
    }

    /// Train text embedder on corpus
    pub fn train(&mut self, corpus: &[String]) -> anyhow::Result<()> {
        // Build vocabulary
        let mut word_counts = HashMap::new();
        let mut document_counts = HashMap::new();
        
        for document in corpus {
            let words = self.tokenize(document);
            let unique_words: std::collections::HashSet<_> = words.iter().cloned().collect();
            
            for word in &words {
                *word_counts.entry(word.clone()).or_insert(0) += 1;
            }
            
            for word in unique_words {
                *document_counts.entry(word).or_insert(0) += 1;
            }
        }

        // Filter vocabulary by frequency
        let mut vocab_pairs: Vec<_> = word_counts.into_iter()
            .filter(|(_, count)| *count >= self.config.min_word_freq)
            .collect();
        vocab_pairs.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by frequency descending
        vocab_pairs.truncate(self.config.max_vocab_size);

        // Build vocabulary and compute IDF scores
        self.vocabulary.clear();
        self.idf_scores.clear();
        
        for (i, (word, _)) in vocab_pairs.iter().enumerate() {
            self.vocabulary.insert(word.clone(), i);
            
            let doc_freq = *document_counts.get(word).unwrap_or(&1) as f32;
            let idf = (corpus.len() as f32 / doc_freq).ln();
            self.idf_scores.push(idf);
        }

        // Initialize random embeddings (in practice, use pre-trained embeddings)
        self.word_embeddings.clear();
        for (word, _) in &vocab_pairs {
            let embedding = self.random_embedding();
            self.word_embeddings.insert(word.clone(), embedding);
        }

        Ok(())
    }

    /// Encode text into embedding
    pub fn encode(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let words = self.tokenize(text);
        let mut embedding = vec![0.0; self.config.embedding_dim];
        let mut total_weight = 0.0;

        for word in words {
            if let (Some(&vocab_idx), Some(word_embedding)) = 
                (self.vocabulary.get(&word), self.word_embeddings.get(&word)) {
                
                let weight = if self.config.use_tfidf && vocab_idx < self.idf_scores.len() {
                    self.idf_scores[vocab_idx]
                } else {
                    1.0
                };

                for (i, &emb_val) in word_embedding.iter().enumerate() {
                    if i < embedding.len() {
                        embedding[i] += emb_val * weight;
                    }
                }
                total_weight += weight;
            }
        }

        // Average the embeddings
        if total_weight > 0.0 {
            for emb_val in &mut embedding {
                *emb_val /= total_weight;
            }
        }

        Ok(embedding)
    }

    /// Simple tokenization
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|word| !word.is_empty())
            .map(|word| word.to_string())
            .collect()
    }

    /// Generate random embedding
    fn random_embedding(&self) -> Vec<f32> {
        (0..self.config.embedding_dim)
            .map(|_| (rand::random::<f32>() - 0.5) * 0.1)
            .collect()
    }
}

impl CategoricalEmbedder {
    pub fn new(config: CategoricalEmbedderConfig) -> Self {
        Self {
            property_type_embeddings: HashMap::new(),
            location_embeddings: HashMap::new(),
            amenity_embeddings: HashMap::new(),
            config,
        }
    }

    /// Train categorical embedder
    pub fn train(&mut self, properties: &[Property]) -> anyhow::Result<()> {
        // Count property types
        let mut type_counts = HashMap::new();
        
        for property in properties {
            *type_counts.entry(property.property_type.clone()).or_insert(0) += 1;
        }

        // Initialize embeddings for frequent categories
        for (prop_type, count) in type_counts {
            if count >= self.config.min_category_freq {
                let embedding = self.random_embedding();
                self.property_type_embeddings.insert(prop_type, embedding);
            }
        }

        Ok(())
    }

    /// Encode property categorical features
    pub fn encode_property(&self, property: &Property) -> anyhow::Result<Vec<f32>> {
        let mut embedding = vec![0.0; self.config.embedding_dim];

        // Property type embedding
        if let Some(type_emb) = self.property_type_embeddings.get(&property.property_type) {
            for (i, &val) in type_emb.iter().enumerate() {
                if i < embedding.len() {
                    embedding[i] += val;
                }
            }
        }

        Ok(embedding)
    }

    /// Generate random embedding
    fn random_embedding(&self) -> Vec<f32> {
        (0..self.config.embedding_dim)
            .map(|_| (rand::random::<f32>() - 0.5) * self.config.unknown_token_init)
            .collect()
    }
}

impl NumericalNormalizer {
    pub fn new(config: NormalizerConfig) -> Self {
        Self {
            feature_stats: HashMap::new(),
            config,
        }
    }

    /// Compute statistics for numerical features
    pub fn compute_statistics(&mut self, properties: &[Property]) -> anyhow::Result<()> {
        if properties.is_empty() {
            return Ok(());
        }

        // Collect feature values
        let prices: Vec<f64> = properties.iter().map(|p| p.price).collect();
        let areas: Vec<f64> = properties.iter().map(|p| p.area_sqm as f64).collect();
        let rooms: Vec<f64> = properties.iter().map(|p| p.number_of_rooms as f64).collect();

        // Compute statistics
        self.feature_stats.insert("price".to_string(), self.compute_feature_stats(&prices));
        self.feature_stats.insert("area_sqm".to_string(), self.compute_feature_stats(&areas));
        self.feature_stats.insert("rooms".to_string(), self.compute_feature_stats(&rooms));

        Ok(())
    }

    /// Normalize a feature value
    pub fn normalize(&self, feature_name: &str, value: f64) -> anyhow::Result<f32> {
        let stats = self.feature_stats.get(feature_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown feature: {}", feature_name))?;

        let normalized = match self.config.normalization_method {
            NormalizationMethod::StandardScore => {
                if stats.std_dev > 0.0 {
                    (value - stats.mean) / stats.std_dev
                } else {
                    0.0
                }
            },
            NormalizationMethod::MinMax => {
                if stats.max > stats.min {
                    (value - stats.min) / (stats.max - stats.min)
                } else {
                    0.0
                }
            },
            NormalizationMethod::Robust => {
                let iqr = stats.q75 - stats.q25;
                if iqr > 0.0 {
                    (value - stats.median) / iqr
                } else {
                    0.0
                }
            },
            NormalizationMethod::Quantile => {
                // Simplified quantile normalization
                if value <= stats.q25 {
                    0.25
                } else if value <= stats.median {
                    0.5
                } else if value <= stats.q75 {
                    0.75
                } else {
                    1.0
                }
            },
        };

        // Handle outliers
        let final_value = if self.config.handle_outliers && self.config.clip_outliers {
            normalized.max(-self.config.outlier_threshold).min(self.config.outlier_threshold)
        } else {
            normalized
        };

        Ok(final_value as f32)
    }

    /// Compute statistics for a feature
    fn compute_feature_stats(&self, values: &[f64]) -> FeatureStats {
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = values.len();
        let mean = values.iter().sum::<f64>() / n as f64;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
        let std_dev = variance.sqrt();

        let min = sorted_values[0];
        let max = sorted_values[n - 1];
        let median = sorted_values[n / 2];
        let q25 = sorted_values[n / 4];
        let q75 = sorted_values[3 * n / 4];

        FeatureStats {
            mean,
            std_dev,
            min,
            max,
            q25,
            median,
            q75,
        }
    }

    /// Extract features from a contact for storage in feature store
    pub fn extract_contact_features(&self, contact: &Contact) -> anyhow::Result<ContactFeatures> {
        let features = ContactFeatures {
            contact_id: contact.id,
            embedding: self.generate_contact_embedding(contact)?,
            preference_embedding: self.generate_preference_embedding(contact),
            location_preferences: self.extract_location_preferences(contact),
            budget_range: (contact.min_budget as f32, contact.max_budget as f32),
            area_range: (contact.min_rooms as f32, contact.min_rooms as f32), // Contact doesn't have max_rooms, using min as placeholder
            property_type_weights: HashMap::new(),
            last_accessed: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
        };
        Ok(features)
    }

    /// Generate property embedding vector (simplified for testing)
    fn generate_property_embedding(&self, property: &Property) -> anyhow::Result<Vec<f32>> {
        let mut embedding = Vec::with_capacity(128);
        
        // Basic numerical features
        embedding.push(property.price as f32 / 1_000_000.0); // Normalize price
        embedding.push(property.area_sqm as f32 / 200.0); // Normalize area
        embedding.push(property.number_of_rooms as f32 / 10.0); // Normalize rooms
        embedding.push(property.location.lat as f32 / 90.0); // Normalize latitude
        embedding.push(property.location.lon as f32 / 180.0); // Normalize longitude
        
        // Property type features
        let type_id = self.get_property_type_id(&property.property_type);
        for i in 0..5 {
            embedding.push(if i == type_id { 1.0 } else { 0.0 });
        }
        
        // Pad to 128 dimensions
        while embedding.len() < 128 {
            embedding.push(0.0);
        }
        
        Ok(embedding)
    }

    /// Generate contact embedding vector (simplified for testing)
    fn generate_contact_embedding(&self, contact: &Contact) -> anyhow::Result<Vec<f32>> {
        let mut embedding = Vec::with_capacity(128);
        
        // Budget features
        embedding.push(contact.min_budget as f32 / 100_000_000.0);
        embedding.push(contact.max_budget as f32 / 100_000_000.0);
        embedding.push((contact.max_budget - contact.min_budget) as f32 / 100_000_000.0);
        
        // Room preferences (using min_rooms only since Contact doesn't have max_rooms)
        embedding.push(contact.min_rooms as f32 / 10.0);
        embedding.push(contact.min_rooms as f32 / 10.0); // Duplicate since we don't have max_rooms
        
        // Property type preference (using first property type if available)
        let type_id = if contact.property_types.is_empty() || contact.property_types.contains(&"any".to_string()) { 4 } else { 
            self.get_property_type_id(&contact.property_types[0]) 
        };
        for i in 0..5 {
            embedding.push(if i == type_id { 1.0 } else { 0.0 });
        }
        
        // Pad to 128 dimensions
        while embedding.len() < 128 {
            embedding.push(0.0);
        }
        
        Ok(embedding)
    }

    /// Generate preference embedding for contact
    fn generate_preference_embedding(&self, contact: &Contact) -> Vec<f32> {
        let mut preferences = Vec::with_capacity(64);
        
        // Budget preference strength
        let budget_range = contact.budget_max - contact.budget_min;
        let budget_specificity = if budget_range > 0.0 {
            1.0 - (budget_range / contact.budget_max).min(1.0)
        } else {
            1.0
        };
        preferences.push(budget_specificity as f32);
        
        // Room preference strength (using only min_rooms)
        let room_range = 0.0; // Contact doesn't have max_rooms, so range is 0
        preferences.push(if room_range > 0.0 { 1.0 / (1.0 + room_range) } else { 1.0 });
        
        // Type specificity
        preferences.push(if contact.property_types.is_empty() || contact.property_types.contains(&"any".to_string()) { 0.1 } else { 0.9 });
        
        // Pad to 64 dimensions
        while preferences.len() < 64 {
            preferences.push(0.5); // Default medium preference strength
        }
        
        preferences
    }

    /// Extract location preferences
    fn extract_location_preferences(&self, contact: &Contact) -> Vec<LocationPreference> {
        let mut preferences = Vec::new();
        for (index, location) in contact.preferred_locations.iter().enumerate() {
            preferences.push(LocationPreference {
                location_id: index as u16, // Using index as ID since NamedLocation doesn't have numeric ID
                weight: 0.8,
                lat: 0.0,  // Would need to be fetched from a location service
                lon: 0.0,  // Would need to be fetched from a location service
                radius_km: 10.0,
            });
        }
        preferences
    }

    /// Generate sparse features for property
    fn generate_sparse_features(&self, property: &Property) -> Vec<f32> {
        let mut features = vec![0.0; 100];
        
        // Property type one-hot
        let type_id = self.get_property_type_id(&property.property_type);
        if type_id < features.len() {
            features[type_id] = 1.0;
        }
        
        features
    }

    /// Generate location embedding
    fn generate_location_embedding(&self, location: &Location) -> Vec<f32> {
        vec![
            location.lat as f32,
            location.lon as f32,
            (location.lat as f32).sin(),
            (location.lon as f32).cos(),
        ]
    }

    /// Get property type ID for categorical encoding
    fn get_property_type_id(&self, property_type: &str) -> usize {
        match property_type {
            "apartment" => 0,
            "house" => 1, 
            "office" => 2,
            "land" => 3,
            _ => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_tokenization() {
        let config = TextEmbedderConfig::default();
        let embedder = TextEmbedder::new(config);
        
        let tokens = embedder.tokenize("Beautiful apartment with modern amenities!");
        assert_eq!(tokens, vec!["beautiful", "apartment", "with", "modern", "amenities"]);
    }

    #[test]
    fn test_numerical_normalization() {
        let config = NormalizerConfig::default();
        let mut normalizer = NumericalNormalizer::new(config);
        
        let values = vec![100.0, 200.0, 300.0, 400.0, 500.0];
        let stats = normalizer.compute_feature_stats(&values);
        
        assert!((stats.mean - 300.0).abs() < 0.001);
        assert!(stats.min == 100.0);
        assert!(stats.max == 500.0);
    }
}
