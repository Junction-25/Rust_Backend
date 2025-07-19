use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::ml::market_trends::{MarketTrend, SupplyLevel, DemandLevel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightConfig {
    pub base_weights: Weights,
    pub adjustment_factors: AdjustmentFactors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weights {
    pub budget: f64,
    pub location: f64,
    pub property_type: f64,
    pub size: f64,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            budget: 0.4,
            location: 0.3,
            property_type: 0.2,
            size: 0.1,
        }
    }
}

/// Market conditions that trigger weight adjustments
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MarketCondition {
    /// Low inventory relative to demand
    LowInventory,
    /// High price volatility
    HighVolatility,
    /// High demand relative to supply
    SellersMarket,
    /// High supply relative to demand
    BuyersMarket,
    /// Seasonally strong period for real estate
    PeakSeason,
    /// Seasonally weak period for real estate
    OffSeason,
    /// High interest rate environment
    HighInterestRates,
    /// Low interest rate environment
    LowInterestRates,
    /// High price growth rate
    HighAppreciation,
    /// Stagnant or decreasing prices
    LowAppreciation,
}

/// Configuration for how different market conditions affect weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustmentFactors {
    // Base adjustment factors
    pub max_adjustment: f64,            // Maximum allowed adjustment to any weight
    
    // Market condition factors
    pub low_inventory_factor: f64,      // How much to reduce location weight when inventory is low
    pub high_volatility_factor: f64,    // How much to increase budget weight when volatility is high
    pub sellers_market_factor: f64,     // How much to increase property_type weight in seller's market
    pub buyers_market_factor: f64,      // How much to increase size weight in buyer's market
    pub peak_season_factor: f64,        // How much to increase location weight during peak season
    pub interest_rate_sensitivity: f64, // How much to adjust budget weight based on interest rates
    pub appreciation_impact: f64,       // How much to adjust weights based on price trends
    
    // Time-based decay factors
    pub trend_decay_rate: f64,          // How quickly recent trends lose influence (0-1)
    pub min_confidence: f64,            // Minimum confidence required to apply adjustments
    
    // Interaction factors
    pub inventory_volatility_interaction: f64, // How much low inventory and high volatility interact
    pub season_market_interaction: f64,        // Interaction between season and market conditions
}

impl Default for AdjustmentFactors {
    fn default() -> Self {
        Self {
            max_adjustment: 0.5,         // Never adjust any weight by more than 50%
            
            // Market condition factors
            low_inventory_factor: 0.5,    // Reduce location weight by up to 50% when inventory is low
            high_volatility_factor: 0.3,  // Increase budget weight by up to 30% when volatility is high
            sellers_market_factor: 0.4,   // Increase property_type weight by up to 40% in seller's market
            buyers_market_factor: 0.3,    // Increase size weight by up to 30% in buyer's market
            peak_season_factor: 0.25,     // Increase location weight by up to 25% during peak season
            interest_rate_sensitivity: 0.35, // Adjust budget weight by up to 35% based on rates
            appreciation_impact: 0.4,     // Adjust weights by up to 40% based on appreciation
            
            // Time-based factors
            trend_decay_rate: 0.9,        // Recent trends have 90% of their original influence each day
            min_confidence: 0.7,          // Require at least 70% confidence to apply adjustments
            
            // Interaction factors
            inventory_volatility_interaction: 0.6,  // Combined effect of low inventory and high volatility
            season_market_interaction: 0.4,         // Combined effect of season and market conditions
        }
    }
}

/// Tracks the strength and recency of market conditions
#[derive(Debug, Clone)]
struct ConditionStrength {
    condition: MarketCondition,
    strength: f64,  // 0.0 to 1.0
    confidence: f64, // 0.0 to 1.0
    last_updated: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub struct WeightAdjuster {
    config: WeightConfig,
    market_trends: HashMap<String, MarketTrend>,
    active_conditions: Vec<ConditionStrength>,
    interest_rate: f64,  // Current interest rate (annual %)
    season_factor: f64,  // 0.0 (off-season) to 1.0 (peak season)
    market_momentum: f64, // -1.0 (strong buyer's market) to 1.0 (strong seller's market)
}

impl WeightAdjuster {
    pub fn new(config: Option<WeightConfig>) -> Self {
        let now = std::time::SystemTime::now();
        
        // Initialize with some default active conditions
        let mut active_conditions = Vec::new();
        
        // Example: Start with no active conditions
        // In a real system, you'd load these from a database or API
        
        Self {
            config: config.unwrap_or_else(|| WeightConfig {
                base_weights: Weights::default(),
                adjustment_factors: AdjustmentFactors::default(),
            }),
            market_trends: HashMap::new(),
            active_conditions,
            interest_rate: 5.0,  // Default 5% interest rate
            season_factor: 0.5,  // Neutral season
            market_momentum: 0.0, // Neutral market
        }
    }

