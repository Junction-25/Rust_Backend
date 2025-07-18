#!/usr/bin/env python3
"""
Plot latency test results from CSV data.

This script reads the CSV output from latency_test.py and creates
various plots to visualize performance metrics, including analysis
of different filtering parameters (top_k, top_percentile, etc.).
"""

import pandas as pd
import matplotlib
matplotlib.use('Agg')  # Use non-interactive backend for headless environments
import matplotlib.pyplot as plt
import seaborn as sns
import argparse
import numpy as np
from datetime import datetime
import os


def load_data(csv_file: str) -> pd.DataFrame:
    """Load latency test data from CSV file."""
    try:
        df = pd.read_csv(csv_file)
        df['timestamp'] = pd.to_datetime(df['timestamp'])
        
        # Parse filtering parameters from request_size_bytes or error_message
        # Since we don't store the actual parameters, we'll infer them from request patterns
        df = infer_filtering_parameters(df)
        
        return df
    except FileNotFoundError:
        print(f"Error: File {csv_file} not found.")
        print("Run latency_test.py first to generate test data.")
        exit(1)
    except Exception as e:
        print(f"Error loading data: {e}")
        exit(1)


def infer_filtering_parameters(df: pd.DataFrame) -> pd.DataFrame:
    """Infer filtering parameters based on request patterns."""
    # Add columns for filtering parameters
    df['top_k'] = None
    df['top_percentile'] = None
    df['scenario'] = 'unknown'
    
    # Group requests by similar request sizes to infer scenarios
    successful_df = df[df['success'] == True].copy()
    
    if len(successful_df) > 0:
        # Categorize based on request size patterns and response times
        request_sizes = successful_df['request_size_bytes'].values
        response_times = successful_df['response_time_ms'].values
        
        # Use clustering or patterns to identify scenarios
        # For now, we'll use simple heuristics based on request order
        for i, row in successful_df.iterrows():
            idx = successful_df.index.get_loc(i)
            scenario_cycle = idx % 5  # 5 scenarios as defined in latency_test.py
            
            if scenario_cycle == 0:
                df.loc[i, 'scenario'] = 'basic_filtering'
                df.loc[i, 'top_k'] = None
                df.loc[i, 'top_percentile'] = None
            elif scenario_cycle == 1:
                df.loc[i, 'scenario'] = 'top_k_5'
                df.loc[i, 'top_k'] = 5
                df.loc[i, 'top_percentile'] = None
            elif scenario_cycle == 2:
                df.loc[i, 'scenario'] = 'top_percentile_20'
                df.loc[i, 'top_k'] = None
                df.loc[i, 'top_percentile'] = 0.2
            elif scenario_cycle == 3:
                df.loc[i, 'scenario'] = 'combined_filtering'
                df.loc[i, 'top_k'] = 5
                df.loc[i, 'top_percentile'] = 0.2
            elif scenario_cycle == 4:
                df.loc[i, 'scenario'] = 'threshold_percentile'
                df.loc[i, 'top_k'] = None
                df.loc[i, 'top_percentile'] = None
    
    return df


