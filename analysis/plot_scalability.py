#!/usr/bin/env python3
"""
Plot scalability test results.

This script reads the CSV output from scalability_test.py and creates
plots to visualize how performance changes with dataset size.
"""

import pandas as pd
import matplotlib
matplotlib.use('Agg')  # Use non-interactive backend
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np
from datetime import datetime
import argparse


def load_scalability_data(csv_file: str) -> pd.DataFrame:
    """Load scalability test data from CSV file."""
    try:
        df = pd.read_csv(csv_file)
        df['timestamp'] = pd.to_datetime(df['timestamp'])
        return df
    except FileNotFoundError:
        print(f"Error: File {csv_file} not found.")
        print("Run scalability_test.py first to generate test data.")
        exit(1)
    except Exception as e:
        print(f"Error loading data: {e}")
        exit(1)


def plot_response_time_vs_dataset_size(df: pd.DataFrame, output_dir: str = "."):
    """Plot how response time changes with dataset size."""
    successful_df = df[df['success'] == True]
    
    if len(successful_df) == 0:
        print("No successful results to plot.")
        return
    
    # Separate single contact and bulk results
    single_contact_df = successful_df[successful_df['endpoint'] == 'single_contact']
    bulk_df = successful_df[successful_df['endpoint'].str.startswith('bulk_recommendations')]
    
    # Group by dataset size and calculate statistics
    single_grouped = single_contact_df.groupby(['total_contacts', 'total_properties']).agg({
        'response_time_ms': ['mean', 'median', 'std', 'min', 'max', 'count'],
        'recommendations_count': 'mean'
    }).reset_index()
    
    # Flatten column names
    single_grouped.columns = ['total_contacts', 'total_properties', 'mean_time', 'median_time', 
                             'std_time', 'min_time', 'max_time', 'test_count', 'avg_recommendations']
    
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    fig.suptitle('Response Time vs Dataset Size Analysis', fontsize=16, fontweight='bold')
    
    # 1. Single contact response time vs number of contacts
    if len(single_grouped) > 0:
        axes[0, 0].errorbar(single_grouped['total_contacts'], single_grouped['mean_time'], 
                           yerr=single_grouped['std_time'], marker='o', capsize=5, capthick=2,
                           label='Single Contact', color='blue')
    
    # Add bulk results if available
    if len(bulk_df) > 0:
        # Group bulk results by endpoint type and dataset size
        bulk_grouped = bulk_df.groupby(['total_contacts', 'total_properties', 'endpoint']).agg({
            'response_time_ms': 'mean'
        }).reset_index()
        
        # Plot different bulk sizes with different colors
        colors = ['red', 'green', 'orange', 'purple', 'brown']
        bulk_types = bulk_grouped['endpoint'].unique()
        
        for i, bulk_type in enumerate(bulk_types):
            bulk_subset = bulk_grouped[bulk_grouped['endpoint'] == bulk_type]
            # Extract bulk size from endpoint name
            bulk_size = bulk_type.split('_')[-1] if '_' in bulk_type else "Unknown"
            color = colors[i % len(colors)]
            
            axes[0, 0].plot(bulk_subset['total_contacts'], bulk_subset['response_time_ms'],
                           marker='s', linewidth=2, label=f'Bulk {bulk_size}', color=color)
    
    axes[0, 0].set_xlabel('Number of Contacts')
    axes[0, 0].set_ylabel('Average Response Time (ms)')
    axes[0, 0].set_title('Response Time vs Number of Contacts')
    axes[0, 0].legend()
    axes[0, 0].grid(True, alpha=0.3)
    
    # 2. Response time vs number of properties (single contact only)
    if len(single_grouped) > 0:
        axes[0, 1].errorbar(single_grouped['total_properties'], single_grouped['mean_time'], 
                           yerr=single_grouped['std_time'], marker='s', capsize=5, capthick=2, color='blue')
        axes[0, 1].set_xlabel('Number of Properties')
        axes[0, 1].set_ylabel('Average Response Time (ms)')
        axes[0, 1].set_title('Single Contact: Response Time vs Properties')
        axes[0, 1].grid(True, alpha=0.3)
    
    # 3. Response time vs total dataset size (contacts + properties)
    if len(single_grouped) > 0:
        single_grouped['total_records'] = single_grouped['total_contacts'] + single_grouped['total_properties']
        axes[1, 0].scatter(single_grouped['total_records'], single_grouped['mean_time'], 
                          s=100, alpha=0.7, c=single_grouped['total_contacts'], cmap='viridis', label='Single Contact')
        
        # Add colorbar for the scatter plot
        scatter = axes[1, 0].collections[0] if axes[1, 0].collections else None
        if scatter:
            cbar = plt.colorbar(scatter, ax=axes[1, 0])
            cbar.set_label('Number of Contacts')
    
    # Add bulk results to the total dataset plot
    if len(bulk_df) > 0:
        bulk_summary = bulk_df.groupby(['total_contacts', 'total_properties']).agg({
            'response_time_ms': 'mean'
        }).reset_index()
        bulk_summary['total_records'] = bulk_summary['total_contacts'] + bulk_summary['total_properties']
        
        axes[1, 0].scatter(bulk_summary['total_records'], bulk_summary['response_time_ms'], 
                          s=100, alpha=0.7, marker='^', color='red', label='Bulk Average')
    
    axes[1, 0].set_xlabel('Total Records (Contacts + Properties)')
    axes[1, 0].set_ylabel('Average Response Time (ms)')
    axes[1, 0].set_title('Response Time vs Total Dataset Size')
    axes[1, 0].legend()
    axes[1, 0].grid(True, alpha=0.3)
    
    # 4. Bulk efficiency analysis
    if len(bulk_df) > 0 and len(single_grouped) > 0:
        # Calculate bulk efficiency compared to single requests
        bulk_efficiency = []
        
        for _, bulk_row in bulk_df.iterrows():
            # Find corresponding single contact performance for same dataset size
            matching_single = single_grouped[
                (single_grouped['total_contacts'] == bulk_row['total_contacts']) &
                (single_grouped['total_properties'] == bulk_row['total_properties'])
            ]
            
            if not matching_single.empty:
                single_time = matching_single['mean_time'].iloc[0]
                bulk_size = int(bulk_row['endpoint'].split('_')[-1]) if '_' in bulk_row['endpoint'] else 1
                expected_time = single_time * bulk_size
                efficiency = bulk_row['response_time_ms'] / expected_time
                
                bulk_efficiency.append({
                    'bulk_size': bulk_size,
                    'efficiency': efficiency,
                    'total_contacts': bulk_row['total_contacts']
                })
        
        if bulk_efficiency:
            efficiency_df = pd.DataFrame(bulk_efficiency)
            
            for bulk_size in efficiency_df['bulk_size'].unique():
                size_data = efficiency_df[efficiency_df['bulk_size'] == bulk_size]
                axes[1, 1].plot(size_data['total_contacts'], size_data['efficiency'],
                               marker='o', label=f'Bulk {bulk_size}')
            
            axes[1, 1].axhline(y=1.0, color='black', linestyle='--', alpha=0.5, label='No optimization')
            axes[1, 1].set_xlabel('Number of Contacts')
            axes[1, 1].set_ylabel('Efficiency (Bulk Time / Expected Time)')
            axes[1, 1].set_title('Bulk Request Efficiency')
            axes[1, 1].legend()
            axes[1, 1].grid(True, alpha=0.3)
    else:
        # If no bulk data, show recommendations count
        if len(single_grouped) > 0:
            axes[1, 1].scatter(single_grouped['total_contacts'], single_grouped['avg_recommendations'], 
                              s=100, alpha=0.7, color='green')
            axes[1, 1].set_xlabel('Number of Contacts')
            axes[1, 1].set_ylabel('Average Recommendations Returned')
            axes[1, 1].set_title('Recommendations Count vs Dataset Size')
            axes[1, 1].grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig(f'{output_dir}/scalability_response_time.png', dpi=300, bbox_inches='tight')
    plt.close()
    print(f"✓ Response time scalability plot saved to {output_dir}/scalability_response_time.png")


