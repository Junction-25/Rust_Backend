#!/usr/bin/env python3
"""
Plot latency test results from CSV data.

This script reads the CSV output from latency_test.py and creates
various plots to visualize performance metrics.
"""

import pandas as pd
import matplotlib
matplotlib.use('Agg')  # Use non-interactive backend for headless environments
import matplotlib.pyplot as plt
import seaborn as sns
import argparse
import numpy as np
from datetime import datetime


def load_data(csv_file: str) -> pd.DataFrame:
    """Load latency test data from CSV file."""
    try:
        df = pd.read_csv(csv_file)
        df['timestamp'] = pd.to_datetime(df['timestamp'])
        return df
    except FileNotFoundError:
        print(f"Error: File {csv_file} not found.")
        print("Run latency_test.py first to generate test data.")
        exit(1)
    except Exception as e:
        print(f"Error loading data: {e}")
        exit(1)


def plot_response_time_distribution(df: pd.DataFrame, output_dir: str = "."):
    """Plot response time distribution by endpoint."""
    successful_df = df[df['success'] == True]
    
    plt.figure(figsize=(12, 8))
    
    # Create subplots
    fig, axes = plt.subplots(2, 2, figsize=(15, 10))
    fig.suptitle('Response Time Analysis', fontsize=16, fontweight='bold')
    
    # 1. Histogram of response times by endpoint
    axes[0, 0].hist(successful_df[successful_df['endpoint'] == 'single_property']['response_time_ms'], 
                    bins=30, alpha=0.7, label='Single Contact', color='blue')
    axes[0, 0].hist(successful_df[successful_df['endpoint'] == 'bulk_recommendations']['response_time_ms'], 
                    bins=30, alpha=0.7, label='Bulk Recommendations', color='red')
    axes[0, 0].set_xlabel('Response Time (ms)')
    axes[0, 0].set_ylabel('Frequency')
    axes[0, 0].set_title('Response Time Distribution')
    axes[0, 0].legend()
    axes[0, 0].grid(True, alpha=0.3)
    
    # 2. Box plot comparison
    response_times_single = successful_df[successful_df['endpoint'] == 'single_property']['response_time_ms']
    response_times_bulk = successful_df[successful_df['endpoint'] == 'bulk_recommendations']['response_time_ms']
    
    box_data = []
    labels = []
    if len(response_times_single) > 0:
        box_data.append(response_times_single)
        labels.append('Single Contact')
    if len(response_times_bulk) > 0:
        box_data.append(response_times_bulk)
        labels.append('Bulk Recommendations')
    
    if box_data:
        axes[0, 1].boxplot(box_data, labels=labels)
        axes[0, 1].set_ylabel('Response Time (ms)')
        axes[0, 1].set_title('Response Time Box Plot')
        axes[0, 1].grid(True, alpha=0.3)
    
    # 3. Response time over time
    axes[1, 0].scatter(successful_df['timestamp'], successful_df['response_time_ms'], 
                      c=successful_df['endpoint'].map({'single_property': 'blue', 'bulk_recommendations': 'red'}),
                      alpha=0.6, s=20)
    axes[1, 0].set_xlabel('Time')
    axes[1, 0].set_ylabel('Response Time (ms)')
    axes[1, 0].set_title('Response Time Over Time')
    axes[1, 0].tick_params(axis='x', rotation=45)
    axes[1, 0].grid(True, alpha=0.3)
    
    # 4. Success rate
    success_rate = df.groupby('endpoint')['success'].agg(['count', 'sum']).reset_index()
    success_rate['success_rate'] = success_rate['sum'] / success_rate['count'] * 100
    
    bars = axes[1, 1].bar(success_rate['endpoint'], success_rate['success_rate'], 
                         color=['blue', 'red'], alpha=0.7)
    axes[1, 1].set_ylabel('Success Rate (%)')
    axes[1, 1].set_title('Success Rate by Endpoint')
    axes[1, 1].set_ylim(0, 105)
    axes[1, 1].grid(True, alpha=0.3)
    
    # Add percentage labels on bars
    for bar, rate in zip(bars, success_rate['success_rate']):
        height = bar.get_height()
        axes[1, 1].text(bar.get_x() + bar.get_width()/2., height + 1,
                       f'{rate:.1f}%', ha='center', va='bottom')
    
    plt.tight_layout()
    plt.savefig(f'{output_dir}/response_time_analysis.png', dpi=300, bbox_inches='tight')
    plt.close()  # Close the figure to free memory
    print(f"✓ Response time analysis plot saved to {output_dir}/response_time_analysis.png")


