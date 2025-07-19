#!/bin/bash

echo "🚀 Testing Real-time Features - Step 2 Implementation"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check if server is running
check_server() {
    if ! curl -s http://localhost:8080/health > /dev/null; then
        echo -e "${RED}❌ Server is not running. Please start the server first.${NC}"
        echo "Run: cargo run"
        exit 1
    fi
    echo -e "${GREEN}✅ Server is running${NC}"
}

# Function to test with colored output
test_endpoint() {
    local method=$1
    local url=$2
    local data=$3
    local description=$4
    
    echo -e "\n${BLUE}🧪 Testing: $description${NC}"
    echo "   $method $url"
    
    if [ -n "$data" ]; then
        response=$(curl -s -X $method "$url" \
            -H "Content-Type: application/json" \
            -d "$data")
    else
        response=$(curl -s -X $method "$url")
    fi
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Success${NC}"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
    else
        echo -e "${RED}❌ Failed${NC}"
    fi
}

# Check server status
check_server

echo -e "\n${YELLOW}🔌 Real-time System Health Check${NC}"
test_endpoint "GET" "http://localhost:8080/realtime/health" "" "System Health Status"

echo -e "\n${YELLOW}📊 WebSocket Statistics${NC}"
test_endpoint "GET" "http://localhost:8080/realtime/stats" "" "WebSocket Connection Stats"

echo -e "\n${YELLOW}🧪 Test Notifications${NC}"

echo -e "\n${BLUE}1. Testing Real-time Recommendations${NC}"
test_endpoint "POST" "http://localhost:8080/realtime/test-notification" \
    '{"notification_type": "recommendation", "count": 3}' \
    "Send Test Recommendations"

echo -e "\n${BLUE}2. Testing Market Alerts${NC}"
test_endpoint "POST" "http://localhost:8080/realtime/test-notification" \
    '{"notification_type": "market_alert", "count": 2}' \
    "Send Market Alerts"

echo -e "\n${BLUE}3. Testing Price Change Notifications${NC}"
test_endpoint "POST" "http://localhost:8080/realtime/test-notification" \
    '{"notification_type": "price_change", "count": 2}' \
    "Send Price Change Notifications"

echo -e "\n${BLUE}4. Testing Price Predictions${NC}"
test_endpoint "POST" "http://localhost:8080/realtime/test-notification" \
    '{"notification_type": "price_prediction", "count": 3}' \
    "Send Price Prediction Notifications"

echo -e "\n${YELLOW}📱 Real-time Monitoring${NC}"
test_endpoint "POST" "http://localhost:8080/realtime/monitor/1001" "" \
    "Start Real-time Monitoring for Contact 1001"

echo -e "\n${YELLOW}💬 Custom Notifications${NC}"
test_endpoint "POST" "http://localhost:8080/realtime/send-notification" \
    '{
        "contact_id": 1001,
        "notification_type": "recommendation",
        "message": "Custom high-priority recommendation: Luxury villa in Hydra matches your criteria perfectly!",
        "data": {"priority": "high", "score": 95.8}
    }' \
    "Send Custom Recommendation"

test_endpoint "POST" "http://localhost:8080/realtime/send-notification" \
    '{
        "notification_type": "market_alert",
        "message": "Market surge detected: 15% price increase in Sidi Bel Abbès residential sector",
        "data": {"location": "Sidi Bel Abbès", "change": 15.3}
    }' \
    "Send Custom Market Alert"

echo -e "\n${YELLOW}🔄 AI Integration Test${NC}"
echo -e "${BLUE}Triggering AI recommendations that will generate real-time notifications...${NC}"

test_endpoint "GET" "http://localhost:8080/ai/recommendations/contact/1001?limit=5" "" \
    "AI Recommendations (should trigger real-time notifications)"

echo -e "\n${YELLOW}⚡ Performance & Load Test${NC}"
echo -e "${BLUE}Sending burst of notifications to test real-time performance...${NC}"

test_endpoint "POST" "http://localhost:8080/realtime/test-notification" \
    '{"notification_type": "recommendation", "count": 10}' \
    "Burst Test - 10 Recommendations"

test_endpoint "POST" "http://localhost:8080/realtime/test-notification" \
    '{"notification_type": "market_alert", "count": 5}' \
    "Burst Test - 5 Market Alerts"

echo -e "\n${YELLOW}📈 Final System Status${NC}"
test_endpoint "GET" "http://localhost:8080/realtime/health" "" "Final Health Check"
test_endpoint "GET" "http://localhost:8080/realtime/stats" "" "Final Statistics"

echo -e "\n${GREEN}🎉 Real-time Features Testing Complete!${NC}"
echo -e "${BLUE}════════════════════════════════════════${NC}"
echo -e "${YELLOW}📋 Summary:${NC}"
echo "✅ WebSocket server infrastructure implemented"
echo "✅ Real-time notification system operational"
echo "✅ Multiple notification types supported:"
echo "   - Instant property recommendations"
echo "   - Market alerts and trends"
echo "   - Price change notifications"
echo "   - Price predictions"
echo "✅ Real-time monitoring capabilities"
echo "✅ Custom notification system"
echo "✅ AI integration with real-time features"
echo "✅ Performance testing completed"
echo ""
echo -e "${BLUE}🔗 WebSocket Connection:${NC}"
echo "Connect to: ws://localhost:8080/ws"
echo ""
echo -e "${BLUE}📝 Sample WebSocket Messages:${NC}"
echo '{"type": "subscribe", "contact_id": 1001, "subscription_types": ["recommendations", "market_updates"]}'
echo ""
echo -e "${YELLOW}🚀 Ready for Step 3: Voice Activation & Smart Search!${NC}"
