#!/bin/bash
# Single Batch Recommendation Script
# Processes all 1000 properties in one API call

echo "🚀 Starting single batch recommendation process..."
echo "This will process ALL properties in the database as one batch"
echo ""

# Set environment variables for single batch processing
export SINGLE_BATCH=true
export LIMIT_PER_PROPERTY=10
export MIN_SCORE=0.1
export API_BASE_URL=http://localhost:8080

# Optional: Set custom output file with timestamp
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
export OUTPUT_FILE="single_batch_recommendations_${TIMESTAMP}.json"

echo "Configuration:"
echo "  Single batch mode: $SINGLE_BATCH"
echo "  Recommendations per property: $LIMIT_PER_PROPERTY"
echo "  Minimum score: $MIN_SCORE"
echo "  Output file: $OUTPUT_FILE"
echo ""

# Run the Python script
python3 batch_recommendations.py

echo ""
echo "✅ Single batch process completed!"
echo "📁 Check the output file: $OUTPUT_FILE"
