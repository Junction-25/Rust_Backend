#!/bin/bash

# Latency Testing Script for Recommendation API
# This script runs the latency test and generates performance plots

set -e

echo "🚀 Starting latency test for recommendation API..."

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is required but not installed."
    exit 1
fi

# Check if pip is available
if ! command -v pip3 &> /dev/null; then
    echo "❌ pip3 is required but not installed."
    exit 1
fi

# Install Python requirements
echo "📦 Installing Python dependencies..."
pip3 install -r requirements.txt

# Check if the server is running
echo "🔍 Checking if the recommendation server is running..."
if ! curl -s http://localhost:8080/health > /dev/null; then
    echo "❌ Server is not running at http://localhost:8080"
    echo "   Please start the Rust backend server first:"
    echo "   cargo run --release"
    exit 1
fi

echo "✅ Server is running"

# Set database URL if not already set
if [ -z "$DATABASE_URL" ]; then
    echo "⚠️  DATABASE_URL not set. Using default..."
    export DATABASE_URL="postgresql://username:password@localhost/real_estate_db"
    echo "   You can set it with: export DATABASE_URL=\"your_db_url\""
fi

# Run latency tests
echo "⏱️  Running latency tests..."
echo "🔗 Using database: $DATABASE_URL"

# Auto-fetch property IDs from database (recommended)
python3 analysis/latency_test.py --iterations 200 --db-url "$DATABASE_URL"

# Alternative: Manually specify property IDs
# python3 analysis/latency_test.py --iterations 200 --property-ids 1 2 3 4 5 --output latency_results.csv

# Alternative: Use range of IDs
# ids=$(seq 6201 6210 | tr '\n' ' ')
# python3 analysis/latency_test.py --iterations 200 --property-ids $ids --output latency_results.csv

# Generate plots
echo "📊 Generating performance plots..."
python3 analysis/plot_latency.py latency_results.csv

echo "✅ Latency testing complete!"
echo ""
echo "Generated files:"
echo "  - latency_results.csv: Raw test data"
echo "  - response_time_analysis.png: Response time distribution plots"
echo "  - percentile_analysis.png: Percentile and size analysis"
echo "  - load_analysis.png: Performance over time analysis"
echo "  - latency_test_report.txt: Summary report"
echo ""
echo "You can now analyze the performance data and plots."