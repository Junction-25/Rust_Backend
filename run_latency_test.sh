#!/bin/bash

echo "🧪 Real Estate API Latency Tester"
echo "================================="

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is not installed. Please install Python 3 first."
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "latency_test.py" ]; then
    echo "❌ Please run this script from the project root directory"
    exit 1
fi

# Install Python dependencies if requirements.txt exists
if [ -f "requirements.txt" ]; then
    echo "📦 Installing Python dependencies..."
    python3 -m pip install -r requirements.txt --user
    
    if [ $? -ne 0 ]; then
        echo "❌ Failed to install dependencies"
        exit 1
    fi
    echo "✅ Dependencies installed"
fi

# Load environment variables
if [ -f ".env" ]; then
    echo "🔧 Loading environment variables..."
    export $(cat .env | grep -v '^#' | xargs)
else
    echo "⚠️  No .env file found. Database connectivity may be limited."
fi

echo ""
echo "🚀 Starting latency tests..."

# Check if server is running
echo "🔍 Checking if server is running..."
SERVER_URL="${1:-http://localhost:8080}"
ITERATIONS="${2:-10}"

if curl -s "$SERVER_URL/health" > /dev/null; then
    echo "✅ Server is accessible at $SERVER_URL"
else
    echo "❌ Server is not accessible at $SERVER_URL"
    echo "Please start the server first with 'cargo run --release' or './start.sh'"
    echo ""
    echo "Usage: $0 [SERVER_URL] [ITERATIONS]"
    echo "  SERVER_URL: Base URL of the API (default: http://localhost:8080)"
    echo "  ITERATIONS: Number of test iterations per endpoint (default: 10)"
    exit 1
fi

echo ""
echo "⚙️  Test Configuration:"
echo "  Server URL: $SERVER_URL"
echo "  Iterations: $ITERATIONS"
echo "  Database URL: ${DATABASE_URL:-Not configured}"
echo ""

# Run the latency test
python3 latency_test.py --url "$SERVER_URL" --iterations "$ITERATIONS"

if [ $? -eq 0 ]; then
    echo ""
    echo "🎉 Latency testing completed successfully!"
    echo ""
    echo "📁 Check the generated CSV file in the analysis/ directory for detailed results"
    echo "📊 You can analyze the data using:"
    echo "   - Excel or Google Sheets"
    echo "   - Python pandas: pandas.read_csv('analysis/latency_test_results_*.csv')"
    echo "   - R: read.csv('analysis/latency_test_results_*.csv')"
    echo "   - Or run: python3 analysis/analyze_latency.py"
else
    echo "❌ Latency testing failed"
    exit 1
fi