def plot_performance_trends(df: pd.DataFrame, output_dir: str = "."):
    """Plot performance trends and variability."""
    successful_df = df[df['success'] == True]
    
    if len(successful_df) == 0:
        print("No successful results to plot.")
        return
    
    # Group by dataset size
    grouped = successful_df.groupby(['total_contacts', 'total_properties']).agg({
        'response_time_ms': ['mean', 'std', 'min', 'max'],
        'response_size_bytes': 'mean'
    }).reset_index()
    
    grouped.columns = ['total_contacts', 'total_properties', 'mean_time', 'std_time', 
                      'min_time', 'max_time', 'avg_response_size']
    grouped['cv'] = grouped['std_time'] / grouped['mean_time']  # Coefficient of variation
    
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    fig.suptitle('Performance Trends and Variability', fontsize=16, fontweight='bold')
    
    # 1. Response time variability (coefficient of variation)
    axes[0, 0].plot(grouped['total_contacts'], grouped['cv'], marker='o', linewidth=2)
    axes[0, 0].set_xlabel('Number of Contacts')
    axes[0, 0].set_ylabel('Coefficient of Variation (std/mean)')
    axes[0, 0].set_title('Response Time Variability')
    axes[0, 0].grid(True, alpha=0.3)
    
    # 2. Min/Max response times
    axes[0, 1].plot(grouped['total_contacts'], grouped['min_time'], 
                   marker='v', label='Min', color='green')
    axes[0, 1].plot(grouped['total_contacts'], grouped['max_time'], 
                   marker='^', label='Max', color='red')
    axes[0, 1].plot(grouped['total_contacts'], grouped['mean_time'], 
                   marker='o', label='Mean', color='blue')
    axes[0, 1].set_xlabel('Number of Contacts')
    axes[0, 1].set_ylabel('Response Time (ms)')
    axes[0, 1].set_title('Response Time Range')
    axes[0, 1].legend()
    axes[0, 1].grid(True, alpha=0.3)
    
    # 3. Response size vs response time
    axes[1, 0].scatter(grouped['avg_response_size'], grouped['mean_time'], 
                      s=100, alpha=0.7, c=grouped['total_contacts'], cmap='plasma')
    axes[1, 0].set_xlabel('Average Response Size (bytes)')
    axes[1, 0].set_ylabel('Average Response Time (ms)')
    axes[1, 0].set_title('Response Size vs Response Time')
    axes[1, 0].grid(True, alpha=0.3)
    
    # 4. Performance degradation rate
    if len(grouped) > 1:
        # Calculate percentage increase in response time
        baseline_time = grouped['mean_time'].iloc[0]
        grouped['performance_degradation'] = ((grouped['mean_time'] - baseline_time) / baseline_time) * 100
        
        axes[1, 1].plot(grouped['total_contacts'], grouped['performance_degradation'], 
                       marker='o', linewidth=2, color='orange')
        axes[1, 1].set_xlabel('Number of Contacts')
        axes[1, 1].set_ylabel('Performance Degradation (%)')
        axes[1, 1].set_title('Performance Degradation vs Baseline')
        axes[1, 1].grid(True, alpha=0.3)
        axes[1, 1].axhline(y=0, color='black', linestyle='--', alpha=0.5)
    
    plt.tight_layout()
    plt.savefig(f'{output_dir}/scalability_trends.png', dpi=300, bbox_inches='tight')
    plt.close()
    print(f"✓ Performance trends plot saved to {output_dir}/scalability_trends.png")