def plot_filtering_analysis(df: pd.DataFrame, output_dir: str = "."):
    """Plot analysis of different filtering parameters."""
    successful_df = df[df['success'] == True]
    
    if len(successful_df) == 0:
        print("Warning: No successful requests found for filtering analysis")
        return
    
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    fig.suptitle('Filtering Parameters Analysis', fontsize=16, fontweight='bold')
    
    # 1. Response time by scenario
    scenarios = successful_df['scenario'].unique()
    scenario_data = []
    scenario_labels = []
    
    for scenario in scenarios:
        if scenario != 'unknown':
            scenario_responses = successful_df[successful_df['scenario'] == scenario]['response_time_ms']
            if len(scenario_responses) > 0:
                scenario_data.append(scenario_responses)
                scenario_labels.append(scenario.replace('_', ' ').title())
    
    if scenario_data:
        bp = axes[0, 0].boxplot(scenario_data, tick_labels=scenario_labels)
        axes[0, 0].set_ylabel('Response Time (ms)')
        axes[0, 0].set_title('Response Time by Filtering Scenario')
        axes[0, 0].tick_params(axis='x', rotation=45)
        axes[0, 0].grid(True, alpha=0.3)
    
    # 2. Mean response time by top_k values
    top_k_analysis = successful_df[successful_df['top_k'].notna()].groupby('top_k')['response_time_ms'].agg([
        'mean', 'median', 'std', 'count'
    ]).reset_index()
    
    if len(top_k_analysis) > 0:
        x_pos = np.arange(len(top_k_analysis))
        bars = axes[0, 1].bar(x_pos, top_k_analysis['mean'], 
                             yerr=top_k_analysis['std'], capsize=5, alpha=0.7,
                             color='skyblue', edgecolor='navy')
        axes[0, 1].set_xlabel('Top K Value')
        axes[0, 1].set_ylabel('Mean Response Time (ms)')
        axes[0, 1].set_title('Performance vs Top K Parameter')
        axes[0, 1].set_xticks(x_pos)
        axes[0, 1].set_xticklabels([f'K={int(k)}' for k in top_k_analysis['top_k']])
        axes[0, 1].grid(True, alpha=0.3)
        
        # Add count labels on bars
        for i, (bar, count) in enumerate(zip(bars, top_k_analysis['count'])):
            height = bar.get_height()
            axes[0, 1].text(bar.get_x() + bar.get_width()/2., height + top_k_analysis.iloc[i]['std'],
                           f'n={int(count)}', ha='center', va='bottom', fontsize=8)
    else:
        axes[0, 1].text(0.5, 0.5, 'No Top K data available', 
                       ha='center', va='center', transform=axes[0, 1].transAxes)
        axes[0, 1].set_title('Performance vs Top K Parameter')
    
    # 3. Mean response time by top_percentile values
    percentile_analysis = successful_df[successful_df['top_percentile'].notna()].groupby('top_percentile')['response_time_ms'].agg([
        'mean', 'median', 'std', 'count'
    ]).reset_index()
    
    if len(percentile_analysis) > 0:
        x_pos = np.arange(len(percentile_analysis))
        bars = axes[1, 0].bar(x_pos, percentile_analysis['mean'], 
                             yerr=percentile_analysis['std'], capsize=5, alpha=0.7,
                             color='lightcoral', edgecolor='darkred')
        axes[1, 0].set_xlabel('Top Percentile Value')
        axes[1, 0].set_ylabel('Mean Response Time (ms)')
        axes[1, 0].set_title('Performance vs Top Percentile Parameter')
        axes[1, 0].set_xticks(x_pos)
        axes[1, 0].set_xticklabels([f'{p:.1%}' for p in percentile_analysis['top_percentile']])
        axes[1, 0].grid(True, alpha=0.3)
        
        # Add count labels on bars
        for i, (bar, count) in enumerate(zip(bars, percentile_analysis['count'])):
            height = bar.get_height()
            axes[1, 0].text(bar.get_x() + bar.get_width()/2., height + percentile_analysis.iloc[i]['std'],
                           f'n={int(count)}', ha='center', va='bottom', fontsize=8)
    else:
        axes[1, 0].text(0.5, 0.5, 'No Top Percentile data available', 
                       ha='center', va='center', transform=axes[1, 0].transAxes)
        axes[1, 0].set_title('Performance vs Top Percentile Parameter')
    
    # 4. Scenario comparison with detailed statistics
    if len(scenarios) > 1 and 'unknown' not in scenarios:
        scenario_stats = []
        for scenario in scenarios:
            if scenario != 'unknown':
                scenario_data = successful_df[successful_df['scenario'] == scenario]['response_time_ms']
                if len(scenario_data) > 0:
                    scenario_stats.append({
                        'scenario': scenario.replace('_', ' ').title(),
                        'mean': scenario_data.mean(),
                        'median': scenario_data.median(),
                        'p95': scenario_data.quantile(0.95),
                        'count': len(scenario_data)
                    })
        
        if scenario_stats:
            scenario_df = pd.DataFrame(scenario_stats)
            x_pos = np.arange(len(scenario_df))
            width = 0.25
            
            axes[1, 1].bar(x_pos - width, scenario_df['mean'], width, label='Mean', alpha=0.8, color='blue')
            axes[1, 1].bar(x_pos, scenario_df['median'], width, label='Median', alpha=0.8, color='green')
            axes[1, 1].bar(x_pos + width, scenario_df['p95'], width, label='P95', alpha=0.8, color='red')
            
            axes[1, 1].set_xlabel('Filtering Scenario')
            axes[1, 1].set_ylabel('Response Time (ms)')
            axes[1, 1].set_title('Detailed Performance Comparison')
            axes[1, 1].set_xticks(x_pos)
            axes[1, 1].set_xticklabels(scenario_df['scenario'], rotation=45, ha='right')
            axes[1, 1].legend()
            axes[1, 1].grid(True, alpha=0.3)
    else:
        axes[1, 1].text(0.5, 0.5, 'Insufficient scenario data', 
                       ha='center', va='center', transform=axes[1, 1].transAxes)
        axes[1, 1].set_title('Detailed Performance Comparison')
    
    plt.tight_layout()
    plt.savefig(os.path.join(output_dir, 'filtering_analysis.png'), dpi=300, bbox_inches='tight')
    plt.close()
    
    print(f"✓ Filtering analysis plot saved to {output_dir}/filtering_analysis.png")