    pub fn update_market_trends(&mut self, trends: HashMap<String, MarketTrend>) {
        self.market_trends = trends;
    }

    /// Update market conditions based on current trends
    fn update_market_conditions(&mut self) {
        let now = std::time::SystemTime::now();
        let config = &self.config.adjustment_factors;
        
        // Update condition strengths with decay
        for condition in &mut self.active_conditions {
            let age_days = now.duration_since(condition.last_updated)
                .unwrap_or_default()
                .as_secs_f64() / 86400.0; // Convert to days
                
            // Apply exponential decay to condition strength
            condition.strength *= config.trend_decay_rate.powf(age_days);
            condition.confidence *= config.trend_decay_rate.powf(age_days);
        }
        
        // Remove conditions that are no longer relevant
        self.active_conditions.retain(|c| c.strength > 0.1 && c.confidence > 0.1);
    }
    
    /// Detect market conditions based on current trends
    fn detect_conditions(&mut self, trend: &MarketTrend) -> Vec<(MarketCondition, f64, f64)> {
        let mut conditions = Vec::new();
        let config = &self.config.adjustment_factors;
        
        // 1. Check inventory levels
        match trend.supply_level {
            SupplyLevel::Scarce | SupplyLevel::Limited => {
                conditions.push((MarketCondition::LowInventory, 1.0, trend.prediction_confidence));
                
                // If demand is high, it's a seller's market
                if matches!(trend.demand_level, DemandLevel::High | DemandLevel::VeryHigh) {
                    conditions.push((MarketCondition::SellersMarket, 0.8, trend.prediction_confidence * 0.9));
                }
            }
            SupplyLevel::Oversupply | SupplyLevel::Abundant => {
                // If demand is low, it's a buyer's market
                if matches!(trend.demand_level, DemandLevel::Low | DemandLevel::VeryLow) {
                    conditions.push((MarketCondition::BuyersMarket, 0.8, trend.prediction_confidence * 0.9));
                }
            }
            _ => {}
        }
        
        // 2. Check price volatility (absolute value of price trend)
        let volatility = trend.price_trend.abs();
        if volatility > 0.1 { // 10% change is considered high volatility
            let strength = (volatility / 0.2).min(1.0); // Cap at 20% change
            conditions.push((MarketCondition::HighVolatility, strength, trend.prediction_confidence));
        }
        
        // 3. Check price appreciation
        if trend.price_trend > 0.05 { // 5%+ annual appreciation
            let strength = (trend.price_trend / 0.2).min(1.0);
            conditions.push((MarketCondition::HighAppreciation, strength, trend.prediction_confidence));
        } else if trend.price_trend < -0.02 { // 2%+ annual depreciation
            let strength = (-trend.price_trend / 0.1).min(1.0);
            conditions.push((MarketCondition::LowAppreciation, strength, trend.prediction_confidence * 0.8));
        }
        
        // 4. Check interest rate environment (using global rate)
        if self.interest_rate > 7.0 { // High interest rates
            conditions.push((MarketCondition::HighInterestRates, 0.9, 1.0));
        } else if self.interest_rate < 4.0 { // Low interest rates
            conditions.push((MarketCondition::LowInterestRates, 0.9, 1.0));
        }
        
        // 5. Check seasonality
        if self.season_factor > 0.7 { // Peak season
            conditions.push((MarketCondition::PeakSeason, self.season_factor, 0.8));
        } else if self.season_factor < 0.3 { // Off season
            conditions.push((MarketCondition::OffSeason, 1.0 - self.season_factor, 0.8));
        }
        
        conditions
    }
    