def plot_scalability_heatmap(df: pd.DataFrame, output_dir: str = "."):
    """Create a heatmap showing response times across different dataset sizes."""
    successful_df = df[df['success'] == True]
    
    if len(successful_df) == 0:
        print("No successful results to plot.")
        return
    
    # Create pivot table for heatmap
    pivot_data = successful_df.groupby(['total_contacts', 'total_properties'])['response_time_ms'].mean().reset_index()
    pivot_table = pivot_data.pivot(index='total_properties', columns='total_contacts', values='response_time_ms')
    
    plt.figure(figsize=(12, 8))
    
    # Create heatmap
    sns.heatmap(pivot_table, annot=True, fmt='.1f', cmap='YlOrRd', 
                cbar_kws={'label': 'Average Response Time (ms)'})
    
    plt.title('Response Time Heatmap: Contacts vs Properties', fontsize=14, fontweight='bold')
    plt.xlabel('Number of Contacts')
    plt.ylabel('Number of Properties')
    
    plt.tight_layout()
    plt.savefig(f'{output_dir}/scalability_heatmap.png', dpi=300, bbox_inches='tight')
    plt.close()
    print(f"✓ Scalability heatmap saved to {output_dir}/scalability_heatmap.png")


def plot_throughput_analysis(df: pd.DataFrame, output_dir: str = "."):
    """Analyze throughput (requests per second equivalent)."""
    successful_df = df[df['success'] == True]
    
    if len(successful_df) == 0:
        print("No successful results to plot.")
        return
    
    # Calculate theoretical throughput (1000ms / avg_response_time)
    grouped = successful_df.groupby(['total_contacts', 'total_properties']).agg({
        'response_time_ms': 'mean',
        'recommendations_count': 'mean',
        'response_size_bytes': 'mean'
    }).reset_index()
    
    grouped['theoretical_throughput'] = 1000 / grouped['response_time_ms']  # requests per second
    grouped['recommendations_per_second'] = grouped['theoretical_throughput'] * grouped['recommendations_count']
    
    fig, axes = plt.subplots(2, 2, figsize=(16, 12))
    fig.suptitle('Throughput and Efficiency Analysis', fontsize=16, fontweight='bold')
    
    # 1. Theoretical throughput vs dataset size
    axes[0, 0].plot(grouped['total_contacts'], grouped['theoretical_throughput'], 
                   marker='o', linewidth=2, color='blue')
    axes[0, 0].set_xlabel('Number of Contacts')
    axes[0, 0].set_ylabel('Theoretical Throughput (req/s)')
    axes[0, 0].set_title('Throughput vs Dataset Size')
    axes[0, 0].grid(True, alpha=0.3)
    
    # 2. Recommendations per second
    axes[0, 1].plot(grouped['total_contacts'], grouped['recommendations_per_second'], 
                   marker='s', linewidth=2, color='green')
    axes[0, 1].set_xlabel('Number of Contacts')
    axes[0, 1].set_ylabel('Recommendations per Second')
    axes[0, 1].set_title('Recommendation Throughput')
    axes[0, 1].grid(True, alpha=0.3)
    
    # 3. Efficiency: response time per recommendation
    grouped['time_per_recommendation'] = grouped['response_time_ms'] / grouped['recommendations_count']
    axes[1, 0].plot(grouped['total_contacts'], grouped['time_per_recommendation'], 
                   marker='D', linewidth=2, color='purple')
    axes[1, 0].set_xlabel('Number of Contacts')
    axes[1, 0].set_ylabel('Time per Recommendation (ms)')
    axes[1, 0].set_title('Processing Efficiency')
    axes[1, 0].grid(True, alpha=0.3)
    
    # 4. Data transfer efficiency: response time per byte
    grouped['time_per_byte'] = grouped['response_time_ms'] / grouped['response_size_bytes'] * 1000  # microseconds per byte
    axes[1, 1].plot(grouped['total_contacts'], grouped['time_per_byte'], 
                   marker='*', linewidth=2, color='orange')
    axes[1, 1].set_xlabel('Number of Contacts')
    axes[1, 1].set_ylabel('Time per Byte (μs/byte)')
    axes[1, 1].set_title('Data Transfer Efficiency')
    axes[1, 1].grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig(f'{output_dir}/scalability_throughput.png', dpi=300, bbox_inches='tight')
    plt.close()
    print(f"✓ Throughput analysis plot saved to {output_dir}/scalability_throughput.png")


