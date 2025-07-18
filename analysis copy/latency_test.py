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
import os
import sys
import psycopg2
from psycopg2.extras import RealDictCursor
from typing import List, Dict, Optional
from dataclasses import dataclass
from datetime import datetime
import argparse


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
    # New fields for K value analysis
    scenario: Optional[str] = None
    top_k_value: Optional[int] = None
    min_score_value: Optional[float] = None
    top_percentile_value: Optional[float] = None
    batch_size: Optional[int] = None


class DatabaseManager:
    def __init__(self, db_url: str):
        self.db_url = db_url
        self.connection = None
        
    def connect(self):
        """Connect to the database."""
        try:
            self.connection = psycopg2.connect(self.db_url)
            return True
        except Exception as e:
            print(f"Failed to connect to database: {e}")
            return False
    
    def disconnect(self):
        """Disconnect from the database."""
        if self.connection:
            self.connection.close()
    
    def get_property_ids(self, limit: int = 1000) -> List[int]:
        """Get property IDs from the database."""
        if not self.connection:
            if not self.connect():
                return []
        
        try:
            with self.connection.cursor() as cursor:
                cursor.execute("SELECT id FROM properties ORDER BY id LIMIT %s", (limit,))
                property_ids = [row[0] for row in cursor.fetchall()]
                print(f"✓ Retrieved {len(property_ids)} property IDs from database")
                return property_ids
        except Exception as e:
            print(f"Error fetching property IDs: {e}")
            return []
    
    def get_contact_ids(self, limit: int = 1000) -> List[int]:
        """Get contact IDs from the database."""
        if not self.connection:
            if not self.connect():
                return []
        
        try:
            with self.connection.cursor() as cursor:
                cursor.execute("SELECT id FROM contacts ORDER BY id LIMIT %s", (limit,))
                contact_ids = [row[0] for row in cursor.fetchall()]
                print(f"✓ Retrieved {len(contact_ids)} contact IDs from database")
                return contact_ids
        except Exception as e:
            print(f"Error fetching contact IDs: {e}")
            return []
    
    def get_random_property_ids(self, count: int = 50) -> List[int]:
        """Get random property IDs from the database."""
        if not self.connection:
            if not self.connect():
                return []
        
        try:
            with self.connection.cursor() as cursor:
                cursor.execute("""
                    SELECT id FROM properties 
                    ORDER BY RANDOM() 
                    LIMIT %s
                """, (count,))
                property_ids = [row[0] for row in cursor.fetchall()]
                print(f"✓ Retrieved {len(property_ids)} random property IDs from database")
                return property_ids
        except Exception as e:
            print(f"Error fetching random property IDs: {e}")
            return []
    
    def get_database_stats(self) -> Dict[str, int]:
        """Get database statistics."""
        if not self.connection:
            if not self.connect():
                return {}
        
        try:
            with self.connection.cursor() as cursor:
                cursor.execute("SELECT COUNT(*) FROM properties")
                property_count = cursor.fetchone()[0]
                
                cursor.execute("SELECT COUNT(*) FROM contacts")
                contact_count = cursor.fetchone()[0]
                
                return {
                    'properties': property_count,
                    'contacts': contact_count
                }
        except Exception as e:
            print(f"Error fetching database stats: {e}")
            return {}


