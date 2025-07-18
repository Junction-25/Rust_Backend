#!/usr/bin/env python3
"""
Latency Test Results Analyzer

This script provides advanced analysis of the latency test results CSV file.
"""

import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import argparse
import glob
import os
from datetime import datetime


def load_latest_results():
    """Load the most recent latency test results file."""
    csv_files = glob.glob("latency_test_results_*.csv")
    if not csv_files:
        raise FileNotFoundError("No latency test results found. Run latency_test.py first.")
    
    latest_file = max(csv_files, key=os.path.getctime)
    print(f"📊 Loading results from: {latest_file}")
    return pd.read_csv(latest_file)


def analyze_results(df, output_dir="analysis_output"):
    """Perform comprehensive analysis of latency results."""
    
    # Create output directory
    os.makedirs(output_dir, exist_ok=True)
    
    print("🔍 Performing latency analysis...")
    
    # Basic statistics
    print("\n📈 BASIC STATISTICS")
    print("=" * 50)
    
    successful_requests = df[df['success'] == True]
    total_requests = len(df)
    success_rate = len(successful_requests) / total_requests * 100
    
    print(f"Total Requests: {total_requests}")
    print(f"Successful Requests: {len(successful_requests)}")
    print(f"Success Rate: {success_rate:.2f}%")
    
    if len(successful_requests) > 0:
        print(f"Overall Average Latency: {successful_requests['latency_ms'].mean():.2f} ms")
        print(f"Overall Median Latency: {successful_requests['latency_ms'].median():.2f} ms")
        print(f"95th Percentile: {successful_requests['latency_ms'].quantile(0.95):.2f} ms")
        print(f"99th Percentile: {successful_requests['latency_ms'].quantile(0.99):.2f} ms")
    
    # Per-endpoint analysis
    print("\n🎯 PER-ENDPOINT ANALYSIS")
    print("=" * 50)
    
    endpoint_stats = []
    for endpoint in df['endpoint'].unique():
        endpoint_data = df[df['endpoint'] == endpoint]
        successful_data = endpoint_data[endpoint_data['success'] == True]
        
        stats = {
            'Endpoint': endpoint,
            'Total Requests': len(endpoint_data),
            'Successful': len(successful_data),
            'Success Rate (%)': len(successful_data) / len(endpoint_data) * 100,
        }
        
        if len(successful_data) > 0:
            stats.update({
                'Avg Latency (ms)': successful_data['latency_ms'].mean(),
                'Median Latency (ms)': successful_data['latency_ms'].median(),
                'Min Latency (ms)': successful_data['latency_ms'].min(),
                'Max Latency (ms)': successful_data['latency_ms'].max(),
                'Std Dev (ms)': successful_data['latency_ms'].std(),
                '95th Percentile (ms)': successful_data['latency_ms'].quantile(0.95),
            })
        
        endpoint_stats.append(stats)
    
    endpoint_df = pd.DataFrame(endpoint_stats)
    print(endpoint_df.to_string(index=False, float_format='%.2f'))
    
    # Save detailed statistics
    endpoint_df.to_csv(f"{output_dir}/endpoint_statistics.csv", index=False)
    
    # Generate visualizations
    generate_visualizations(df, successful_requests, output_dir)
    
    # Generate detailed CSV reports
    generate_detailed_reports(df, successful_requests, output_dir)
    
    print(f"\n✅ Analysis complete! Results saved in '{output_dir}/' directory")