    /// Calculate adjustment factors based on current conditions
    fn calculate_adjustments(&self, conditions: &[(MarketCondition, f64, f64)]) -> (f64, f64, f64, f64) {
        let config = &self.config.adjustment_factors;
        let mut budget_adj = 0.0;
        let mut location_adj = 0.0;
        let mut property_type_adj = 0.0;
        let mut size_adj = 0.0;
        
        for (condition, strength, confidence) in conditions {
            if *confidence < config.min_confidence {
                continue;
            }
            
            match condition {
                MarketCondition::LowInventory => {
                    // Reduce location importance when inventory is low
                    let adjustment = config.low_inventory_factor * strength * confidence;
                    location_adj -= adjustment;
                }
                MarketCondition::HighVolatility => {
                    // Increase budget importance when volatility is high
                    let adjustment = config.high_volatility_factor * strength * confidence;
                    budget_adj += adjustment;
                }
                MarketCondition::SellersMarket => {
                    // In seller's market, property type becomes more important
                    let adjustment = config.sellers_market_factor * strength * confidence;
                    property_type_adj += adjustment;
                }
                MarketCondition::BuyersMarket => {
                    // In buyer's market, size becomes more important
                    let adjustment = config.buyers_market_factor * strength * confidence;
                    size_adj += adjustment;
                }
                MarketCondition::PeakSeason => {
                    // In peak season, location becomes more important
                    let adjustment = config.peak_season_factor * strength * confidence;
                    location_adj += adjustment;
                }
                MarketCondition::HighInterestRates => {
                    // With high rates, budget becomes more constrained
                    let adjustment = config.interest_rate_sensitivity * strength * confidence;
                    budget_adj += adjustment * 0.5; // Moderate increase
                }
                MarketCondition::HighAppreciation => {
                    // In high appreciation markets, all factors matter more
                    let adjustment = config.appreciation_impact * strength * confidence * 0.5;
                    budget_adj += adjustment;
                    location_adj += adjustment;
                    property_type_adj += adjustment;
                }
                _ => {}
            }
        }
        
        // Apply interaction effects
        let has_low_inventory = conditions.iter().any(|(c, _, _)| matches!(c, MarketCondition::LowInventory));
        let has_high_volatility = conditions.iter().any(|(c, _, _)| matches!(c, MarketCondition::HighVolatility));
        
        if has_low_inventory && has_high_volatility {
            // When both conditions exist, amplify the budget adjustment
            let interaction = config.inventory_volatility_interaction * budget_adj.abs();
            budget_adj += interaction * budget_adj.signum();
        }
        
        // Ensure adjustments don't exceed maximum allowed
        let max_adj = config.max_adjustment;
        (
            budget_adj.clamp(-max_adj, max_adj),
            location_adj.clamp(-max_adj, max_adj),
            property_type_adj.clamp(-max_adj, max_adj),
            size_adj.clamp(-max_adj, max_adj),
        )
    }
    
    pub fn get_adjusted_weights(&self, location: &str, property_type: &str) -> Weights {
        let mut adjusted_weights = self.config.base_weights.clone();
        
        // Get the relevant market trend if available
        let trend_key = format!("{}:{}", location, property_type);
        if let Some(trend) = self.market_trends.get(&trend_key) {
            // Detect current market conditions
            let conditions = self.detect_conditions(trend);
            
            // Calculate adjustments based on conditions
            let (budget_adj, location_adj, property_type_adj, size_adj) = 
                self.calculate_adjustments(&conditions);
            
            // Apply adjustments
            adjusted_weights.budget *= 1.0 + budget_adj;
            adjusted_weights.location *= 1.0 + location_adj;
            adjusted_weights.property_type *= 1.0 + property_type_adj;
            adjusted_weights.size *= 1.0 + size_adj;
            
            // Ensure weights are still valid
            self.normalize_weights(&mut adjusted_weights);
        }
        
        adjusted_weights
    }
        
        // Get the relevant market trend if available
        let trend_key = format!("{}:{}", location, property_type);
        if let Some(trend) = self.market_trends.get(&trend_key) {
            // Adjust weights based on inventory levels
            self.adjust_for_inventory(trend, &mut adjusted_weights);
            
            // Adjust weights based on price volatility
            self.adjust_for_volatility(trend, &mut adjusted_weights);
            
            // Ensure weights are still valid (sum to 1.0 and within bounds)
            self.normalize_weights(&mut adjusted_weights);
        }
        
        adjusted_weights
    }
    
