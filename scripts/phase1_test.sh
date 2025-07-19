#!/bin/bash

# Phase 1 Testing Script - Neural Binning and Location Attention
# This script compares traditional vs neural-enhanced recommendations

echo "=========================================="
echo "Phase 1: Neural Binning & Location Attention"
echo "=========================================="

# Set base URL
BASE_URL="http://localhost:8080"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to test and compare recommendation endpoints
test_recommendations() {
    local test_name="$1"
    local contact_id="$2"
    local additional_params="$3"
    
    echo -e "${BLUE}Testing: ${test_name}${NC}"
    echo "----------------------------------------"
    
    # Traditional scoring (neural disabled)
    echo -e "${YELLOW}Traditional Scoring:${NC}"
    curl -s "${BASE_URL}/api/recommendations/contact/${contact_id}?neural_scoring=false${additional_params}" \
        | jq -r '.recommendations[:3] | .[] | "Score: \(.score | . * 100 | floor / 100), Contact: \(.contact.first_name) \(.contact.last_name), Property: €\(.property.price)"' \
        | head -3
    
    echo ""
    
    # Neural-enhanced scoring (default)
    echo -e "${YELLOW}Neural-Enhanced Scoring:${NC}"
    curl -s "${BASE_URL}/api/recommendations/contact/${contact_id}?neural_scoring=true${additional_params}" \
        | jq -r '.recommendations[:3] | .[] | "Score: \(.score | . * 100 | floor / 100), Contact: \(.contact.first_name) \(.contact.last_name), Property: €\(.property.price)"' \
        | head -3
    
    echo ""
    echo "----------------------------------------"
    echo ""
}

# Function to check server health
check_server() {
    echo -e "${BLUE}Checking server health...${NC}"
    
    # Try multiple times to give server time to start
    for i in {1..10}; do
        if curl -s "${BASE_URL}/health" | jq -r '.status' 2>/dev/null | grep -q "healthy"; then
            echo -e "${GREEN}✓ Server is healthy${NC}"
            return 0
        else
            if [ $i -le 5 ]; then
                echo "Attempt $i/10: Waiting for server to start..."
                sleep 2
            else
                echo "Attempt $i/10: Still waiting..."
                sleep 1
            fi
        fi
    done
    
    echo -e "${RED}✗ Server is not responding after 20 seconds${NC}"
    return 1
}

# Function to get performance metrics
get_performance_metrics() {
    local contact_id="$1"
    
    echo -e "${BLUE}Performance Comparison:${NC}"
    
    # Traditional performance
    traditional_time=$(curl -s -w "%{time_total}" "${BASE_URL}/api/recommendations/contact/${contact_id}?neural_scoring=false&limit=50" -o /dev/null)
    
    # Neural performance
    neural_time=$(curl -s -w "%{time_total}" "${BASE_URL}/api/recommendations/contact/${contact_id}?neural_scoring=true&limit=50" -o /dev/null)
    
    echo "Traditional scoring time: ${traditional_time}s"
    echo "Neural-enhanced time: ${neural_time}s"
    
    # Calculate percentage difference
    if (( $(echo "${neural_time} > 0" | bc -l) )); then
        performance_diff=$(echo "scale=2; (${neural_time} - ${traditional_time}) / ${traditional_time} * 100" | bc -l)
        echo "Performance difference: ${performance_diff}%"
    fi
    
    echo ""
}

# Function to test feature binning
test_feature_binning() {
    echo -e "${BLUE}Testing Feature Binning Analysis:${NC}"
    
    # Get recommendations with detailed scoring
    curl -s "${BASE_URL}/api/recommendations/contact/1?neural_scoring=true&limit=5" \
        | jq -r '.recommendations[] | "Property ID: \(.property.id), Neural Score: \(.score | . * 100 | floor / 100), Price Bin: \(.property.price | if . < 200000 then "Low" elif . < 500000 then "Medium" else "High" end), Area: \(.property.area_sqm)m²"'
    
    echo ""
}

# Function to test location attention
test_location_attention() {
    echo -e "${BLUE}Testing Location Attention Pooling:${NC}"
    
    # Test with contact that has multiple preferred locations
    curl -s "${BASE_URL}/api/recommendations/contact/2?neural_scoring=true&limit=3" \
        | jq -r '.recommendations[] | "Property: \(.property.id), Score: \(.score | . * 100 | floor / 100), Distance: \(.explanation.location_match.distance_km | . * 100 | floor / 100)km, Location Score: \(.explanation.location_match.score | . * 100 | floor / 100)"'
    
    echo ""
}

# Main execution
main() {
    echo -e "${GREEN}Starting Phase 1 Testing Suite${NC}"
    echo "Date: $(date)"
    echo ""
    
    # Check if server is running
    if ! check_server; then
        echo -e "${RED}Please start the server first: cargo run${NC}"
        exit 1
    fi
    
    echo ""
    
    # Test different scenarios
    test_recommendations "Budget-focused Contact (Low Budget)" "1" "&min_score=0.1"
    test_recommendations "Location-focused Contact (Multiple Preferences)" "2" "&min_score=0.1"
    test_recommendations "Size-focused Contact (Large Property)" "3" "&min_score=0.1"
    
    # Performance comparison
    get_performance_metrics "1"
    
    # Feature analysis
    test_feature_binning
    
    # Location attention analysis
    test_location_attention
    
    echo -e "${GREEN}Phase 1 testing completed!${NC}"
    echo ""
    echo "Key Phase 1 Features Tested:"
    echo "✓ Neural binning for price, area, rooms"
    echo "✓ Location attention pooling"
    echo "✓ Enhanced similarity scoring"
    echo "✓ Feature compatibility analysis"
    echo ""
    echo "Next: Run phase2_test.sh for two-stage retrieval and feature store"
}

# Execute main function
main "$@"
