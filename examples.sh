#!/bin/bash

BASE_URL="http://localhost:8080"

echo "🏠 Real Estate Recommendation System - API Examples"
echo "=================================================="

# Function to check if jq is installed
check_jq() {
    if ! command -v jq &> /dev/null; then
        echo "⚠️  jq is not installed. Installing jq for better JSON formatting..."
        if command -v apt &> /dev/null; then
            sudo apt install -y jq
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y jq
        elif command -v yum &> /dev/null; then
            sudo yum install -y jq
        elif command -v brew &> /dev/null; then
            brew install jq
        else
            echo "⚠️  Could not install jq. JSON output will not be formatted."
            return 1
        fi
    fi
    return 0
}

# Check if jq is available
check_jq
JQ_AVAILABLE=$?

# Load environment variables
if [ -f ".env" ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo ""
echo "🔍 Checking server health..."
HEALTH_RESPONSE=$(curl -s "$BASE_URL/health")

if [ $JQ_AVAILABLE -eq 0 ]; then
    echo "$HEALTH_RESPONSE" | jq '.'
else
    echo "$HEALTH_RESPONSE"
fi

if echo "$HEALTH_RESPONSE" | grep -q "healthy"; then
    echo "✅ Server is running"
else
    echo "❌ Server is not responding. Please start the server with 'cargo run --release' or './start.sh'"
    exit 1
fi

echo ""
echo "📋 Getting sample data from database..."

# Get property IDs from database
if [ ! -z "$DATABASE_URL" ]; then
    PROPERTY_IDS=($(psql "$DATABASE_URL" -t -c "SELECT id FROM properties LIMIT 3;" 2>/dev/null | tr -d ' '))
    CONTACT_IDS=($(psql "$DATABASE_URL" -t -c "SELECT id FROM contacts LIMIT 2;" 2>/dev/null | tr -d ' '))
    
    if [ ${#PROPERTY_IDS[@]} -gt 0 ]; then
        echo "✅ Found ${#PROPERTY_IDS[@]} sample properties"
        echo "✅ Found ${#CONTACT_IDS[@]} sample contacts"
    else
        echo "❌ No sample data found. Please run './setup.sh' to initialize the database."
        exit 1
    fi
else
    echo "❌ DATABASE_URL not set. Please run './setup.sh' first."
    exit 1
fi

echo ""
echo "🎯 Example 1: Get recommendations for a contact"
echo "================================================"
echo "GET $BASE_URL/recommendations/contact/${CONTACT_IDS[0]}?limit=3&min_score=0.3"
echo ""

RECOMMENDATIONS=$(curl -s "$BASE_URL/recommendations/contact/${CONTACT_IDS[0]}?limit=3&min_score=0.3")

if [ $JQ_AVAILABLE -eq 0 ]; then
    echo "$RECOMMENDATIONS" | jq '.'
else
    echo "$RECOMMENDATIONS"
fi

echo ""
echo "🎯 Example 2: Bulk recommendations for multiple contacts"
echo "========================================================"
echo "POST $BASE_URL/recommendations/bulk"
echo ""

BULK_REQUEST=$(cat << EOF
{
    "limit_per_contact": 3,
    "min_score": 0.3,
    "contact_ids": [${CONTACT_IDS[0]}, ${CONTACT_IDS[1]}]
}
EOF
)

BULK_RESPONSE=$(curl -s -X POST "$BASE_URL/recommendations/bulk" \
    -H "Content-Type: application/json" \
    -d "$BULK_REQUEST")

if [ $JQ_AVAILABLE -eq 0 ]; then
    echo "$BULK_RESPONSE" | jq '.'
else
    echo "$BULK_RESPONSE"
fi

echo ""
echo "🎯 Example 3: Compare two properties"
echo "===================================="
if [ ${#PROPERTY_IDS[@]} -ge 2 ]; then
    echo "GET $BASE_URL/comparisons/properties?property1_id=${PROPERTY_IDS[0]}&property2_id=${PROPERTY_IDS[1]}"
    echo ""
    
    COMPARISON=$(curl -s "$BASE_URL/comparisons/properties?property1_id=${PROPERTY_IDS[0]}&property2_id=${PROPERTY_IDS[1]}")
    
    if [ $JQ_AVAILABLE -eq 0 ]; then
        echo "$COMPARISON" | jq '.comparison_metrics'
    else
        echo "$COMPARISON"
    fi
else
    echo "⚠️  Need at least 2 properties for comparison"
fi

echo ""
echo "🎯 Example 4: Generate a quote PDF"
echo "=================================="
if [ ${#PROPERTY_IDS[@]} -gt 0 ] && [ ${#CONTACT_IDS[@]} -gt 0 ]; then
    echo "POST $BASE_URL/quotes/generate"
    echo ""
    
    QUOTE_REQUEST=$(cat << EOF
{
    "property_id": "${PROPERTY_IDS[0]}",
    "contact_id": "${CONTACT_IDS[0]}",
    "additional_costs": [
        {"description": "Legal Fees", "amount": 150000},
        {"description": "Property Inspection", "amount": 50000}
    ],
    "custom_message": "Thank you for your interest in this property"
}
EOF
)

    echo "Generating PDF quote..."
    curl -s -X POST "$BASE_URL/quotes/generate" \
        -H "Content-Type: application/json" \
        -d "$QUOTE_REQUEST" \
        --output quote.pdf

    if [ -f "quote.pdf" ]; then
        echo "✅ Quote saved as quote.pdf"
        ls -la quote.pdf
    else
        echo "❌ Failed to generate quote PDF"
    fi
else
    echo "⚠️  Need at least 1 property and 1 contact for quote generation"
fi

echo ""
echo "🎯 Example 5: Generate comparison quote PDF"
echo "==========================================="
if [ ${#PROPERTY_IDS[@]} -ge 2 ] && [ ${#CONTACT_IDS[@]} -gt 0 ]; then
    echo "POST $BASE_URL/quotes/comparison"
    echo ""
    
    COMPARISON_QUOTE_REQUEST=$(cat << EOF
{
    "property1_id": "${PROPERTY_IDS[0]}",
    "property2_id": "${PROPERTY_IDS[1]}",
    "contact_id": "${CONTACT_IDS[0]}",
    "custom_message": "Property comparison report"
}
EOF
)

    echo "Generating comparison PDF..."
    curl -s -X POST "$BASE_URL/quotes/comparison" \
        -H "Content-Type: application/json" \
        -d "$COMPARISON_QUOTE_REQUEST" \
        --output comparison.pdf

    if [ -f "comparison.pdf" ]; then
        echo "✅ Comparison saved as comparison.pdf"
        ls -la comparison.pdf
    else
        echo "❌ Failed to generate comparison PDF"
    fi
else
    echo "⚠️  Need at least 2 properties and 1 contact for comparison quote"
fi

echo ""
echo "🎯 Example 6: Generate recommendation report PDF"
echo "==============================================="
if [ ${#PROPERTY_IDS[@]} -gt 0 ]; then
    echo "GET $BASE_URL/quotes/recommendations?property_id=${PROPERTY_IDS[0]}"
    echo ""
    
    echo "Generating recommendations PDF..."
    curl -s "$BASE_URL/quotes/recommendations?property_id=${PROPERTY_IDS[0]}" \
        --output recommendations.pdf

    if [ -f "recommendations.pdf" ]; then
        echo "✅ Recommendations report saved as recommendations.pdf"
        ls -la recommendations.pdf
    else
        echo "❌ Failed to generate recommendations PDF"
    fi
else
    echo "⚠️  Need at least 1 property for recommendations report"
fi

echo ""
echo "📊 Sample data summary:"
echo "======================"
echo "Properties in database:"
if [ $JQ_AVAILABLE -eq 0 ]; then
    psql "$DATABASE_URL" -c "SELECT id, address, property_type, price as price_dinars FROM properties;" 2>/dev/null
else
    psql "$DATABASE_URL" -c "SELECT id, address, property_type FROM properties;" 2>/dev/null
fi

echo ""
echo "Contacts in database:"
psql "$DATABASE_URL" -c "SELECT id, name, min_budget, max_budget FROM contacts;" 2>/dev/null

echo ""
echo "🎉 All API examples completed!"
echo ""
echo "📝 Manual testing commands:"
echo "=========================="
echo "Health check:"
echo "  curl $BASE_URL/health"
echo ""
echo "Get recommendations for contact:"
echo "  curl \"$BASE_URL/recommendations/contact/${CONTACT_IDS[0]}?limit=3&min_score=0.3\""
echo ""
echo "Compare properties:"
if [ ${#PROPERTY_IDS[@]} -ge 2 ]; then
    echo "  curl \"$BASE_URL/comparisons/properties?property1_id=${PROPERTY_IDS[0]}&property2_id=${PROPERTY_IDS[1]}\""
fi
echo ""
echo "🚀 Server is ready for production use!"

# Function to format JSON output
format_json() {
    if [ $JQ_AVAILABLE -eq 0 ]; then
        jq '.'
    else
        cat
    fi
}

# Check if server is running
echo "🔍 Checking server health..."
health_response=$(curl -s "${BASE_URL}/health")
if [ $? -ne 0 ] || [ -z "$health_response" ]; then
    echo "❌ Server is not running or not accessible."
    echo ""
    echo "Please start the server first:"
    echo "  cargo run"
    echo ""
    echo "Then wait for the message: 'Server running at http://127.0.0.1:8080'"
    exit 1
fi

echo "$health_response" | format_json
echo "✅ Server is running"
echo ""

# Get sample property and contact IDs from the health endpoint or use defaults
echo "📋 Getting sample data..."
echo ""
echo "🎯 Example 1: Get recommendations for a property"
echo "================================================"
echo "GET ${BASE_URL}/recommendations/property/{property_id}?limit=3&min_score=0.3"
echo ""
echo "Note: Replace {property_id} with an actual property ID from your database"
echo ""

echo "Example curl command:"
cat << 'EOF'
curl -X GET "http://localhost:8080/recommendations/property/PROPERTY_ID_HERE?limit=3&min_score=0.3" \
  -H "Accept: application/json" | jq '.'
EOF

echo ""
echo ""
echo "🎯 Example 2: Bulk recommendations"
echo "=================================="
echo "POST ${BASE_URL}/recommendations/bulk"
echo ""

echo "Example curl command:"
cat << 'EOF'
curl -X POST "http://localhost:8080/recommendations/bulk" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{
    "limit_per_property": 5,
    "min_score": 0.4
  }' | jq '.'
EOF

echo ""
echo ""
echo "🎯 Example 3: Compare two properties"
echo "===================================="
echo "GET ${BASE_URL}/comparisons/properties"
echo ""

echo "Example curl command:"
cat << 'EOF'
curl -X GET "http://localhost:8080/comparisons/properties?property1_id=PROPERTY1_ID&property2_id=PROPERTY2_ID" \
  -H "Accept: application/json" | jq '.'
EOF

echo ""
echo ""
echo "🎯 Example 4: Generate a quote PDF"
echo "=================================="
echo "POST ${BASE_URL}/quotes/generate"
echo ""

echo "Example curl command:"
cat << 'EOF'
curl -X POST "http://localhost:8080/quotes/generate" \
  -H "Content-Type: application/json" \
  -d '{
    "property_id": "PROPERTY_ID_HERE",
    "contact_id": "CONTACT_ID_HERE",
    "additional_costs": [
      {"description": "Legal Fees", "amount": 150000},
      {"description": "Property Inspection", "amount": 50000}
    ],
    "custom_message": "Thank you for your interest in this property"
  }' \
  --output quote.pdf

echo "Quote saved as quote.pdf"
EOF

echo ""
echo ""
echo "🎯 Example 5: Generate comparison quote PDF"
echo "==========================================="
echo "POST ${BASE_URL}/quotes/comparison"
echo ""

echo "Example curl command:"
cat << 'EOF'
curl -X POST "http://localhost:8080/quotes/comparison" \
  -H "Content-Type: application/json" \
  -d '{
    "property1_id": "PROPERTY1_ID_HERE",
    "property2_id": "PROPERTY2_ID_HERE",
    "contact_id": "CONTACT_ID_HERE",
    "custom_message": "Property comparison report"
  }' \
  --output comparison.pdf

echo "Comparison saved as comparison.pdf"
EOF

echo ""
echo ""
echo "🎯 Example 6: Generate recommendation report PDF"
echo "==============================================="
echo "GET ${BASE_URL}/quotes/recommendations"
echo ""

echo "Example curl command:"
cat << 'EOF'
curl -X GET "http://localhost:8080/quotes/recommendations?property_id=PROPERTY_ID_HERE" \
  --output recommendations.pdf

echo "Recommendations report saved as recommendations.pdf"
EOF

echo ""
echo ""
echo "📝 To get actual IDs for testing:"
echo "================================="
echo "1. Connect to your PostgreSQL database:"
echo "   psql -U your_username -d real_estate_db"
echo ""
echo "2. Get property IDs:"
echo "   SELECT id, title FROM properties LIMIT 5;"
echo ""
echo "3. Get contact IDs:"
echo "   SELECT id, first_name, last_name FROM contacts LIMIT 5;"
echo ""
echo "4. Replace the placeholder IDs in the commands above with actual IDs"
echo ""
echo "📊 Sample data is automatically inserted during migration"
echo "You can use the sample property and contact IDs from the database"
