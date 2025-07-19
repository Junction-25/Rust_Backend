use crate::models::{Contact, Property, Recommendation};
use crate::ml::{CollaborativeFilteringEngine, MarketTrendsEngine, MLRecommendation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveMatchResult {
    pub contact_id: i32,
    pub property_id: i32,
    pub compatibility_score: f64,
    pub purchase_probability: f64,
    pub time_to_decision_days: i32,
    pub confidence_level: f64,
    pub risk_factors: Vec<RiskFactor>,
    pub success_indicators: Vec<SuccessIndicator>,
    pub predicted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor: String,
    pub impact: f64, // -1.0 to 1.0
    pub severity: RiskSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessIndicator {
    pub indicator: String,
    pub strength: f64, // 0.0 to 1.0
    pub category: IndicatorCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorCategory {
    Financial,
    Location,
    PropertyType,
    Timing,
    Market,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactBehaviorProfile {
    pub contact_id: i32,
    pub decisiveness_score: f64, // How quickly they make decisions
    pub price_sensitivity: f64,
    pub location_flexibility: f64,
    pub property_type_flexibility: f64,
    pub market_timing_preference: MarketTimingPreference,
    pub communication_frequency: CommunicationFrequency,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MarketTimingPreference {
    EarlyAdopter,    // Buys in rising markets
    ValueHunter,     // Waits for deals
    MarketFollower,  // Average timing
    TrendAverse,     // Avoids volatile periods
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CommunicationFrequency {
    VeryHigh,   // Daily engagement
    High,       // Weekly engagement  
    Medium,     // Monthly engagement
    Low,        // Quarterly engagement
    VeryLow,    // Rare engagement
}

#[derive(Clone)]
pub struct PredictiveMatchingEngine {
    cf_engine: CollaborativeFilteringEngine,
    market_engine: MarketTrendsEngine,
    behavior_profiles: HashMap<i32, ContactBehaviorProfile>,
    historical_matches: Vec<HistoricalMatch>,
}

#[derive(Debug, Clone)]
struct HistoricalMatch {
    contact_id: i32,
    property_id: i32,
    initial_score: f64,
    outcome: MatchOutcome,
    days_to_decision: i32,
    match_date: DateTime<Utc>,
}

#[derive(Debug, Clone)]
enum MatchOutcome {
    Purchased,
    Visited,
    Interested,
    Declined,
    NoResponse,
}

impl PredictiveMatchingEngine {
    pub fn new() -> Self {
        Self {
            cf_engine: CollaborativeFilteringEngine::new(),
            market_engine: MarketTrendsEngine::new(),
            behavior_profiles: HashMap::new(),
            historical_matches: Vec::new(),
        }
    }

    /// Initialize the engine with training data
    pub fn initialize_with_data(
        &mut self,
        contacts: &[Contact],
        properties: &[Property],
        recommendations: &[Recommendation],
    ) -> anyhow::Result<()> {
        // Initialize collaborative filtering
        self.cf_engine.build_matrix_from_recommendations(recommendations)?;
        self.cf_engine.extract_user_features(contacts);
        self.cf_engine.extract_item_features(properties);

        // Initialize market trends
        self.market_engine.analyze_market_trends(properties)?;

        // Generate behavior profiles
        self.generate_behavior_profiles(contacts, recommendations);

        // Simulate some historical matches for training
        self.generate_simulated_historical_data(contacts, properties);

        Ok(())
    }

    /// Generate behavior profiles for contacts
    fn generate_behavior_profiles(&mut self, contacts: &[Contact], recommendations: &[Recommendation]) {
        for contact in contacts {
            let profile = self.analyze_contact_behavior(contact, recommendations);
            self.behavior_profiles.insert(contact.id, profile);
        }
    }

    /// Analyze individual contact behavior
    fn analyze_contact_behavior(&self, contact: &Contact, recommendations: &[Recommendation]) -> ContactBehaviorProfile {
        let contact_recs: Vec<&Recommendation> = recommendations
            .iter()
            .filter(|r| r.contact.id == contact.id)
            .collect();

        // Analyze decisiveness (based on budget range tightness)
        let budget_range = contact.max_budget - contact.min_budget;
        let decisiveness_score = if budget_range < 5_000_000.0 {
            0.8 // Tight budget = decisive
        } else if budget_range < 15_000_000.0 {
            0.6
        } else {
            0.3 // Wide budget = indecisive
        };

        // Analyze price sensitivity (based on budget utilization patterns)
        let avg_recommended_price = if !contact_recs.is_empty() {
            contact_recs.iter().map(|r| r.property.price).sum::<f64>() / contact_recs.len() as f64
        } else {
            contact.max_budget
        };
        
        let price_sensitivity = if avg_recommended_price < contact.min_budget * 1.1 {
            0.9 // Very price sensitive
        } else if avg_recommended_price < contact.max_budget * 0.8 {
            0.6
        } else {
            0.3 // Not very price sensitive
        };

        // Analyze location flexibility
        let location_flexibility = if contact.preferred_locations.len() > 3 {
            0.8 // Very flexible
        } else if contact.preferred_locations.len() > 1 {
            0.5
        } else {
            0.2 // Not flexible
        };

        // Analyze property type flexibility
        let property_type_flexibility = contact.property_types.len() as f64 / 5.0; // Normalize by max types

        // Determine market timing preference
        let market_timing_preference = if price_sensitivity > 0.7 {
            MarketTimingPreference::ValueHunter
        } else if decisiveness_score > 0.7 {
            MarketTimingPreference::EarlyAdopter
        } else {
            MarketTimingPreference::MarketFollower
        };

        // Simulate communication frequency based on engagement patterns
        let communication_frequency = if contact_recs.len() > 10 {
            CommunicationFrequency::VeryHigh
        } else if contact_recs.len() > 5 {
            CommunicationFrequency::High
        } else if contact_recs.len() > 2 {
            CommunicationFrequency::Medium
        } else {
            CommunicationFrequency::Low
        };

        ContactBehaviorProfile {
            contact_id: contact.id,
            decisiveness_score,
            price_sensitivity,
            location_flexibility,
            property_type_flexibility,
            market_timing_preference,
            communication_frequency,
            last_updated: Utc::now(),
        }
    }

    /// Generate simulated historical data for training
    fn generate_simulated_historical_data(&mut self, contacts: &[Contact], properties: &[Property]) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for contact in contacts.iter().take(contacts.len().min(50)) {
            for property in properties.iter().take(properties.len().min(20)) {
                if rng.gen::<f64>() < 0.3 { // 30% chance of historical interaction
                    let outcome = match rng.gen_range(0..5) {
                        0 => MatchOutcome::Purchased,
                        1 => MatchOutcome::Visited,
                        2 => MatchOutcome::Interested,
                        3 => MatchOutcome::Declined,
                        _ => MatchOutcome::NoResponse,
                    };

                    let days_to_decision = match outcome {
                        MatchOutcome::Purchased => rng.gen_range(7..90),
                        MatchOutcome::Visited => rng.gen_range(3..30),
                        MatchOutcome::Interested => rng.gen_range(1..14),
                        MatchOutcome::Declined => rng.gen_range(1..7),
                        MatchOutcome::NoResponse => rng.gen_range(30..180),
                    };

                    // Calculate initial score based on basic compatibility
                    let initial_score = self.calculate_basic_compatibility(contact, property);

                    self.historical_matches.push(HistoricalMatch {
                        contact_id: contact.id,
                        property_id: property.id,
                        initial_score,
                        outcome,
                        days_to_decision,
                        match_date: Utc::now() - Duration::days(rng.gen_range(30..365)),
                    });
                }
            }
        }
    }

    /// Calculate basic compatibility score
    fn calculate_basic_compatibility(&self, contact: &Contact, property: &Property) -> f64 {
        let mut score = 0.0;

        // Budget compatibility
        if property.price >= contact.min_budget && property.price <= contact.max_budget {
            score += 0.3;
        }

        // Area compatibility
        if property.area_sqm >= contact.min_area_sqm && property.area_sqm <= contact.max_area_sqm {
            score += 0.2;
        }

        // Room compatibility
        if property.number_of_rooms >= contact.min_rooms {
            score += 0.2;
        }

        // Property type compatibility
        if contact.property_types.contains(&property.property_type) {
            score += 0.3;
        }

        score
    }

    /// Predict match outcome with comprehensive analysis
    pub fn predict_match(&mut self, contact_id: i32, property_id: i32) -> anyhow::Result<PredictiveMatchResult> {
        let behavior_profile = self.behavior_profiles.get(&contact_id)
            .ok_or_else(|| anyhow::anyhow!("No behavior profile found for contact {}", contact_id))?;

        // Get ML recommendation score
        let (ml_score, ml_confidence) = self.cf_engine.predict_user_item_rating(contact_id, property_id, 5);

        // Calculate purchase probability using multiple factors
        let purchase_probability = self.calculate_purchase_probability(
            behavior_profile,
            ml_score,
            ml_confidence,
            property_id,
        )?;

        // Predict time to decision
        let time_to_decision_days = self.predict_decision_timing(behavior_profile, purchase_probability);

        // Calculate overall compatibility score
        let compatibility_score = self.calculate_comprehensive_compatibility(
            contact_id,
            property_id,
            ml_score,
            behavior_profile,
        )?;

        // Identify risk factors
        let risk_factors = self.identify_risk_factors(contact_id, property_id, behavior_profile)?;

        // Identify success indicators
        let success_indicators = self.identify_success_indicators(contact_id, property_id, behavior_profile)?;

        // Calculate overall confidence
        let confidence_level = self.calculate_prediction_confidence(
            ml_confidence,
            behavior_profile,
            &risk_factors,
            &success_indicators,
        );

        Ok(PredictiveMatchResult {
            contact_id,
            property_id,
            compatibility_score,
            purchase_probability,
            time_to_decision_days,
            confidence_level,
            risk_factors,
            success_indicators,
            predicted_at: Utc::now(),
        })
    }

    /// Calculate purchase probability
    fn calculate_purchase_probability(
        &self,
        profile: &ContactBehaviorProfile,
        ml_score: f64,
        ml_confidence: f64,
        _property_id: i32,
    ) -> anyhow::Result<f64> {
        let mut probability = ml_score * 0.4; // Base ML score contribution

        // Adjust based on decisiveness
        probability += profile.decisiveness_score * 0.2;

        // Adjust based on historical patterns
        let historical_success_rate = self.calculate_historical_success_rate(profile.contact_id);
        probability += historical_success_rate * 0.2;

        // Adjust based on market timing
        let market_timing_bonus = match profile.market_timing_preference {
            MarketTimingPreference::EarlyAdopter => 0.1,
            MarketTimingPreference::ValueHunter => 0.05,
            MarketTimingPreference::MarketFollower => 0.0,
            MarketTimingPreference::TrendAverse => -0.05,
        };
        probability += market_timing_bonus;

        // Adjust based on communication frequency (higher engagement = higher probability)
        let engagement_bonus = match profile.communication_frequency {
            CommunicationFrequency::VeryHigh => 0.15,
            CommunicationFrequency::High => 0.1,
            CommunicationFrequency::Medium => 0.05,
            CommunicationFrequency::Low => 0.0,
            CommunicationFrequency::VeryLow => -0.1,
        };
        probability += engagement_bonus;

        // Apply confidence weighting
        probability = probability * ml_confidence + (1.0 - ml_confidence) * 0.3;

        Ok(probability.max(0.0).min(1.0))
    }

    /// Calculate historical success rate for a contact
    fn calculate_historical_success_rate(&self, contact_id: i32) -> f64 {
        let contact_matches: Vec<&HistoricalMatch> = self.historical_matches
            .iter()
            .filter(|m| m.contact_id == contact_id)
            .collect();

        if contact_matches.is_empty() {
            return 0.5; // Default neutral rate
        }

        let successful_matches = contact_matches
            .iter()
            .filter(|m| matches!(m.outcome, MatchOutcome::Purchased | MatchOutcome::Visited))
            .count();

        successful_matches as f64 / contact_matches.len() as f64
    }

    /// Predict decision timing
    fn predict_decision_timing(&self, profile: &ContactBehaviorProfile, purchase_probability: f64) -> i32 {
        let mut base_days = 30; // Base decision time

        // Adjust based on decisiveness
        base_days = (base_days as f64 * (1.0 - profile.decisiveness_score * 0.5)) as i32;

        // Adjust based on purchase probability
        if purchase_probability > 0.8 {
            base_days = (base_days as f64 * 0.7) as i32; // High probability = faster decision
        } else if purchase_probability < 0.3 {
            base_days = (base_days as f64 * 1.5) as i32; // Low probability = slower decision
        }

        // Adjust based on communication frequency
        let frequency_multiplier = match profile.communication_frequency {
            CommunicationFrequency::VeryHigh => 0.5,
            CommunicationFrequency::High => 0.7,
            CommunicationFrequency::Medium => 1.0,
            CommunicationFrequency::Low => 1.3,
            CommunicationFrequency::VeryLow => 2.0,
        };
        base_days = (base_days as f64 * frequency_multiplier) as i32;

        base_days.max(1).min(365)
    }

    /// Calculate comprehensive compatibility score
    fn calculate_comprehensive_compatibility(
        &self,
        contact_id: i32,
        property_id: i32,
        ml_score: f64,
        profile: &ContactBehaviorProfile,
    ) -> anyhow::Result<f64> {
        let mut score = ml_score * 0.5; // Base ML score

        // Add behavior-based adjustments
        score += profile.decisiveness_score * 0.1;
        score += (1.0 - profile.price_sensitivity) * 0.1; // Less sensitive = better compatibility
        score += profile.location_flexibility * 0.1;
        score += profile.property_type_flexibility * 0.1;

        // Market timing compatibility
        let market_timing_score = match profile.market_timing_preference {
            MarketTimingPreference::EarlyAdopter => 0.8,
            MarketTimingPreference::ValueHunter => 0.6,
            MarketTimingPreference::MarketFollower => 0.7,
            MarketTimingPreference::TrendAverse => 0.4,
        };
        score += market_timing_score * 0.1;

        Ok(score.max(0.0).min(1.0))
    }

    /// Identify risk factors
    fn identify_risk_factors(
        &self,
        _contact_id: i32,
        _property_id: i32,
        profile: &ContactBehaviorProfile,
    ) -> anyhow::Result<Vec<RiskFactor>> {
        let mut risk_factors = Vec::new();

        // Low decisiveness risk
        if profile.decisiveness_score < 0.3 {
            risk_factors.push(RiskFactor {
                factor: "Low decisiveness - may delay or abandon purchase".to_string(),
                impact: -0.3,
                severity: RiskSeverity::Medium,
            });
        }

        // High price sensitivity risk
        if profile.price_sensitivity > 0.8 {
            risk_factors.push(RiskFactor {
                factor: "High price sensitivity - may seek lower prices".to_string(),
                impact: -0.2,
                severity: RiskSeverity::Low,
            });
        }

        // Low communication frequency risk
        if matches!(profile.communication_frequency, CommunicationFrequency::Low | CommunicationFrequency::VeryLow) {
            risk_factors.push(RiskFactor {
                factor: "Low engagement - difficult to reach and convert".to_string(),
                impact: -0.25,
                severity: RiskSeverity::Medium,
            });
        }

        // Market timing risk
        if matches!(profile.market_timing_preference, MarketTimingPreference::TrendAverse) {
            risk_factors.push(RiskFactor {
                factor: "Market timing averse - may wait for better conditions".to_string(),
                impact: -0.15,
                severity: RiskSeverity::Low,
            });
        }

        Ok(risk_factors)
    }

    /// Identify success indicators
    fn identify_success_indicators(
        &self,
        _contact_id: i32,
        _property_id: i32,
        profile: &ContactBehaviorProfile,
    ) -> anyhow::Result<Vec<SuccessIndicator>> {
        let mut indicators = Vec::new();

        // High decisiveness indicator
        if profile.decisiveness_score > 0.7 {
            indicators.push(SuccessIndicator {
                indicator: "High decisiveness score".to_string(),
                strength: profile.decisiveness_score,
                category: IndicatorCategory::Timing,
            });
        }

        // High engagement indicator
        if matches!(profile.communication_frequency, CommunicationFrequency::High | CommunicationFrequency::VeryHigh) {
            indicators.push(SuccessIndicator {
                indicator: "High engagement level".to_string(),
                strength: 0.8,
                category: IndicatorCategory::Financial,
            });
        }

        // Flexibility indicators
        if profile.location_flexibility > 0.6 {
            indicators.push(SuccessIndicator {
                indicator: "High location flexibility".to_string(),
                strength: profile.location_flexibility,
                category: IndicatorCategory::Location,
            });
        }

        if profile.property_type_flexibility > 0.6 {
            indicators.push(SuccessIndicator {
                indicator: "Open to multiple property types".to_string(),
                strength: profile.property_type_flexibility,
                category: IndicatorCategory::PropertyType,
            });
        }

        // Market timing indicators
        if matches!(profile.market_timing_preference, MarketTimingPreference::EarlyAdopter) {
            indicators.push(SuccessIndicator {
                indicator: "Early adopter - quick to act on opportunities".to_string(),
                strength: 0.8,
                category: IndicatorCategory::Market,
            });
        }

        Ok(indicators)
    }

    /// Calculate prediction confidence
    fn calculate_prediction_confidence(
        &self,
        ml_confidence: f64,
        profile: &ContactBehaviorProfile,
        risk_factors: &[RiskFactor],
        success_indicators: &[SuccessIndicator],
    ) -> f64 {
        let mut confidence = ml_confidence * 0.4; // Base ML confidence

        // Adjust based on behavior profile completeness
        confidence += 0.2; // We have complete profile

        // Adjust based on communication frequency (higher engagement = more predictable)
        let engagement_confidence = match profile.communication_frequency {
            CommunicationFrequency::VeryHigh => 0.2,
            CommunicationFrequency::High => 0.15,
            CommunicationFrequency::Medium => 0.1,
            CommunicationFrequency::Low => 0.05,
            CommunicationFrequency::VeryLow => 0.0,
        };
        confidence += engagement_confidence;

        // Adjust based on risk/success balance
        let risk_impact: f64 = risk_factors.iter().map(|r| r.impact.abs()).sum();
        let success_strength: f64 = success_indicators.iter().map(|s| s.strength).sum();
        
        if success_strength > risk_impact {
            confidence += 0.1; // More success indicators = higher confidence
        } else if risk_impact > success_strength * 1.5 {
            confidence -= 0.1; // Many risk factors = lower confidence
        }

        confidence.max(0.1).min(0.95)
    }

    /// Get behavior profile for a contact
    pub fn get_behavior_profile(&self, contact_id: i32) -> Option<&ContactBehaviorProfile> {
        self.behavior_profiles.get(&contact_id)
    }

    /// Update behavior profile with new interaction data
    pub fn update_behavior_profile(
        &mut self,
        contact_id: i32,
        interaction_type: &str,
        outcome: &str,
    ) -> anyhow::Result<()> {
        if let Some(profile) = self.behavior_profiles.get_mut(&contact_id) {
            // Update based on interaction
            match interaction_type {
                "quick_response" => {
                    profile.decisiveness_score = (profile.decisiveness_score + 0.1).min(1.0);
                }
                "price_negotiation" => {
                    profile.price_sensitivity = (profile.price_sensitivity + 0.1).min(1.0);
                }
                "location_inquiry" => {
                    profile.location_flexibility = (profile.location_flexibility + 0.05).min(1.0);
                }
                _ => {}
            }

            // Update communication frequency based on outcome
            if outcome == "positive_response" {
                profile.communication_frequency = match profile.communication_frequency {
                    CommunicationFrequency::VeryLow => CommunicationFrequency::Low,
                    CommunicationFrequency::Low => CommunicationFrequency::Medium,
                    CommunicationFrequency::Medium => CommunicationFrequency::High,
                    CommunicationFrequency::High => CommunicationFrequency::VeryHigh,
                    CommunicationFrequency::VeryHigh => CommunicationFrequency::VeryHigh,
                };
            }

            profile.last_updated = Utc::now();
        }

        Ok(())
    }
}
