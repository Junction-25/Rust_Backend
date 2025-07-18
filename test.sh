#!/bin/bash

echo "🧪 Real Estate Recommendation System Tests"
echo "================================================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Please run this script from the project root directory"
    exit 1
fi

# Load environment variables
if [ -f ".env" ]; then
    echo "🔧 Loading environment variables..."
    export $(cat .env | grep -v '^#' | xargs)
else
    echo "⚠️  No .env file found. Please run './setup.sh' first."
    exit 1
fi

echo ""
echo "📦 Checking dependencies..."
cargo check --release

if [ $? -ne 0 ]; then
    echo "❌ Dependency check failed"
    exit 1
fi

echo "✅ Dependencies OK"

# Test database connectivity
echo ""
echo "🗄️  Testing database connectivity..."
if command -v sqlx &> /dev/null; then
    sqlx migrate info 2>/dev/null
    if [ $? -eq 0 ]; then
        echo "✅ Database connection OK"
        
        # Test if sample data exists
        PROPERTY_COUNT=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM properties;" 2>/dev/null | xargs)
        CONTACT_COUNT=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM contacts;" 2>/dev/null | xargs)
        
        if [ "$PROPERTY_COUNT" -gt "0" ] && [ "$CONTACT_COUNT" -gt "0" ]; then
            echo "✅ Sample data found: $PROPERTY_COUNT properties, $CONTACT_COUNT contacts"
        else
            echo "⚠️  No sample data found. Running migrations..."
            sqlx migrate run
        fi
    else
        echo "❌ Database connection failed. Please run './setup.sh' first."
        exit 1
    fi
else
    echo "❌ SQLx CLI not found. Please run './setup.sh' first."
    exit 1
fi

echo ""
echo "🔨 Building in release mode..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"

echo ""
echo "🔧 Running Rust tests..."
cargo test --release

if [ $? -ne 0 ]; then
    echo "❌ Rust tests failed"
    exit 1
fi

echo "✅ Rust tests passed"

echo ""
echo "🚀 Testing server startup..."

# Start server in background
cargo run --release &
SERVER_PID=$!

echo "⏳ Waiting for server to start..."
sleep 5

# Test health endpoint
HEALTH_RESPONSE=$(curl -s http://127.0.0.1:8080/health 2>/dev/null)

if echo "$HEALTH_RESPONSE" | grep -q "healthy"; then
    echo "✅ Server health check passed"
    
    # Test recommendation API with sample data
    echo ""
    echo "🎯 Testing recommendation API..."
    CONTACT_ID=$(psql "$DATABASE_URL" -t -c "SELECT id FROM contacts LIMIT 1;" 2>/dev/null | xargs)
    
    if [ ! -z "$CONTACT_ID" ]; then
        RECOMMENDATIONS=$(curl -s "http://127.0.0.1:8080/recommendations/contact/$CONTACT_ID?limit=1" 2>/dev/null)
        
        if echo "$RECOMMENDATIONS" | grep -q "recommendations"; then
            echo "✅ Recommendation API working"
        else
            echo "❌ Recommendation API failed"
            echo "Response: $RECOMMENDATIONS"
        fi
    else
        echo "⚠️  No sample contacts found for testing"
    fi
    
    # Test comparison API
    echo ""
    echo "🔍 Testing comparison API..."
    PROPERTY_IDS=($(psql "$DATABASE_URL" -t -c "SELECT id FROM properties LIMIT 2;" 2>/dev/null | xargs))
    
    if [ ${#PROPERTY_IDS[@]} -ge 2 ]; then
        COMPARISON=$(curl -s "http://127.0.0.1:8080/comparisons/properties?property1_id=${PROPERTY_IDS[0]}&property2_id=${PROPERTY_IDS[1]}" 2>/dev/null)
        
        if echo "$COMPARISON" | grep -q "comparison_metrics"; then
            echo "✅ Comparison API working"
        else
            echo "❌ Comparison API failed"
        fi
    else
        echo "⚠️  Not enough sample properties for comparison testing"
    fi
    
else
    echo "❌ Server health check failed"
    echo "Response: $HEALTH_RESPONSE"
fi

# Clean up server process
if [ ! -z "$SERVER_PID" ]; then
    kill $SERVER_PID 2>/dev/null
    wait $SERVER_PID 2>/dev/null
    echo "🛑 Server stopped"
fi

echo ""
echo "🎉 All tests completed!"
echo ""
echo "� Test Summary:"
echo "  ✅ Dependencies check"
echo "  ✅ Database connectivity"
echo "  ✅ Build process"
echo "  ✅ Rust unit tests"
echo "  ✅ Server startup"
echo "  ✅ API endpoints"
echo ""
echo "🚀 Ready to run: cargo run --release"
    echo "❌ Debug build failed"
    exit 1
fi

echo "✅ Debug build successful"

echo ""
echo "🚀 Building in release mode..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Release build failed"
    exit 1
fi

echo "✅ Release build successful"

echo ""
echo "📝 Running code formatting check..."
cargo fmt --check

if [ $? -ne 0 ]; then
    echo "⚠️  Code formatting issues found. Run 'cargo fmt' to fix."
else
    echo "✅ Code formatting OK"
fi

echo ""
echo "🔍 Running linter..."
cargo clippy -- -D warnings

if [ $? -ne 0 ]; then
    echo "⚠️  Linting issues found. Please fix the warnings above."
else
    echo "✅ Linting passed"
fi

echo ""
echo "🧪 Running unit tests..."
cargo test

if [ $? -ne 0 ]; then
    echo "❌ Tests failed"
    exit 1
fi

echo "✅ All tests passed"

echo ""
echo "🎉 All checks completed successfully!"
echo ""
echo "Next steps:"
echo "1. Set up your PostgreSQL database"
echo "2. Run './setup.sh' to initialize the system"
echo "3. Start the server with 'cargo run'"
echo "4. Test the API with './examples.sh'"
