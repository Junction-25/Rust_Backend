#!/usr/bin/env python3
"""
Latency test for the recommendation endpoint.

This script tests the performance of the recommendation API endpoints
and generates CSV data for plotting performance metrics.
"""

import time
import requests
import csv
import json
import statistics
from typing import List, Dict, Optional
from dataclasses import dataclass
from datetime import datetime
import argparse
import sys


@dataclass
class LatencyResult:
    timestamp: str
    endpoint: str
    contact_id: Optional[int]
    response_time_ms: float
    status_code: int
    request_size_bytes: int
    response_size_bytes: int
    success: bool
    error_message: Optional[str] = None


class RecommendationLatencyTester:
    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
        self.results: List[LatencyResult] = []
        
    def test_health_endpoint(self) -> bool:
        """Test if the server is running by hitting the health endpoint."""
        try:
            response = self.session.get(f"{self.base_url}/health", timeout=5)
            return response.status_code == 200
        except requests.exceptions.RequestException:
            return False

    def test_single_property_recommendation(self, property_id: int, limit: Optional[int] = None,
                                             min_score: Optional[float] = None, top_k: Optional[int] = None,
                                             top_percentile: Optional[float] = None, 
                                             score_threshold_percentile: Optional[float] = None) -> LatencyResult:
        """Test the single property recommendation endpoint with advanced filtering."""
        print(f"Testing single property recommendation for ID: {property_id}")
        url = f"{self.base_url}/recommendations/property/{property_id}"
        params = {}
        if limit is not None:
            params['limit'] = limit
        if min_score is not None:
            params['min_score'] = min_score
        if top_k is not None:
            params['top_k'] = top_k
        if top_percentile is not None:
            params['top_percentile'] = top_percentile
        if score_threshold_percentile is not None:
            params['score_threshold_percentile'] = score_threshold_percentile
        
        request_size = len(str(params).encode('utf-8'))
        start_time = time.time()
        timestamp = datetime.now().isoformat()
        
        try:
            response = self.session.get(url, params=params, timeout=30)
            end_time = time.time()
            
            response_time_ms = (end_time - start_time) * 1000
            response_size = len(response.content)
            success = response.status_code == 200
            error_message = None if success else f"HTTP {response.status_code}: {response.text[:100]}"
            
            return LatencyResult(
                timestamp=timestamp,
                endpoint="single_property",
                contact_id=property_id,
                response_time_ms=response_time_ms,
                status_code=response.status_code,
                request_size_bytes=request_size,
                response_size_bytes=response_size,
                success=success,
                error_message=error_message
            )
        
        except requests.exceptions.RequestException as e:
            end_time = time.time()
            response_time_ms = (end_time - start_time) * 1000
            
            return LatencyResult(
                timestamp=timestamp,
                endpoint="single_property",
                contact_id=property_id,
                response_time_ms=response_time_ms,
                status_code=0,
                request_size_bytes=request_size,
                response_size_bytes=0,
                success=False,
                error_message=str(e)
            )
    
    def test_bulk_recommendations(self, property_ids: List[int], limit_per_property: Optional[int] = None,
                                min_score: Optional[float] = None, top_k: Optional[int] = None,
                                top_percentile: Optional[float] = None, 
                                score_threshold_percentile: Optional[float] = None) -> LatencyResult:
        """Test the bulk recommendations endpoint with advanced filtering."""
        url = f"{self.base_url}/recommendations/bulk"
        payload = {
            "property_ids": property_ids
        }
        if limit_per_property is not None:
            payload['limit_per_property'] = limit_per_property
        if min_score is not None:
            payload['min_score'] = min_score
        if top_k is not None:
            payload['top_k'] = top_k
        if top_percentile is not None:
            payload['top_percentile'] = top_percentile
        if score_threshold_percentile is not None:
            payload['score_threshold_percentile'] = score_threshold_percentile
        
        request_data = json.dumps(payload)
        request_size = len(request_data.encode('utf-8'))
        start_time = time.time()
        timestamp = datetime.now().isoformat()
        
        try:
            response = self.session.post(
                url, 
                json=payload,
                headers={'Content-Type': 'application/json'},
                timeout=60
            )
            end_time = time.time()
            
            response_time_ms = (end_time - start_time) * 1000
            response_size = len(response.content)
            success = response.status_code == 200
            error_message = None if success else f"HTTP {response.status_code}: {response.text[:100]}"
            
            return LatencyResult(
                timestamp=timestamp,
                endpoint="bulk_recommendations",
                contact_id=None,
                response_time_ms=response_time_ms,
                status_code=response.status_code,
                request_size_bytes=request_size,
                response_size_bytes=response_size,
                success=success,
                error_message=error_message
            )
        
        except requests.exceptions.RequestException as e:
            end_time = time.time()
            response_time_ms = (end_time - start_time) * 1000
            
            return LatencyResult(
                timestamp=timestamp,
                endpoint="bulk_recommendations",
                contact_id=None,
                response_time_ms=response_time_ms,
                status_code=0,
                request_size_bytes=request_size,
                response_size_bytes=0,
                success=False,
                error_message=str(e)
            )
    
    def run_test_suite(self, iterations: int = 100, property_ids: List[int] = None) -> None:
        """Run a comprehensive test suite."""
        if property_ids is None:
            property_ids = [2000 + i + 1 for i in range(999)]  # Default property IDs from sample data

        print(f"Running latency tests for {iterations} iterations...")
        print(f"Testing property IDs: {property_ids}")
        print(f"Base URL: {self.base_url}")
        
        # Test server health first
        if not self.test_health_endpoint():
            print("ERROR: Server health check failed. Make sure the server is running.")
            sys.exit(1)
        
        print("✓ Server health check passed")
        
        # Test single property recommendations with different parameter combinations
        print("\nTesting single property recommendations...")
        test_scenarios = [
            {"limit": 10, "min_score": 0.6},  # Basic filtering
            {"limit": 10, "min_score": 0.6, "top_k": 5},  # Top K filtering
            {"limit": 10, "min_score": 0.6, "top_percentile": 0.2},  # Top percentile filtering
            {"limit": 10, "min_score": 0.6, "top_k": 5, "top_percentile": 0.2},  # Combined filtering
            {"limit": 10, "min_score": 0.6, "score_threshold_percentile": 0.8},  # Threshold percentile
        ]
        
        for i in range(iterations):
            property_id = property_ids[i % len(property_ids)]
            scenario = test_scenarios[i % len(test_scenarios)]
            
            result = self.test_single_property_recommendation(
                property_id=property_id,
                limit=scenario.get("limit"),
                min_score=scenario.get("min_score"),
                top_k=scenario.get("top_k"),
                top_percentile=scenario.get("top_percentile"),
                score_threshold_percentile=scenario.get("score_threshold_percentile")
            )
            self.results.append(result)
            
            if (i + 1) % 20 == 0:
                print(f"  Completed {i + 1}/{iterations} single property tests")

        # Test bulk recommendations with different parameter combinations
        print("\nTesting bulk recommendations...")
        bulk_scenarios = [
            {"limit_per_property": 5, "min_score": 0.5},  # Basic filtering
            {"limit_per_property": 5, "min_score": 0.6, "top_k": 3},  # Top K filtering
            {"limit_per_property": 5, "min_score": 0.6, "top_percentile": 0.3},  # Top percentile
            {"limit_per_property": 5, "min_score": 0.6, "top_k": 3, "top_percentile": 0.2},  # Combined
        ]
        
        for i in range(iterations // 2):  # Half as many bulk tests since they're more expensive
            # Use different batch sizes
            batch_size = 2 + (i % 4)  # Batch sizes of 2, 3, 4, 5
            batch_property_ids = property_ids[:batch_size]
            scenario = bulk_scenarios[i % len(bulk_scenarios)]
            
            result = self.test_bulk_recommendations(
                property_ids=batch_property_ids,
                limit_per_property=scenario.get("limit_per_property"),
                min_score=scenario.get("min_score"),
                top_k=scenario.get("top_k"),
                top_percentile=scenario.get("top_percentile"),
                score_threshold_percentile=scenario.get("score_threshold_percentile")
            )
            self.results.append(result)
            
            if (i + 1) % 10 == 0:
                print(f"  Completed {i + 1}/{iterations // 2} bulk tests")
        
        print(f"\n✓ Completed all tests. Total results: {len(self.results)}")
    
    def save_results_to_csv(self, filename: str = "analysis/latency_results.csv") -> None:
        """Save the test results to a CSV file."""
        if not self.results:
            print("No results to save.")
            return
        
        with open(filename, 'w', newline='') as csvfile:
            fieldnames = [
                'timestamp', 'endpoint', 'property_id', 'response_time_ms',
                'status_code', 'request_size_bytes', 'response_size_bytes',
                'success', 'error_message'
            ]
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writeheader()
            
            for result in self.results:
                writer.writerow({
                    'timestamp': result.timestamp,
                    'endpoint': result.endpoint,
                    'property_id': result.contact_id,  # Note: keeping the field name for compatibility
                    'response_time_ms': result.response_time_ms,
                    'status_code': result.status_code,
                    'request_size_bytes': result.request_size_bytes,
                    'response_size_bytes': result.response_size_bytes,
                    'success': result.success,
                    'error_message': result.error_message
                })
        
        print(f"✓ Results saved to {filename}")
    
    def print_summary_stats(self) -> None:
        """Print summary statistics of the test results."""
        if not self.results:
            print("No results to analyze.")
            return
        
        successful_results = [r for r in self.results if r.success]
        failed_results = [r for r in self.results if not r.success]
        
        print("\n" + "="*60)
        print("LATENCY TEST SUMMARY")
        print("="*60)
        
        print(f"Total tests: {len(self.results)}")
        print(f"Successful: {len(successful_results)} ({len(successful_results)/len(self.results)*100:.1f}%)")
        print(f"Failed: {len(failed_results)} ({len(failed_results)/len(self.results)*100:.1f}%)")
        
        if successful_results:
            response_times = [r.response_time_ms for r in successful_results]
            
            print(f"\nResponse Time Statistics (ms):")
            print(f"  Mean: {statistics.mean(response_times):.2f}")
            print(f"  Median: {statistics.median(response_times):.2f}")
            print(f"  Min: {min(response_times):.2f}")
            print(f"  Max: {max(response_times):.2f}")
            print(f"  95th percentile: {sorted(response_times)[int(len(response_times) * 0.95)]:.2f}")
            print(f"  99th percentile: {sorted(response_times)[int(len(response_times) * 0.99)]:.2f}")
            
            # Break down by endpoint
            single_property_results = [r for r in successful_results if r.endpoint == "single_property"]
            bulk_results = [r for r in successful_results if r.endpoint == "bulk_recommendations"]
            
            if single_property_results:
                single_times = [r.response_time_ms for r in single_property_results]
                print(f"\nSingle Property Endpoint:")
                print(f"  Tests: {len(single_property_results)}")
                print(f"  Mean response time: {statistics.mean(single_times):.2f} ms")
                print(f"  Median response time: {statistics.median(single_times):.2f} ms")
            
            if bulk_results:
                bulk_times = [r.response_time_ms for r in bulk_results]
                print(f"\nBulk Recommendations Endpoint:")
                print(f"  Tests: {len(bulk_results)}")
                print(f"  Mean response time: {statistics.mean(bulk_times):.2f} ms")
                print(f"  Median response time: {statistics.median(bulk_times):.2f} ms")
        
        if failed_results:
            print(f"\nFailure Analysis:")
            error_counts = {}
            for result in failed_results:
                error_key = f"HTTP {result.status_code}" if result.status_code > 0 else "Connection Error"
                error_counts[error_key] = error_counts.get(error_key, 0) + 1
            
            for error, count in error_counts.items():
                print(f"  {error}: {count} occurrences")


def main():
    parser = argparse.ArgumentParser(description='Run latency tests for recommendation endpoints')
    parser.add_argument('--url', default='http://localhost:8080', 
                       help='Base URL of the API server (default: http://localhost:8080)')
    parser.add_argument('--iterations', type=int, default=100,
                       help='Number of test iterations (default: 100)')
    parser.add_argument('--property-ids', nargs='+', type=int,
                       help='Property IDs to test (default: auto-generated from sample data)')
    parser.add_argument('--output', default='analysis/latency_results.csv',
                       help='Output CSV file name (default: analysis/latency_results.csv)')
    
    args = parser.parse_args()
    
    tester = RecommendationLatencyTester(args.url)
    tester.run_test_suite(args.iterations, args.property_ids)
    tester.save_results_to_csv(args.output)
    tester.print_summary_stats()


if __name__ == "__main__":
    main()
