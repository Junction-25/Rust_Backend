use crate::db::Repository;
use crate::ml::{CollaborativeFilteringEngine, MarketTrendsEngine, PredictiveMatchingEngine};
use crate::ml::{MLRecommendation, PredictiveMatchResult, MarketTrend, PricePredictor};
use crate::models::{Contact, Property, Recommendation};
use crate::services::recommendation::RecommendationService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRecommendationRequest {
    pub contact_id: i32,
    pub property_ids: Option<Vec<i32>>,
    pub enable_ml_scoring: bool,
    pub enable_market_analysis: bool,
    pub enable_predictive_matching: bool,
    pub include_price_predictions: bool,
    pub min_confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRecommendationResponse {
    pub recommendations: Vec<EnhancedRecommendation>,
    pub market_insights: Vec<String>,
    pub price_predictions: HashMap<i32, PricePredictor>,
    pub contact_behavior_insights: Option<ContactBehaviorSummary>,
    pub total_count: usize,
    pub processing_time_ms: u64,
    pub ai_model_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedRecommendation {
    pub recommendation: Recommendation,
    pub ml_enhancement: Option<MLRecommendation>,
    pub predictive_analysis: Option<PredictiveMatchResult>,
    pub market_trend: Option<MarketTrend>,
    pub ai_insights: Vec<String>,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactBehaviorSummary {
    pub contact_id: i32,
    pub decisiveness_level: String,
    pub price_sensitivity_level: String,
    pub flexibility_score: f64,
    pub predicted_timeline: String,
    pub engagement_level: String,
    pub recommendations: Vec<String>,
}

#[derive(Clone)]
pub struct AIRecommendationService {
    repository: Arc<Repository>,
    traditional_service: Arc<RecommendationService>,
    cf_engine: Arc<tokio::sync::Mutex<CollaborativeFilteringEngine>>,
    market_engine: Arc<tokio::sync::Mutex<MarketTrendsEngine>>,
    predictive_engine: Arc<tokio::sync::Mutex<PredictiveMatchingEngine>>,
    is_initialized: Arc<tokio::sync::Mutex<bool>>,
}

impl AIRecommendationService {
    pub fn new(
        repository: Arc<Repository>,
        traditional_service: Arc<RecommendationService>,
    ) -> Self {
        Self {
            repository,
            traditional_service,
            cf_engine: Arc::new(tokio::sync::Mutex::new(CollaborativeFilteringEngine::new())),
            market_engine: Arc::new(tokio::sync::Mutex::new(MarketTrendsEngine::new())),
            predictive_engine: Arc::new(tokio::sync::Mutex::new(PredictiveMatchingEngine::new())),
            is_initialized: Arc::new(tokio::sync::Mutex::new(false)),
        }
    }

    /// Initialize AI models with current data
    pub async fn initialize_models(&self) -> Result<()> {
        let start_time = std::time::Instant::now();
        log::info!("🤖 Initializing AI recommendation models...");

        // Get all data for training
        let contacts = self.repository.get_all_active_contacts().await?;
        let properties = self.repository.get_all_active_properties().await?;
        
        log::info!("📊 Training with {} contacts and {} properties", contacts.len(), properties.len());

        // Generate some training recommendations using traditional algorithm
        let mut training_recommendations = Vec::new();
        
        // Sample a subset for training to avoid overwhelming the system
        let sample_contacts = contacts.iter().take(50).collect::<Vec<_>>();
        let sample_properties = properties.iter().take(100).collect::<Vec<_>>();

        for contact in &sample_contacts {
            let response = self.traditional_service
                .get_recommendations_for_contact(contact.id, Some(10), Some(0.3), None, None, None)
                .await?;
            training_recommendations.extend(response.recommendations);
        }

        log::info!("🎯 Generated {} training recommendations", training_recommendations.len());

        // Initialize collaborative filtering
        {
            let mut cf_engine = self.cf_engine.lock().await;
            cf_engine.build_matrix_from_recommendations(&training_recommendations)?;
            cf_engine.extract_user_features(&contacts);
            cf_engine.extract_item_features(&properties);
        }

        // Initialize market trends
        {
            let mut market_engine = self.market_engine.lock().await;
            market_engine.analyze_market_trends(&properties)?;
        }

        // Initialize predictive matching
        {
            let mut predictive_engine = self.predictive_engine.lock().await;
            predictive_engine.initialize_with_data(&contacts, &properties, &training_recommendations)?;
        }

        *self.is_initialized.lock().await = true;
        
        let elapsed = start_time.elapsed();
        log::info!("✅ AI models initialized in {:.2}s", elapsed.as_secs_f64());

        Ok(())
    }

    /// Get AI-enhanced recommendations
    pub async fn get_ai_recommendations(
        &self,
        request: AIRecommendationRequest,
    ) -> Result<AIRecommendationResponse> {
        let start_time = std::time::Instant::now();

        // Ensure models are initialized
        if !*self.is_initialized.lock().await {
            self.initialize_models().await?;
        }

        // Get traditional recommendations first
        let traditional_response = self.traditional_service
            .get_recommendations_for_contact(
                request.contact_id,
                None,
                request.min_confidence,
                None,
                None,
                None,
            )
            .await?;

        let mut enhanced_recommendations = Vec::new();
        let mut price_predictions = HashMap::new();
        let mut market_insights = Vec::new();

        // Process each recommendation with AI enhancements
        for recommendation in traditional_response.recommendations {
            let mut enhanced = EnhancedRecommendation {
                recommendation: recommendation.clone(),
                ml_enhancement: None,
                predictive_analysis: None,
                market_trend: None,
                ai_insights: Vec::new(),
                confidence_score: recommendation.score,
            };

            // ML Enhancement
            if request.enable_ml_scoring {
                if let Ok(ml_rec) = self.get_ml_recommendation(
                    request.contact_id,
                    recommendation.property.id,
                    recommendation.score,
                ).await {
                    enhanced.ml_enhancement = Some(ml_rec.clone());
                    enhanced.confidence_score = ml_rec.hybrid_score;
                    
                    // Add ML insights
                    if ml_rec.prediction_confidence > 0.7 {
                        enhanced.ai_insights.push("🎯 High ML prediction confidence".to_string());
                    }
                    
                    match ml_rec.recommendation_type {
                        crate::ml::RecommendationType::CollaborativeFiltering => {
                            enhanced.ai_insights.push("👥 Based on similar user preferences".to_string());
                        }
                        crate::ml::RecommendationType::Hybrid => {
                            enhanced.ai_insights.push("🔄 Hybrid ML + traditional analysis".to_string());
                        }
                        _ => {}
                    }
                }
            }

            // Predictive Analysis
            if request.enable_predictive_matching {
                if let Ok(prediction) = self.get_predictive_analysis(
                    request.contact_id,
                    recommendation.property.id,
                ).await {
                    enhanced.predictive_analysis = Some(prediction.clone());
                    
                    // Add predictive insights
                    if prediction.purchase_probability > 0.7 {
                        enhanced.ai_insights.push(format!(
                            "🎯 High purchase probability: {:.1}%",
                            prediction.purchase_probability * 100.0
                        ));
                    }
                    
                    if prediction.time_to_decision_days <= 7 {
                        enhanced.ai_insights.push("⚡ Quick decision expected".to_string());
                    } else if prediction.time_to_decision_days >= 90 {
                        enhanced.ai_insights.push("⏳ Long consideration period expected".to_string());
                    }
                    
                    // Add risk/success insights
                    if !prediction.risk_factors.is_empty() {
                        enhanced.ai_insights.push(format!(
                            "⚠️ {} risk factor(s) identified",
                            prediction.risk_factors.len()
                        ));
                    }
                    
                    if prediction.success_indicators.len() > 2 {
                        enhanced.ai_insights.push("✨ Multiple success indicators present".to_string());
                    }
                }
            }

            // Market Analysis
            if request.enable_market_analysis {
                if let Some(trend) = self.get_market_trend_for_property(&recommendation.property).await {
                    enhanced.market_trend = Some(trend.clone());
                    
                    // Add market insights
                    if trend.price_trend > 0.05 {
                        enhanced.ai_insights.push(format!(
                            "📈 Strong market growth: +{:.1}% annually",
                            trend.price_trend * 100.0
                        ));
                    }
                    
                    match trend.demand_level {
                        crate::ml::DemandLevel::VeryHigh => {
                            enhanced.ai_insights.push("🔥 Very high demand market".to_string());
                        }
                        crate::ml::DemandLevel::High => {
                            enhanced.ai_insights.push("📊 High demand market".to_string());
                        }
                        _ => {}
                    }
                }
            }

            // Price Predictions
            if request.include_price_predictions {
                let prediction = self.get_price_prediction(&recommendation.property).await;
                price_predictions.insert(recommendation.property.id, prediction);
            }

            enhanced_recommendations.push(enhanced);
        }

        // Generate market insights
        if request.enable_market_analysis {
            market_insights = self.generate_market_insights().await;
        }

        // Generate contact behavior insights
        let contact_behavior_insights = if request.enable_predictive_matching {
            self.generate_contact_behavior_summary(request.contact_id).await
        } else {
            None
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(AIRecommendationResponse {
            total_count: enhanced_recommendations.len(),
            recommendations: enhanced_recommendations,
            market_insights,
            price_predictions,
            contact_behavior_insights,
            processing_time_ms: processing_time,
            ai_model_version: "v1.0.0-hackathon".to_string(),
        })
    }

    /// Get ML recommendation for a specific contact-property pair
    async fn get_ml_recommendation(
        &self,
        contact_id: i32,
        property_id: i32,
        traditional_score: f64,
    ) -> Result<MLRecommendation> {
        let mut cf_engine = self.cf_engine.lock().await;
        
        let mut traditional_scores = HashMap::new();
        traditional_scores.insert(property_id, traditional_score);
        
        let ml_recs = cf_engine.generate_ml_recommendations(
            contact_id,
            &[property_id],
            &traditional_scores,
            5, // k_neighbors
        );

        ml_recs.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("No ML recommendation generated"))
    }

    /// Get predictive analysis
    async fn get_predictive_analysis(
        &self,
        contact_id: i32,
        property_id: i32,
    ) -> Result<PredictiveMatchResult> {
        let mut predictive_engine = self.predictive_engine.lock().await;
        predictive_engine.predict_match(contact_id, property_id)
    }

    /// Get market trend for property
    async fn get_market_trend_for_property(&self, property: &Property) -> Option<MarketTrend> {
        let market_engine = self.market_engine.lock().await;
        
        // Extract location from address (simplified)
        let location = property.address.split(',').next()?.trim();
        market_engine.get_market_trend(location, &property.property_type).cloned()
    }

    /// Get price prediction
    async fn get_price_prediction(&self, property: &Property) -> PricePredictor {
        let market_engine = self.market_engine.lock().await;
        market_engine.predict_property_price(property)
    }

    /// Generate market insights
    pub async fn generate_market_insights(&self) -> Vec<String> {
        let market_engine = self.market_engine.lock().await;
        market_engine.generate_market_insights()
    }

    /// Generate contact behavior summary
    async fn generate_contact_behavior_summary(&self, contact_id: i32) -> Option<ContactBehaviorSummary> {
        let predictive_engine = self.predictive_engine.lock().await;
        
        if let Some(profile) = predictive_engine.get_behavior_profile(contact_id) {
            let decisiveness_level = if profile.decisiveness_score > 0.7 {
                "High - Makes quick decisions"
            } else if profile.decisiveness_score > 0.4 {
                "Medium - Considers options carefully"
            } else {
                "Low - Takes time to decide"
            }.to_string();

            let price_sensitivity_level = if profile.price_sensitivity > 0.7 {
                "High - Very price conscious"
            } else if profile.price_sensitivity > 0.4 {
                "Medium - Balanced approach"
            } else {
                "Low - Value focused"
            }.to_string();

            let flexibility_score = (profile.location_flexibility + profile.property_type_flexibility) / 2.0;

            let predicted_timeline = match profile.communication_frequency {
                crate::ml::CommunicationFrequency::VeryHigh => "1-2 weeks",
                crate::ml::CommunicationFrequency::High => "2-4 weeks",
                crate::ml::CommunicationFrequency::Medium => "1-2 months",
                crate::ml::CommunicationFrequency::Low => "2-3 months",
                crate::ml::CommunicationFrequency::VeryLow => "3+ months",
            }.to_string();

            let engagement_level = format!("{:?}", profile.communication_frequency);

            let mut recommendations = Vec::new();
            
            if profile.decisiveness_score < 0.3 {
                recommendations.push("Provide clear comparisons and detailed information".to_string());
            }
            
            if profile.price_sensitivity > 0.7 {
                recommendations.push("Emphasize value and cost-effectiveness".to_string());
            }
            
            if flexibility_score > 0.6 {
                recommendations.push("Show diverse options across locations and types".to_string());
            }
            
            match profile.market_timing_preference {
                crate::ml::MarketTimingPreference::EarlyAdopter => {
                    recommendations.push("Highlight new opportunities and market trends".to_string());
                }
                crate::ml::MarketTimingPreference::ValueHunter => {
                    recommendations.push("Focus on deals and investment potential".to_string());
                }
                _ => {}
            }

            Some(ContactBehaviorSummary {
                contact_id,
                decisiveness_level,
                price_sensitivity_level,
                flexibility_score,
                predicted_timeline,
                engagement_level,
                recommendations,
            })
        } else {
            None
        }
    }

    /// Update AI models with feedback
    pub async fn update_with_feedback(
        &self,
        contact_id: i32,
        property_id: i32,
        feedback_type: &str,
        outcome: &str,
    ) -> Result<()> {
        // Update collaborative filtering
        {
            let mut cf_engine = self.cf_engine.lock().await;
            let rating = match outcome {
                "purchased" => 1.0,
                "visited" => 0.8,
                "interested" => 0.6,
                "declined" => 0.2,
                _ => 0.3,
            };
            cf_engine.update_with_feedback(contact_id, property_id, rating, 0.9)?;
        }

        // Update predictive matching
        {
            let mut predictive_engine = self.predictive_engine.lock().await;
            predictive_engine.update_behavior_profile(contact_id, feedback_type, outcome)?;
        }

        Ok(())
    }

    /// Get AI model statistics
    pub async fn get_model_stats(&self) -> serde_json::Value {
        let is_init = *self.is_initialized.lock().await;
        
        serde_json::json!({
            "initialized": is_init,
            "model_version": "v1.0.0-hackathon",
            "features": {
                "collaborative_filtering": true,
                "market_trends": true,
                "predictive_matching": true,
                "price_prediction": true
            },
            "last_updated": chrono::Utc::now()
        })
    }
}
