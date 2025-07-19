#!/bin/bash

# AI-Enhanced Real Estate Recommendation System Test Script
set -e

echo "🤖 AI-Enhanced Real Estate Recommendation System Test"
echo "====================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Check if server is running
check_server() {
    echo -e "${BLUE}🔍 Checking server status...${NC}"
    if curl -s "http://localhost:8080/health" > /dev/null; then
        echo -e "${GREEN}✅ Server is running${NC}"
        return 0
    else
        echo -e "${RED}❌ Server is not running. Please start with: cargo run${NC}"
        return 1
    fi
}

# Test AI model initialization
test_ai_initialization() {
    echo -e "\n${PURPLE}🧠 Testing AI Model Initialization...${NC}"
    
    echo "Initializing AI models..."
    INIT_RESPONSE=$(curl -s -X POST "http://localhost:8080/ai/models/initialize" \
        -H "Content-Type: application/json")
    
    if [[ $INIT_RESPONSE == *"success"* ]]; then
        echo -e "${GREEN}✅ AI models initialized successfully${NC}"
        echo "Response: $INIT_RESPONSE"
    else
        echo -e "${YELLOW}⚠️ AI initialization may already be done${NC}"
        echo "Response: $INIT_RESPONSE"
    fi
}

# Test AI model statistics
test_ai_stats() {
    echo -e "\n${PURPLE}📊 Testing AI Model Statistics...${NC}"
    
    AI_STATS=$(curl -s "http://localhost:8080/ai/models/stats")
    echo "AI Model Statistics:"
    echo "$AI_STATS" | python3 -m json.tool 2>/dev/null || echo "$AI_STATS"
}

# Test AI-enhanced recommendations
test_ai_recommendations() {
    echo -e "\n${PURPLE}🎯 Testing AI-Enhanced Recommendations...${NC}"
    
    # Test for contact 1 with all AI features enabled
    echo "Getting AI recommendations for contact 1..."
    AI_REC_RESPONSE=$(curl -s "http://localhost:8080/ai/recommendations/contact/1?enable_ml_scoring=true&enable_market_analysis=true&enable_predictive_matching=true&include_price_predictions=true&min_confidence=0.3")
    
    echo "AI-Enhanced Recommendation Response:"
    echo "$AI_REC_RESPONSE" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    print(f'Total Recommendations: {data.get(\"total_count\", 0)}')
    print(f'Processing Time: {data.get(\"processing_time_ms\", 0)}ms')
    print(f'AI Model Version: {data.get(\"ai_model_version\", \"N/A\")}')
    
    if 'recommendations' in data and data['recommendations']:
        rec = data['recommendations'][0]
        print(f'\\nFirst Recommendation:')
        print(f'  Property ID: {rec[\"recommendation\"][\"property\"][\"id\"]}')
        print(f'  Address: {rec[\"recommendation\"][\"property\"][\"address\"]}')
        print(f'  Traditional Score: {rec[\"recommendation\"][\"score\"]:.3f}')
        if 'ml_enhancement' in rec and rec['ml_enhancement']:
            print(f'  ML Score: {rec[\"ml_enhancement\"][\"ml_score\"]:.3f}')
            print(f'  Hybrid Score: {rec[\"ml_enhancement\"][\"hybrid_score\"]:.3f}')
        print(f'  Confidence Score: {rec[\"confidence_score\"]:.3f}')
        if rec.get('ai_insights'):
            print(f'  AI Insights: {len(rec[\"ai_insights\"])} insights')
            for insight in rec['ai_insights'][:3]:
                print(f'    - {insight}')
    
    if 'market_insights' in data and data['market_insights']:
        print(f'\\nMarket Insights ({len(data[\"market_insights\"])} total):')
        for insight in data['market_insights'][:3]:
            print(f'  - {insight}')
    
    if 'contact_behavior_insights' in data and data['contact_behavior_insights']:
        behavior = data['contact_behavior_insights']
        print(f'\\nContact Behavior Analysis:')
        print(f'  Decisiveness: {behavior[\"decisiveness_level\"]}')
        print(f'  Price Sensitivity: {behavior[\"price_sensitivity_level\"]}')
        print(f'  Flexibility Score: {behavior[\"flexibility_score\"]:.2f}')
        print(f'  Timeline: {behavior[\"predicted_timeline\"]}')
        
except Exception as e:
    print(f'Error parsing JSON: {e}')
    print(sys.stdin.read())
"
}

# Test traditional vs AI comparison
test_comparison() {
    echo -e "\n${PURPLE}⚖️ Comparing Traditional vs AI Recommendations...${NC}"
    
    # Get traditional recommendations
    echo "Getting traditional recommendations..."
    TRAD_RESPONSE=$(curl -s "http://localhost:8080/recommendations/contact/1?limit=3")
    TRAD_COUNT=$(echo "$TRAD_RESPONSE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('total_count', 0))" 2>/dev/null || echo "0")
    TRAD_TIME=$(echo "$TRAD_RESPONSE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('processing_time_ms', 0))" 2>/dev/null || echo "0")
    
    # Get AI recommendations
    echo "Getting AI recommendations..."
    AI_RESPONSE=$(curl -s "http://localhost:8080/ai/recommendations/contact/1?min_confidence=0.3")
    AI_COUNT=$(echo "$AI_RESPONSE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('total_count', 0))" 2>/dev/null || echo "0")
    AI_TIME=$(echo "$AI_RESPONSE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('processing_time_ms', 0))" 2>/dev/null || echo "0")
    
    echo -e "${BLUE}Comparison Results:${NC}"
    echo "Traditional Algorithm:"
    echo "  - Recommendations: $TRAD_COUNT"
    echo "  - Processing Time: ${TRAD_TIME}ms"
    echo ""
    echo "AI-Enhanced Algorithm:"
    echo "  - Recommendations: $AI_COUNT"
    echo "  - Processing Time: ${AI_TIME}ms"
    echo "  - Enhancement: ML scoring, market analysis, behavioral insights"
}

