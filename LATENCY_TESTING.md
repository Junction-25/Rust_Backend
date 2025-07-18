# Real Estate API Latency Testing Suite

This directory contains a comprehensive latency testing suite for the Real Estate Recommendation System API. The suite includes tools for testing endpoint performance, analyzing results, and generating detailed reports.

## 📁 Files Overview

### Core Testing Files
- **`latency_test.py`** - Main latency testing script
- **`analyze_latency.py`** - Advanced results analysis and visualization
- **`run_latency_test.sh`** - Convenient shell script to run tests
- **`requirements.txt`** - Python dependencies

### Generated Output Files
- **`latency_test_results_*.csv`** - Raw test results (timestamped)
- **`analysis_output/`** - Directory containing analysis reports and charts

## 🚀 Quick Start

### 1. Prerequisites
- Python 3.6 or higher
- Running Real Estate API server (port 8080 by default)
- PostgreSQL database (optional, mock data used if not available)

### 2. Install Dependencies
```bash
pip install -r requirements.txt
```

### 3. Run Basic Latency Tests
```bash
# Simple test with default settings
./run_latency_test.sh

# Custom server URL and iterations
./run_latency_test.sh http://localhost:8080 20

# Or run directly with Python
python3 latency_test.py --iterations 15
```

### 4. Analyze Results
```bash
# Analyze the latest results
python3 analyze_latency.py

# Analyze a specific file
python3 analyze_latency.py --file latency_test_results_20250718_143022.csv
```

## 🔧 Detailed Usage

### Latency Testing (`latency_test.py`)

Tests all API endpoints and measures:
- Response latency (in milliseconds)
- Success/failure rates
- Response sizes
- HTTP status codes

**Tested Endpoints:**
- `GET /health` - Health check
- `GET /recommendations/property/{id}` - Property recommendations
- `POST /recommendations/bulk` - Bulk recommendations
- `GET /comparisons/properties` - Property comparisons
- `POST /quotes/generate` - Quote generation
- `POST /quotes/comparison` - Comparison quote
- `GET /quotes/recommendations` - Recommendation quote

**Command Line Options:**
```bash
python3 latency_test.py [OPTIONS]

Options:
  --url TEXT              Base URL (default: http://localhost:8080)
  --iterations INTEGER    Number of test iterations (default: 10)
  --database-url TEXT     PostgreSQL connection string
  --output TEXT           Custom CSV output filename
  --no-summary           Skip console summary report
```

**Examples:**
```bash
# Test with 50 iterations per endpoint
python3 latency_test.py --iterations 50

# Test against staging server
python3 latency_test.py --url https://staging.example.com --iterations 25

# Custom output file
python3 latency_test.py --output my_test_results.csv
```

### Results Analysis (`analyze_latency.py`)

Provides comprehensive analysis including:
- Statistical summaries (mean, median, percentiles)
- Per-endpoint performance breakdown
- Success rate analysis
- Time-based trends
- Visual charts and graphs

**Generated Reports:**
- `endpoint_statistics.csv` - Summary stats per endpoint
- `percentile_analysis.csv` - Latency percentiles (50th, 75th, 90th, 95th, 99th)
- `time_based_analysis.csv` - Performance trends over time
- `successful_requests.csv` - All successful request data
- `latency_analysis.png` - Overview visualization
- `endpoint_distributions.png` - Detailed distribution charts

**Command Line Options:**
```bash
python3 analyze_latency.py [OPTIONS]

Options:
  --file TEXT            Specific CSV file to analyze
  --output-dir TEXT      Output directory (default: analysis_output)
  --no-plots            Skip generating visualization charts
```

## 📊 Understanding the Results

### CSV Output Format

The main results CSV contains the following columns:

| Column | Description |
|--------|-------------|
| `timestamp` | When the request was made (ISO format) |
| `endpoint` | API endpoint name (health, recommendations, etc.) |
| `method` | HTTP method (GET, POST) |
| `iteration` | Test iteration number |
| `url` | Full URL that was tested |
| `success` | Boolean indicating if request succeeded |
| `latency_ms` | Response time in milliseconds |
| `status_code` | HTTP status code (200, 500, etc.) |
| `response_size` | Response body size in bytes |
| `error` | Error message (if request failed) |

