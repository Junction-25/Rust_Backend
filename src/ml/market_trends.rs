use crate::models::{Property, Contact};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTrend {
    pub location: String,
    pub property_type: String,
    pub avg_price_per_sqm: f64,
    pub price_trend: f64, // Percentage change
    pub demand_level: DemandLevel,
    pub supply_level: SupplyLevel,
    pub prediction_confidence: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DemandLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SupplyLevel {
    Scarce,
    Limited,
    Balanced,
    Abundant,
    Oversupply,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePredictor {
    pub property_id: i32,
    pub current_price: f64,
    pub predicted_price_3m: f64,
    pub predicted_price_6m: f64,
    pub predicted_price_12m: f64,
    pub confidence_3m: f64,
    pub confidence_6m: f64,
    pub confidence_12m: f64,
    pub factors: Vec<PriceFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFactor {
    pub name: String,
    pub impact: f64, // -1.0 to 1.0
    pub confidence: f64,
}

#[derive(Clone)]
pub struct MarketTrendsEngine {
    trends: HashMap<String, MarketTrend>,
    price_history: HashMap<String, Vec<(DateTime<Utc>, f64)>>,
    demand_indicators: HashMap<String, f64>,
    supply_indicators: HashMap<String, f64>,
}

impl MarketTrendsEngine {
    pub fn new() -> Self {
        Self {
            trends: HashMap::new(),
            price_history: HashMap::new(),
            demand_indicators: HashMap::new(),
            supply_indicators: HashMap::new(),
        }
    }

    /// Analyze market trends from property data
    pub fn analyze_market_trends(&mut self, properties: &[Property]) -> anyhow::Result<()> {
        // Group properties by location and type
        let mut location_type_groups: HashMap<String, Vec<&Property>> = HashMap::new();

        for property in properties {
            // Extract location (could be city from address)
            let location = self.extract_location_from_address(&property.address);
            let key = format!("{}_{}", location, property.property_type);
            location_type_groups.entry(key).or_default().push(property);
        }

        // Analyze each group
        for (key, group_properties) in location_type_groups {
            let parts: Vec<&str> = key.split('_').collect();
            if parts.len() >= 2 {
                let location = parts[0].to_string();
                let property_type = parts[1].to_string();
                
                let trend = self.calculate_trend_for_group(&location, &property_type, &group_properties)?;
                self.trends.insert(key, trend);
            }
        }

        Ok(())
    }

    /// Extract location from property address
    fn extract_location_from_address(&self, address: &str) -> String {
        // Simple extraction - look for known Algerian cities
        let cities = vec![
            "Alger", "Algiers", "Constantine", "Oran", "Annaba", "Setif", "Batna", 
            "Biskra", "Tlemcen", "Bejaïa", "Blida", "Sidi Bel Abbès"
        ];

        for city in cities {
            if address.to_lowercase().contains(&city.to_lowercase()) {
                return city.to_string();
            }
        }

        // Fallback to first word that looks like a city
        address.split(',').next().unwrap_or("Unknown").trim().to_string()
    }

    /// Calculate trend for a specific location-type group
    fn calculate_trend_for_group(
        &self,
        location: &str,
        property_type: &str,
        properties: &[&Property],
    ) -> anyhow::Result<MarketTrend> {
        if properties.is_empty() {
            return Err(anyhow::anyhow!("No properties in group"));
        }

        // Calculate average price per sqm
        let total_value: f64 = properties.iter().map(|p| p.price).sum();
        let total_area: i32 = properties.iter().map(|p| p.area_sqm).sum();
        let avg_price_per_sqm = if total_area > 0 {
            total_value / total_area as f64
        } else {
            0.0
        };

        // Simulate price trend analysis (in real scenario, you'd have historical data)
        let price_trend = self.calculate_simulated_price_trend(location, property_type, properties);

        // Analyze demand and supply
        let demand_level = self.analyze_demand_level(location, property_type, properties);
        let supply_level = self.analyze_supply_level(location, property_type, properties);

        // Calculate prediction confidence based on sample size and variance
        let prediction_confidence = self.calculate_prediction_confidence(properties);

        Ok(MarketTrend {
            location: location.to_string(),
            property_type: property_type.to_string(),
            avg_price_per_sqm,
            price_trend,
            demand_level,
            supply_level,
            prediction_confidence,
            last_updated: Utc::now(),
        })
    }

    /// Simulate price trend calculation
    fn calculate_simulated_price_trend(&self, location: &str, property_type: &str, properties: &[&Property]) -> f64 {
        // Simulate trend based on various factors
        let mut trend: f64 = 0.0;

        // Location factor
        match location {
            "Alger" | "Algiers" => trend += 0.05, // Capital cities tend to appreciate
            "Constantine" => trend += 0.03,
            "Oran" => trend += 0.02,
            _ => trend += 0.01,
        }

        // Property type factor
        match property_type {
            "apartment" => trend += 0.02, // High demand
            "house" => trend += 0.01,
            "office" => trend += 0.03, // Commercial growth
            "land" => trend -= 0.01,   // Slower appreciation
            _ => trend += 0.0,
        }

        // Size factor (larger properties might appreciate differently)
        let avg_size: f64 = properties.iter().map(|p| p.area_sqm as f64).sum::<f64>() / properties.len() as f64;
        if avg_size > 150.0 {
            trend += 0.01; // Luxury properties
        } else if avg_size < 80.0 {
            trend += 0.02; // Affordable housing in demand
        }

        // Add some randomness but keep it realistic (-5% to +15% annually)
        trend = trend.max(-0.05).min(0.15);
        trend
    }

    /// Analyze demand level
    fn analyze_demand_level(&self, location: &str, property_type: &str, properties: &[&Property]) -> DemandLevel {
        let mut demand_score = 0.0;

        // Location demand
        match location {
            "Alger" | "Algiers" => demand_score += 0.8,
            "Constantine" | "Oran" => demand_score += 0.6,
            _ => demand_score += 0.4,
        }

        // Property type demand
        match property_type {
            "apartment" => demand_score += 0.7,
            "house" => demand_score += 0.5,
            "office" => demand_score += 0.6,
            _ => demand_score += 0.3,
        }

        // Price competitiveness (lower prices = higher demand)
        let avg_price: f64 = properties.iter().map(|p| p.price).sum::<f64>() / properties.len() as f64;
        if avg_price < 20_000_000.0 {
            demand_score += 0.3; // Affordable
        } else if avg_price > 40_000_000.0 {
            demand_score -= 0.2; // Luxury market
        }

        match demand_score {
            s if s >= 1.2 => DemandLevel::VeryHigh,
            s if s >= 0.9 => DemandLevel::High,
            s if s >= 0.6 => DemandLevel::Medium,
            s if s >= 0.3 => DemandLevel::Low,
            _ => DemandLevel::VeryLow,
        }
    }

    /// Analyze supply level
    fn analyze_supply_level(&self, _location: &str, _property_type: &str, properties: &[&Property]) -> SupplyLevel {
        let supply_count = properties.len();

        // Simple supply analysis based on available properties
        match supply_count {
            0..=2 => SupplyLevel::Scarce,
            3..=5 => SupplyLevel::Limited,
            6..=10 => SupplyLevel::Balanced,
            11..=20 => SupplyLevel::Abundant,
            _ => SupplyLevel::Oversupply,
        }
    }

    /// Calculate prediction confidence
    fn calculate_prediction_confidence(&self, properties: &[&Property]) -> f64 {
        let sample_size = properties.len() as f64;
        let min_confidence = 0.2;
        let max_confidence = 0.9;

        // Confidence increases with sample size but plateaus
        let size_factor = (sample_size / (sample_size + 10.0)).max(0.3);

        // Calculate price variance to adjust confidence
        if properties.len() > 1 {
            let prices: Vec<f64> = properties.iter().map(|p| p.price).collect();
            let mean = prices.iter().sum::<f64>() / prices.len() as f64;
            let variance = prices.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / prices.len() as f64;
            let std_dev = variance.sqrt();
            let cv = if mean > 0.0 { std_dev / mean } else { 1.0 }; // Coefficient of variation

            // Lower variance = higher confidence
            let variance_factor = (1.0 - cv.min(1.0)).max(0.1);
            
            (size_factor * variance_factor).max(min_confidence).min(max_confidence)
        } else {
            min_confidence
        }
    }

    /// Predict property price based on market trends
    pub fn predict_property_price(&self, property: &Property) -> PricePredictor {
        let location = self.extract_location_from_address(&property.address);
        let key = format!("{}_{}", location, property.property_type);
        
        let trend = self.trends.get(&key);
        let base_trend = trend.map(|t| t.price_trend).unwrap_or(0.02); // Default 2% growth

        // Calculate predictions for different time horizons
        let current_price = property.price;
        
        // 3-month prediction
        let predicted_3m = current_price * (1.0 + base_trend * 0.25);
        let confidence_3m = trend.map(|t| t.prediction_confidence * 0.9).unwrap_or(0.3);

        // 6-month prediction
        let predicted_6m = current_price * (1.0 + base_trend * 0.5);
        let confidence_6m = trend.map(|t| t.prediction_confidence * 0.8).unwrap_or(0.25);

        // 12-month prediction
        let predicted_12m = current_price * (1.0 + base_trend);
        let confidence_12m = trend.map(|t| t.prediction_confidence * 0.7).unwrap_or(0.2);

        // Generate factors affecting price
        let factors = self.generate_price_factors(property, trend);

        PricePredictor {
            property_id: property.id,
            current_price,
            predicted_price_3m: predicted_3m,
            predicted_price_6m: predicted_6m,
            predicted_price_12m: predicted_12m,
            confidence_3m,
            confidence_6m,
            confidence_12m,
            factors,
        }
    }

    /// Generate factors affecting price prediction
    fn generate_price_factors(&self, property: &Property, trend: Option<&MarketTrend>) -> Vec<PriceFactor> {
        let mut factors = Vec::new();

        // Location factor
        let location = self.extract_location_from_address(&property.address);
        let location_impact = match location.as_str() {
            "Alger" | "Algiers" => 0.3,
            "Constantine" | "Oran" => 0.2,
            _ => 0.1,
        };
        factors.push(PriceFactor {
            name: "Location Premium".to_string(),
            impact: location_impact,
            confidence: 0.8,
        });

        // Property type factor
        let type_impact = match property.property_type.as_str() {
            "apartment" => 0.1,
            "house" => 0.0,
            "office" => 0.2,
            "land" => -0.1,
            _ => 0.0,
        };
        factors.push(PriceFactor {
            name: "Property Type Demand".to_string(),
            impact: type_impact,
            confidence: 0.7,
        });

        // Size factor
        let size_impact = if property.area_sqm > 150 {
            0.15 // Luxury premium
        } else if property.area_sqm < 80 {
            0.1 // Affordability demand
        } else {
            0.0
        };
        factors.push(PriceFactor {
            name: "Size Category".to_string(),
            impact: size_impact,
            confidence: 0.6,
        });

        // Market trend factor
        if let Some(trend) = trend {
            factors.push(PriceFactor {
                name: "Market Trend".to_string(),
                impact: trend.price_trend,
                confidence: trend.prediction_confidence,
            });

            // Demand/supply balance
            let supply_demand_impact = match (&trend.demand_level, &trend.supply_level) {
                (DemandLevel::VeryHigh, SupplyLevel::Scarce) => 0.25,
                (DemandLevel::High, SupplyLevel::Limited) => 0.15,
                (DemandLevel::Low, SupplyLevel::Abundant) => -0.15,
                (DemandLevel::VeryLow, SupplyLevel::Oversupply) => -0.25,
                _ => 0.0,
            };
            factors.push(PriceFactor {
                name: "Supply-Demand Balance".to_string(),
                impact: supply_demand_impact,
                confidence: trend.prediction_confidence * 0.8,
            });
        }

        factors
    }

    /// Get market trends for a location and property type
    pub fn get_market_trend(&self, location: &str, property_type: &str) -> Option<&MarketTrend> {
        let key = format!("{}_{}", location, property_type);
        self.trends.get(&key)
    }

    /// Get all market trends
    pub fn get_all_trends(&self) -> &HashMap<String, MarketTrend> {
        &self.trends
    }

    /// Update trend with new data point
    pub fn update_trend_with_sale(&mut self, location: &str, property_type: &str, sale_price: f64, area_sqm: i32) -> anyhow::Result<()> {
        let key = format!("{}_{}", location, property_type);
        
        if let Some(trend) = self.trends.get_mut(&key) {
            let new_price_per_sqm = sale_price / area_sqm as f64;
            
            // Update with exponential moving average
            let alpha = 0.1; // Learning rate
            trend.avg_price_per_sqm = alpha * new_price_per_sqm + (1.0 - alpha) * trend.avg_price_per_sqm;
            trend.last_updated = Utc::now();
        }

        Ok(())
    }

    /// Generate market insights
    pub fn generate_market_insights(&self) -> Vec<String> {
        let mut insights = Vec::new();

        for (key, trend) in &self.trends {
            let parts: Vec<&str> = key.split('_').collect();
            if parts.len() >= 2 {
                let location = parts[0];
                let property_type = parts[1];

                if trend.price_trend > 0.1 {
                    insights.push(format!(
                        "🔥 Strong growth in {} {} market: {:.1}% annual appreciation",
                        location, property_type, trend.price_trend * 100.0
                    ));
                }

                if matches!(trend.demand_level, DemandLevel::VeryHigh) && matches!(trend.supply_level, SupplyLevel::Scarce) {
                    insights.push(format!(
                        "⚡ Hot market alert: Very high demand but scarce supply for {} {} properties",
                        location, property_type
                    ));
                }

                if trend.price_trend < -0.02 {
                    insights.push(format!(
                        "📉 Buyer's market: {} {} prices declining by {:.1}% annually",
                        location, property_type, trend.price_trend * 100.0
                    ));
                }
            }
        }

        insights
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Property, Location};

    #[test]
    fn test_location_extraction() {
        let engine = MarketTrendsEngine::new();
        assert_eq!(engine.extract_location_from_address("123 Main St, Algiers"), "Algiers");
        assert_eq!(engine.extract_location_from_address("456 Rue de Constantine"), "Constantine");
    }

    #[test]
    fn test_price_prediction() {
        let mut engine = MarketTrendsEngine::new();
        let property = Property {
            id: 1,
            address: "Test St, Algiers".to_string(),
            location: Location { lat: 36.7, lon: 3.2 },
            price: 30_000_000.0,
            area_sqm: 100,
            property_type: "apartment".to_string(),
            number_of_rooms: 3,
        };

        let prediction = engine.predict_property_price(&property);
        assert!(prediction.predicted_price_12m >= property.price);
    }
}