def plot_percentile_analysis(df: pd.DataFrame, output_dir: str = "."):
    """Plot percentile analysis of response times."""
    successful_df = df[df['success'] == True]
    
    plt.figure(figsize=(12, 6))
    
    endpoints = successful_df['endpoint'].unique()
    percentiles = [50, 75, 90, 95, 99]
    
    fig, axes = plt.subplots(1, 2, figsize=(15, 6))
    fig.suptitle('Percentile Analysis', fontsize=16, fontweight='bold')
    
    # Percentile comparison
    percentile_data = {}
    for endpoint in endpoints:
        endpoint_data = successful_df[successful_df['endpoint'] == endpoint]['response_time_ms']
        percentile_data[endpoint] = [np.percentile(endpoint_data, p) for p in percentiles]
    
    x = np.arange(len(percentiles))
    width = 0.35
    
    colors = ['blue', 'red', 'green', 'orange', 'purple', 'cyan', 'magenta', 'yellow']
    for i, (endpoint, values) in enumerate(percentile_data.items()):
        # print("Color:", colors[i % len(colors)], "Endpoint:", endpoint, "Values:", values)
        axes[0].bar(x + i * width, values, width, label=endpoint, color=colors[i % len(colors)], alpha=0.7)
    
    axes[0].set_xlabel('Percentile')
    axes[0].set_ylabel('Response Time (ms)')
    axes[0].set_title('Response Time Percentiles')
    axes[0].set_xticks(x + width / 2)
    axes[0].set_xticklabels([f'P{p}' for p in percentiles])
    axes[0].legend()
    axes[0].grid(True, alpha=0.3)
    
    print(successful_df["endpoint"].unique())
    # Response size vs response time
    axes[1].scatter(successful_df['response_size_bytes'], successful_df['response_time_ms'],
                   c=successful_df['endpoint'].map({'single_property': 'blue', 'bulk_recommendations': 'red'}),
                   alpha=0.6, s=20)
    axes[1].set_xlabel('Response Size (bytes)')
    axes[1].set_ylabel('Response Time (ms)')
    axes[1].set_title('Response Time vs Response Size')
    axes[1].grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig(f'{output_dir}/percentile_analysis.png', dpi=300, bbox_inches='tight')
    plt.close()  # Close the figure to free memory
    print(f"✓ Percentile analysis plot saved to {output_dir}/percentile_analysis.png")


def plot_load_analysis(df: pd.DataFrame, output_dir: str = "."):
    """Plot load analysis showing how performance changes over time."""
    successful_df = df[df['success'] == True]
    
    # Create time windows (e.g., every 10 requests)
    successful_df = successful_df.copy()
    successful_df['request_order'] = range(len(successful_df))
    successful_df['time_window'] = successful_df['request_order'] // 10
    
    window_stats = successful_df.groupby(['time_window', 'endpoint'])['response_time_ms'].agg([
        'mean', 'median', 'std', 'count'
    ]).reset_index()
    
    plt.figure(figsize=(15, 8))
    
    fig, axes = plt.subplots(2, 2, figsize=(15, 10))
    fig.suptitle('Load Analysis - Performance Over Time', fontsize=16, fontweight='bold')
    
    # Mean response time by window
    for endpoint in window_stats['endpoint'].unique():
        endpoint_data = window_stats[window_stats['endpoint'] == endpoint]
        color = 'blue' if endpoint == 'single_property' else 'red'
        axes[0, 0].plot(endpoint_data['time_window'], endpoint_data['mean'], 
                       label=f'{endpoint} - Mean', color=color, marker='o')
    
    axes[0, 0].set_xlabel('Time Window (groups of 10 requests)')
    axes[0, 0].set_ylabel('Mean Response Time (ms)')
    axes[0, 0].set_title('Mean Response Time Over Time')
    axes[0, 0].legend()
    axes[0, 0].grid(True, alpha=0.3)
    
    # Standard deviation over time
    for endpoint in window_stats['endpoint'].unique():
        endpoint_data = window_stats[window_stats['endpoint'] == endpoint]
        color = 'blue' if endpoint == 'single_property' else 'red'
        axes[0, 1].plot(endpoint_data['time_window'], endpoint_data['std'], 
                       label=f'{endpoint} - Std Dev', color=color, marker='s')
    
    axes[0, 1].set_xlabel('Time Window (groups of 10 requests)')
    axes[0, 1].set_ylabel('Response Time Std Dev (ms)')
    axes[0, 1].set_title('Response Time Variability Over Time')
    axes[0, 1].legend()
    axes[0, 1].grid(True, alpha=0.3)
    
    # Cumulative average
    successful_df['cumulative_avg'] = successful_df.groupby('endpoint')['response_time_ms'].expanding().mean().reset_index(level=0, drop=True)
    
    for endpoint in successful_df['endpoint'].unique():
        endpoint_data = successful_df[successful_df['endpoint'] == endpoint]
        color = 'blue' if endpoint == 'single_property' else 'red'
        axes[1, 0].plot(endpoint_data['request_order'], endpoint_data['cumulative_avg'], 
                       label=f'{endpoint}', color=color, alpha=0.8)
    
    axes[1, 0].set_xlabel('Request Number')
    axes[1, 0].set_ylabel('Cumulative Average Response Time (ms)')
    axes[1, 0].set_title('Cumulative Average Response Time')
    axes[1, 0].legend()
    axes[1, 0].grid(True, alpha=0.3)
    
    # Error rate over time
    df_with_windows = df.copy()
    df_with_windows['request_order'] = range(len(df))
    df_with_windows['time_window'] = df_with_windows['request_order'] // 10
    
    error_stats = df_with_windows.groupby(['time_window', 'endpoint']).agg({
        'success': ['count', 'sum']
    }).reset_index()
    error_stats.columns = ['time_window', 'endpoint', 'total_requests', 'successful_requests']
    error_stats['error_rate'] = (error_stats['total_requests'] - error_stats['successful_requests']) / error_stats['total_requests'] * 100
    
    for endpoint in error_stats['endpoint'].unique():
        endpoint_data = error_stats[error_stats['endpoint'] == endpoint]
        color = 'blue' if endpoint == 'single_property' else 'red'
        axes[1, 1].plot(endpoint_data['time_window'], endpoint_data['error_rate'], 
                       label=f'{endpoint}', color=color, marker='x')
    
    axes[1, 1].set_xlabel('Time Window (groups of 10 requests)')
    axes[1, 1].set_ylabel('Error Rate (%)')
    axes[1, 1].set_title('Error Rate Over Time')
    axes[1, 1].legend()
    axes[1, 1].grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig(f'{output_dir}/load_analysis.png', dpi=300, bbox_inches='tight')
    plt.close()  # Close the figure to free memory
    print(f"✓ Load analysis plot saved to {output_dir}/load_analysis.png")