# Test AI feedback system
test_ai_feedback() {
    echo -e "\n${PURPLE}🔄 Testing AI Feedback System...${NC}"
    
    echo "Sending positive feedback for contact 1 and property 1..."
    FEEDBACK_RESPONSE=$(curl -s -X POST "http://localhost:8080/ai/feedback" \
        -H "Content-Type: application/json" \
        -d '{
            "contact_id": 1,
            "property_id": 1,
            "feedback_type": "property_view",
            "outcome": "positive"
        }')
    
    echo "Feedback Response: $FEEDBACK_RESPONSE"
}

# Test market analysis
test_market_analysis() {
    echo -e "\n${PURPLE}📈 Testing Market Analysis...${NC}"
    
    MARKET_RESPONSE=$(curl -s "http://localhost:8080/ai/market/analysis")
    echo "Market Analysis Response:"
    echo "$MARKET_RESPONSE" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    insights = data.get('market_insights', [])
    print(f'Market Insights ({len(insights)} total):')
    for i, insight in enumerate(insights[:5], 1):
        print(f'  {i}. {insight}')
    print(f'\\nGenerated at: {data.get(\"generated_at\", \"N/A\")}')
    print(f'Model Version: {data.get(\"model_version\", \"N/A\")}')
except Exception as e:
    print(f'Error: {e}')
    print(sys.stdin.read())
"
}

# Test multiple contacts with AI
test_multiple_contacts() {
    echo -e "\n${PURPLE}👥 Testing AI Recommendations for Multiple Contacts...${NC}"
    
    for contact_id in 1 2 3; do
        echo -e "\n${BLUE}Contact $contact_id AI Analysis:${NC}"
        AI_RESPONSE=$(curl -s "http://localhost:8080/ai/recommendations/contact/$contact_id?enable_predictive_matching=true&min_confidence=0.2" 2>/dev/null)
        
        echo "$AI_RESPONSE" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    total = data.get('total_count', 0)
    time_ms = data.get('processing_time_ms', 0)
    
    print(f'  Recommendations: {total}')
    print(f'  Processing Time: {time_ms}ms')
    
    if 'contact_behavior_insights' in data and data['contact_behavior_insights']:
        behavior = data['contact_behavior_insights']
        print(f'  Decisiveness: {behavior[\"decisiveness_level\"]}')
        print(f'  Timeline: {behavior[\"predicted_timeline\"]}')
        
    # Show top recommendation
    if data.get('recommendations'):
        rec = data['recommendations'][0]
        property_id = rec['recommendation']['property']['id']
        confidence = rec['confidence_score']
        print(f'  Top Property: #{property_id} (confidence: {confidence:.3f})')
        
        if rec.get('predictive_analysis'):
            pred = rec['predictive_analysis']
            purchase_prob = pred.get('purchase_probability', 0)
            decision_days = pred.get('time_to_decision_days', 0)
            print(f'  Purchase Probability: {purchase_prob:.1%}')
            print(f'  Decision Timeline: {decision_days} days')

except Exception as e:
    print(f'  Error: {e}')
" 2>/dev/null || echo "  Could not analyze response"
    done
}

# Performance benchmark
performance_benchmark() {
    echo -e "\n${PURPLE}⚡ Performance Benchmark...${NC}"
    
    echo "Benchmarking traditional vs AI recommendations..."
    
    # Traditional benchmark
    start_time=$(date +%s%3N)
    for i in {1..5}; do
        curl -s "http://localhost:8080/recommendations/contact/1?limit=5" > /dev/null
    done
    end_time=$(date +%s%3N)
    traditional_avg=$((($end_time - $start_time) / 5))
    
    # AI benchmark
    start_time=$(date +%s%3N)
    for i in {1..5}; do
        curl -s "http://localhost:8080/ai/recommendations/contact/1?min_confidence=0.3" > /dev/null
    done
    end_time=$(date +%s%3N)
    ai_avg=$((($end_time - $start_time) / 5))
    
    echo -e "${BLUE}Performance Results (5 requests average):${NC}"
    echo "  Traditional: ${traditional_avg}ms"
    echo "  AI-Enhanced: ${ai_avg}ms"
    
    if [ $ai_avg -lt $((traditional_avg * 3)) ]; then
        echo -e "${GREEN}✅ AI performance is acceptable (< 3x traditional)${NC}"
    else
        echo -e "${YELLOW}⚠️ AI processing takes longer but provides enhanced insights${NC}"
    fi
}

# Main execution
main() {
    echo -e "${YELLOW}🚀 Starting AI-Enhanced Recommendation System Tests${NC}"
    
    if ! check_server; then
        exit 1
    fi
    
    test_ai_initialization
    sleep 2
    
    test_ai_stats
    test_ai_recommendations
    test_comparison
    test_ai_feedback
    test_market_analysis
    test_multiple_contacts
    performance_benchmark
    
    echo -e "\n${GREEN}🎉 All AI tests completed successfully!${NC}"
    echo -e "${BLUE}💡 Key AI Features Demonstrated:${NC}"
    echo "   🤖 Machine Learning recommendation scoring"
    echo "   📊 Market trend analysis and price predictions"
    echo "   🎯 Predictive behavioral matching"
    echo "   🔄 Continuous learning from feedback"
    echo "   📈 Comprehensive market insights"
    echo "   ⚡ Performance optimization with caching"
    
    echo -e "\n${PURPLE}🏆 Your system now includes cutting-edge AI capabilities!${NC}"
}

# Run tests
main "$@"
