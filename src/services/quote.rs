use crate::db::Repository;
use crate::models::*;
use crate::utils::pdf::*;
use anyhow::Result;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct QuoteService {
    repository: Arc<Repository>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteRequest {
    pub property_id: uuid::Uuid,
    pub contact_id: uuid::Uuid,
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
    pub quote_id: uuid::Uuid,
    pub property: Property,
    pub contact: Contact,
    pub base_price: i64,
    pub additional_costs: Vec<AdditionalCost>,
    pub total_amount: i64,
    pub pdf_data: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonQuoteRequest {
    pub property1_id: uuid::Uuid,
    pub property2_id: uuid::Uuid,
    pub contact_id: uuid::Uuid,
    pub custom_message: Option<String>,
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
        let total_amount = property.price + additional_total;

        // Generate PDF
        let additional_costs_for_pdf: Vec<(String, i64)> = additional_costs
            .iter()
            .map(|cost| (cost.description.clone(), cost.amount))
            .collect();

        let pdf_data = generate_quote_pdf(
            &property,
            &contact,
            if additional_costs_for_pdf.is_empty() {
                None
            } else {
                Some(&additional_costs_for_pdf)
            },
        )?;

        Ok(QuoteResponse {
            quote_id: uuid::Uuid::new_v4(),
            property: property.clone(),
            contact,
            base_price: property.price,
            additional_costs,
            total_amount,
            pdf_data,
            created_at: chrono::Utc::now(),
        })
    }

    pub async fn generate_comparison_quote(&self, request: ComparisonQuoteRequest) -> Result<Vec<u8>> {
        // Get properties and contact
        let _property1 = self.repository.get_property_by_id(request.property1_id).await?
            .ok_or_else(|| anyhow::anyhow!("First property not found"))?;
        
        let _property2 = self.repository.get_property_by_id(request.property2_id).await?
            .ok_or_else(|| anyhow::anyhow!("Second property not found"))?;

        let _contact = self.repository.get_contact_by_id(request.contact_id).await?
            .ok_or_else(|| anyhow::anyhow!("Contact not found"))?;

        // Create comparison service to get metrics
        let comparison_service = crate::services::comparison::ComparisonService::new(self.repository.clone());
        let comparison = comparison_service.compare_properties(request.property1_id, request.property2_id).await?;

        // Generate PDF
        let pdf_data = generate_comparison_pdf(&comparison)?;

        Ok(pdf_data)
    }

    pub async fn generate_recommendation_quote(
        &self,
        property_id: uuid::Uuid,
        recommendations: &[Recommendation],
    ) -> Result<Vec<u8>> {
        // Get property
        let property = self.repository.get_property_by_id(property_id).await?
            .ok_or_else(|| anyhow::anyhow!("Property not found"))?;

        // Generate PDF
        let pdf_data = generate_recommendation_report_pdf(recommendations, &property)?;

        Ok(pdf_data)
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