def generate_visualizations(df, successful_requests, output_dir):
    """Generate visualization charts."""
    try:
        print("📊 Generating visualizations...")
        
        plt.style.use('default')
        sns.set_palette("husl")
        
        # 1. Latency distribution by endpoint
        plt.figure(figsize=(12, 8))
        
        # Box plot
        plt.subplot(2, 2, 1)
        sns.boxplot(data=successful_requests, x='endpoint', y='latency_ms')
        plt.title('Latency Distribution by Endpoint')
        plt.xticks(rotation=45)
        plt.ylabel('Latency (ms)')
        
        # Histogram
        plt.subplot(2, 2, 2)
        plt.hist(successful_requests['latency_ms'], bins=30, alpha=0.7, edgecolor='black')
        plt.title('Overall Latency Distribution')
        plt.xlabel('Latency (ms)')
        plt.ylabel('Frequency')
        
        # Success rate by endpoint
        plt.subplot(2, 2, 3)
        success_rates = df.groupby('endpoint')['success'].mean() * 100
        success_rates.plot(kind='bar')
        plt.title('Success Rate by Endpoint')
        plt.ylabel('Success Rate (%)')
        plt.xticks(rotation=45)
        
        # Latency over time
        plt.subplot(2, 2, 4)
        df['timestamp'] = pd.to_datetime(df['timestamp'])
        successful_with_time = successful_requests.copy()
        successful_with_time['timestamp'] = pd.to_datetime(successful_with_time['timestamp'])
        
        for endpoint in successful_with_time['endpoint'].unique():
            endpoint_data = successful_with_time[successful_with_time['endpoint'] == endpoint]
            plt.plot(endpoint_data['timestamp'], endpoint_data['latency_ms'], 
                    marker='o', label=endpoint, alpha=0.7)
        
        plt.title('Latency Over Time')
        plt.xlabel('Time')
        plt.ylabel('Latency (ms)')
        plt.legend()
        plt.xticks(rotation=45)
        
        plt.tight_layout()
        plt.savefig(f"{output_dir}/latency_analysis.png", dpi=300, bbox_inches='tight')
        plt.close()
        
        # 2. Detailed endpoint comparison
        plt.figure(figsize=(14, 10))
        
        endpoints = successful_requests['endpoint'].unique()
        
        for i, endpoint in enumerate(endpoints, 1):
            plt.subplot(3, 3, i)
            endpoint_data = successful_requests[successful_requests['endpoint'] == endpoint]
            
            plt.hist(endpoint_data['latency_ms'], bins=20, alpha=0.7, edgecolor='black')
            plt.title(f'{endpoint.replace("_", " ").title()}\n'
                     f'Avg: {endpoint_data["latency_ms"].mean():.1f}ms')
            plt.xlabel('Latency (ms)')
            plt.ylabel('Frequency')
        
        plt.tight_layout()
        plt.savefig(f"{output_dir}/endpoint_distributions.png", dpi=300, bbox_inches='tight')
        plt.close()
        
        print("✅ Visualizations saved as PNG files")
        
    except ImportError:
        print("⚠️  Matplotlib/Seaborn not available. Skipping visualizations.")
        print("   Install with: pip install matplotlib seaborn")


def generate_detailed_reports(df, successful_requests, output_dir):
    """Generate detailed CSV reports."""
    print("📋 Generating detailed reports...")
    
    # 1. Percentile analysis
    percentiles = [50, 75, 90, 95, 99]
    percentile_data = []
    
    for endpoint in successful_requests['endpoint'].unique():
        endpoint_data = successful_requests[successful_requests['endpoint'] == endpoint]
        row = {'endpoint': endpoint}
        
        for p in percentiles:
            row[f'p{p}'] = endpoint_data['latency_ms'].quantile(p/100)
        
        percentile_data.append(row)
    
    percentile_df = pd.DataFrame(percentile_data)
    percentile_df.to_csv(f"{output_dir}/percentile_analysis.csv", index=False)
    
    # 2. Time-based analysis (if multiple timestamps)
    df['timestamp'] = pd.to_datetime(df['timestamp'])
    df['minute'] = df['timestamp'].dt.floor('T')  # Round to minute
    
    time_analysis = df.groupby(['minute', 'endpoint']).agg({
        'latency_ms': ['mean', 'median', 'std', 'count'],
        'success': 'mean'
    }).round(2)
    
    time_analysis.to_csv(f"{output_dir}/time_based_analysis.csv")
    
    # 3. Failure analysis
    failures = df[df['success'] == False]
    if len(failures) > 0:
        failure_summary = failures.groupby('endpoint').agg({
            'error': 'count',
            'status_code': lambda x: x.value_counts().to_dict()
        })
        failure_summary.to_csv(f"{output_dir}/failure_analysis.csv")
    
    # 4. Raw successful requests for further analysis
    successful_requests.to_csv(f"{output_dir}/successful_requests.csv", index=False)
    
    print("✅ Detailed reports saved as CSV files")


def main():
    parser = argparse.ArgumentParser(description='Analyze latency test results')
    parser.add_argument('--file', help='CSV file to analyze (uses latest if not specified)')
    parser.add_argument('--output-dir', default='analysis_output', 
                       help='Output directory for analysis results')
    parser.add_argument('--no-plots', action='store_true',
                       help='Skip generating plots')
    
    args = parser.parse_args()
    
    try:
        if args.file:
            if not os.path.exists(args.file):
                print(f"❌ File not found: {args.file}")
                return
            df = pd.read_csv(args.file)
            print(f"📊 Loading results from: {args.file}")
        else:
            df = load_latest_results()
        
        analyze_results(df, args.output_dir)
        
        print(f"\n📁 Analysis results saved in: {args.output_dir}/")
        print("Files generated:")
        print("  - endpoint_statistics.csv: Summary statistics per endpoint")
        print("  - percentile_analysis.csv: Latency percentiles")
        print("  - time_based_analysis.csv: Performance over time")
        print("  - successful_requests.csv: All successful request data")
        if not args.no_plots:
            print("  - latency_analysis.png: Overview charts")
            print("  - endpoint_distributions.png: Detailed distributions")
        
    except Exception as e:
        print(f"❌ Error analyzing results: {e}")


if __name__ == "__main__":
    main()
