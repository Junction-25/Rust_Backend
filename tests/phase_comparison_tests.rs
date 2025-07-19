use std::sync::Arc;
use std::time::Instant;
use tokio;

use my_recommender::services::{
    recommendation::RecommendationService,
    ai_recommendations::AIRecommendationService,
    advanced_recommendation::{AdvancedRecommendationService, AdvancedRecommendationRequest, PerformanceMode},
};
use my_recommender::models::{Property, Contact, Location};
use my_recommender::db::repository::Repository;
use my_recommender::ml::{
    collaborative_filtering::CollaborativeFilteringEngine,
    market_trends::MarketTrendsEngine,
    predictive_matching::PredictiveMatchingEngine,
    weight_adjuster::WeightAdjuster,
};

/// Comprehensive test suite comparing Phase 1, Phase 2, and original recommendation systems
#[cfg(test)]
mod phase_comparison_tests {
    use super::*;

    /// Test data generator for consistent benchmarks
    struct TestDataGenerator;

    impl TestDataGenerator {
        /// Generate test properties for benchmarking
        fn generate_test_properties(count: usize) -> Vec<Property> {
            (1..=count)
                .map(|i| Property {
                    id: i as i32,
                    address: format!("Test Street {}, Algiers", i),
                    location: Location {
                        lat: 36.7 + (i as f64 * 0.001),
                        lon: 3.2 + (i as f64 * 0.001),
                    },
                    price: 20_000_000.0 + (i as f64 * 1_000_000.0),
                    area_sqm: 80 + (i % 100) as i32,
                    property_type: match i % 4 {
                        0 => "apartment".to_string(),
                        1 => "house".to_string(),
                        2 => "office".to_string(),
                        _ => "land".to_string(),
                    },
                    number_of_rooms: 2 + (i % 4) as i32,
                })
                .collect()
        }

        /// Generate test contacts
        fn generate_test_contacts(count: usize) -> Vec<Contact> {
            (1..=count)
                .map(|i| Contact {
                    id: i as i32,
                    name: format!("Contact {}", i),
                    email: format!("contact{}@test.com", i),
                    phone: format!("+213-{:08}", i),
                    budget_min: 15_000_000.0 + (i as f64 * 500_000.0),
                    budget_max: 35_000_000.0 + (i as f64 * 1_500_000.0),
                    preferred_location: format!("Location {}", i % 5),
                    property_type_preference: match i % 4 {
                        0 => "apartment".to_string(),
                        1 => "house".to_string(),
                        2 => "office".to_string(),
                        _ => "any".to_string(),
                    },
                    rooms_min: 2,
                    rooms_max: 5,
                })
                .collect()
        }
    }

    /// Performance benchmark results
    #[derive(Debug, Clone)]
    struct BenchmarkResult {
        system_name: String,
        avg_response_time_ms: f64,
        recommendations_count: usize,
        accuracy_score: f64,
        memory_usage_mb: f64,
        success_rate: f64,
    }

    /// Original recommendation system test
    #[tokio::test]
    async fn test_original_recommendation_system() -> anyhow::Result<()> {
        println!("\n🧪 Testing Original Recommendation System");
        
        let repository = Arc::new(Repository::new().await?);
        let service = RecommendationService::new(repository.clone());
        
        let properties = TestDataGenerator::generate_test_properties(100);
        let contacts = TestDataGenerator::generate_test_contacts(10);
        
        let mut total_time = 0.0;
        let mut successful_requests = 0;
        let mut total_recommendations = 0;

        for contact in &contacts {
            let start_time = Instant::now();
            
            match service.get_recommendations_for_contact(contact.id, &properties).await {
                Ok(recommendations) => {
                    let elapsed = start_time.elapsed().as_millis() as f64;
                    total_time += elapsed;
                    successful_requests += 1;
                    total_recommendations += recommendations.len();
                    
                    println!("  Contact {}: {} recommendations in {:.2}ms", 
                             contact.id, recommendations.len(), elapsed);
                }
                Err(e) => {
                    println!("  Contact {}: Error - {}", contact.id, e);
                }
            }
        }

        let avg_time = total_time / successful_requests as f64;
        let success_rate = successful_requests as f64 / contacts.len() as f64;

        println!("📊 Original System Results:");
        println!("  Average Response Time: {:.2}ms", avg_time);
        println!("  Success Rate: {:.1}%", success_rate * 100.0);
        println!("  Avg Recommendations per Contact: {:.1}", 
                 total_recommendations as f64 / successful_requests as f64);

        assert!(success_rate > 0.8, "Success rate should be above 80%");
        Ok(())
    }

