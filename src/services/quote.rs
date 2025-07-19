use crate::db::Repository;
use crate::models::*;
use anyhow::Result;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct QuoteService {
    repository: Arc<Repository>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteRequest {
    pub property_id: i32,
    pub contact_id: i32,
    pub additional_costs: Option<Vec<AdditionalCost>>,
    pub custom_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdditionalCost {
    pub description: String,
    pub amount: i64, // in cents
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub quote_id: i32,
    pub property: Property,
    pub contact_details: ContactQuoteDetails,
    pub base_price: i64,
    pub additional_costs: Vec<AdditionalCost>,
    pub total_amount: i64,
    pub financial_details: FinancialDetails,
    pub quote_summary: QuoteSummary,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinancialDetails {
    pub property_price: f64,
    pub estimated_down_payment: f64,
    pub estimated_monthly_payment: f64,
    pub estimated_closing_costs: f64,
    pub financing_options: Vec<FinancingOption>,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinancingOption {
    pub loan_type: String,
    pub interest_rate: f64,
    pub loan_term_years: i32,
    pub monthly_payment: f64,
    pub total_interest: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteSummary {
    pub affordability_score: f64,
    pub recommendation_level: String,
    pub key_highlights: Vec<String>,
    pub potential_concerns: Vec<String>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactQuoteDetails {
    pub name: String,
    pub budget_min: f64,
    pub budget_max: f64,
    pub preferred_locations: Vec<String>,
    pub property_types: Vec<String>,
    pub min_rooms: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonQuoteRequest {
    pub property1_id: i32,
    pub property2_id: i32,
    pub contact_id: i32,
    pub custom_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonQuoteResponse {
    pub quote_id: String,
    pub contact_details: ContactQuoteDetails,
    pub property1: Property,
    pub property2: Property,
    pub comparison_details: ComparisonDetails,
    pub financial_comparison: FinancialComparison,
    pub recommendation: ComparisonRecommendation,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonDetails {
    pub price_difference: i64,
    pub area_difference: i32,
    pub location_comparison: String,
    pub amenities_comparison: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinancialComparison {
    pub property1_financial: FinancialDetails,
    pub property2_financial: FinancialDetails,
    pub monthly_payment_difference: f64,
    pub total_cost_difference: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonRecommendation {
    pub recommended_property_id: i32,
    pub reasoning: Vec<String>,
    pub score_difference: f64,
}

impl QuoteService {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { repository }
    }

    pub async fn generate_property_quote(&self, request: QuoteRequest) -> Result<QuoteResponse> {
        // Get property and contact
        let property = self.repository.get_property_by_id(request.property_id).await?
            .ok_or_else(|| anyhow::anyhow!("Property not found"))?;
        
        let contact = self.repository.get_contact_by_id(request.contact_id).await?
            .ok_or_else(|| anyhow::anyhow!("Contact not found"))?;

        // Calculate total amount
        let additional_costs = request.additional_costs.unwrap_or_default();
        let additional_total: i64 = additional_costs.iter().map(|cost| cost.amount).sum();
        let total_amount = property.price as i64 + additional_total;

        // Generate financial details
        let financial_details = self.calculate_financial_details(&property, &contact);
        
        // Generate quote summary
        let quote_summary = self.generate_quote_summary(&property, &contact);

        // Create contact details for quote
        let contact_details = ContactQuoteDetails {
            name: contact.name.clone(),
            budget_min: contact.min_budget,
            budget_max: contact.max_budget,
            preferred_locations: contact.preferred_locations.iter().map(|loc| loc.name.clone()).collect(),
            property_types: contact.property_types.clone(),
            min_rooms: contact.min_rooms,
        };

        Ok(QuoteResponse {
            quote_id: 1, // Simple ID for now - could be generated from DB
            property: property.clone(),
            contact_details,
            base_price: property.price as i64,
            additional_costs,
            total_amount,
            financial_details,
            quote_summary,
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::days(30),
        })
    }

    pub async fn generate_comparison_quote(&self, request: ComparisonQuoteRequest) -> Result<ComparisonQuoteResponse> {
        // Get properties and contact
        let property1 = self.repository.get_property_by_id(request.property1_id).await?
            .ok_or_else(|| anyhow::anyhow!("First property not found"))?;
        
        let property2 = self.repository.get_property_by_id(request.property2_id).await?
            .ok_or_else(|| anyhow::anyhow!("Second property not found"))?;

        let contact = self.repository.get_contact_by_id(request.contact_id).await?
            .ok_or_else(|| anyhow::anyhow!("Contact not found"))?;

        // Generate comparison details
        let comparison_details = ComparisonDetails {
            price_difference: property2.price as i64 - property1.price as i64,
            area_difference: property2.area_sqm - property1.area_sqm,
            location_comparison: format!("Property 1: {} vs Property 2: {}", property1.location.lat, property2.location.lat),
            amenities_comparison: vec![
                format!("Property 1 type: {}", property1.property_type),
                format!("Property 2 type: {}", property2.property_type),
            ],
        };

        // Generate financial comparison
        let property1_financial = self.calculate_financial_details(&property1, &contact);
        let property2_financial = self.calculate_financial_details(&property2, &contact);
        
        let financial_comparison = FinancialComparison {
            monthly_payment_difference: property2_financial.estimated_monthly_payment - property1_financial.estimated_monthly_payment,
            total_cost_difference: (property2.price - property1.price) as f64,
            property1_financial,
            property2_financial,
        };

        // Generate recommendation
        let recommendation = self.generate_comparison_recommendation(&property1, &property2, &contact);

        // Create contact details for quote
        let contact_details = ContactQuoteDetails {
            name: contact.name.clone(),
            budget_min: contact.min_budget,
            budget_max: contact.max_budget,
            preferred_locations: contact.preferred_locations.iter().map(|loc| loc.name.clone()).collect(),
            property_types: contact.property_types.clone(),
            min_rooms: contact.min_rooms,
        };

        // Generate quote ID
        let quote_id = format!("CMP_{}_{}_{}", request.property1_id, request.property2_id, chrono::Utc::now().timestamp());

        Ok(ComparisonQuoteResponse {
            quote_id,
            contact_details,
            property1,
            property2,
            comparison_details,
            financial_comparison,
            recommendation,
            created_at: chrono::Utc::now(),
        })
    }

    pub async fn generate_recommendation_quote(
        &self,
        property_id: i32,
        recommendations: &[Recommendation],
    ) -> Result<serde_json::Value> {
        // Get property
        let property = self.repository.get_property_by_id(property_id).await?
            .ok_or_else(|| anyhow::anyhow!("Property not found"))?;

        // Convert recommendations to a structured JSON response
        let recommendation_data = serde_json::json!({
            "property_id": property_id,
            "property_address": property.address,
            "property_price": property.price,
            "recommendations_count": recommendations.len(),
            "recommendations": recommendations.iter().map(|r| serde_json::json!({
                "contact_id": r.contact.id,
                "contact_name": r.contact.name.clone(),
                "score": r.score,
                "reasons": r.explanation.reasons
            })).collect::<Vec<_>>(),
            "generated_at": chrono::Utc::now().to_rfc3339()
        });

        Ok(recommendation_data)
    }

    fn calculate_financial_details(&self, property: &Property, _contact: &Contact) -> FinancialDetails {
        let property_price = property.price as f64;
        
        // Calculate down payment (typically 20%)
        let down_payment_percentage = 0.20;
        let estimated_down_payment = property_price * down_payment_percentage;
        
        // Calculate loan amount
        let loan_amount = property_price - estimated_down_payment;
        
        // Estimate closing costs (typically 2-3% of property price)
        let estimated_closing_costs = property_price * 0.025;
        
        // Calculate financing options
        let financing_options = vec![
            self.calculate_financing_option("Conventional 30-year", loan_amount, 6.5, 30),
            self.calculate_financing_option("Conventional 15-year", loan_amount, 6.0, 15),
            self.calculate_financing_option("FHA 30-year", loan_amount, 6.25, 30),
        ];
        
        // Use the first financing option for estimated monthly payment
        let estimated_monthly_payment = financing_options[0].monthly_payment;

        FinancialDetails {
            property_price,
            estimated_down_payment,
            estimated_monthly_payment,
            estimated_closing_costs,
            financing_options,
            currency: "DZD".to_string(),
        }
    }

    fn calculate_financing_option(&self, loan_type: &str, loan_amount: f64, annual_interest_rate: f64, loan_term_years: i32) -> FinancingOption {
        let monthly_interest_rate = annual_interest_rate / 100.0 / 12.0;
        let total_payments = loan_term_years * 12;
        
        // Calculate monthly payment using mortgage formula
        let monthly_payment = if monthly_interest_rate > 0.0 {
            loan_amount * (monthly_interest_rate * (1.0 + monthly_interest_rate).powi(total_payments))
                / ((1.0 + monthly_interest_rate).powi(total_payments) - 1.0)
        } else {
            loan_amount / total_payments as f64
        };
        
        let total_paid = monthly_payment * total_payments as f64;
        let total_interest = total_paid - loan_amount;

        FinancingOption {
            loan_type: loan_type.to_string(),
            interest_rate: annual_interest_rate,
            loan_term_years,
            monthly_payment,
            total_interest,
        }
    }

    fn generate_quote_summary(&self, property: &Property, contact: &Contact) -> QuoteSummary {
        let mut key_highlights = Vec::new();
        let mut potential_concerns = Vec::new();
        let next_steps = vec![
            "Schedule property viewing".to_string(),
            "Get mortgage pre-approval".to_string(),
            "Conduct property inspection".to_string(),
            "Review HOA documents if applicable".to_string(),
        ];
        
        // Analyze budget compatibility
        let affordability_score = if property.price >= contact.min_budget && property.price <= contact.max_budget {
            let budget_utilization = (property.price - contact.min_budget) / (contact.max_budget - contact.min_budget);
            key_highlights.push("Property is within your budget range".to_string());
            1.0 - (budget_utilization - 0.5).abs()
        } else if property.price < contact.min_budget {
            potential_concerns.push("Property is below your minimum budget - consider if it meets your needs".to_string());
            0.6
        } else {
            potential_concerns.push("Property exceeds your maximum budget".to_string());
            0.3
        };

        // Analyze property features
        if property.number_of_rooms >= contact.min_rooms {
            key_highlights.push(format!("Property has {} rooms, meeting your requirement", property.number_of_rooms));
        } else {
            potential_concerns.push("Property has fewer rooms than requested".to_string());
        }

        // Property type check
        if contact.property_types.contains(&property.property_type) {
            key_highlights.push("Property type matches your preferences".to_string());
        } else {
            potential_concerns.push("Property type differs from your preferences".to_string());
        }

        let recommendation_level = if affordability_score > 0.8 && potential_concerns.is_empty() {
            "Highly Recommended"
        } else if affordability_score > 0.6 && potential_concerns.len() <= 1 {
            "Recommended"
        } else if affordability_score > 0.4 {
            "Consider with Caution"
        } else {
            "Not Recommended"
        };

        QuoteSummary {
            affordability_score,
            recommendation_level: recommendation_level.to_string(),
            key_highlights,
            potential_concerns,
            next_steps,
        }
    }

    fn generate_comparison_recommendation(&self, property1: &Property, property2: &Property, contact: &Contact) -> ComparisonRecommendation {
        let mut reasoning = Vec::new();
        
        // Price comparison
        let price_diff = property2.price - property1.price;
        if price_diff.abs() > 10000.0 {
            if price_diff > 0.0 {
                reasoning.push(format!("Property 2 is ${:.0} more expensive than Property 1", price_diff));
            } else {
                reasoning.push(format!("Property 1 is ${:.0} more expensive than Property 2", price_diff.abs()));
            }
        }

        // Area comparison
        let area_diff = property2.area_sqm - property1.area_sqm;
        if area_diff != 0 {
            if area_diff > 0 {
                reasoning.push(format!("Property 2 has {} sqm more space", area_diff));
            } else {
                reasoning.push(format!("Property 1 has {} sqm more space", area_diff.abs()));
            }
        }

        // Budget fit analysis
        let prop1_budget_fit = property1.price >= contact.min_budget && property1.price <= contact.max_budget;
        let prop2_budget_fit = property2.price >= contact.min_budget && property2.price <= contact.max_budget;

        let (recommended_property_id, score_difference) = if prop1_budget_fit && !prop2_budget_fit {
            reasoning.push("Property 1 fits better within your budget".to_string());
            (property1.id, 0.3)
        } else if prop2_budget_fit && !prop1_budget_fit {
            reasoning.push("Property 2 fits better within your budget".to_string());
            (property2.id, 0.3)
        } else if prop1_budget_fit && prop2_budget_fit {
            // Both fit, choose based on value
            if property1.area_sqm as f64 / property1.price > property2.area_sqm as f64 / property2.price {
                reasoning.push("Property 1 offers better value per square meter".to_string());
                (property1.id, 0.1)
            } else {
                reasoning.push("Property 2 offers better value per square meter".to_string());
                (property2.id, 0.1)
            }
        } else {
            // Neither fits perfectly
            if (property1.price - contact.max_budget).abs() < (property2.price - contact.max_budget).abs() {
                reasoning.push("Property 1 is closer to your budget range".to_string());
                (property1.id, 0.05)
            } else {
                reasoning.push("Property 2 is closer to your budget range".to_string());
                (property2.id, 0.05)
            }
        };

        ComparisonRecommendation {
            recommended_property_id,
            reasoning,
            score_difference,
        }
    }

    pub fn get_standard_additional_costs() -> Vec<AdditionalCost> {
        vec![
            AdditionalCost {
                description: "Legal Fees".to_string(),
                amount: 150000, // $1,500
            },
            AdditionalCost {
                description: "Property Inspection".to_string(),
                amount: 50000, // $500
            },
            AdditionalCost {
                description: "Transfer Tax".to_string(),
                amount: 0, // To be calculated based on property value
            },
            AdditionalCost {
                description: "Real Estate Commission".to_string(),
                amount: 0, // To be calculated as percentage
            },
        ]
    }

    pub fn calculate_transfer_tax(property_value: i64) -> i64 {
        // Example: 1% transfer tax
        property_value / 100
    }

    pub fn calculate_commission(property_value: i64, commission_percentage: f64) -> i64 {
        (property_value as f64 * commission_percentage / 100.0) as i64
    }
}