def generate_summary_report(df: pd.DataFrame, output_dir: str = "."):
    """Generate a summary report of the test results."""
    successful_df = df[df['success'] == True]
    
    report = []
    report.append("LATENCY TEST REPORT")
    report.append("=" * 50)
    report.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    report.append(f"Total tests: {len(df)}")
    report.append(f"Successful tests: {len(successful_df)} ({len(successful_df)/len(df)*100:.1f}%)")
    report.append("")
    
    # Overall statistics
    if len(successful_df) > 0:
        response_times = successful_df['response_time_ms']
        report.append("OVERALL RESPONSE TIME STATISTICS")
        report.append("-" * 40)
        report.append(f"Mean: {response_times.mean():.2f} ms")
        report.append(f"Median: {response_times.median():.2f} ms")
        report.append(f"Min: {response_times.min():.2f} ms")
        report.append(f"Max: {response_times.max():.2f} ms")
        report.append(f"Standard Deviation: {response_times.std():.2f} ms")
        report.append(f"95th Percentile: {np.percentile(response_times, 95):.2f} ms")
        report.append(f"99th Percentile: {np.percentile(response_times, 99):.2f} ms")
        report.append("")
        
        # By endpoint
        for endpoint in successful_df['endpoint'].unique():
            endpoint_data = successful_df[successful_df['endpoint'] == endpoint]
            endpoint_times = endpoint_data['response_time_ms']
            
            report.append(f"{endpoint.upper()} ENDPOINT STATISTICS")
            report.append("-" * 40)
            report.append(f"Tests: {len(endpoint_data)}")
            report.append(f"Mean response time: {endpoint_times.mean():.2f} ms")
            report.append(f"Median response time: {endpoint_times.median():.2f} ms")
            report.append(f"Min response time: {endpoint_times.min():.2f} ms")
            report.append(f"Max response time: {endpoint_times.max():.2f} ms")
            report.append(f"95th Percentile: {np.percentile(endpoint_times, 95):.2f} ms")
            report.append("")
    
    # Error analysis
    failed_df = df[df['success'] == False]
    if len(failed_df) > 0:
        report.append("ERROR ANALYSIS")
        report.append("-" * 40)
        error_counts = failed_df.groupby('status_code').size()
        for status_code, count in error_counts.items():
            report.append(f"HTTP {status_code}: {count} occurrences")
        report.append("")
    
    # Save report
    with open(f'{output_dir}/latency_test_report.txt', 'w') as f:
        f.write('\n'.join(report))
    
    print(f"✓ Summary report saved to {output_dir}/latency_test_report.txt")
    print('\n'.join(report))


def main():
    parser = argparse.ArgumentParser(description='Plot latency test results')
    parser.add_argument('csv_file', default='latency_results.csv', nargs='?',
                       help='CSV file with latency test results (default: latency_results.csv)')
    parser.add_argument('--output-dir', default='./analysis',
                       help='Directory to save plots (default: current directory)')
    
    args = parser.parse_args()
    
    # Set style for better-looking plots
    plt.style.use('seaborn-v0_8')
    sns.set_palette("husl")
    
    # Load data
    df = load_data(args.csv_file)
    
    print(f"Loaded {len(df)} test results from {args.csv_file}")
    
    # Generate plots
    print("Generating response time analysis...")
    plot_response_time_distribution(df, args.output_dir)
    
    print("Generating percentile analysis...")
    plot_percentile_analysis(df, args.output_dir)
    
    print("Generating load analysis...")
    plot_load_analysis(df, args.output_dir)
    
    print("Generating summary report...")
    generate_summary_report(df, args.output_dir)
    
    print(f"\nAll plots and reports saved to {args.output_dir}/")


if __name__ == "__main__":
    main()
