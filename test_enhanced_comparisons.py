#!/usr/bin/env python3
"""
Test script for enhanced JSON comparison API
"""

import requests
import json
import sys

BASE_URL = "http://localhost:8080"

def test_property_comparison():
    """Test the enhanced property comparison with detailed analysis"""
    print("Testing enhanced property comparison...")
    
    url = f"{BASE_URL}/comparisons/properties"
    params = {
        "property1_id": 1,
        "property2_id": 2
    }
    
    response = requests.get(url, params=params)
    print(f"Response status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        print(f"✅ Enhanced comparison generated successfully!")
        
        # Display basic comparison metrics
        print(f"\n📊 Basic Metrics:")
        print(f"Price difference: ${data['comparison_metrics']['price_difference']:,.2f}")
        print(f"Price difference percentage: {data['comparison_metrics']['price_difference_percentage']:.1f}%")
        print(f"Area difference: {data['comparison_metrics']['area_difference']} sqm")
        print(f"Distance between properties: {data['comparison_metrics']['location_distance_km']:.1f} km")
        print(f"Overall similarity score: {data['comparison_metrics']['overall_similarity_score']:.2f}")
        
        # Display detailed analysis
        if 'detailed_analysis' in data:
            analysis = data['detailed_analysis']
            print(f"\n🔍 Detailed Analysis:")
            
            # Price analysis
            if 'price_analysis' in analysis:
                price = analysis['price_analysis']
                cheaper_prop = price['cheaper_property']
                print(f"Cheaper property: Property {cheaper_prop}")
                print(f"Price savings: ${price['price_savings']:,.2f}")
                print(f"Affordability rating: {price['affordability_rating']}")
                print(f"Price per sqm: Property 1: ${price['price_per_sqm_comparison'][0]:.0f}, Property 2: ${price['price_per_sqm_comparison'][1]:.0f}")
            
            # Space analysis
            if 'space_analysis' in analysis:
                space = analysis['space_analysis']
                print(f"\nLarger property: Property {space['larger_property']}")
                print(f"Space advantage: {space['space_advantage']} sqm")
                print(f"Room comparison: {space['room_comparison']}")
                print(f"Space efficiency: Property 1: {space['space_efficiency'][0]:.1f} sqm/room, Property 2: {space['space_efficiency'][1]:.1f} sqm/room")
            
            # Location analysis
            if 'location_analysis' in analysis:
                location = analysis['location_analysis']
                print(f"\nLocation similarity: {location['location_similarity']}")
                print(f"Accessibility notes: {', '.join(location['accessibility_notes'])}")
            
            # Value analysis
            if 'value_analysis' in analysis:
                value = analysis['value_analysis']
                print(f"\nBetter value property: Property {value['better_value_property']}")
                print(f"Value scores: Property 1: {value['value_score'][0]:.2f}, Property 2: {value['value_score'][1]:.2f}")
                print(f"Investment potential: {value['investment_potential']}")
        
        # Display recommendation
        if 'recommendation' in data:
            rec = data['recommendation']
            print(f"\n🎯 Recommendation:")
            print(f"Recommended property: Property {rec['recommended_property']}")
            print(f"Confidence score: {rec['confidence_score']:.2f}")
            print(f"Summary: {rec['summary']}")
            
            if rec['key_reasons']:
                print(f"Key reasons:")
                for reason in rec['key_reasons']:
                    print(f"  • {reason}")
            
            if rec['considerations']:
                print(f"Considerations:")
                for consideration in rec['considerations']:
                    print(f"  • {consideration}")
        
        return True
    else:
        print(f"❌ Error: {response.text}")
        return False

def test_comparison_edge_cases():
    """Test comparison with edge cases"""
    print("\nTesting comparison edge cases...")
    
    # Test with non-existent property
    url = f"{BASE_URL}/comparisons/properties"
    params = {
        "property1_id": 999999,  # Non-existent
        "property2_id": 1
    }
    
    response = requests.get(url, params=params)
    print(f"Non-existent property response status: {response.status_code}")
    
    if response.status_code == 500:
        print("✅ Properly handles non-existent properties")
        return True
    else:
        print(f"❌ Unexpected response: {response.text}")
        return False

def validate_json_structure(data):
    """Validate that the JSON response has the expected structure"""
    required_fields = [
        'property1', 'property2', 'comparison_metrics', 
        'detailed_analysis', 'recommendation'
    ]
    
    missing_fields = []
    for field in required_fields:
        if field not in data:
            missing_fields.append(field)
    
    if missing_fields:
        print(f"❌ Missing required fields: {', '.join(missing_fields)}")
        return False
    
    # Validate detailed_analysis structure
    analysis_fields = [
        'price_analysis', 'space_analysis', 'location_analysis',
        'feature_analysis', 'value_analysis'
    ]
    
    analysis = data.get('detailed_analysis', {})
    missing_analysis = []
    for field in analysis_fields:
        if field not in analysis:
            missing_analysis.append(field)
    
    if missing_analysis:
        print(f"❌ Missing analysis fields: {', '.join(missing_analysis)}")
        return False
    
    print("✅ JSON structure validation passed")
    return True

def main():
    """Main test function"""
    print("Testing Enhanced JSON Comparison API")
    print("=" * 45)
    
    try:
        # Test if server is running
        response = requests.get(f"{BASE_URL}/health", timeout=5)
        if response.status_code != 200:
            print("Health check failed. Is the server running?")
            return False
    except requests.RequestException as e:
        print(f"Cannot connect to server at {BASE_URL}. Is it running?")
        print(f"Error: {e}")
        return False
    
    success = True
    
    # Run comparison test
    comparison_success = test_property_comparison()
    success &= comparison_success
    
    if comparison_success:
        # Test with actual data to validate structure
        url = f"{BASE_URL}/comparisons/properties"
        params = {"property1_id": 1, "property2_id": 2}
        response = requests.get(url, params=params)
        
        if response.status_code == 200:
            success &= validate_json_structure(response.json())
    
    # Test edge cases
    success &= test_comparison_edge_cases()
    
    print("\n" + "=" * 45)
    if success:
        print("✅ All comparison tests passed! Enhanced JSON comparison is working correctly.")
        print("\n🚀 Enhanced Comparison Features:")
        print("  • Detailed price analysis with affordability ratings")
        print("  • Space efficiency and room comparison analysis")
        print("  • Location proximity and accessibility notes")
        print("  • Feature-by-feature comparison with advantages")
        print("  • Value analysis with investment potential")
        print("  • Intelligent recommendation with confidence scoring")
        print("  • Comprehensive reasoning and considerations")
        print("  • JSON format for easy integration")
        
        print("\n📋 JSON Structure includes:")
        print("  • Basic comparison metrics")
        print("  • Detailed multi-dimensional analysis")
        print("  • Smart recommendations with reasoning")
        print("  • Confidence scoring and considerations")
        print("  • All data accessible programmatically")
    else:
        print("❌ Some comparison tests failed.")
    
    return success

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