def plot_performance_heatmap(df: pd.DataFrame, output_dir: str = "."):
    """Create a heatmap showing performance across different parameter combinations."""
    successful_df = df[df['success'] == True]
    
    if len(successful_df) == 0:
        print("Warning: No successful requests found for heatmap analysis")
        return
    
    # Create parameter combination analysis
    param_combinations = successful_df.groupby(['top_k', 'top_percentile', 'endpoint'])['response_time_ms'].agg([
        'mean', 'count'
    ]).reset_index()
    
    # Filter combinations with sufficient data
    param_combinations = param_combinations[param_combinations['count'] >= 3]
    
    if len(param_combinations) == 0:
        print("Warning: Insufficient data for parameter combination analysis")
        return
    
    fig, axes = plt.subplots(1, 2, figsize=(16, 6))
    fig.suptitle('Parameter Combination Performance Heatmap', fontsize=16, fontweight='bold')
    
    # Heatmap for single_property endpoint
    single_data = param_combinations[param_combinations['endpoint'] == 'single_property']
    if len(single_data) > 0:
        # Create pivot table for heatmap
        heatmap_data = single_data.pivot_table(values='mean', index='top_k', columns='top_percentile', fill_value=np.nan)
        
        if not heatmap_data.empty:
            sns.heatmap(heatmap_data, annot=True, fmt='.1f', cmap='YlOrRd', 
                       ax=axes[0], cbar_kws={'label': 'Mean Response Time (ms)'})
            axes[0].set_title('Single Property Endpoint')
            axes[0].set_xlabel('Top Percentile')
            axes[0].set_ylabel('Top K')
    
    # Heatmap for bulk_recommendations endpoint
    bulk_data = param_combinations[param_combinations['endpoint'] == 'bulk_recommendations']
    if len(bulk_data) > 0:
        heatmap_data = bulk_data.pivot_table(values='mean', index='top_k', columns='top_percentile', fill_value=np.nan)
        
        if not heatmap_data.empty:
            sns.heatmap(heatmap_data, annot=True, fmt='.1f', cmap='YlOrRd', 
                       ax=axes[1], cbar_kws={'label': 'Mean Response Time (ms)'})
            axes[1].set_title('Bulk Recommendations Endpoint')
            axes[1].set_xlabel('Top Percentile')
            axes[1].set_ylabel('Top K')
    
    # If no data for heatmaps, show message
    if len(single_data) == 0:
        axes[0].text(0.5, 0.5, 'No data for Single Property endpoint', 
                    ha='center', va='center', transform=axes[0].transAxes)
        axes[0].set_title('Single Property Endpoint')
    
    if len(bulk_data) == 0:
        axes[1].text(0.5, 0.5, 'No data for Bulk Recommendations endpoint', 
                    ha='center', va='center', transform=axes[1].transAxes)
        axes[1].set_title('Bulk Recommendations Endpoint')
    
    plt.tight_layout()
    plt.savefig(os.path.join(output_dir, 'parameter_heatmap.png'), dpi=300, bbox_inches='tight')
    plt.close()
    
    print(f"✓ Parameter heatmap plot saved to {output_dir}/parameter_heatmap.png")