class RecommendationLatencyTester:
    def __init__(self, base_url: str = "http://localhost:8080", db_url: str = None):
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
        self.results: List[LatencyResult] = []
        self.db_manager = DatabaseManager(db_url) if db_url else None
        
    def test_health_endpoint(self) -> bool:
        """Test if the server is running by hitting the health endpoint."""
        try:
            response = self.session.get(f"{self.base_url}/health", timeout=5)
            return response.status_code == 200
        except requests.exceptions.RequestException:
            return False

    def get_property_ids_for_testing(self, property_ids: List[int] = None, limit: int = 1000) -> List[int]:
        """Get property IDs for testing - either from parameter, database, or default."""
        if property_ids:
            print(f"Using provided property IDs: {len(property_ids)} properties")
            return property_ids
        
        if self.db_manager:
            print("Fetching property IDs from database...")
            db_property_ids = self.db_manager.get_property_ids(limit)
            if db_property_ids:
                return db_property_ids
            else:
                print("⚠️ No property IDs found in database, falling back to default range")
        
        # Fallback to default range
        default_ids = [6200 + i + 1 for i in range(min(limit, 999))]
        print(f"Using default property ID range: {len(default_ids)} properties")
        return default_ids

    def test_single_property_recommendation(self, property_id: int, limit: Optional[int] = None,
                                             min_score: Optional[float] = None, top_k: Optional[int] = None,
                                             top_percentile: Optional[float] = None, 
                                             score_threshold_percentile: Optional[float] = None) -> LatencyResult:
        """Test the single property recommendation endpoint with advanced filtering."""
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
        """Run a comprehensive test suite with specific K value scenarios."""
        # Get property IDs for testing
        test_property_ids = self.get_property_ids_for_testing(property_ids, limit=1000)
        
        if not test_property_ids:
            print("ERROR: No property IDs available for testing.")
            sys.exit(1)

        print(f"Running latency tests for {iterations} iterations...")
        print(f"Testing with {len(test_property_ids)} property IDs")
        print(f"Base URL: {self.base_url}")
        print(f"Testing K values: 5, 10, 50, 100")
        
        # Show database stats if available
        if self.db_manager:
            stats = self.db_manager.get_database_stats()
            if stats:
                print(f"Database stats: {stats['properties']} properties, {stats['contacts']} contacts")
        
        # Test server health first
        if not self.test_health_endpoint():
            print("ERROR: Server health check failed. Make sure the server is running.")
            sys.exit(1)
        
        print("✓ Server health check passed")
        
        # Test single property recommendations with specific K values
        print("\nTesting single property recommendations with different K values...")
        
        # Define test scenarios with specific K values
        test_scenarios = [
            # Basic scenarios without K filtering
            {"scenario_name": "baseline", "limit": 100, "min_score": 0.5},
            {"scenario_name": "high_threshold", "limit": 100, "min_score": 0.7},
            
            # Top K scenarios - the main focus
            {"scenario_name": "top_k_5", "limit": 100, "min_score": 0.5, "top_k": 5},
            {"scenario_name": "top_k_10", "limit": 100, "min_score": 0.5, "top_k": 10},
            {"scenario_name": "top_k_50", "limit": 100, "min_score": 0.5, "top_k": 50},
            {"scenario_name": "top_k_100", "limit": 100, "min_score": 0.5, "top_k": 100},
            
            # Top K with higher min_score
            {"scenario_name": "top_k_5_high_score", "limit": 100, "min_score": 0.7, "top_k": 5},
            {"scenario_name": "top_k_10_high_score", "limit": 100, "min_score": 0.7, "top_k": 10},
            {"scenario_name": "top_k_50_high_score", "limit": 100, "min_score": 0.7, "top_k": 50},
            {"scenario_name": "top_k_100_high_score", "limit": 100, "min_score": 0.7, "top_k": 100},
            
            # Combined filtering scenarios
            {"scenario_name": "top_k_5_percentile", "limit": 100, "min_score": 0.5, "top_k": 5, "top_percentile": 0.2},
            {"scenario_name": "top_k_10_percentile", "limit": 100, "min_score": 0.5, "top_k": 10, "top_percentile": 0.2},
            {"scenario_name": "top_k_50_percentile", "limit": 100, "min_score": 0.5, "top_k": 50, "top_percentile": 0.3},
            {"scenario_name": "top_k_100_percentile", "limit": 100, "min_score": 0.5, "top_k": 100, "top_percentile": 0.4},
        ]
        
        # Calculate iterations per scenario
        iterations_per_scenario = max(1, iterations // len(test_scenarios))
        total_single_tests = iterations_per_scenario * len(test_scenarios)
        
        print(f"Running {iterations_per_scenario} iterations per scenario ({len(test_scenarios)} scenarios)")
        print(f"Total single property tests: {total_single_tests}")
        
        test_count = 0
        for scenario_idx, scenario in enumerate(test_scenarios):
            scenario_name = scenario.pop("scenario_name")
            print(f"\n  Testing scenario '{scenario_name}' (K={scenario.get('top_k', 'None')})")
            
            for i in range(iterations_per_scenario):
                property_id = test_property_ids[(test_count) % len(test_property_ids)]
                
                result = self.test_single_property_recommendation(
                    property_id=property_id,
                    **scenario
                )
                
                # Add scenario info to the result for later analysis
                result.scenario = scenario_name
                result.top_k_value = scenario.get('top_k')
                result.min_score_value = scenario.get('min_score')
                result.top_percentile_value = scenario.get('top_percentile')
                
                self.results.append(result)
                test_count += 1
                
                if (i + 1) % 10 == 0:
                    print(f"    Completed {i + 1}/{iterations_per_scenario} tests for {scenario_name}")
            
            print(f"  ✓ Completed scenario '{scenario_name}'")

        print(f"\n✓ Completed all single property tests ({test_count} total)")

        # Test bulk recommendations with K value scenarios
        print("\nTesting bulk recommendations with different K values...")
        
        bulk_scenarios = [
            # Different K values for bulk operations
            {"scenario_name": "bulk_baseline", "limit_per_property": 50, "min_score": 0.5},
            {"scenario_name": "bulk_top_k_5", "limit_per_property": 50, "min_score": 0.5, "top_k": 5},
            {"scenario_name": "bulk_top_k_10", "limit_per_property": 50, "min_score": 0.5, "top_k": 10},
            {"scenario_name": "bulk_top_k_50", "limit_per_property": 50, "min_score": 0.5, "top_k": 50},
            {"scenario_name": "bulk_top_k_100", "limit_per_property": 50, "min_score": 0.5, "top_k": 100},
            
            # Combined scenarios for bulk
            {"scenario_name": "bulk_top_k_5_percentile", "limit_per_property": 50, "min_score": 0.5, "top_k": 5, "top_percentile": 0.2},
            {"scenario_name": "bulk_top_k_10_percentile", "limit_per_property": 50, "min_score": 0.5, "top_k": 10, "top_percentile": 0.2},
        ]
        
        bulk_iterations = max(1, iterations // 4)  # Fewer bulk tests since they're more expensive
        
        bulk_test_count = 0
        for scenario_idx, scenario in enumerate(bulk_scenarios):
            scenario_name = scenario.pop("scenario_name")
            print(f"\n  Testing bulk scenario '{scenario_name}' (K={scenario.get('top_k', 'None')})")
            
            iterations_this_scenario = max(1, bulk_iterations // len(bulk_scenarios))
            
            for i in range(iterations_this_scenario):
                # Use different batch sizes (2-5 properties per bulk request)
                batch_size = 2 + (i % 4)  
                batch_property_ids = test_property_ids[:batch_size]
                
                result = self.test_bulk_recommendations(
                    property_ids=batch_property_ids,
                    **scenario
                )
                
                # Add scenario info to the result
                result.scenario = scenario_name
                result.top_k_value = scenario.get('top_k')
                result.min_score_value = scenario.get('min_score')
                result.top_percentile_value = scenario.get('top_percentile')
                result.batch_size = batch_size
                
                self.results.append(result)
                bulk_test_count += 1
                
                if (i + 1) % 5 == 0:
                    print(f"    Completed {i + 1}/{iterations_this_scenario} bulk tests for {scenario_name}")
            
            print(f"  ✓ Completed bulk scenario '{scenario_name}'")
        
        print(f"\n✓ Completed all bulk tests ({bulk_test_count} total)")
        print(f"\n🎯 Total tests completed: {len(self.results)}")
        print(f"   - Single property tests: {test_count}")
        print(f"   - Bulk recommendation tests: {bulk_test_count}")
        
        # Show scenario distribution
        if hasattr(self.results[0], 'scenario'):
            scenario_counts = {}
            for result in self.results:
                scenario = getattr(result, 'scenario', 'unknown')
                scenario_counts[scenario] = scenario_counts.get(scenario, 0) + 1
            
            print(f"\nTest distribution by scenario:")
            for scenario, count in sorted(scenario_counts.items()):
                k_value = next((getattr(r, 'top_k_value', None) for r in self.results if getattr(r, 'scenario', None) == scenario), None)
                k_info = f" (K={k_value})" if k_value else ""
                print(f"   {scenario}{k_info}: {count} tests")
        
        # Cleanup database connection
        if self.db_manager:
            self.db_manager.disconnect()
    
    def save_results_to_csv(self, filename: str = "analysis/latency_results.csv") -> None:
        """Save the test results to a CSV file."""
        if not self.results:
            print("No results to save.")
            return
        
        # Create the directory if it doesn't exist
        os.makedirs(os.path.dirname(filename), exist_ok=True)
        
        with open(filename, 'w', newline='') as csvfile:
            fieldnames = [
                'timestamp', 'endpoint', 'property_id', 'response_time_ms',
                'status_code', 'request_size_bytes', 'response_size_bytes',
                'success', 'error_message', 'scenario', 'top_k_value',
                'min_score_value', 'top_percentile_value', 'batch_size'
            ]
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writeheader()
            
            for result in self.results:
                writer.writerow({
                    'timestamp': result.timestamp,
                    'endpoint': result.endpoint,
                    'property_id': result.contact_id,  # Note: keeping field name for compatibility
                    'response_time_ms': result.response_time_ms,
                    'status_code': result.status_code,
                    'request_size_bytes': result.request_size_bytes,
                    'response_size_bytes': result.response_size_bytes,
                    'success': result.success,
                    'error_message': result.error_message,
                    'scenario': getattr(result, 'scenario', None),
                    'top_k_value': getattr(result, 'top_k_value', None),
                    'min_score_value': getattr(result, 'min_score_value', None),
                    'top_percentile_value': getattr(result, 'top_percentile_value', None),
                    'batch_size': getattr(result, 'batch_size', None)
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
    parser.add_argument('--db-url', 
                       help='Database connection URL (e.g., postgresql://user:pass@localhost/db)')
    parser.add_argument('--iterations', type=int, default=100,
                       help='Number of test iterations (default: 100)')
    parser.add_argument('--property-ids', nargs='+', type=int,
                       help='Property IDs to test (default: auto-fetch from database)')
    parser.add_argument('--output', default='analysis/latency_results.csv',
                       help='Output CSV file name (default: analysis/latency_results.csv)')
    
    args = parser.parse_args()
    
    # Try to get database URL from environment if not provided
    db_url = args.db_url
    if not db_url:
        db_url = os.environ.get('DATABASE_URL')
        if db_url:
            print(f"Using DATABASE_URL from environment")
    
    tester = RecommendationLatencyTester(args.url, db_url)
    tester.run_test_suite(args.iterations, args.property_ids)
    tester.save_results_to_csv(args.output)
    tester.print_summary_stats()


if __name__ == "__main__":
    main()