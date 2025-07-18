#!/usr/bin/env python3
"""
Test script for configurable scoring weights API feature
"""

import requests
import json
import sys

BASE_URL = "http://localhost:8080"

def test_property_recommendations_with_weights():
    """Test property recommendations with custom weights"""
    print("Testing property recommendations with custom weights...")
    
    # Test with default weights (no weights provided)
    url = f"{BASE_URL}/recommendations/property/1"
    params = {
        "limit": 5,
        "min_score": 0.1
    }
    
    response = requests.get(url, params=params)
    print(f"Default weights response status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        print(f"Default weights - Got {data['total_count']} recommendations")
        if data['recommendations']:
            print(f"First recommendation score: {data['recommendations'][0]['score']:.3f}")
    else:
        print(f"Error: {response.text}")
        return False
    
    # Test with custom weights that sum to 1.0
    params_custom = {
        "limit": 5,
        "min_score": 0.1,
        "budget_weight": 0.4,      # Higher weight on budget
        "location_weight": 0.3,    # Higher weight on location
        "property_type_weight": 0.2,
        "size_weight": 0.1         # Lower weight on size
    }
    
    response = requests.get(url, params=params_custom)
    print(f"Custom weights response status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        print(f"Custom weights - Got {data['total_count']} recommendations")
        if data['recommendations']:
            print(f"First recommendation score: {data['recommendations'][0]['score']:.3f}")
    else:
        print(f"Error: {response.text}")
        return False
    
    # Test with invalid weights (don't sum to 1.0)
    params_invalid = {
        "budget_weight": 0.5,
        "location_weight": 0.5,
        "property_type_weight": 0.5,  # This will sum to 1.5, should fail
        "size_weight": 0.0
    }
    
    response = requests.get(url, params=params_invalid)
    print(f"Invalid weights response status: {response.status_code}")
    
    if response.status_code == 400:
        error_data = response.json()
        print(f"Expected validation error: {error_data['message']}")
    else:
        print(f"Unexpected response: {response.text}")
        return False
    
    return True

def test_bulk_recommendations_with_weights():
    """Test bulk recommendations with custom weights"""
    print("\nTesting bulk recommendations with custom weights...")
    
    url = f"{BASE_URL}/recommendations/bulk"
    
    # Test with default weights
    payload_default = {
        "property_ids": [1, 2],
        "limit_per_property": 3,
        "min_score": 0.1
    }
    
    response = requests.post(url, json=payload_default)
    print(f"Bulk default weights response status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        print(f"Default weights - Got recommendations for {data['total_properties']} properties")
        print(f"Total recommendations: {data['total_recommendations']}")
    else:
        print(f"Error: {response.text}")
        return False
    
    # Test with custom weights
    payload_custom = {
        "property_ids": [1, 2],
        "limit_per_property": 3,
        "min_score": 0.1,
        "budget_weight": 0.5,
        "location_weight": 0.2,
        "property_type_weight": 0.2,
        "size_weight": 0.1
    }
    
    response = requests.post(url, json=payload_custom)
    print(f"Bulk custom weights response status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        print(f"Custom weights - Got recommendations for {data['total_properties']} properties")
        print(f"Total recommendations: {data['total_recommendations']}")
    else:
        print(f"Error: {response.text}")
        return False
    
    # Test with invalid weights
    payload_invalid = {
        "property_ids": [1],
        "budget_weight": 0.6,
        "location_weight": 0.6,  # Sum = 1.2, should fail
        "property_type_weight": 0.0,
        "size_weight": 0.0
    }
    
    response = requests.post(url, json=payload_invalid)
    print(f"Bulk invalid weights response status: {response.status_code}")
    
    if response.status_code == 400:
        error_data = response.json()
        print(f"Expected validation error: {error_data['message']}")
    else:
        print(f"Unexpected response: {response.text}")
        return False
    
    return True

def main():
    """Main test function"""
    print("Testing configurable scoring weights API feature")
    print("=" * 50)
    
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
    
    # Run tests
    success &= test_property_recommendations_with_weights()
    success &= test_bulk_recommendations_with_weights()
    
    print("\n" + "=" * 50)
    if success:
        print("✅ All tests passed! Configurable weights feature is working correctly.")
    else:
        print("❌ Some tests failed.")
    
    return success

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