def plot_response_time_distribution(df: pd.DataFrame, output_dir: str = "."):
    """Plot response time distribution by endpoint."""
    successful_df = df[df['success'] == True]
    
    plt.figure(figsize=(12, 8))
    
    # Create subplots
    fig, axes = plt.subplots(2, 2, figsize=(15, 10))
    fig.suptitle('Response Time Analysis', fontsize=16, fontweight='bold')
    
    # 1. Histogram of response times by endpoint
    single_property_data = successful_df[successful_df['endpoint'] == 'single_property']['response_time_ms']
    bulk_data = successful_df[successful_df['endpoint'] == 'bulk_recommendations']['response_time_ms']
    
    if len(single_property_data) > 0:
        axes[0, 0].hist(single_property_data, bins=30, alpha=0.7, label='Single Property', color='blue')
    if len(bulk_data) > 0:
        axes[0, 0].hist(bulk_data, bins=30, alpha=0.7, label='Bulk Recommendations', color='red')
    
    axes[0, 0].set_xlabel('Response Time (ms)')
    axes[0, 0].set_ylabel('Frequency')
    axes[0, 0].set_title('Response Time Distribution')
    axes[0, 0].legend()
    axes[0, 0].grid(True, alpha=0.3)
    
    # 2. Box plot comparison
    box_data = []
    labels = []
    if len(single_property_data) > 0:
        box_data.append(single_property_data)
        labels.append('Single Property')
    if len(bulk_data) > 0:
        box_data.append(bulk_data)
        labels.append('Bulk Recommendations')
    
    if box_data:
        axes[0, 1].boxplot(box_data, tick_labels=labels)
        axes[0, 1].set_ylabel('Response Time (ms)')
        axes[0, 1].set_title('Response Time Box Plot')
        axes[0, 1].grid(True, alpha=0.3)
    
    # 3. Response time over time with scenario coloring
    if 'scenario' in successful_df.columns:
        # Create color map for scenarios
        scenarios = successful_df['scenario'].unique()
        colors = plt.cm.Set3(np.linspace(0, 1, len(scenarios)))
        scenario_colors = {scenario: colors[i] for i, scenario in enumerate(scenarios)}
        
        point_colors = [scenario_colors.get(scenario, 'gray') for scenario in successful_df['scenario']]
        
        scatter = axes[1, 0].scatter(successful_df['timestamp'], successful_df['response_time_ms'], 
                                   c=point_colors, alpha=0.6, s=20)
        axes[1, 0].set_xlabel('Time')
        axes[1, 0].set_ylabel('Response Time (ms)')
        axes[1, 0].set_title('Response Time Over Time (by Scenario)')
        axes[1, 0].tick_params(axis='x', rotation=45)
        axes[1, 0].grid(True, alpha=0.3)
        
        # Add legend for scenarios
        for scenario, color in scenario_colors.items():
            if scenario != 'unknown':
                axes[1, 0].scatter([], [], c=[color], label=scenario.replace('_', ' ').title())
        axes[1, 0].legend(bbox_to_anchor=(1.05, 1), loc='upper left')
    else:
        axes[1, 0].scatter(successful_df['timestamp'], successful_df['response_time_ms'], 
                          alpha=0.6, s=20, color='blue')
        axes[1, 0].set_xlabel('Time')
        axes[1, 0].set_ylabel('Response Time (ms)')
        axes[1, 0].set_title('Response Time Over Time')
        axes[1, 0].tick_params(axis='x', rotation=45)
        axes[1, 0].grid(True, alpha=0.3)
    
    # 4. Success rate
    success_rate = df.groupby('endpoint')['success'].agg(['count', 'sum']).reset_index()
    success_rate['success_rate'] = success_rate['sum'] / success_rate['count'] * 100
    
    if len(success_rate) > 0:
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
    plt.close()
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
        if len(endpoint_data) > 0:
            percentile_data[endpoint] = [np.percentile(endpoint_data, p) for p in percentiles]
    
    x = np.arange(len(percentiles))
    width = 0.35
    
    colors = ['blue', 'red', 'green', 'orange', 'purple', 'cyan', 'magenta', 'yellow']
    for i, (endpoint, values) in enumerate(percentile_data.items()):
        axes[0].bar(x + i * width, values, width, label=endpoint, color=colors[i % len(colors)], alpha=0.7)
    
    axes[0].set_xlabel('Percentile')
    axes[0].set_ylabel('Response Time (ms)')
    axes[0].set_title('Response Time Percentiles')
    axes[0].set_xticks(x + width / 2)
    axes[0].set_xticklabels([f'P{p}' for p in percentiles])
    axes[0].legend()
    axes[0].grid(True, alpha=0.3)
    
    # Response size vs response time
    if 'response_size_bytes' in successful_df.columns:
        endpoint_colors = {'single_property': 'blue', 'bulk_recommendations': 'red'}
        point_colors = [endpoint_colors.get(endpoint, 'gray') for endpoint in successful_df['endpoint']]
        
        axes[1].scatter(successful_df['response_size_bytes'], successful_df['response_time_ms'],
                       c=point_colors, alpha=0.6, s=20)
        axes[1].set_xlabel('Response Size (bytes)')
        axes[1].set_ylabel('Response Time (ms)')
        axes[1].set_title('Response Time vs Response Size')
        axes[1].grid(True, alpha=0.3)
        
        # Add legend
        for endpoint, color in endpoint_colors.items():
            if endpoint in successful_df['endpoint'].values:
                axes[1].scatter([], [], c=color, label=endpoint.replace('_', ' ').title())
        axes[1].legend()
    
    plt.tight_layout()
    plt.savefig(f'{output_dir}/percentile_analysis.png', dpi=300, bbox_inches='tight')
    plt.close()
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
    plt.close()
    print(f"✓ Load analysis plot saved to {output_dir}/load_analysis.png")