### Key Metrics to Monitor

**Response Time Targets:**
- Health endpoint: < 10ms
- Recommendations: < 100ms
- Comparisons: < 200ms
- Quote generation: < 2000ms (PDF generation is slower)

**Success Rate Targets:**
- All endpoints should maintain > 99% success rate
- Monitor for 5xx errors indicating server issues
- Watch for timeout errors under load

### Sample Analysis Output

```
📊 LATENCY TEST SUMMARY REPORT
================================================

🎯 HEALTH
----------------------------------------
Total Requests: 10
Successful: 10
Failed: 0
Success Rate: 100.0%
Average Latency: 8.45 ms
Median Latency: 7.23 ms
95th Percentile: 15.67 ms

🎯 RECOMMENDATIONS
----------------------------------------
Total Requests: 10
Successful: 10
Failed: 0
Success Rate: 100.0%
Average Latency: 89.23 ms
Median Latency: 85.12 ms
95th Percentile: 120.45 ms
```

## 🔍 Advanced Usage

### Load Testing

For load testing, run multiple iterations:

```bash
# High-intensity test
python3 latency_test.py --iterations 100

# Stress test with analysis
python3 latency_test.py --iterations 200
python3 analyze_latency.py
```

### Continuous Monitoring

Set up regular testing with cron:

```bash
# Add to crontab for hourly testing
0 * * * * cd /path/to/project && ./run_latency_test.sh > /dev/null 2>&1
```

### Database Integration

The script automatically uses the `DATABASE_URL` environment variable:

```bash
export DATABASE_URL="postgresql://user:pass@localhost:5432/dbname"
python3 latency_test.py
```

### Custom Analysis

Use the CSV data for custom analysis:

```python
import pandas as pd

# Load results
df = pd.read_csv('latency_test_results_20250718_143022.csv')

# Custom analysis
slow_requests = df[df['latency_ms'] > 1000]
endpoint_averages = df.groupby('endpoint')['latency_ms'].mean()
```

## 🐛 Troubleshooting

### Common Issues

**"Cannot connect to server"**
- Ensure the API server is running: `cargo run --release`
- Check the correct URL and port
- Verify firewall settings

**"No sample data found"**
- Run database setup: `./setup.sh`
- Check `DATABASE_URL` environment variable
- Script will use mock data if database is unavailable

**"Import errors"**
- Install dependencies: `pip install -r requirements.txt`
- Use virtual environment for isolation

**"Permission denied"**
- Make scripts executable: `chmod +x *.sh`
- Check file permissions

### Performance Optimization

**For faster testing:**
- Reduce iterations for development testing
- Skip PDF generation endpoints for quick checks
- Use `--no-summary` flag for automated testing

**For comprehensive analysis:**
- Use higher iteration counts (50-100)
- Include all endpoints
- Run during different load conditions

## 📈 Integration with CI/CD

### GitHub Actions Example

```yaml
name: API Latency Tests
on: [push, pull_request]

jobs:
  latency-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Start API Server
        run: cargo run --release &
      - name: Install Python Dependencies
        run: pip install -r requirements.txt
      - name: Run Latency Tests
        run: python3 latency_test.py --iterations 20
      - name: Upload Results
        uses: actions/upload-artifact@v2
        with:
          name: latency-results
          path: latency_test_results_*.csv
```

## 📝 Best Practices

1. **Regular Testing**: Run latency tests as part of your deployment pipeline
2. **Baseline Establishment**: Keep historical data to track performance trends
3. **Environment Consistency**: Test in environments similar to production
4. **Load Simulation**: Test with realistic data volumes and user patterns
5. **Monitoring Integration**: Combine with APM tools for comprehensive monitoring

## 🤝 Contributing

To extend the latency testing suite:

1. Add new endpoints to `latency_test.py`
2. Update the analysis scripts for new metrics
3. Add visualization for new data points
4. Update this documentation

---

**Happy Testing! 🚀**

For questions or issues, check the main project documentation or open an issue.
