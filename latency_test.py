#!/usr/bin/env python3
"""
Real Estate Recommendation System - API Latency Testing Script

This script tests the latency of all API endpoints and stores the results in CSV format
for further analysis.
"""

import requests
import time
import csv
import json
import os
import sys
import statistics
from datetime import datetime
from typing import List, Dict, Any, Optional
import psycopg2
from urllib.parse import urlparse
import argparse


class LatencyTester:
    def __init__(self, base_url: str = "http://localhost:8080", database_url: str = None):
        self.base_url = base_url.rstrip('/')
        self.database_url = database_url
        self.session = requests.Session()
        self.results = []
        self.property_ids = []
        self.contact_ids = []
        
    def load_sample_data(self):
        """Load sample property and contact IDs from the database."""
        if not self.database_url:
            print("❌ DATABASE_URL not provided. Using mock data.")
            # Use mock UUIDs for testing when DB is not available
            self.property_ids = [
                "550e8400-e29b-41d4-a716-446655440000",
                "550e8400-e29b-41d4-a716-446655440001",
                "550e8400-e29b-41d4-a716-446655440002"
            ]
            self.contact_ids = [
                "660e8400-e29b-41d4-a716-446655440000",
                "660e8400-e29b-41d4-a716-446655440001"
            ]
            return
            
        try:
            conn = psycopg2.connect(self.database_url)
            cursor = conn.cursor()
            
            # Get property IDs
            cursor.execute("SELECT id FROM properties WHERE is_active = true LIMIT 5;")
            self.property_ids = [str(row[0]) for row in cursor.fetchall()]
            
            # Get contact IDs
            cursor.execute("SELECT id FROM contacts WHERE is_active = true LIMIT 3;")
            self.contact_ids = [str(row[0]) for row in cursor.fetchall()]
            
            cursor.close()
            conn.close()
            
            print(f"✅ Loaded {len(self.property_ids)} properties and {len(self.contact_ids)} contacts")
            
        except Exception as e:
            print(f"⚠️  Database connection failed: {e}")
            print("Using mock data for testing.")
            self.load_sample_data()  # Fallback to mock data

    def measure_latency(self, method: str, url: str, **kwargs) -> Dict[str, Any]:
        """Measure the latency of a single HTTP request."""
        start_time = time.time()
        
        try:
            if method.upper() == 'GET':
                response = self.session.get(url, **kwargs)
            elif method.upper() == 'POST':
                response = self.session.post(url, **kwargs)
            else:
                raise ValueError(f"Unsupported HTTP method: {method}")
            
            end_time = time.time()
            latency = (end_time - start_time) * 1000  # Convert to milliseconds
            
            return {
                'success': True,
                'latency_ms': latency,
                'status_code': response.status_code,
                'response_size': len(response.content),
                'error': None
            }
            
        except Exception as e:
            end_time = time.time()
            latency = (end_time - start_time) * 1000
            
            return {
                'success': False,
                'latency_ms': latency,
                'status_code': None,
                'response_size': 0,
                'error': str(e)
            }

    def test_health_endpoint(self, iterations: int = 10):
        """Test the health check endpoint."""
        print("🔍 Testing health endpoint...")
        endpoint_name = "health"
        url = f"{self.base_url}/health"
        
        for i in range(iterations):
            result = self.measure_latency('GET', url)
            self.results.append({
                'timestamp': datetime.now().isoformat(),
                'endpoint': endpoint_name,
                'method': 'GET',
                'iteration': i + 1,
                'url': url,
                **result
            })
            time.sleep(0.1)  # Small delay between requests

    def test_recommendations_endpoint(self, iterations: int = 10):
        """Test the recommendations endpoint."""
        if not self.property_ids:
            print("⚠️  No property IDs available, skipping recommendations test")
            return
            
        print("🎯 Testing recommendations endpoint...")
        endpoint_name = "recommendations"
        
        for i in range(iterations):
            property_id = self.property_ids[i % len(self.property_ids)]
            url = f"{self.base_url}/recommendations/property/{property_id}"
            params = {'limit': 3, 'min_score': 0.3}
            
            result = self.measure_latency('GET', url, params=params)
            self.results.append({
                'timestamp': datetime.now().isoformat(),
                'endpoint': endpoint_name,
                'method': 'GET',
                'iteration': i + 1,
                'url': f"{url}?limit=3&min_score=0.3",
                **result
            })
            time.sleep(0.1)

    def test_bulk_recommendations_endpoint(self, iterations: int = 5):
        """Test the bulk recommendations endpoint."""
        print("📦 Testing bulk recommendations endpoint...")
        endpoint_name = "bulk_recommendations"
        url = f"{self.base_url}/recommendations/bulk"
        
        payload = {
            "limit_per_property": 2,
            "min_score": 0.2
        }
        
        for i in range(iterations):
            result = self.measure_latency(
                'POST', 
                url, 
                json=payload,
                headers={'Content-Type': 'application/json'}
            )
            self.results.append({
                'timestamp': datetime.now().isoformat(),
                'endpoint': endpoint_name,
                'method': 'POST',
                'iteration': i + 1,
                'url': url,
                **result
            })
            time.sleep(0.2)  # Slightly longer delay for bulk operations

    def test_comparison_endpoint(self, iterations: int = 10):
        """Test the property comparison endpoint."""
        if len(self.property_ids) < 2:
            print("⚠️  Need at least 2 property IDs, skipping comparison test")
            return
            
        print("🔍 Testing comparison endpoint...")
        endpoint_name = "comparison"
        url = f"{self.base_url}/comparisons/properties"
        
        for i in range(iterations):
            prop1_id = self.property_ids[i % len(self.property_ids)]
            prop2_id = self.property_ids[(i + 1) % len(self.property_ids)]
            params = {
                'property1_id': prop1_id,
                'property2_id': prop2_id
            }
            
            result = self.measure_latency('GET', url, params=params)
            self.results.append({
                'timestamp': datetime.now().isoformat(),
                'endpoint': endpoint_name,
                'method': 'GET',
                'iteration': i + 1,
                'url': f"{url}?property1_id={prop1_id}&property2_id={prop2_id}",
                **result
            })
            time.sleep(0.1)

    def test_quote_generation_endpoint(self, iterations: int = 3):
        """Test the quote generation endpoint."""
        if not self.property_ids or not self.contact_ids:
            print("⚠️  Need property and contact IDs, skipping quote generation test")
            return
            
        print("📄 Testing quote generation endpoint...")
        endpoint_name = "quote_generation"
        url = f"{self.base_url}/quotes/generate"
        
        for i in range(iterations):
            property_id = self.property_ids[i % len(self.property_ids)]
            contact_id = self.contact_ids[i % len(self.contact_ids)]
            
            payload = {
                "property_id": property_id,
                "contact_id": contact_id,
                "additional_costs": [
                    {"description": "Legal Fees", "amount": 150000},
                    {"description": "Property Inspection", "amount": 50000}
                ],
                "custom_message": "Thank you for your interest in this property"
            }
            
            result = self.measure_latency(
                'POST',
                url,
                json=payload,
                headers={'Content-Type': 'application/json'}
            )
            self.results.append({
                'timestamp': datetime.now().isoformat(),
                'endpoint': endpoint_name,
                'method': 'POST',
                'iteration': i + 1,
                'url': url,
                **result
            })
            time.sleep(0.5)  # Longer delay for PDF generation

    def test_comparison_quote_endpoint(self, iterations: int = 3):
        """Test the comparison quote generation endpoint."""
        if len(self.property_ids) < 2 or not self.contact_ids:
            print("⚠️  Need at least 2 properties and 1 contact, skipping comparison quote test")
            return
            
        print("📊 Testing comparison quote endpoint...")
        endpoint_name = "comparison_quote"
        url = f"{self.base_url}/quotes/comparison"
        
        for i in range(iterations):
            prop1_id = self.property_ids[i % len(self.property_ids)]
            prop2_id = self.property_ids[(i + 1) % len(self.property_ids)]
            contact_id = self.contact_ids[i % len(self.contact_ids)]
            
            payload = {
                "property1_id": prop1_id,
                "property2_id": prop2_id,
                "contact_id": contact_id,
                "custom_message": "Property comparison report"
            }
            
            result = self.measure_latency(
                'POST',
                url,
                json=payload,
                headers={'Content-Type': 'application/json'}
            )
            self.results.append({
                'timestamp': datetime.now().isoformat(),
                'endpoint': endpoint_name,
                'method': 'POST',
                'iteration': i + 1,
                'url': url,
                **result
            })
            time.sleep(0.5)

    def test_recommendation_quote_endpoint(self, iterations: int = 3):
        """Test the recommendation quote generation endpoint."""
        if not self.property_ids:
            print("⚠️  No property IDs available, skipping recommendation quote test")
            return
            
        print("📈 Testing recommendation quote endpoint...")
        endpoint_name = "recommendation_quote"
        
        for i in range(iterations):
            property_id = self.property_ids[i % len(self.property_ids)]
            url = f"{self.base_url}/quotes/recommendations"
            params = {'property_id': property_id}
            
            result = self.measure_latency('GET', url, params=params)
            self.results.append({
                'timestamp': datetime.now().isoformat(),
                'endpoint': endpoint_name,
                'method': 'GET',
                'iteration': i + 1,
                'url': f"{url}?property_id={property_id}",
                **result
            })
            time.sleep(0.5)

    def run_all_tests(self, iterations: int = 10):
        """Run all latency tests."""
        print("🚀 Starting comprehensive latency testing...")
        print(f"Base URL: {self.base_url}")
        print(f"Iterations per endpoint: {iterations}")
        print("=" * 60)
        
        # Load sample data first
        self.load_sample_data()
        
        # Run all tests
        test_methods = [
            (self.test_health_endpoint, iterations),
            (self.test_recommendations_endpoint, iterations),
            (self.test_bulk_recommendations_endpoint, max(1, iterations // 2)),
            (self.test_comparison_endpoint, iterations),
            (self.test_quote_generation_endpoint, max(1, iterations // 3)),
            (self.test_comparison_quote_endpoint, max(1, iterations // 3)),
            (self.test_recommendation_quote_endpoint, max(1, iterations // 3)),
        ]
        
        for test_method, test_iterations in test_methods:
            try:
                test_method(test_iterations)
            except Exception as e:
                print(f"❌ Error in {test_method.__name__}: {e}")
            
        print("\n✅ All tests completed!")

    def save_results_to_csv(self, filename: str = None):
        """Save the latency test results to a CSV file."""
        if not filename:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"analysis/latency_test_results_{timestamp}.csv"
        
        # Ensure analysis directory exists
        os.makedirs("analysis", exist_ok=True)
        
        if not self.results:
            print("❌ No results to save")
            return
        
        fieldnames = [
            'timestamp', 'endpoint', 'method', 'iteration', 'url',
            'success', 'latency_ms', 'status_code', 'response_size', 'error'
        ]
        
        with open(filename, 'w', newline='', encoding='utf-8') as csvfile:
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(self.results)
        
        print(f"📊 Results saved to: {filename}")
        return filename

    def generate_summary_report(self):
        """Generate a summary report of the latency test results."""
        if not self.results:
            print("❌ No results available for summary")
            return
        
        print("\n" + "=" * 60)
        print("📊 LATENCY TEST SUMMARY REPORT")
        print("=" * 60)
        
        # Group results by endpoint
        endpoint_stats = {}
        
        for result in self.results:
            endpoint = result['endpoint']
            if endpoint not in endpoint_stats:
                endpoint_stats[endpoint] = {
                    'latencies': [],
                    'successes': 0,
                    'failures': 0,
                    'total_requests': 0
                }
            
            stats = endpoint_stats[endpoint]
            stats['total_requests'] += 1
            
            if result['success']:
                stats['successes'] += 1
                stats['latencies'].append(result['latency_ms'])
            else:
                stats['failures'] += 1
        
        # Print summary for each endpoint
        for endpoint, stats in endpoint_stats.items():
            print(f"\n🎯 {endpoint.upper()}")
            print("-" * 40)
            print(f"Total Requests: {stats['total_requests']}")
            print(f"Successful: {stats['successes']}")
            print(f"Failed: {stats['failures']}")
            print(f"Success Rate: {(stats['successes'] / stats['total_requests']) * 100:.1f}%")
            
            if stats['latencies']:
                latencies = stats['latencies']
                print(f"Average Latency: {statistics.mean(latencies):.2f} ms")
                print(f"Median Latency: {statistics.median(latencies):.2f} ms")
                print(f"Min Latency: {min(latencies):.2f} ms")
                print(f"Max Latency: {max(latencies):.2f} ms")
                if len(latencies) > 1:
                    print(f"Std Deviation: {statistics.stdev(latencies):.2f} ms")
        
        # Overall statistics
        all_latencies = [r['latency_ms'] for r in self.results if r['success']]
        total_requests = len(self.results)
        successful_requests = len(all_latencies)
        
        print(f"\n🌟 OVERALL STATISTICS")
        print("-" * 40)
        print(f"Total Requests: {total_requests}")
        print(f"Successful Requests: {successful_requests}")
        print(f"Failed Requests: {total_requests - successful_requests}")
        print(f"Overall Success Rate: {(successful_requests / total_requests) * 100:.1f}%")
        
        if all_latencies:
            print(f"Overall Average Latency: {statistics.mean(all_latencies):.2f} ms")
            print(f"Overall Median Latency: {statistics.median(all_latencies):.2f} ms")


def main():
    parser = argparse.ArgumentParser(description='Real Estate API Latency Tester')
    parser.add_argument('--url', default='http://localhost:8080', 
                       help='Base URL of the API (default: http://localhost:8080)')
    parser.add_argument('--iterations', type=int, default=10,
                       help='Number of iterations per endpoint (default: 10)')
    parser.add_argument('--database-url', 
                       help='PostgreSQL database URL (uses DATABASE_URL env var if not provided)')
    parser.add_argument('--output', 
                       help='CSV output filename (auto-generated if not provided)')
    parser.add_argument('--no-summary', action='store_true',
                       help='Skip the summary report')
    
    args = parser.parse_args()
    
    # Get database URL from environment if not provided
    database_url = args.database_url or os.getenv('DATABASE_URL')
    
    # Create tester instance
    tester = LatencyTester(base_url=args.url, database_url=database_url)
    
    # Check if server is accessible
    try:
        response = requests.get(f"{args.url}/health", timeout=5)
        if response.status_code != 200:
            print(f"❌ Server health check failed with status {response.status_code}")
            sys.exit(1)
        print(f"✅ Server is accessible at {args.url}")
    except Exception as e:
        print(f"❌ Cannot connect to server at {args.url}: {e}")
        print("Please make sure the server is running with 'cargo run --release'")
        sys.exit(1)
    
    # Run tests
    start_time = time.time()
    tester.run_all_tests(iterations=args.iterations)
    end_time = time.time()
    
    # Save results
    filename = tester.save_results_to_csv(args.output)
    
    # Generate summary
    if not args.no_summary:
        tester.generate_summary_report()
    
    print(f"\n⏱️  Total test duration: {end_time - start_time:.2f} seconds")
    print(f"📁 Results saved in: {filename}")
    print("\n🎉 Latency testing completed successfully!")


if __name__ == "__main__":
    main()