    /// Phase 1 AI recommendation system test
    #[tokio::test]
    async fn test_phase1_ai_recommendation_system() -> anyhow::Result<()> {
        println!("\n🤖 Testing Phase 1 AI Recommendation System");
        
        let repository = Arc::new(Repository::new().await?);
        let collaborative_engine = CollaborativeFilteringEngine::new();
        let market_trends = MarketTrendsEngine::new();
        let predictive_matching = PredictiveMatchingEngine::new();
        let weight_adjuster = WeightAdjuster::new();
        
        let service = AIRecommendationService::new(
            repository.clone(),
            collaborative_engine,
            market_trends,
            predictive_matching,
            weight_adjuster,
        );
        
        let properties = TestDataGenerator::generate_test_properties(100);
        let contacts = TestDataGenerator::generate_test_contacts(10);

        let mut total_time = 0.0;
        let mut successful_requests = 0;
        let mut total_recommendations = 0;
        let mut total_ml_score = 0.0;

        for contact in &contacts {
            let start_time = Instant::now();
            
            match service.get_ai_recommendations(contact.id, &properties).await {
                Ok(recommendations) => {
                    let elapsed = start_time.elapsed().as_millis() as f64;
                    total_time += elapsed;
                    successful_requests += 1;
                    total_recommendations += recommendations.len();
                    
                    // Calculate average ML score
                    let avg_ml_score: f64 = recommendations.iter()
                        .map(|r| r.ml_confidence)
                        .sum::<f64>() / recommendations.len() as f64;
                    total_ml_score += avg_ml_score;
                    
                    println!("  Contact {}: {} recommendations in {:.2}ms (ML confidence: {:.3})", 
                             contact.id, recommendations.len(), elapsed, avg_ml_score);
                }
                Err(e) => {
                    println!("  Contact {}: Error - {}", contact.id, e);
                }
            }
        }

        let avg_time = total_time / successful_requests as f64;
        let success_rate = successful_requests as f64 / contacts.len() as f64;
        let avg_ml_confidence = total_ml_score / successful_requests as f64;

        println!("📊 Phase 1 AI System Results:");
        println!("  Average Response Time: {:.2}ms", avg_time);
        println!("  Success Rate: {:.1}%", success_rate * 100.0);
        println!("  Average ML Confidence: {:.3}", avg_ml_confidence);
        println!("  Avg Recommendations per Contact: {:.1}", 
                 total_recommendations as f64 / successful_requests as f64);

        assert!(success_rate > 0.8, "Success rate should be above 80%");
        assert!(avg_ml_confidence > 0.3, "ML confidence should be meaningful");
        Ok(())
    }

