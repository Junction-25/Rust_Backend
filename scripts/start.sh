#!/bin/bash

echo "🚀 Real Estate Recommendation System - Server Startup"
echo "====================================================="

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
    echo "❌ No .env file found. Please run './setup.sh' first."
    exit 1
fi

# Quick dependency check
echo ""
echo "📦 Quick dependency check..."
if ! command -v psql &> /dev/null; then
    echo "❌ PostgreSQL not found. Please run './setup.sh' first."
    exit 1
fi

if ! command -v sqlx &> /dev/null; then
    echo "❌ SQLx CLI not found. Please run './setup.sh' first."
    exit 1
fi

# Test database connection
echo ""
echo "🗄️  Testing database connection..."
sqlx migrate info >/dev/null 2>&1
if [ $? -ne 0 ]; then
    echo "❌ Database connection failed. Please run './setup.sh' first."
    exit 1
fi

echo "✅ Database connection OK"

# Build the project
echo ""
echo "🔨 Building project in release mode..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"

# Start the server
echo ""
echo "🚀 Starting Real Estate Recommendation System Server..."
echo "====================================================="
echo ""
echo "📍 Server will be available at: http://$SERVER_HOST:$SERVER_PORT"
echo "🏥 Health check: curl http://$SERVER_HOST:$SERVER_PORT/health"
echo ""
echo "📱 Example API calls:"
echo "  • Health: curl http://$SERVER_HOST:$SERVER_PORT/health"
echo "  • Recommendations: curl \"http://$SERVER_HOST:$SERVER_PORT/recommendations/contact/{id}?limit=3\""
echo "  • Comparisons: curl \"http://$SERVER_HOST:$SERVER_PORT/comparisons/properties?property1_id={id1}&property2_id={id2}\""
echo ""
echo "📄 Run './examples.sh' for detailed API examples with real property IDs"
echo ""
echo "🛑 Press Ctrl+C to stop the server"
echo ""

# Run the server
cargo run --release
