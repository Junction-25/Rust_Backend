#!/bin/bash
"""
Configuration script for batch recommendations with different scenarios
"""

# Set common environment variables
export API_BASE_URL="http://localhost:8080"
export DATABASE_URL="postgresql:///real_estate_db"

echo "🚀 Real Estate Batch Recommendations - Configuration Script"
echo "=========================================================="

# Check if server is running
echo "🔌 Checking if API server is running..."
if curl -s "$API_BASE_URL/health" > /dev/null 2>&1; then
    echo "✅ API server is running"
else
    echo "❌ API server is not running. Please start it first with:"
    echo "   cargo run --release"
    echo "   or"
    echo "   ./start.sh"
    exit 1
fi

# Function to run batch recommendations with specific settings
run_batch() {
    local scenario_name="$1"
    local batch_size="$2"
    local limit_per_property="$3"
    local min_score="$4"
    local top_k="$5"
    local additional_params="$6"
    
    echo ""
    echo "🔄 Running scenario: $scenario_name"
    echo "   Batch size: $batch_size"
    echo "   Limit per property: $limit_per_property"
    echo "   Min score: $min_score"
    if [ ! -z "$top_k" ]; then
        echo "   Top K: $top_k"
    fi
    echo "   Additional params: $additional_params"
    
    # Set environment variables
    export BATCH_SIZE="$batch_size"
    export LIMIT_PER_PROPERTY="$limit_per_property"
    export MIN_SCORE="$min_score"
    export OUTPUT_FILE="batch_recommendations_${scenario_name}_$(date +%Y%m%d_%H%M%S).json"
    
    if [ ! -z "$top_k" ]; then
        export TOP_K="$top_k"
    else
        unset TOP_K
    fi
    
    # Run the batch process
    python3 batch_recommendations.py
    
    echo "✅ Scenario '$scenario_name' completed!"
}

# Show menu
echo ""
echo "📋 Available scenarios:"
echo "1. Quick test (10 properties, 5 recommendations each)"
echo "2. Standard batch (50 properties per batch, 10 recommendations each)"
echo "3. High-quality only (min score 0.5, top 5 per property)"
echo "4. Top performers (top 10% of scores, unlimited recommendations)"
echo "5. Custom configuration"
echo "6. Full dataset processing"

read -p "🎯 Select scenario (1-6): " choice

case $choice in
    1)
        # Quick test scenario
        export TOP_K=10  # Only process first 10 properties
        run_batch "quick_test" 5 5 0.1 10
        ;;
    2)
        # Standard batch processing
        run_batch "standard" 50 10 0.1
        ;;
    3)
        # High-quality recommendations only
        run_batch "high_quality" 30 5 0.5
        ;;
    4)
        # Top performers with percentile filtering
        export TOP_PERCENTILE=0.1  # Top 10%
        run_batch "top_performers" 25 20 0.1
        ;;
    5)
        # Custom configuration
        echo ""
        echo "🔧 Custom Configuration:"
        read -p "Batch size (default 50): " custom_batch_size
        read -p "Limit per property (default 10): " custom_limit
        read -p "Min score (default 0.1): " custom_min_score
        read -p "Top K properties to process (optional): " custom_top_k
        
        custom_batch_size=${custom_batch_size:-50}
        custom_limit=${custom_limit:-10}
        custom_min_score=${custom_min_score:-0.1}
        
        if [ ! -z "$custom_top_k" ]; then
            run_batch "custom" "$custom_batch_size" "$custom_limit" "$custom_min_score" "$custom_top_k"
        else
            run_batch "custom" "$custom_batch_size" "$custom_limit" "$custom_min_score"
        fi
        ;;
    6)
        # Full dataset processing
        echo ""
        echo "⚠️  WARNING: This will process ALL properties in the database!"
        echo "This may take a significant amount of time and generate a large file."
        read -p "Are you sure you want to continue? (y/N): " confirm
        
        if [[ $confirm =~ ^[Yy]$ ]]; then
            run_batch "full_dataset" 100 15 0.05
        else
            echo "❌ Full dataset processing cancelled."
        fi
        ;;
    *)
        echo "❌ Invalid choice. Exiting."
        exit 1
        ;;
esac

echo ""
echo "🎉 Batch recommendation process completed!"
echo "📁 Check the generated JSON file for results."
echo ""
echo "💡 Tip: You can also run the script directly with environment variables:"
echo "   BATCH_SIZE=25 LIMIT_PER_PROPERTY=5 MIN_SCORE=0.3 python3 batch_recommendations.py"