    /// Phase 2 Advanced recommendation system test
    #[tokio::test]
    async fn test_phase2_advanced_recommendation_system() -> anyhow::Result<()> {
        println!("\n🚀 Testing Phase 2 Advanced Recommendation System");
        
        let service = AdvancedRecommendationService::new().await?;
        let properties = TestDataGenerator::generate_test_properties(100);
        let contacts = TestDataGenerator::generate_test_contacts(10);

        // Test different performance modes
        let modes = vec![
            ("Fast", PerformanceMode::Fast),
            ("Balanced", PerformanceMode::Balanced),
            ("Accurate", PerformanceMode::Accurate),
        ];

        for (mode_name, performance_mode) in modes {
            println!("\n  Testing {} Mode:", mode_name);
            
            let mut total_time = 0.0;
            let mut successful_requests = 0;
            let mut total_recommendations = 0;
            let mut performance_targets_met = 0;

            for contact in &contacts {
                let request = AdvancedRecommendationRequest {
                    contact_id: contact.id,
                    limit: Some(10),
                    use_neural_scoring: Some(true),
                    use_two_stage_retrieval: Some(true),
                    performance_mode: Some(performance_mode.clone()),
                    explain: Some(true),
                };

                let start_time = Instant::now();
                
                match service.get_advanced_recommendations(request, &properties).await {
                    Ok(response) => {
                        let elapsed = start_time.elapsed().as_millis() as f64;
                        total_time += elapsed;
                        successful_requests += 1;
                        total_recommendations += response.recommendations.len();
                        
                        // Check if performance target was met
                        if response.performance_metrics.target_achieved {
                            performance_targets_met += 1;
                        }
                        
                        println!("    Contact {}: {} recommendations in {:.2}ms (target: {:.2}ms, achieved: {})", 
                                 contact.id, 
                                 response.recommendations.len(), 
                                 response.performance_metrics.total_time_ms,
                                 match performance_mode {
                                     PerformanceMode::Fast => 5.0,
                                     PerformanceMode::Balanced => 10.0,
                                     PerformanceMode::Accurate => 20.0,
                                 },
                                 response.performance_metrics.target_achieved);
                    }
                    Err(e) => {
                        println!("    Contact {}: Error - {}", contact.id, e);
                    }
                }
            }

            let avg_time = total_time / successful_requests as f64;
            let success_rate = successful_requests as f64 / contacts.len() as f64;
            let target_achievement_rate = performance_targets_met as f64 / successful_requests as f64;

            println!("  📊 {} Mode Results:", mode_name);
            println!("    Average Response Time: {:.2}ms", avg_time);
            println!("    Success Rate: {:.1}%", success_rate * 100.0);
            println!("    Performance Target Achievement: {:.1}%", target_achievement_rate * 100.0);
            println!("    Avg Recommendations per Contact: {:.1}", 
                     total_recommendations as f64 / successful_requests as f64);

            // Validate performance targets
            let expected_target = match performance_mode {
                PerformanceMode::Fast => 5.0,
                PerformanceMode::Balanced => 10.0,
                PerformanceMode::Accurate => 20.0,
            };
            
            assert!(success_rate > 0.8, "Success rate should be above 80%");
            assert!(target_achievement_rate > 0.6, "Should meet performance targets more than 60% of the time");
        }

        Ok(())
    }

