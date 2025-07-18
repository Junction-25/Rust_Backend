# Latency Testing for Recommendation API

This directory contains tools for testing the latency and performance of the recommendation API endpoints.

## Files

- `latency_test.py` - Main latency testing script
- `plot_latency.py` - Script to generate performance plots from test data
- `run_latency_test.sh` - Convenience script to run the complete test suite
- `requirements.txt` - Python dependencies

## Quick Start

1. **Start the Rust backend server:**
   ```bash
   cargo run --release
   ```

2. **Run the complete test suite:**
   ```bash
   ./run_latency_test.sh
   ```

This will automatically install dependencies, run tests, and generate plots.

## Manual Usage

### Prerequisites

Install Python dependencies:
```bash
pip3 install -r requirements.txt
```

### Running Latency Tests

Basic usage:
```bash
python3 latency_test.py
```

Advanced options:
```bash
python3 latency_test.py \
    --url http://localhost:8080 \
    --iterations 500 \
    --contact-ids 1 2 3 4 5 \
    --output my_results.csv
```

Options:
- `--url`: Base URL of the API server (default: http://localhost:8080)
- `--iterations`: Number of test iterations (default: 100)
- `--contact-ids`: Contact IDs to test (default: 1 2 3 4 5)
- `--output`: Output CSV file name (default: latency_results.csv)

### Generating Plots

```bash
python3 plot_latency.py latency_results.csv --output-dir ./plots
```

## Test Endpoints

The latency test covers two main endpoints:

1. **Single Contact Recommendations**: `GET /recommendations/contact/{contact_id}`
   - Tests individual contact recommendation requests
   - Supports query parameters: `limit`, `min_score`

2. **Bulk Recommendations**: `POST /recommendations/bulk`
   - Tests bulk recommendation requests
   - Supports multiple contact IDs in a single request

## Output Files

After running tests, you'll get:

1. **latency_results.csv**: Raw test data with columns:
   - `timestamp`: When the test was run
   - `endpoint`: Which endpoint was tested
   - `contact_id`: Contact ID used (for single contact tests)
   - `response_time_ms`: Response time in milliseconds
   - `status_code`: HTTP status code
   - `request_size_bytes`: Size of request payload
   - `response_size_bytes`: Size of response payload
   - `success`: Whether the request was successful
   - `error_message`: Error details (if any)

2. **Performance Plots**:
   - `response_time_analysis.png`: Response time distributions and success rates
   - `percentile_analysis.png`: Percentile analysis and response size correlation
   - `load_analysis.png`: Performance trends over time

3. **latency_test_report.txt**: Text summary of test results

## Performance Metrics

The tools measure and report:

- **Response Time Statistics**: Mean, median, min, max, percentiles
- **Success Rate**: Percentage of successful requests
- **Performance Trends**: How latency changes over time
- **Error Analysis**: Breakdown of failed requests
- **Load Analysis**: Performance under sustained load

## Customization

### Testing Different Scenarios

You can modify the test scenarios by:

1. **Adding more contact IDs**: Update the `--contact-ids` parameter
2. **Testing different request patterns**: Modify the `run_test_suite()` method
3. **Adding custom endpoints**: Extend the tester class with new methods
4. **Changing load patterns**: Adjust iteration counts and timing

### Custom Analysis

The CSV output can be imported into any data analysis tool:

```python
import pandas as pd
df = pd.read_csv('latency_results.csv')
# Your custom analysis here
```

## Troubleshooting

1. **Server not running**: Make sure the Rust backend is running on the specified port
2. **Connection errors**: Check firewall settings and network connectivity
3. **Import errors**: Install required Python packages with `pip3 install -r requirements.txt`
4. **Permission errors**: Make sure the script has write permissions for output files

## Example Output

```
Running latency tests for 200 iterations...
Testing contact IDs: [1, 2, 3, 4, 5]
Base URL: http://localhost:8080
✓ Server health check passed

Testing single contact recommendations...
  Completed 20/200 single contact tests
  ...

Testing bulk recommendations...
  Completed 10/100 bulk tests
  ...

✓ Completed all tests. Total results: 300
✓ Results saved to latency_results.csv

============================================================
LATENCY TEST SUMMARY
============================================================
Total tests: 300
Successful: 298 (99.3%)
Failed: 2 (0.7%)

Response Time Statistics (ms):
  Mean: 45.23
  Median: 42.10
  Min: 12.34
  Max: 156.78
  95th percentile: 89.45
  99th percentile: 134.56
```

This provides comprehensive latency testing for your recommendation API with detailed performance analysis and visualization.
