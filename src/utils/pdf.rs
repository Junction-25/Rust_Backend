use printpdf::*;
use crate::models::{Property, Contact, PropertyComparison, Recommendation};
use anyhow::Result;
use chrono::Utc;
use std::io::BufWriter;

pub fn generate_comparison_pdf(comparison: &PropertyComparison) -> Result<Vec<u8>> {
    let (doc, page1, layer1) = PdfDocument::new("Property Comparison", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);
    
    // Add fonts
    let font = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let regular_font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    // Title
    current_layer.use_text("Property Comparison Report", 18.0, Mm(20.0), Mm(270.0), &font);
    current_layer.use_text(&format!("Generated on: {}", Utc::now().format("%Y-%m-%d %H:%M UTC")), 10.0, Mm(20.0), Mm(260.0), &regular_font);

    // Property 1 details
    let mut y_position = 240.0;
    current_layer.use_text("Property 1:", 14.0, Mm(20.0), Mm(y_position), &font);
    y_position -= 10.0;
    
    current_layer.use_text(&format!("Address: {}", comparison.property1.address), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Price: ${}", comparison.property1.price / 100.0), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Area: {} sqm", comparison.property1.area_sqm), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Rooms: {}", comparison.property1.number_of_rooms), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Location: {:.4}, {:.4}", comparison.property1.location.lat, comparison.property1.location.lon), 12.0, Mm(25.0), Mm(y_position), &regular_font);

    // Property 2 details
    y_position -= 20.0;
    current_layer.use_text("Property 2:", 14.0, Mm(20.0), Mm(y_position), &font);
    y_position -= 10.0;
    
    current_layer.use_text(&format!("Address: {}", comparison.property2.address), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Price: ${}", comparison.property2.price / 100.0), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Area: {} sqm", comparison.property2.area_sqm), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Rooms: {}", comparison.property2.number_of_rooms), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Location: {:.4}, {:.4}", comparison.property2.location.lat, comparison.property2.location.lon), 12.0, Mm(25.0), Mm(y_position), &regular_font);

    // Comparison metrics
    y_position -= 20.0;
    current_layer.use_text("Comparison Analysis:", 14.0, Mm(20.0), Mm(y_position), &font);
    y_position -= 10.0;
    
    current_layer.use_text(&format!("Price Difference: ${} ({:.1}%)", 
        comparison.comparison_metrics.price_difference / 100.0,
        comparison.comparison_metrics.price_difference_percentage), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Area Difference: {} sqm ({:.1}%)", 
        comparison.comparison_metrics.area_difference,
        comparison.comparison_metrics.area_difference_percentage), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Distance Between Properties: {:.2} km", 
        comparison.comparison_metrics.location_distance_km), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Feature Similarity: {:.1}%", 
        comparison.comparison_metrics.overall_similarity_score * 100.0), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Overall Similarity: {:.1}%", 
        comparison.comparison_metrics.overall_similarity_score * 100.0), 12.0, Mm(25.0), Mm(y_position), &regular_font);

    // Save to bytes
    let mut pdf_bytes = Vec::new();
    let mut buf_writer = BufWriter::new(&mut pdf_bytes);
    doc.save(&mut buf_writer)?;
    drop(buf_writer); // Ensure buffer is flushed
    Ok(pdf_bytes)
}