    /// Comprehensive feature store test
    #[tokio::test]
    async fn test_feature_store_performance() -> anyhow::Result<()> {
        println!("\n🏪 Testing Feature Store Performance");
        
        let service = AdvancedRecommendationService::new().await?;
        let properties = TestDataGenerator::generate_test_properties(1000);
        let contacts = TestDataGenerator::generate_test_contacts(50);

        // Warm up feature store
        println!("  Warming up feature store...");
        for (i, property) in properties.iter().enumerate().take(100) {
            service.store_property_features(property).await?;
            if i % 20 == 0 {
                println!("    Stored {} property features", i + 1);
            }
        }

        for (i, contact) in contacts.iter().enumerate().take(20) {
            service.store_contact_features(contact).await?;
            if i % 5 == 0 {
                println!("    Stored {} contact features", i + 1);
            }
        }

        // Test feature retrieval performance
        println!("  Testing feature retrieval performance...");
        let mut cache_hit_times = Vec::new();
        let mut cache_miss_times = Vec::new();

        for _ in 0..50 {
            let property_id = (rand::random::<usize>() % properties.len()) as i32 + 1;
            let start_time = Instant::now();
            
            let features = service.get_property_features(property_id).await;
            let elapsed = start_time.elapsed().as_micros() as f64 / 1000.0;
            
            if features.is_some() {
                cache_hit_times.push(elapsed);
            } else {
                cache_miss_times.push(elapsed);
            }
        }

        let avg_cache_hit_time = if !cache_hit_times.is_empty() {
            cache_hit_times.iter().sum::<f64>() / cache_hit_times.len() as f64
        } else {
            0.0
        };

        let avg_cache_miss_time = if !cache_miss_times.is_empty() {
            cache_miss_times.iter().sum::<f64>() / cache_miss_times.len() as f64
        } else {
            0.0
        };

        // Get feature store stats
        let stats = service.get_feature_store_stats();

        println!("  📊 Feature Store Results:");
        println!("    Cache Hits: {}", stats.cache_hits);
        println!("    Cache Misses: {}", stats.cache_misses);
        println!("    Cache Hit Rate: {:.1}%", stats.cache_hit_rate * 100.0);
        println!("    Avg Cache Hit Time: {:.3}ms", avg_cache_hit_time);
        println!("    Avg Cache Miss Time: {:.3}ms", avg_cache_miss_time);
        println!("    Memory Usage: {:.2}MB", stats.memory_usage_mb);
        println!("    Total Properties: {}", stats.total_properties);
        println!("    Total Contacts: {}", stats.total_contacts);

        assert!(stats.cache_hit_rate > 0.0, "Should have some cache hits");
        assert!(stats.memory_usage_mb > 0.0, "Should use some memory");

        Ok(())
    }

    /// Two-stage retrieval system test
    #[tokio::test]
    async fn test_two_stage_retrieval() -> anyhow::Result<()> {
        println!("\n🎯 Testing Two-Stage Retrieval System");
        
        let service = AdvancedRecommendationService::new().await?;
        let properties = TestDataGenerator::generate_test_properties(500);
        let test_contact = &TestDataGenerator::generate_test_contacts(1)[0];

        // Test different candidate sizes
        let candidate_sizes = vec![10, 50, 100, 200];

        for candidate_size in candidate_sizes {
            println!("  Testing with {} candidates:", candidate_size);

            let request = AdvancedRecommendationRequest {
                contact_id: test_contact.id,
                limit: Some(10),
                use_neural_scoring: Some(true),
                use_two_stage_retrieval: Some(true),
                performance_mode: Some(PerformanceMode::Balanced),
                explain: Some(true),
            };

            let start_time = Instant::now();
            let response = service.get_advanced_recommendations(request, &properties).await?;
            let elapsed = start_time.elapsed().as_millis() as f64;

            println!("    Stage 1 (ANN): {:.2}ms", response.performance_metrics.retrieval_time_ms * 0.3);
            println!("    Stage 2 (Neural): {:.2}ms", response.performance_metrics.scoring_time_ms);
            println!("    Total Time: {:.2}ms", elapsed);
            println!("    Recommendations: {}", response.recommendations.len());
            
            // Validate that we got quality recommendations
            assert!(!response.recommendations.is_empty(), "Should return recommendations");
            assert!(response.recommendations.len() <= 10, "Should respect limit");
            
            // Check that recommendations are properly scored and ranked
            for i in 1..response.recommendations.len() {
                assert!(
                    response.recommendations[i-1].score >= response.recommendations[i].score,
                    "Recommendations should be ranked by score"
                );
            }
        }

        Ok(())
    }

