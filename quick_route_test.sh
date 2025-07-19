#!/bin/bash

# Quick Route Verification Script
# Fast validation of all major API endpoints and system health

set -e

BASE_URL="http://localhost:8080"
SERVER_PID=""
TOTAL_TESTS=0
PASSED_TESTS=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║              🔧 MY-RECOMMENDER QUICK VALIDATION              ║${NC}"
echo -e "${CYAN}║                   Fast Route Verification                    ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Test function
test_endpoint() {
    local method="$1"
    local url="$2"
    local data="$3"
    local description="$4"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    local start_time=$(date +%s.%N)
    
    if [ "$method" = "POST" ] && [ -n "$data" ]; then
        local response=$(curl -s -w "%{http_code}" -X POST \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$url" 2>/dev/null)
    else
        local response=$(curl -s -w "%{http_code}" "$url" 2>/dev/null)
    fi
    
    local end_time=$(date +%s.%N)
    local duration=$(echo "$end_time - $start_time" | bc -l 2>/dev/null || echo "0")
    local duration_ms=$(echo "$duration * 1000" | bc -l 2>/dev/null | cut -d. -f1)
    
    local http_code=$(echo "$response" | tail -c 4)
    
    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        echo -e "${GREEN}✅ $description - ${duration_ms}ms (HTTP $http_code)${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}❌ $description - ${duration_ms}ms (HTTP $http_code)${NC}"
        return 1
    fi
}

# Start server
echo -e "${YELLOW}🏗️ Building and starting server...${NC}"
cargo build --release --quiet

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

./target/release/my-recommender > /dev/null 2>&1 &
SERVER_PID=$!

echo -e "${BLUE}Server PID: $SERVER_PID${NC}"
echo -e "${YELLOW}⏳ Waiting for server startup (5s)...${NC}"
sleep 5

echo ""
echo -e "${PURPLE}🧪 RUNNING QUICK VALIDATION TESTS${NC}"
echo -e "${PURPLE}===================================${NC}"

# Core System Tests
echo -e "\n${BLUE}🏥 Core System Health${NC}"
test_endpoint "GET" "$BASE_URL/health" "" "System Health Check"

# Basic Recommendations
echo -e "\n${BLUE}🎯 Basic Recommendations${NC}"
test_endpoint "GET" "$BASE_URL/recommendations/property/1?limit=3" "" "Property Recommendations"
test_endpoint "GET" "$BASE_URL/recommendations/contact/1?limit=3" "" "Contact Recommendations"

# Advanced Features
echo -e "\n${BLUE}⚡ Advanced Features${NC}"
test_endpoint "GET" "$BASE_URL/advanced/stats" "" "Advanced Service Stats"
test_endpoint "GET" "$BASE_URL/advanced/health" "" "Advanced Health Check"
test_endpoint "GET" "$BASE_URL/advanced/recommendations/fast?user_id=user1&limit=3" "" "Fast Advanced Recommendations"

# AI & ML
echo -e "\n${BLUE}🧠 AI & Machine Learning${NC}"
test_endpoint "GET" "$BASE_URL/ai/recommendations?user_id=user1&limit=3" "" "AI Recommendations"
test_endpoint "GET" "$BASE_URL/ai/stats" "" "AI Model Stats"

# Real-time Features  
echo -e "\n${BLUE}📡 Real-time Features${NC}"
test_endpoint "GET" "$BASE_URL/realtime/stats" "" "WebSocket Stats"
test_endpoint "GET" "$BASE_URL/realtime/health" "" "Real-time Health Check"

# Property Comparisons
echo -e "\n${BLUE}📊 Property Comparisons${NC}"
local comparison_data='{
    "property_ids": [1, 2, 3],
    "comparison_criteria": ["price", "location"],
    "include_market_data": true
}'
test_endpoint "POST" "$BASE_URL/comparisons/compare" "$comparison_data" "Property Comparison"

# PDF Generation (if available)
echo -e "\n${BLUE}📄 Document Generation${NC}"
local pdf_data='{
    "property_id": 1,
    "contact_id": 1,
    "format": "professional"
}'
test_endpoint "POST" "$BASE_URL/quotes/generate" "$pdf_data" "PDF Quote Generation"

# Performance Stress Test (Quick)
echo -e "\n${BLUE}💪 Quick Performance Test${NC}"
echo -e "${YELLOW}Running 5 concurrent requests...${NC}"

local pids=()
for i in {1..5}; do
    (curl -s "$BASE_URL/recommendations/property/1?limit=3" > /dev/null) &
    pids+=($!)
done

# Wait for all concurrent requests
for pid in "${pids[@]}"; do
    wait "$pid"
done

test_endpoint "GET" "$BASE_URL/health" "" "System Recovery After Load"

# Calculate success rate
echo ""
echo -e "${CYAN}═══════════════════════════════════════${NC}"
echo -e "${CYAN}         QUICK VALIDATION RESULTS        ${NC}"  
echo -e "${CYAN}═══════════════════════════════════════${NC}"

local success_rate=0
if [ $TOTAL_TESTS -gt 0 ]; then
    success_rate=$(echo "scale=1; $PASSED_TESTS * 100 / $TOTAL_TESTS" | bc -l)
fi

echo -e "${BLUE}📊 Test Results:${NC}"
echo -e "   Total Tests: $TOTAL_TESTS"
echo -e "   Passed: $PASSED_TESTS"
echo -e "   Failed: $((TOTAL_TESTS - PASSED_TESTS))"
echo -e "   Success Rate: ${success_rate}%"

if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED! System is ready for use.${NC}"
else
    echo -e "${YELLOW}⚠️  Some tests failed. Check the output above for details.${NC}"
fi

# Cleanup
echo -e "\n${YELLOW}🛑 Stopping server...${NC}"
kill $SERVER_PID 2>/dev/null || true
sleep 2

echo -e "${GREEN}✅ Quick validation completed${NC}"
echo ""
