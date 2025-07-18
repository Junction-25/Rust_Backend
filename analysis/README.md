# Analysis Directory

This directory contains all the latency testing analysis tools and results.

## 📁 Structure

```
analysis/
├── analyze_latency.py          # Analysis script
├── latency_test_results_*.csv  # Test result files
└── results/                    # Generated analysis reports
    ├── endpoint_statistics.csv
    ├── percentile_analysis.csv
    ├── time_based_analysis.csv
    ├── successful_requests.csv
    ├── latency_analysis.png
    └── endpoint_distributions.png
```

## 🚀 Quick Usage

```bash
# Analyze the latest results
python3 analysis/analyze_latency.py

# Analyze a specific file
python3 analysis/analyze_latency.py --file analysis/latency_test_results_20250718_143022.csv

# Custom output directory
python3 analysis/analyze_latency.py --output-dir custom_results
```

## 📊 Generated Files

- **endpoint_statistics.csv** - Summary statistics per endpoint
- **percentile_analysis.csv** - Latency percentiles (50th, 75th, 90th, 95th, 99th)
- **time_based_analysis.csv** - Performance trends over time
- **successful_requests.csv** - All successful request data
- **latency_analysis.png** - Overview visualization charts
- **endpoint_distributions.png** - Detailed distribution charts

For complete documentation, see `../LATENCY_TESTING.md`