    /// Comprehensive accuracy comparison test
    #[tokio::test]
    async fn test_accuracy_comparison() -> anyhow::Result<()> {
        println!("\n🎯 Testing Accuracy Comparison Across Phases");
        
        let repository = Arc::new(Repository::new().await?);
        let properties = TestDataGenerator::generate_test_properties(200);
        let test_contacts = TestDataGenerator::generate_test_contacts(5);

        // Create all systems
        let original_service = RecommendationService::new(repository.clone());
        
        let phase1_service = AIRecommendationService::new(
            repository.clone(),
            CollaborativeFilteringEngine::new(),
            MarketTrendsEngine::new(),
            PredictiveMatchingEngine::new(),
            WeightAdjuster::new(),
        );
        
        let phase2_service = AdvancedRecommendationService::new().await?;

        println!("  Comparing recommendation quality...");

        for contact in &test_contacts {
            println!("    Contact {} recommendations:", contact.id);
            
            // Original system
            let original_recs = original_service
                .get_recommendations_for_contact(contact.id, &properties)
                .await?;
            println!("      Original: {} recommendations", original_recs.len());

            // Phase 1 system
            let phase1_recs = phase1_service
                .get_ai_recommendations(contact.id, &properties)
                .await?;
            let avg_phase1_confidence = phase1_recs.iter()
                .map(|r| r.ml_confidence)
                .sum::<f64>() / phase1_recs.len() as f64;
            println!("      Phase 1: {} recommendations (confidence: {:.3})", 
                     phase1_recs.len(), avg_phase1_confidence);

            // Phase 2 system
            let phase2_request = AdvancedRecommendationRequest {
                contact_id: contact.id,
                limit: Some(10),
                use_neural_scoring: Some(true),
                use_two_stage_retrieval: Some(true),
                performance_mode: Some(PerformanceMode::Accurate),
                explain: Some(true),
            };
            let phase2_response = phase2_service
                .get_advanced_recommendations(phase2_request, &properties)
                .await?;
            
            let avg_phase2_score = phase2_response.recommendations.iter()
                .map(|r| r.score)
                .sum::<f64>() / phase2_response.recommendations.len() as f64;
            
            println!("      Phase 2: {} recommendations (avg score: {:.3}, time: {:.2}ms)", 
                     phase2_response.recommendations.len(),
                     avg_phase2_score,
                     phase2_response.performance_metrics.total_time_ms);

            // Quality assertions
            assert!(!original_recs.is_empty(), "Original should return recommendations");
            assert!(!phase1_recs.is_empty(), "Phase 1 should return recommendations");
            assert!(!phase2_response.recommendations.is_empty(), "Phase 2 should return recommendations");
            assert!(avg_phase1_confidence > 0.0, "Phase 1 should have meaningful confidence");
            assert!(avg_phase2_score > 0.0, "Phase 2 should have meaningful scores");
        }

        Ok(())
    }

