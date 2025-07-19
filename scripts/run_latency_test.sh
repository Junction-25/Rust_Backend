#!/bin/bash

# Latency Testing Script for Recommendation API with K Value Analysis
# This script runs comprehensive latency tests focusing on K values: 5, 10, 50, 100

set -e

echo "🚀 Starting comprehensive K value latency test for recommendation API..."
echo "🎯 Testing K values: 5, 10, 50, 100"

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

# Create output directory
mkdir -p analysis

# Run comprehensive K value latency tests
echo "⏱️  Running comprehensive K value latency tests..."
echo "🔗 Using database: $DATABASE_URL"
echo "📊 Test scenarios include:"
echo "   - Baseline (no K filtering)"
echo "   - K=5 (top 5 results)"
echo "   - K=10 (top 10 results)" 
echo "   - K=50 (top 50 results)"
echo "   - K=100 (top 100 results)"
echo "   - Combined K + percentile filtering"
echo ""

# Run with more iterations to get statistical significance for K value analysis
python3 analysis/latency_test.py \
    --iterations 2000 \
    --db-url "$DATABASE_URL" \
    --output analysis/k_value_latency_results.csv

# Generate comprehensive plots focusing on K value analysis
echo "📊 Generating K value performance plots..."
python3 analysis/plot_latency.py analysis/k_value_latency_results.csv --output-dir analysis/

echo "✅ K value latency testing complete!"
echo ""
echo "Generated files in analysis/ directory:"
echo "  📈 k_value_latency_results.csv: Raw test data with K value scenarios"
echo "  📊 k_value_analysis.png: Detailed K value performance comparison"
echo "  📈 response_time_analysis.png: Response time distribution plots"
echo "  📊 percentile_analysis.png: Percentile and size analysis"
echo "  📈 load_analysis.png: Performance over time analysis"
echo "  📊 filtering_analysis.png: Analysis of different filtering scenarios"
echo "  🔥 parameter_heatmap.png: Heatmap of parameter combinations"
echo "  📝 latency_test_report.txt: Comprehensive summary report"
echo ""
echo "🎯 Key insights available:"
echo "  • Performance comparison of K=5, 10, 50, 100"
echo "  • Optimal K value identification"
echo "  • Performance improvement percentages"
echo "  • Statistical significance analysis"
echo ""
echo "View the k_value_analysis.png for the most relevant insights!"