    fn adjust_for_inventory(&self, trend: &MarketTrend, weights: &mut Weights) {
        match trend.supply_level {
            SupplyLevel::Scarce | SupplyLevel::Limited => {
                // When inventory is low, reduce the importance of location
                let reduction = self.config.adjustment_factors.low_inventory_factor * weights.location;
                let new_location_weight = (weights.location - reduction).max(0.0);
                
                // Distribute the reduction proportionally to other weights
                let total_other_weights = weights.budget + weights.property_type + weights.size;
                if total_other_weights > 0.0 {
                    let budget_ratio = weights.budget / total_other_weights;
                    let property_type_ratio = weights.property_type / total_other_weights;
                    let size_ratio = weights.size / total_other_weights;
                    
                    let reduction_per_weight = reduction / total_other_weights;
                    
                    weights.location = new_location_weight;
                    weights.budget += reduction * budget_ratio;
                    weights.property_type += reduction * property_type_ratio;
                    weights.size += reduction * size_ratio;
                }
            }
            _ => {}
        }
    }
    
    fn adjust_for_volatility(&self, trend: &MarketTrend, weights: &mut Weights) {
        // Assuming price_trend is a percentage change (e.g., 0.05 for 5% change)
        let volatility = trend.price_trend.abs();
        
        // Define a threshold for what's considered high volatility
        let high_volatility_threshold = 0.1; // 10% change is considered high volatility
        
        if volatility > high_volatility_threshold {
            // When volatility is high, increase the importance of budget
            let max_increase = weights.budget * self.config.adjustment_factors.high_volatility_factor;
            let increase = (volatility / high_volatility_threshold).min(1.0) * max_increase;
            
            // Distribute the decrease proportionally to other weights
            let total_other_weights = weights.location + weights.property_type + weights.size;
            if total_other_weights > 0.0 {
                let location_ratio = weights.location / total_other_weights;
                let property_type_ratio = weights.property_type / total_other_weights;
                let size_ratio = weights.size / total_other_weights;
                
                let decrease_per_weight = increase / 3.0; // Distribute evenly for simplicity
                
                weights.budget = (weights.budget + increase).min(weights.budget * (1.0 + self.config.adjustment_factors.max_adjustment));
                weights.location = (weights.location - decrease_per_weight * location_ratio).max(0.0);
                weights.property_type = (weights.property_type - decrease_per_weight * property_type_ratio).max(0.0);
                weights.size = (weights.size - decrease_per_weight * size_ratio).max(0.0);
            }
        }
    }
    
    fn normalize_weights(&self, weights: &mut Weights) {
        let total = weights.budget + weights.location + weights.property_type + weights.size;
        if total > 0.0 {
            weights.budget /= total;
            weights.location /= total;
            weights.property_type /= total;
            weights.size /= total;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::market_trends::{MarketTrend, SupplyLevel};
    use chrono::Utc;

    fn create_test_trend(supply_level: SupplyLevel, price_trend: f64) -> MarketTrend {
        MarketTrend {
            location: "TestLocation".to_string(),
            property_type: "Apartment".to_string(),
            avg_price_per_sqm: 5000.0,
            price_trend,
            demand_level: DemandLevel::Medium,
            supply_level: supply_level,
            prediction_confidence: 0.8,
            last_updated: Utc::now(),
        }
    }

    #[test]
    fn test_low_inventory_adjustment() {
        let mut adjuster = WeightAdjuster::new(None);
        let mut trends = HashMap::new();
        trends.insert("TestLocation:Apartment".to_string(), create_test_trend(SupplyLevel::Scarce, 0.05));
        
        adjuster.update_market_trends(trends);
        let weights = adjuster.get_adjusted_weights("TestLocation", "Apartment");
        
        // Location weight should be reduced
        assert!(weights.location < adjuster.config.base_weights.location);
        // Other weights should increase
        assert!(weights.budget > adjuster.config.base_weights.budget ||
                weights.property_type > adjuster.config.base_weights.property_type ||
                weights.size > adjuster.config.base_weights.size);
    }

    #[test]
    fn test_high_volatility_adjustment() {
        let mut adjuster = WeightAdjuster::new(None);
        let mut trends = HashMap::new();
        trends.insert("TestLocation:Apartment".to_string(), create_test_trend(SupplyLevel::Balanced, 0.15));
        
        adjuster.update_market_trends(trends);
        let weights = adjuster.get_adjusted_weights("TestLocation", "Apartment");
        
        // Budget weight should increase with high volatility
        assert!(weights.budget > adjuster.config.base_weights.budget);
    }

    #[test]
    fn test_weight_normalization() {
        let mut adjuster = WeightAdjuster::new(None);
        let weights = adjuster.get_adjusted_weights("Nonexistent", "Location");
        
        let total = weights.budget + weights.location + weights.property_type + weights.size;
        assert!((total - 1.0).abs() < 0.0001); // Should sum to 1.0
    }
}
