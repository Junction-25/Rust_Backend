# Batch Recommendation Configuration Examples

## Single Batch Mode (All 1000 properties at once)
```bash
export SINGLE_BATCH=true
export LIMIT_PER_PROPERTY=10
export MIN_SCORE=0.1
python3 batch_recommendations.py
```

## Custom Scoring Weights (Budget-focused)
```bash
export SINGLE_BATCH=true
export BUDGET_WEIGHT=0.5
export LOCATION_WEIGHT=0.2
export PROPERTY_TYPE_WEIGHT=0.2
export SIZE_WEIGHT=0.1
python3 batch_recommendations.py
```

## High-quality recommendations only
```bash
export SINGLE_BATCH=true
export MIN_SCORE=0.5
export TOP_K=5
python3 batch_recommendations.py
```

## Top 10% recommendations
```bash
export SINGLE_BATCH=true
export TOP_PERCENTILE=0.1
export LIMIT_PER_PROPERTY=20
python3 batch_recommendations.py
```

## Multi-batch mode (if needed for memory constraints)
```bash
export SINGLE_BATCH=false
export BATCH_SIZE=100
export LIMIT_PER_PROPERTY=10
python3 batch_recommendations.py
```

## Environment Variables Reference

| Variable | Default | Description |
|----------|---------|-------------|
| `SINGLE_BATCH` | `true` | Process all properties in one batch |
| `BATCH_SIZE` | `50` | Size of each batch (only if SINGLE_BATCH=false) |
| `LIMIT_PER_PROPERTY` | `10` | Max recommendations per property |
| `MIN_SCORE` | `0.1` | Minimum recommendation score |
| `TOP_K` | `None` | Return only top K recommendations |
| `TOP_PERCENTILE` | `None` | Return only top X% of recommendations |
| `BUDGET_WEIGHT` | `0.3` | Weight for budget matching |
| `LOCATION_WEIGHT` | `0.25` | Weight for location proximity |
| `PROPERTY_TYPE_WEIGHT` | `0.2` | Weight for property type matching |
| `SIZE_WEIGHT` | `0.25` | Weight for size requirements |
| `API_BASE_URL` | `http://localhost:8080` | API server URL |
| `OUTPUT_FILE` | `None` | Custom output filename |

## Quick Start

1. **Default single batch run:**
   ```bash
   ./run_single_batch.sh
   ```

2. **Custom run:**
   ```bash
   export LIMIT_PER_PROPERTY=20
   export MIN_SCORE=0.3
   python3 batch_recommendations.py
   ```
