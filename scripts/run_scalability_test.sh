#!/bin/bash

# Scalability Testing Script for Recommendation API
# This script runs comprehensive scalability tests and generates analysis plots

set -e

echo "🚀 Starting scalability test for recommendation API..."

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

# Install Python requirements (including psycopg2 for database operations)
echo "📦 Installing Python dependencies..."
pip3 install -r requirements.txt
pip3 install psycopg2-binary  # For database operations

# Check if the server is running
echo "🔍 Checking if the recommendation server is running..."
if ! curl -s http://localhost:8080/health > /dev/null; then
    echo "❌ Server is not running at http://localhost:8080"
    echo "   Please start the Rust backend server first:"
    echo "   cargo run --release"
    exit 1
fi

echo "✅ Server is running"

# Default database URL (can be overridden with environment variable)
DB_URL=${DATABASE_URL:-"postgresql://username:password@localhost/real_estate_db"}

echo "🔍 Testing database connection..."
python3 -c "
import psycopg2
try:
    conn = psycopg2.connect('$DB_URL')
    conn.close()
    print('✅ Database connection successful')
except Exception as e:
    print(f'❌ Database connection failed: {e}')
    print('Please check your DATABASE_URL environment variable')
    exit(1)
"

if [ $? -ne 0 ]; then
    exit 1
fi

# Create results directory
mkdir -p scalability_results

# Run scalability tests
echo "⏱️  Running scalability tests..."
echo "   This will test performance using real contact and property data"
echo "   Testing both single and bulk recommendation endpoints"
echo "   Expected duration: 10-20 minutes depending on dataset size"

python3 analysis/scalability_test.py \
    --db-url "$DB_URL" \
    --data-dir data \
    --max-contacts 500 \
    --max-properties 1000 \
    --contact-batch-size 50 \
    --property-batch-size 100 \
    --tests-per-step 5 \
    --include-bulk \
    --bulk-sizes 2 5 10 20 \
    --output scalability_results/scalability_results.csv

# Generate plots
echo "📊 Generating scalability analysis plots..."
python3 analysis/plot_scalability.py \
    scalability_results/scalability_results.csv \
    --output-dir scalability_results

echo "✅ Scalability testing complete!"
echo ""
echo "Generated files in scalability_results/:"
echo "  - scalability_results.csv: Raw test data"
echo "  - scalability_response_time.png: Response time vs dataset size"
echo "  - scalability_trends.png: Performance trends and variability"
echo "  - scalability_heatmap.png: Performance heatmap"
echo "  - scalability_throughput.png: Throughput analysis"
echo "  - scalability_report.txt: Comprehensive analysis report"
echo ""
echo "📈 Key insights:"
echo "  1. Check how response time grows with dataset size"
echo "  2. Identify performance bottlenecks"
echo "  3. Understand scalability characteristics"
echo "  4. Plan for production capacity"