pub fn generate_quote_pdf(property: &Property, contact: &Contact, additional_costs: Option<&[(String, i64)]>) -> Result<Vec<u8>> {
    let (doc, page1, layer1) = PdfDocument::new("Property Quote", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);
    
    let font = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let regular_font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    // Header
    current_layer.use_text("PROPERTY QUOTE", 20.0, Mm(20.0), Mm(270.0), &font);
    current_layer.use_text(&format!("Quote Date: {}", Utc::now().format("%Y-%m-%d")), 10.0, Mm(20.0), Mm(260.0), &regular_font);

    // Client information
    let mut y_position = 240.0;
    current_layer.use_text("Client Information:", 14.0, Mm(20.0), Mm(y_position), &font);
    y_position -= 10.0;
    current_layer.use_text(&format!("Name: {}", contact.name), 12.0, Mm(25.0), Mm(y_position), &regular_font);

    // Property information
    y_position -= 20.0;
    current_layer.use_text("Property Details:", 14.0, Mm(20.0), Mm(y_position), &font);
    y_position -= 10.0;
    current_layer.use_text(&format!("Address: {}", property.address), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Location: {:.4}, {:.4}", property.location.lat, property.location.lon), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Property Type: {:?}", property.property_type), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Area: {} sqm", property.area_sqm), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Rooms: {}", property.number_of_rooms), 12.0, Mm(25.0), Mm(y_position), &regular_font);

    // Pricing breakdown
    y_position -= 20.0;
    current_layer.use_text("Pricing Breakdown:", 14.0, Mm(20.0), Mm(y_position), &font);
    y_position -= 10.0;
    current_layer.use_text(&format!("Property Price: ${:.2}", property.price as f64 / 100.0), 12.0, Mm(25.0), Mm(y_position), &regular_font);
    
    let mut total = property.price as i64;
    
    if let Some(costs) = additional_costs {
        for (description, amount) in costs {
            y_position -= 8.0;
            current_layer.use_text(&format!("{}: ${:.2}", description, *amount as f64 / 100.0), 12.0, Mm(25.0), Mm(y_position), &regular_font);
            total += *amount;
        }
    }

    y_position -= 15.0;
    current_layer.use_text(&format!("TOTAL: ${:.2}", total as f64 / 100.0), 14.0, Mm(25.0), Mm(y_position), &font);

    // Terms and conditions
    y_position -= 25.0;
    current_layer.use_text("Terms & Conditions:", 12.0, Mm(20.0), Mm(y_position), &font);
    y_position -= 8.0;
    current_layer.use_text("• This quote is valid for 30 days from the date of issue", 10.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 6.0;
    current_layer.use_text("• Final price subject to property inspection and legal review", 10.0, Mm(25.0), Mm(y_position), &regular_font);
    y_position -= 6.0;
    current_layer.use_text("• Additional fees may apply for legal, inspection, and transfer costs", 10.0, Mm(25.0), Mm(y_position), &regular_font);

    let mut pdf_bytes = Vec::new();
    let mut buf_writer = BufWriter::new(&mut pdf_bytes);
    doc.save(&mut buf_writer)?;
    drop(buf_writer); // Ensure buffer is flushed
    Ok(pdf_bytes)
}

pub fn generate_recommendation_report_pdf(recommendations: &[Recommendation], property: &Property) -> Result<Vec<u8>> {
    let (doc, page1, layer1) = PdfDocument::new("Recommendation Report", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);
    
    let font = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let regular_font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    // Header
    current_layer.use_text("RECOMMENDATION REPORT", 18.0, Mm(20.0), Mm(270.0), &font);
    current_layer.use_text(&format!("Generated: {}", Utc::now().format("%Y-%m-%d %H:%M UTC")), 10.0, Mm(20.0), Mm(260.0), &regular_font);

    // Property summary
    let mut y_position = 240.0;
    current_layer.use_text(&format!("Property: {}", property.address), 14.0, Mm(20.0), Mm(y_position), &font);
    y_position -= 8.0;
    current_layer.use_text(&format!("Price: ${:.2} | Area: {} sqm | Rooms: {}", 
        property.price as f64 / 100.0, property.area_sqm, property.number_of_rooms), 11.0, Mm(20.0), Mm(y_position), &regular_font);

    y_position -= 20.0;
    current_layer.use_text(&format!("Top {} Recommended Contacts:", recommendations.len()), 14.0, Mm(20.0), Mm(y_position), &font);

    for (i, recommendation) in recommendations.iter().enumerate() {
        y_position -= 15.0;
        
        if y_position < 50.0 {
            // Add new page if needed
            break;
        }

        current_layer.use_text(&format!("{}. {} (Score: {:.1}%)", 
            i + 1, 
            recommendation.contact.name,
            recommendation.score * 100.0), 12.0, Mm(25.0), Mm(y_position), &font);
        
        y_position -= 8.0;
        current_layer.use_text(&format!("Budget: ${}-${}", 
            recommendation.contact.min_budget,
            recommendation.contact.max_budget), 10.0, Mm(30.0), Mm(y_position), &regular_font);
        
        y_position -= 6.0;
        if !recommendation.explanation.reasons.is_empty() {
            let reason = &recommendation.explanation.reasons[0]; // Show first reason
            current_layer.use_text(&format!("Key Match: {}", reason), 10.0, Mm(30.0), Mm(y_position), &regular_font);
        }
    }

    let mut pdf_bytes = Vec::new();
    let mut buf_writer = BufWriter::new(&mut pdf_bytes);
    doc.save(&mut buf_writer)?;
    drop(buf_writer); // Ensure buffer is flushed
    Ok(pdf_bytes)
}
