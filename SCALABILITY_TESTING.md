# Scalability Testing for Recommendation API

This directory contains tools for testing how the recommendation API performance scales with increasing dataset size. The tests start with a relatively empty database and gradually add more contacts and properties while measuring response times.

## Overview

The scalability test measures:
- **Response time** as database size increases
- **Throughput** degradation with larger datasets  
- **Performance variability** across different data volumes
- **Scalability characteristics** and bottlenecks
- **Bulk request efficiency** compared to individual requests
- **Performance comparison** between single and bulk endpoints

## Files

- `scalability_test.py` - Main scalability testing script
- `plot_scalability.py` - Script to generate scalability analysis plots
- `run_scalability_test.sh` - Convenience script to run the complete test suite

## Quick Start

1. **Start the Rust backend server:**
   ```bash
   cargo run --release
   ```

2. **Set your database URL** (if different from default):
   ```bash
   export DATABASE_URL="postgresql://your_user:your_password@localhost/your_db"
   ```

3. **Run the complete scalability test:**
   ```bash
   ./run_scalability_test.sh
   ```

## How It Works

### Test Process

1. **Data Loading**: Loads real contact and property data from `data/contacts.json` and `data/properties.json`
2. **Database Preparation**: Clears existing test data
3. **Gradual Growth**: Adds real data in batches:
   - Starts with 100 properties (baseline dataset)
   - Adds 50 contacts per step (from real data)
   - Adds 100 properties per step (from real data)
   - Tests performance at each step
4. **Performance Measurement**: For each dataset size:
   - Runs 5 single recommendation requests
   - Runs bulk recommendation requests (sizes: 2, 5, 10, 20 contacts)
   - Measures response time, success rate, recommendation count
   - Records database metrics and efficiency comparisons

### Dataset Progression Example

```
Step 1:   50 contacts,  200 properties →  5 single tests + 4 bulk tests (real data)
Step 2:  100 contacts,  300 properties →  5 single tests + 4 bulk tests (real data)
Step 3:  150 contacts,  400 properties →  5 single tests + 4 bulk tests (real data)
...
Step 10: 500 contacts, 1000 properties →  5 single tests + 4 bulk tests (real data)
```

Each bulk test includes different batch sizes (2, 5, 10, 20 contacts) to measure efficiency.

## Manual Usage

### Prerequisites

```bash
pip3 install -r requirements.txt
```

### Running Scalability Tests

Basic usage:
```bash
python3 analysis/scalability_test.py --db-url "postgresql://user:pass@localhost/db"
```

Advanced options:
```bash
python3 analysis/scalability_test.py \
    --url http://localhost:8080 \
    --db-url "postgresql://user:pass@localhost/db" \
    --data-dir data \
    --max-contacts 1000 \
    --max-properties 2000 \
    --contact-batch-size 50 \
    --property-batch-size 100 \
    --tests-per-step 10 \
    --output scalability_results.csv
```

### Generating Analysis Plots

```bash
python3 analysis/plot_scalability.py scalability_results.csv --output-dir ./plots
```

## Configuration Options

| Parameter | Default | Description |
|-----------|---------|-------------|
| `--max-contacts` | 1000 | Maximum contacts to test |
| `--max-properties` | 2000 | Maximum properties to test |
| `--contact-batch-size` | 50 | Contacts added per step |
| `--property-batch-size` | 100 | Properties added per step |
| `--tests-per-step` | 5 | Tests run per dataset size |
| `--data-dir` | data | Directory with JSON data files |
| `--include-bulk` | True | Include bulk recommendation tests |
| `--no-bulk` | - | Disable bulk recommendation tests |
| `--bulk-sizes` | [2,5,10,20] | Bulk batch sizes to test |

## Output Files

### CSV Data (`scalability_results.csv`)

Contains detailed test results with columns:
- `total_contacts`, `total_properties`: Dataset size at test time
- `response_time_ms`: Response time in milliseconds
- `recommendations_count`: Number of recommendations returned
- `success`: Whether the request succeeded
- `timestamp`: When the test was run

### Analysis Plots

1. **`scalability_response_time.png`**:
   - Single contact response time vs dataset size
   - Bulk response time comparisons by batch size  
   - Bulk efficiency analysis (time vs expected time)
   - Response time vs total dataset size

2. **`scalability_trends.png`**:
   - Response time variability (coefficient of variation)
   - Min/max/mean response times
   - Performance degradation percentage
   - Response size correlation

3. **`scalability_heatmap.png`**:
   - 2D heatmap showing response times across different contact/property combinations
   - Helps identify performance patterns

4. **`scalability_throughput.png`**:
   - Theoretical throughput (requests/second)
   - Recommendations per second
   - Processing efficiency metrics
   - Data transfer efficiency

### Report (`scalability_report.txt`)

Text summary including:
- Performance degradation analysis
- Dataset progression details
- Scalability assessment and recommendations

## Interpreting Results

### Good Scalability Indicators

- **Linear growth**: Response time grows linearly (or sub-linearly) with data size
- **Low variability**: Consistent response times across tests
- **Scalability factor < 1.5**: Performance degrades slower than data growth

### Performance Concerns

- **Exponential growth**: Response time grows exponentially with data size  
- **High variability**: Inconsistent response times
- **Scalability factor > 2.0**: Performance degrades faster than data growth

### Example Analysis

```
SCALABILITY INSIGHTS
--------------------
Dataset grew by: 12.0x
Response time grew by: 3.2x  
Scalability factor: 0.27
Assessment: Excellent scalability
```

This shows the dataset grew 12x but response time only grew 3.2x, indicating excellent scalability.

## Database Requirements

The test requires:
- **PostgreSQL database** with your schema
- **Write permissions** to clear and populate test data
- **Connection string** in format: `postgresql://user:password@host:port/database`

⚠️ **Warning**: The test clears all contacts and properties data before running. Use a test database, not production data.

## Performance Benchmarks

Expected performance characteristics:

| Dataset Size | Expected Response Time | Notes |
|--------------|----------------------|-------|
| 25 contacts, 100 properties | 20-50ms | Baseline performance |
| 100 contacts, 300 properties | 30-80ms | Good scalability |
| 300 contacts, 600 properties | 50-150ms | Acceptable for most use cases |
| 500+ contacts, 1000+ properties | 100-300ms | May need optimization |

## Troubleshooting

### Common Issues

1. **Database connection errors**:
   - Verify DATABASE_URL is correct
   - Check database is running and accessible
   - Ensure user has CREATE/DROP permissions

2. **Server not responding**:
   - Verify Rust backend is running on correct port
   - Check server logs for errors

3. **Out of memory errors**:
   - Reduce batch sizes or max dataset size
   - Monitor system resources during testing

4. **Slow test execution**:
   - Reduce `--tests-per-step` for faster results
   - Use smaller batch sizes for more granular analysis

### Debug Mode

For debugging, run individual components:

```bash
# Test database connection
python3 -c "import psycopg2; psycopg2.connect('your_db_url')"

# Test API endpoint
curl http://localhost:8080/health

# Run minimal scalability test
python3 analysis/scalability_test.py --max-contacts 50 --max-properties 100
```

This comprehensive scalability testing will help you understand how your recommendation system performs as your data grows and identify any bottlenecks that need optimization.