def plot_k_value_analysis(df: pd.DataFrame, output_dir: str = "."):
    """Plot detailed analysis of K value performance."""
    successful_df = df[df['success'] == True]
    
    if len(successful_df) == 0:
        print("Warning: No successful requests found for K value analysis")
        return
    
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    fig.suptitle('K Value Performance Analysis', fontsize=16, fontweight='bold')
    
    # 1. Response time by K value - Single Property
    single_property_df = successful_df[successful_df['endpoint'] == 'single_property']
    k_values = [5, 10, 50, 100]
    
    if len(single_property_df) > 0:
        k_stats = []
        k_data_for_box = []
        k_labels = []
        
        for k in k_values:
            k_data = single_property_df[single_property_df['top_k_value'] == k]['response_time_ms']
            baseline_data = single_property_df[single_property_df['top_k_value'].isna()]['response_time_ms']
            
            if len(k_data) > 0:
                k_data_for_box.append(k_data)
                k_labels.append(f'K={k}')
                k_stats.append({
                    'k_value': k,
                    'mean': k_data.mean(),
                    'median': k_data.median(),
                    'std': k_data.std(),
                    'count': len(k_data),
                    'p95': np.percentile(k_data, 95)
                })
        
        # Add baseline (no K filtering)
        if len(baseline_data) > 0:
            k_data_for_box.append(baseline_data)
            k_labels.append('Baseline\n(No K)')
            k_stats.append({
                'k_value': 'baseline',
                'mean': baseline_data.mean(),
                'median': baseline_data.median(),
                'std': baseline_data.std(),
                'count': len(baseline_data),
                'p95': np.percentile(baseline_data, 95)
            })
        
        # Box plot
        if k_data_for_box:
            bp = axes[0, 0].boxplot(k_data_for_box, tick_labels=k_labels, patch_artist=True)
            
            # Color the boxes
            colors = ['lightblue', 'lightgreen', 'lightcoral', 'lightyellow', 'lightgray']
            for patch, color in zip(bp['boxes'], colors[:len(bp['boxes'])]):
                patch.set_facecolor(color)
            
            axes[0, 0].set_ylabel('Response Time (ms)')
            axes[0, 0].set_title('Single Property: Response Time by K Value')
            axes[0, 0].grid(True, alpha=0.3)
        
        # Bar chart with error bars
        if k_stats:
            k_df = pd.DataFrame([s for s in k_stats if isinstance(s['k_value'], int)])
            if len(k_df) > 0:
                x_pos = np.arange(len(k_df))
                bars = axes[0, 1].bar(x_pos, k_df['mean'], yerr=k_df['std'], 
                                     capsize=5, alpha=0.7, color='steelblue')
                axes[0, 1].set_xlabel('K Value')
                axes[0, 1].set_ylabel('Mean Response Time (ms)')
                axes[0, 1].set_title('Single Property: Mean Response Time vs K')
                axes[0, 1].set_xticks(x_pos)
                axes[0, 1].set_xticklabels([f'K={int(k)}' for k in k_df['k_value']])
                axes[0, 1].grid(True, alpha=0.3)
                
                # Add sample size labels
                for i, (bar, count) in enumerate(zip(bars, k_df['count'])):
                    height = bar.get_height()
                    axes[0, 1].text(bar.get_x() + bar.get_width()/2., height + k_df.iloc[i]['std'],
                                   f'n={int(count)}', ha='center', va='bottom', fontsize=8)
    
    # 2. Bulk recommendations K value analysis
    bulk_df = successful_df[successful_df['endpoint'] == 'bulk_recommendations']
    
    if len(bulk_df) > 0:
        bulk_k_data = []
        bulk_k_labels = []
        
        for k in k_values:
            k_data = bulk_df[bulk_df['top_k_value'] == k]['response_time_ms']
            if len(k_data) > 0:
                bulk_k_data.append(k_data)
                bulk_k_labels.append(f'K={k}')
        
        # Add baseline
        baseline_bulk = bulk_df[bulk_df['top_k_value'].isna()]['response_time_ms']
        if len(baseline_bulk) > 0:
            bulk_k_data.append(baseline_bulk)
            bulk_k_labels.append('Baseline')
        
        if bulk_k_data:
            bp = axes[1, 0].boxplot(bulk_k_data, tick_labels=bulk_k_labels, patch_artist=True)
            
            # Color the boxes
            colors = ['lightblue', 'lightgreen', 'lightcoral', 'lightyellow', 'lightgray']
            for patch, color in zip(bp['boxes'], colors[:len(bp['boxes'])]):
                patch.set_facecolor(color)
            
            axes[1, 0].set_ylabel('Response Time (ms)')
            axes[1, 0].set_title('Bulk Recommendations: Response Time by K Value')
            axes[1, 0].grid(True, alpha=0.3)
    else:
        axes[1, 0].text(0.5, 0.5, 'No bulk recommendation data', 
                       ha='center', va='center', transform=axes[1, 0].transAxes)
        axes[1, 0].set_title('Bulk Recommendations: Response Time by K Value')
    
    # 3. Performance improvement analysis
    if len(single_property_df) > 0:
        # Calculate percentage improvement over baseline
        baseline_mean = single_property_df[single_property_df['top_k_value'].isna()]['response_time_ms'].mean()
        
        improvements = []
        k_values_with_data = []
        
        for k in k_values:
            k_data = single_property_df[single_property_df['top_k_value'] == k]['response_time_ms']
            if len(k_data) > 0:
                k_mean = k_data.mean()
                improvement = ((baseline_mean - k_mean) / baseline_mean) * 100
                improvements.append(improvement)
                k_values_with_data.append(k)
        
        if improvements:
            bars = axes[1, 1].bar(range(len(improvements)), improvements, 
                                 color=['green' if x > 0 else 'red' for x in improvements],
                                 alpha=0.7)
            axes[1, 1].set_xlabel('K Value')
            axes[1, 1].set_ylabel('Performance Improvement (%)')
            axes[1, 1].set_title('Performance Improvement vs Baseline')
            axes[1, 1].set_xticks(range(len(improvements)))
            axes[1, 1].set_xticklabels([f'K={k}' for k in k_values_with_data])
            axes[1, 1].axhline(y=0, color='black', linestyle='-', alpha=0.5)
            axes[1, 1].grid(True, alpha=0.3)
            
            # Add percentage labels on bars
            for bar, improvement in zip(bars, improvements):
                height = bar.get_height()
                axes[1, 1].text(bar.get_x() + bar.get_width()/2., 
                               height + (1 if height > 0 else -1),
                               f'{improvement:.1f}%', ha='center', 
                               va='bottom' if height > 0 else 'top')
    else:
        axes[1, 1].text(0.5, 0.5, 'Insufficient data for improvement analysis', 
                       ha='center', va='center', transform=axes[1, 1].transAxes)
        axes[1, 1].set_title('Performance Improvement vs Baseline')
    
    plt.tight_layout()
    plt.savefig(os.path.join(output_dir, 'k_value_analysis.png'), dpi=300, bbox_inches='tight')
    plt.close()
    
    print(f"✓ K value analysis plot saved to {output_dir}/k_value_analysis.png")


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
        
        # Filtering scenario analysis
        if 'scenario' in successful_df.columns:
            scenarios = successful_df['scenario'].unique()
            if len(scenarios) > 1 and 'unknown' not in scenarios:
                report.append("FILTERING SCENARIO ANALYSIS")
                report.append("-" * 40)
                for scenario in scenarios:
                    if scenario != 'unknown':
                        scenario_data = successful_df[successful_df['scenario'] == scenario]
                        scenario_times = scenario_data['response_time_ms']
                        
                        report.append(f"{scenario.replace('_', ' ').title()}:")
                        report.append(f"  Tests: {len(scenario_data)}")
                        report.append(f"  Mean: {scenario_times.mean():.2f} ms")
                        report.append(f"  Median: {scenario_times.median():.2f} ms")
                        report.append(f"  95th Percentile: {np.percentile(scenario_times, 95):.2f} ms")
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
                       help='Directory to save plots (default: ./analysis)')
    
    args = parser.parse_args()
    
    # Create output directory if it doesn't exist
    os.makedirs(args.output_dir, exist_ok=True)
    
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
    
    print("Generating K value analysis...")
    plot_k_value_analysis(df, args.output_dir)  # New plot
    
    print("Generating filtering parameter analysis...")
    plot_filtering_analysis(df, args.output_dir)
    
    print("Generating parameter combination heatmap...")
    plot_performance_heatmap(df, args.output_dir)
    
    print("Generating summary report...")
    generate_summary_report(df, args.output_dir)
    
    print(f"\nAll plots and reports saved to {args.output_dir}/")
    print("\nGenerated files:")
    print("  - response_time_analysis.png: Basic response time analysis")
    print("  - percentile_analysis.png: Percentile and size analysis")
    print("  - load_analysis.png: Performance over time analysis")
    print("  - k_value_analysis.png: Detailed K value performance analysis")  # New
    print("  - filtering_analysis.png: Analysis of different filtering scenarios")
    print("  - parameter_heatmap.png: Heatmap of parameter combinations")
    print("  - latency_test_report.txt: Detailed summary report")


if __name__ == "__main__":
    main()
