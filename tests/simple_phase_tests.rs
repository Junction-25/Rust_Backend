use std::sync::Arc;
use std::time::Instant;
use tokio;

use my_recommender::services::recommendation::RecommendationService;
use my_recommender::models::{Property, Contact, Location};
use my_recommender::db::repository::Repository;

/// Simple Phase Comparison Test - focuses on basic functionality and performance
#[cfg(test)]
mod simple_phase_tests {
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
                    preferred_locations: vec![format!("Location {}", i % 5)],
                    min_budget: 15_000_000.0 + (i as f64 * 500_000.0),
                    max_budget: 35_000_000.0 + (i as f64 * 1_500_000.0),
                    property_type: match i % 4 {
                        0 => "apartment".to_string(),
                        1 => "house".to_string(),
                        2 => "office".to_string(),
                        _ => "any".to_string(),
                    },
                    min_rooms: 2,
                    max_rooms: 5,
                    contact_info: format!("contact{}@test.com", i),
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
        success_rate: f64,
        total_requests: usize,
        properties_processed: usize,
    }

    /// Test basic recommendation system performance
    #[tokio::test]
    async fn test_basic_recommendation_performance() -> anyhow::Result<()> {
        println!("\n🧪 Testing Basic Recommendation System Performance");
        
        let repository = Arc::new(Repository::new().await?);
        let service = RecommendationService::new(repository.clone());
        
        // Test with different data sizes
        let test_sizes = vec![
            (50, 5, "Small Dataset"),
            (100, 10, "Medium Dataset"),
            (200, 15, "Large Dataset"),
        ];

        let mut results = Vec::new();

        for (property_count, contact_count, size_name) in test_sizes {
            println!("\n  Testing with {} ({}P, {}C):", size_name, property_count, contact_count);
            
            let properties = TestDataGenerator::generate_test_properties(property_count);
            let contacts = TestDataGenerator::generate_test_contacts(contact_count);
            
            let mut total_time = 0.0;
            let mut successful_requests = 0;
            let mut total_recommendations = 0;
            let start_benchmark = Instant::now();

            for contact in &contacts {
                let start_time = Instant::now();
                
                match service.get_recommendations_for_contact(contact.id, &properties).await {
                    Ok(recommendations) => {
                        let elapsed = start_time.elapsed().as_millis() as f64;
                        total_time += elapsed;
                        successful_requests += 1;
                        total_recommendations += recommendations.len();
                        
                        if successful_requests <= 3 {
                            println!("    Contact {}: {} recommendations in {:.2}ms", 
                                     contact.id, recommendations.len(), elapsed);
                        }
                    }
                    Err(e) => {
                        println!("    Contact {}: Error - {}", contact.id, e);
                    }
                }
            }

            let benchmark_total = start_benchmark.elapsed().as_millis() as f64;
            let avg_time = total_time / successful_requests as f64;
            let success_rate = successful_requests as f64 / contacts.len() as f64;

            let result = BenchmarkResult {
                system_name: format!("Basic-{}", size_name),
                avg_response_time_ms: avg_time,
                recommendations_count: total_recommendations,
                success_rate,
                total_requests: contacts.len(),
                properties_processed: property_count,
            };
            results.push(result.clone());

            println!("  📊 {} Results:", size_name);
            println!("    Average Response Time: {:.2}ms", avg_time);
            println!("    Success Rate: {:.1}%", success_rate * 100.0);
            println!("    Total Benchmark Time: {:.2}ms", benchmark_total);
            println!("    Recommendations per Contact: {:.1}", 
                     total_recommendations as f64 / successful_requests as f64);
            println!("    Throughput: {:.2} requests/second", 
                     successful_requests as f64 / (benchmark_total / 1000.0));

            assert!(success_rate > 0.8, "Success rate should be above 80% for {}", size_name);
        }

        // Print comparison summary
        println!("\n📈 PERFORMANCE SCALING ANALYSIS");
        println!("==================================================================");
        println!("{:<15} {:<12} {:<8} {:<10} {:<12} {:<10}", 
                 "Dataset", "Avg Time(ms)", "Recs", "Success%", "Properties", "Req/sec");
        println!("==================================================================");
        
        for result in &results {
            let throughput = result.total_requests as f64 / (result.avg_response_time_ms / 1000.0 * result.total_requests as f64);
            println!("{:<15} {:<12.2} {:<8} {:<10.1} {:<12} {:<10.2}",
                     result.system_name.replace("Basic-", ""),
                     result.avg_response_time_ms,
                     result.recommendations_count,
                     result.success_rate * 100.0,
                     result.properties_processed,
                     throughput);
        }
        println!("==================================================================");

        // Analyze scaling characteristics
        if results.len() >= 2 {
            let small = &results[0];
            let large = &results[results.len() - 1];
            
            let time_scaling = large.avg_response_time_ms / small.avg_response_time_ms;
            let data_scaling = large.properties_processed as f64 / small.properties_processed as f64;
            
            println!("\n🔍 SCALING ANALYSIS:");
            println!("  Data Size Increase: {:.1}x", data_scaling);
            println!("  Response Time Increase: {:.1}x", time_scaling);
            println!("  Scaling Efficiency: {:.1}% (lower is better)", 
                     (time_scaling / data_scaling) * 100.0);
            
            if time_scaling < data_scaling * 1.5 {
                println!("  ✅ Good scaling characteristics - sublinear performance degradation");
            } else {
                println!("  ⚠️  Performance scales worse than linearly with data size");
            }
        }

        Ok(())
    }

    /// Test recommendation accuracy and quality
    #[tokio::test]
    async fn test_recommendation_quality() -> anyhow::Result<()> {
        println!("\n🎯 Testing Recommendation Quality and Relevance");
        
        let repository = Arc::new(Repository::new().await?);
        let service = RecommendationService::new(repository.clone());
        
        // Create targeted test scenarios
        let properties = vec![
            // Perfect match scenario
            Property {
                id: 1,
                address: "Perfect Street, Algiers".to_string(),
                location: Location { lat: 36.7, lon: 3.2 },
                price: 25_000_000.0,
                area_sqm: 100,
                property_type: "apartment".to_string(),
                number_of_rooms: 3,
            },
            // Close match scenario
            Property {
                id: 2,
                address: "Good Street, Algiers".to_string(),
                location: Location { lat: 36.71, lon: 3.21 },
                price: 27_000_000.0,
                area_sqm: 95,
                property_type: "apartment".to_string(),
                number_of_rooms: 3,
            },
            // Poor match scenario
            Property {
                id: 3,
                address: "Expensive Avenue, Oran".to_string(),
                location: Location { lat: 35.7, lon: -0.6 },
                price: 50_000_000.0,
                area_sqm: 200,
                property_type: "house".to_string(),
                number_of_rooms: 5,
            },
        ];

        // Specific contact looking for an apartment in Algiers
        let target_contact = Contact {
            id: 1,
            name: "Test Buyer".to_string(),
            preferred_locations: vec!["Algiers".to_string()],
            min_budget: 20_000_000.0,
            max_budget: 30_000_000.0,
            property_type: "apartment".to_string(),
            min_rooms: 2,
            max_rooms: 4,
            contact_info: "buyer@test.com".to_string(),
        };

        let recommendations = service.get_recommendations_for_contact(target_contact.id, &properties).await?;
        
        println!("  Target Contact Profile:");
        println!("    Budget: {:.1}M - {:.1}M DZD", 
                 target_contact.min_budget / 1_000_000.0,
                 target_contact.max_budget / 1_000_000.0);
        println!("    Type: {}", target_contact.property_type);
        println!("    Rooms: {} - {}", target_contact.min_rooms, target_contact.max_rooms);
        println!("    Location: {:?}", target_contact.preferred_locations);

        println!("\n  Recommendation Results:");
        for (i, rec) in recommendations.iter().enumerate() {
            let property = properties.iter().find(|p| p.id == rec.property_id).unwrap();
            println!("    {}. Property {} (Score: {:.3})", 
                     i + 1, property.id, rec.score);
            println!("       Price: {:.1}M DZD, {}, {} rooms", 
                     property.price / 1_000_000.0,
                     property.property_type, 
                     property.number_of_rooms);
            println!("       Address: {}", property.address);
            
            // Quality checks
            let budget_match = property.price >= target_contact.min_budget && 
                             property.price <= target_contact.max_budget;
            let type_match = property.property_type == target_contact.property_type;
            let room_match = property.number_of_rooms >= target_contact.min_rooms && 
                           property.number_of_rooms <= target_contact.max_rooms;
            
            println!("       Quality: Budget:{} Type:{} Rooms:{}", 
                     if budget_match { "✅" } else { "❌" },
                     if type_match { "✅" } else { "❌" },
                     if room_match { "✅" } else { "❌" });
        }

        // Quality assertions
        assert!(!recommendations.is_empty(), "Should return recommendations");
        assert!(recommendations.len() <= properties.len(), "Should not exceed available properties");
        
        // Check that recommendations are sorted by score
        for i in 1..recommendations.len() {
            assert!(recommendations[i-1].score >= recommendations[i].score,
                   "Recommendations should be sorted by score (descending)");
        }

        // Check that the perfect match (Property 1) has the highest score
        if let Some(perfect_match) = recommendations.iter().find(|r| r.property_id == 1) {
            let perfect_score = perfect_match.score;
            let all_other_scores: Vec<f64> = recommendations.iter()
                .filter(|r| r.property_id != 1)
                .map(|r| r.score)
                .collect();
            
            for other_score in all_other_scores {
                assert!(perfect_score >= other_score, 
                       "Perfect match should have highest or equal score");
            }
            println!("  ✅ Perfect match property has optimal score: {:.3}", perfect_score);
        }

        println!("  📊 Quality Summary:");
        let budget_matches = recommendations.iter().filter(|r| {
            let prop = properties.iter().find(|p| p.id == r.property_id).unwrap();
            prop.price >= target_contact.min_budget && prop.price <= target_contact.max_budget
        }).count();
        
        let type_matches = recommendations.iter().filter(|r| {
            let prop = properties.iter().find(|p| p.id == r.property_id).unwrap();
            prop.property_type == target_contact.property_type
        }).count();

        println!("    Budget Matches: {}/{} ({:.1}%)", 
                 budget_matches, recommendations.len(),
                 budget_matches as f64 / recommendations.len() as f64 * 100.0);
        println!("    Type Matches: {}/{} ({:.1}%)", 
                 type_matches, recommendations.len(),
                 type_matches as f64 / recommendations.len() as f64 * 100.0);

        Ok(())
    }

    /// Test system behavior under different load conditions
    #[tokio::test]
    async fn test_system_load_behavior() -> anyhow::Result<()> {
        println!("\n⚡ Testing System Load Behavior");
        
        let repository = Arc::new(Repository::new().await?);
        let service = RecommendationService::new(repository.clone());
        
        let properties = TestDataGenerator::generate_test_properties(100);
        
        // Test concurrent requests
        let contact_count = 20;
        let contacts = TestDataGenerator::generate_test_contacts(contact_count);
        
        println!("  Testing {} concurrent requests...", contact_count);
        
        let start_time = Instant::now();
        
        // Create concurrent tasks
        let mut tasks = Vec::new();
        for contact in contacts {
            let service_clone = service.clone();
            let properties_clone = properties.clone();
            
            let task = tokio::spawn(async move {
                let request_start = Instant::now();
                let result = service_clone
                    .get_recommendations_for_contact(contact.id, &properties_clone)
                    .await;
                let request_time = request_start.elapsed().as_millis() as f64;
                (contact.id, result, request_time)
            });
            
            tasks.push(task);
        }
        
        // Wait for all tasks to complete
        let mut successful = 0;
        let mut total_time = 0.0;
        let mut max_time = 0.0;
        let mut min_time = f64::INFINITY;
        
        for task in tasks {
            if let Ok((contact_id, result, request_time)) = task.await {
                match result {
                    Ok(recommendations) => {
                        successful += 1;
                        total_time += request_time;
                        max_time = max_time.max(request_time);
                        min_time = min_time.min(request_time);
                        
                        if successful <= 3 {
                            println!("    Contact {}: {} recs in {:.2}ms", 
                                     contact_id, recommendations.len(), request_time);
                        }
                    }
                    Err(e) => {
                        println!("    Contact {}: Error - {}", contact_id, e);
                    }
                }
            }
        }
        
        let total_elapsed = start_time.elapsed().as_millis() as f64;
        let avg_response_time = total_time / successful as f64;
        let throughput = successful as f64 / (total_elapsed / 1000.0);
        
        println!("  📊 Concurrent Load Test Results:");
        println!("    Successful Requests: {}/{}", successful, contact_count);
        println!("    Total Time: {:.2}ms", total_elapsed);
        println!("    Average Response Time: {:.2}ms", avg_response_time);
        println!("    Min Response Time: {:.2}ms", min_time);
        println!("    Max Response Time: {:.2}ms", max_time);
        println!("    Throughput: {:.2} requests/second", throughput);
        println!("    Response Time Variance: {:.2}ms", max_time - min_time);
        
        // Performance assertions
        assert!(successful as f64 / contact_count as f64 > 0.9, 
               "Should handle at least 90% of concurrent requests successfully");
        assert!(avg_response_time < 1000.0, 
               "Average response time should be under 1 second");
        assert!(throughput > 5.0, 
               "Should achieve at least 5 requests per second throughput");
        
        // Check for reasonable response time consistency
        let variance_ratio = (max_time - min_time) / avg_response_time;
        if variance_ratio < 2.0 {
            println!("  ✅ Good response time consistency (variance ratio: {:.2})", variance_ratio);
        } else {
            println!("  ⚠️  High response time variance (ratio: {:.2})", variance_ratio);
        }

        Ok(())
    }
}