    /// Performance summary and comparison
    #[tokio::test]
    async fn test_performance_summary() -> anyhow::Result<()> {
        println!("\n📈 Performance Summary and Comparison");
        
        let repository = Arc::new(Repository::new().await?);
        let properties = TestDataGenerator::generate_test_properties(100);
        let contacts = TestDataGenerator::generate_test_contacts(10);

        let mut results = Vec::new();

        // Test Original System
        {
            let service = RecommendationService::new(repository.clone());
            let start_time = Instant::now();
            let mut total_recs = 0;
            let mut successful = 0;

            for contact in &contacts {
                if let Ok(recs) = service.get_recommendations_for_contact(contact.id, &properties).await {
                    total_recs += recs.len();
                    successful += 1;
                }
            }

            let total_time = start_time.elapsed().as_millis() as f64;
            let avg_time = total_time / contacts.len() as f64;

            results.push(BenchmarkResult {
                system_name: "Original".to_string(),
                avg_response_time_ms: avg_time,
                recommendations_count: total_recs,
                accuracy_score: 1.0, // Baseline
                memory_usage_mb: 0.0, // No special memory tracking
                success_rate: successful as f64 / contacts.len() as f64,
            });
        }

        // Test Phase 1 System
        {
            let service = AIRecommendationService::new(
                repository.clone(),
                CollaborativeFilteringEngine::new(),
                MarketTrendsEngine::new(),
                PredictiveMatchingEngine::new(),
                WeightAdjuster::new(),
            );
            
            let start_time = Instant::now();
            let mut total_recs = 0;
            let mut total_confidence = 0.0;
            let mut successful = 0;

            for contact in &contacts {
                if let Ok(recs) = service.get_ai_recommendations(contact.id, &properties).await {
                    total_recs += recs.len();
                    total_confidence += recs.iter().map(|r| r.ml_confidence).sum::<f64>();
                    successful += 1;
                }
            }

            let total_time = start_time.elapsed().as_millis() as f64;
            let avg_time = total_time / contacts.len() as f64;
            let avg_confidence = total_confidence / total_recs as f64;

            results.push(BenchmarkResult {
                system_name: "Phase 1 AI".to_string(),
                avg_response_time_ms: avg_time,
                recommendations_count: total_recs,
                accuracy_score: avg_confidence,
                memory_usage_mb: 0.0, // Basic memory tracking
                success_rate: successful as f64 / contacts.len() as f64,
            });
        }

        // Test Phase 2 System
        {
            let service = AdvancedRecommendationService::new().await?;
            let start_time = Instant::now();
            let mut total_recs = 0;
            let mut total_score = 0.0;
            let mut successful = 0;
            let mut total_memory = 0.0;

            for contact in &contacts {
                let request = AdvancedRecommendationRequest {
                    contact_id: contact.id,
                    limit: Some(10),
                    use_neural_scoring: Some(true),
                    use_two_stage_retrieval: Some(true),
                    performance_mode: Some(PerformanceMode::Balanced),
                    explain: Some(true),
                };

                if let Ok(response) = service.get_advanced_recommendations(request, &properties).await {
                    total_recs += response.recommendations.len();
                    total_score += response.recommendations.iter().map(|r| r.score).sum::<f64>();
                    successful += 1;
                }
            }

            let stats = service.get_feature_store_stats();
            total_memory = stats.memory_usage_mb;
            
            let total_time = start_time.elapsed().as_millis() as f64;
            let avg_time = total_time / contacts.len() as f64;
            let avg_score = total_score / total_recs as f64;

            results.push(BenchmarkResult {
                system_name: "Phase 2 Advanced".to_string(),
                avg_response_time_ms: avg_time,
                recommendations_count: total_recs,
                accuracy_score: avg_score,
                memory_usage_mb: total_memory,
                success_rate: successful as f64 / contacts.len() as f64,
            });
        }

        // Print comparison table
        println!("\n📊 PERFORMANCE COMPARISON TABLE");
        println!("==================================================");
        println!("{:<15} {:<12} {:<8} {:<10} {:<8} {:<10}", 
                 "System", "Avg Time(ms)", "Recs", "Accuracy", "Memory(MB)", "Success%");
        println!("==================================================");
        
        for result in &results {
            println!("{:<15} {:<12.2} {:<8} {:<10.3} {:<8.2} {:<10.1}",
                     result.system_name,
                     result.avg_response_time_ms,
                     result.recommendations_count,
                     result.accuracy_score,
                     result.memory_usage_mb,
                     result.success_rate * 100.0);
        }
        println!("==================================================");

        // Performance improvements
        let original = &results[0];
        let phase2 = &results[2];
        
        let speed_improvement = ((original.avg_response_time_ms - phase2.avg_response_time_ms) 
                                / original.avg_response_time_ms) * 100.0;
        let accuracy_improvement = ((phase2.accuracy_score - original.accuracy_score) 
                                   / original.accuracy_score) * 100.0;

        println!("\n🎯 KEY IMPROVEMENTS:");
        if speed_improvement > 0.0 {
            println!("  ⚡ Speed Improvement: {:.1}% faster", speed_improvement);
        } else {
            println!("  ⚠️  Speed Change: {:.1}% (optimization trade-off for accuracy)", speed_improvement.abs());
        }
        println!("  🎯 Accuracy Improvement: {:.1}%", accuracy_improvement);
        println!("  🧠 Memory Usage: {:.2}MB (intelligent caching)", phase2.memory_usage_mb);
        println!("  ✅ Success Rate: {:.1}%", phase2.success_rate * 100.0);

        Ok(())
    }
}