def generate_scalability_report(df: pd.DataFrame, output_dir: str = "."):
    """Generate a comprehensive scalability report."""
    successful_df = df[df['success'] == True]
    
    report = []
    report.append("SCALABILITY TEST REPORT")
    report.append("=" * 60)
    report.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    report.append(f"Total tests: {len(df)}")
    report.append(f"Successful tests: {len(successful_df)} ({len(successful_df)/len(df)*100:.1f}%)")
    report.append("")
    
    if len(successful_df) > 0:
        # Separate single contact and bulk results
        single_contact_df = successful_df[successful_df['endpoint'] == 'single_contact']
        bulk_df = successful_df[successful_df['endpoint'].str.startswith('bulk_recommendations')]
        
        # Single contact performance summary
        if len(single_contact_df) > 0:
            report.append("SINGLE CONTACT PERFORMANCE SUMMARY")
            report.append("-" * 40)
            
            single_grouped = single_contact_df.groupby(['total_contacts', 'total_properties']).agg({
                'response_time_ms': ['mean', 'std'],
                'recommendations_count': 'mean'
            }).reset_index()
            
            single_grouped.columns = ['total_contacts', 'total_properties', 'mean_time', 'std_time', 'avg_recommendations']
            
            baseline_time = single_grouped['mean_time'].iloc[0]
            final_time = single_grouped['mean_time'].iloc[-1]
            degradation = ((final_time - baseline_time) / baseline_time) * 100
            
            report.append(f"Baseline response time: {baseline_time:.2f} ms")
            report.append(f"Final response time: {final_time:.2f} ms")
            report.append(f"Performance degradation: {degradation:.1f}%")
            report.append("")
            
            # Dataset size progression for single contacts
            report.append("SINGLE CONTACT DATASET PROGRESSION")
            report.append("-" * 40)
            report.append(f"{'Contacts':<10} {'Properties':<12} {'Avg Time':<12} {'Recommendations':<15}")
            report.append("-" * 50)
            
            for _, row in single_grouped.iterrows():
                report.append(f"{int(row['total_contacts']):<10} {int(row['total_properties']):<12} "
                             f"{row['mean_time']:<12.2f} {row['avg_recommendations']:<15.1f}")
            
            report.append("")
        
        # Bulk performance summary
        if len(bulk_df) > 0:
            report.append("BULK RECOMMENDATIONS PERFORMANCE SUMMARY")
            report.append("-" * 40)
            
            bulk_grouped = bulk_df.groupby(['total_contacts', 'total_properties', 'endpoint']).agg({
                'response_time_ms': 'mean',
                'recommendations_count': 'mean'
            }).reset_index()
            
            # Show bulk performance by batch size
            bulk_types = bulk_grouped['endpoint'].unique()
            for bulk_type in sorted(bulk_types):
                bulk_subset = bulk_grouped[bulk_grouped['endpoint'] == bulk_type]
                bulk_size = bulk_type.split('_')[-1] if '_' in bulk_type else "Unknown"
                
                if len(bulk_subset) > 0:
                    avg_time = bulk_subset['response_time_ms'].mean()
                    avg_recommendations = bulk_subset['recommendations_count'].mean()
                    
                    report.append(f"Bulk size {bulk_size}:")
                    report.append(f"  Average response time: {avg_time:.2f} ms")
                    report.append(f"  Average recommendations: {avg_recommendations:.1f}")
            
            report.append("")
            
            # Bulk efficiency analysis
            if len(single_contact_df) > 0:
                report.append("BULK EFFICIENCY ANALYSIS")
                report.append("-" * 40)
                
                efficiency_results = []
                
                for _, bulk_row in bulk_df.iterrows():
                    # Find corresponding single contact performance
                    matching_single = single_contact_df[
                        (single_contact_df['total_contacts'] == bulk_row['total_contacts']) &
                        (single_contact_df['total_properties'] == bulk_row['total_properties'])
                    ]
                    
                    if not matching_single.empty:
                        single_time = matching_single['response_time_ms'].mean()
                        bulk_size = int(bulk_row['endpoint'].split('_')[-1]) if '_' in bulk_row['endpoint'] else 1
                        expected_time = single_time * bulk_size
                        efficiency = bulk_row['response_time_ms'] / expected_time
                        
                        efficiency_results.append({
                            'bulk_size': bulk_size,
                            'efficiency': efficiency,
                            'bulk_time': bulk_row['response_time_ms'],
                            'expected_time': expected_time
                        })
                
                if efficiency_results:
                    efficiency_df = pd.DataFrame(efficiency_results)
                    
                    report.append(f"{'Bulk Size':<10} {'Efficiency':<12} {'Status':<15}")
                    report.append("-" * 40)
                    
                    for bulk_size in sorted(efficiency_df['bulk_size'].unique()):
                        size_data = efficiency_df[efficiency_df['bulk_size'] == bulk_size]
                        avg_efficiency = size_data['efficiency'].mean()
                        
                        if avg_efficiency < 0.8:
                            status = "Excellent"
                        elif avg_efficiency < 1.0:
                            status = "Good"
                        elif avg_efficiency < 1.5:
                            status = "Acceptable"
                        else:
                            status = "Poor"
                        
                        report.append(f"{bulk_size:<10} {avg_efficiency:<12.3f} {status:<15}")
                    
                    report.append("")
                    report.append("Efficiency < 0.8: Excellent bulk optimization")
                    report.append("Efficiency < 1.0: Good bulk optimization")
                    report.append("Efficiency > 1.5: Poor bulk optimization - needs improvement")
                    report.append("")
        
        # Overall scalability insights
        report.append("SCALABILITY INSIGHTS")
        report.append("-" * 40)
        
        if len(single_contact_df) > 0:
            single_grouped = single_contact_df.groupby(['total_contacts', 'total_properties']).agg({
                'response_time_ms': 'mean'
            }).reset_index()
            
            if len(single_grouped) > 1:
                contacts_growth = single_grouped['total_contacts'].iloc[-1] / single_grouped['total_contacts'].iloc[0]
                time_growth = single_grouped['response_time_ms'].iloc[-1] / single_grouped['response_time_ms'].iloc[0]
                
                if contacts_growth > 1:
                    scalability_factor = time_growth / contacts_growth
                    report.append(f"Dataset grew by: {contacts_growth:.1f}x")
                    report.append(f"Response time grew by: {time_growth:.1f}x")
                    report.append(f"Scalability factor: {scalability_factor:.2f}")
                    
                    if scalability_factor < 1.2:
                        assessment = "Excellent scalability"
                    elif scalability_factor < 1.5:
                        assessment = "Good scalability"
                    elif scalability_factor < 2.0:
                        assessment = "Moderate scalability"
                    else:
                        assessment = "Poor scalability - optimization needed"
                    
                    report.append(f"Assessment: {assessment}")
        
        if len(bulk_df) > 0:
            report.append("")
            report.append("BULK RECOMMENDATIONS INSIGHTS")
            report.append("-" * 40)
            
            bulk_sizes = []
            for endpoint in bulk_df['endpoint'].unique():
                if '_' in endpoint:
                    try:
                        size = int(endpoint.split('_')[-1])
                        bulk_sizes.append(size)
                    except ValueError:
                        pass
            
            if bulk_sizes:
                report.append(f"Tested bulk sizes: {sorted(set(bulk_sizes))}")
                report.append("Bulk operations can significantly improve throughput")
                report.append("for scenarios requiring multiple recommendations.")
    
    # Save report
    with open(f'{output_dir}/scalability_report.txt', 'w') as f:
        f.write('\n'.join(report))
    
    print(f"✓ Scalability report saved to {output_dir}/scalability_report.txt")
    print('\n'.join(report))


def main():
    parser = argparse.ArgumentParser(description='Plot scalability test results')
    parser.add_argument('csv_file', default='scalability_results.csv', nargs='?',
                       help='CSV file with scalability test results')
    parser.add_argument('--output-dir', default='.',
                       help='Directory to save plots (default: current directory)')
    
    args = parser.parse_args()
    
    # Load data
    df = load_scalability_data(args.csv_file)
    print(f"Loaded {len(df)} scalability test results from {args.csv_file}")
    
    # Generate plots
    print("Generating response time vs dataset size plots...")
    plot_response_time_vs_dataset_size(df, args.output_dir)
    
    print("Generating performance trends plots...")
    plot_performance_trends(df, args.output_dir)
    
    print("Generating scalability heatmap...")
    plot_scalability_heatmap(df, args.output_dir)
    
    print("Generating throughput analysis...")
    plot_throughput_analysis(df, args.output_dir)
    
    print("Generating scalability report...")
    generate_scalability_report(df, args.output_dir)
    
    print(f"\nAll scalability plots and reports saved to {args.output_dir}/")


if __name__ == "__main__":
    main()
