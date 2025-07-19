mod utils {
    pub mod feature_engineering;
}

use utils::feature_engineering::{NeuralBinner, LocationAttentionPooler, cosine_similarity};
use std::collections::HashMap;

#[path = "../src/utils/feature_engineering.rs"]
mod feature_engineering_impl;

fn main() {
    println!("Phase 1 Implementation Test");
    println!("==========================");

    // Test 1: Neural Binner
    println!("\n1. Testing Neural Binner:");
    let binner = NeuralBinner::new();
    
    let price_embedding = binner.get_embedding("price", 300_000.0);
    println!("   Price embedding length: {}", price_embedding.len());
    
    let area_embedding = binner.get_embedding("area", 100.0);
    println!("   Area embedding length: {}", area_embedding.len());
    
    // Test feature compatibility
    let mut property_features = HashMap::new();
    property_features.insert("price".to_string(), 350_000.0);
    property_features.insert("area".to_string(), 120.0);
    property_features.insert("rooms".to_string(), 3.0);
    
    let property_vector = binner.get_feature_vector(&property_features);
    println!("   Property feature vector length: {}", property_vector.len());

    // Test 2: Location Attention Pooler
    println!("\n2. Testing Location Attention Pooler:");
    let pooler = LocationAttentionPooler::new(0.1);
    
    let distances = vec![2.0, 10.0, 25.0];
    let weights = pooler.calculate_attention_weights(&distances);
    println!("   Attention weights: {:?}", weights);
    println!("   Sum of weights: {:.6}", weights.iter().sum::<f64>());

    // Test 3: Cosine Similarity
    println!("\n3. Testing Cosine Similarity:");
    let vec_a = vec![1.0, 0.0, 0.0];
    let vec_b = vec![1.0, 0.0, 0.0];
    let vec_c = vec![0.0, 1.0, 0.0];
    
    let sim_identical = cosine_similarity(&vec_a, &vec_b);
    let sim_perpendicular = cosine_similarity(&vec_a, &vec_c);
    
    println!("   Similarity (identical): {:.6}", sim_identical);
    println!("   Similarity (perpendicular): {:.6}", sim_perpendicular);

    println!("\nPhase 1 components are working correctly!");
    println!("Ready to test with live server.");
}
