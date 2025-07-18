#!/usr/bin/env python3
"""
Scalability test for the recommendation endpoint.

This script tests how recommendation response times change as the database
grows by gradually adding more contacts and properties, measuring performance
at each step.
"""

import time
import requests
import csv
import json
import random
from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass
from datetime import datetime
import argparse
import sys
import psycopg2
from psycopg2.extras import RealDictCursor
import subprocess


@dataclass
class ScalabilityResult:
    timestamp: str
    total_contacts: int
    total_properties: int
    endpoint: str
    contact_id: Optional[int]
    response_time_ms: float
    status_code: int
    response_size_bytes: int
    recommendations_count: int
    success: bool
    error_message: Optional[str] = None


class DatabaseManager:
    def __init__(self, db_url: str, data_dir: str = "data"):
        self.db_url = db_url
        self.data_dir = data_dir
        self.connection = None
        self.contacts_data = []
        self.properties_data = []
        self.load_data_files()
        
    def load_data_files(self):
        """Load contact and property data from JSON files."""
        import os
        
        contacts_file = os.path.join(self.data_dir, "contacts.json")
        properties_file = os.path.join(self.data_dir, "properties.json")
        
        try:
            with open(contacts_file, 'r') as f:
                self.contacts_data = json.load(f)
            print(f"✓ Loaded {len(self.contacts_data)} contacts from {contacts_file}")
        except FileNotFoundError:
            print(f"❌ Contact data file not found: {contacts_file}")
            sys.exit(1)
        except Exception as e:
            print(f"❌ Error loading contact data: {e}")
            sys.exit(1)
        
        try:
            with open(properties_file, 'r') as f:
                self.properties_data = json.load(f)
            print(f"✓ Loaded {len(self.properties_data)} properties from {properties_file}")
        except FileNotFoundError:
            print(f"❌ Property data file not found: {properties_file}")
            sys.exit(1)
        except Exception as e:
            print(f"❌ Error loading property data: {e}")
            sys.exit(1)
        
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
    
    def clear_tables(self):
        """Clear contacts and properties tables."""
        with self.connection.cursor() as cursor:
            cursor.execute("DELETE FROM contacts")
            cursor.execute("DELETE FROM properties")
            self.connection.commit()
    
    def get_counts(self) -> Tuple[int, int]:
        """Get current count of contacts and properties."""
        with self.connection.cursor() as cursor:
            cursor.execute("SELECT COUNT(*) FROM contacts")
            contact_count = cursor.fetchone()[0]
            
            cursor.execute("SELECT COUNT(*) FROM properties")
            property_count = cursor.fetchone()[0]
            
            return contact_count, property_count
    
    def add_contacts_batch(self, batch_size: int) -> List[int]:
        """Add a batch of contacts from the loaded JSON data and return their IDs."""
        contact_ids = []
        
        if not self.contacts_data:
            print("❌ No contact data available")
            return contact_ids
        
        # Get current count to know which contacts to add next
        current_count, _ = self.get_counts()
        
        # Select contacts from the JSON data
        start_idx = current_count
        end_idx = min(start_idx + batch_size, len(self.contacts_data))
        
        if start_idx >= len(self.contacts_data):
            print(f"⚠️ Reached end of contact data ({len(self.contacts_data)} contacts)")
            return contact_ids
        
        contacts_to_add = self.contacts_data[start_idx:end_idx]
        
        with self.connection.cursor() as cursor:
            for contact_data in contacts_to_add:
                # Extract data from the JSON structure
                name = contact_data['name']
                preferred_locations = contact_data['preferred_locations']
                min_budget = contact_data['min_budget']
                max_budget = contact_data['max_budget']
                min_area_sqm = contact_data['min_area_sqm']
                max_area_sqm = contact_data['max_area_sqm']
                property_types = contact_data['property_types']
                min_rooms = contact_data['min_rooms']
                
                cursor.execute("""
                    INSERT INTO contacts (name, preferred_locations, min_budget, max_budget, 
                                        min_area_sqm, max_area_sqm, property_types, min_rooms)
                    VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
                    RETURNING id
                """, (
                    name,
                    json.dumps(preferred_locations),
                    min_budget,
                    max_budget,
                    min_area_sqm,
                    max_area_sqm,
                    json.dumps(property_types),
                    min_rooms
                ))
                
                contact_id = cursor.fetchone()[0]
                contact_ids.append(contact_id)
            
            self.connection.commit()
        
        return contact_ids
    
    def add_properties_batch(self, batch_size: int) -> List[int]:
        """Add a batch of properties from the loaded JSON data and return their IDs."""
        property_ids = []
        
        if not self.properties_data:
            print("❌ No property data available")
            return property_ids
        
        # Get current property count to know which properties to add next
        _, current_count = self.get_counts()
        
        # Select properties from the JSON data
        start_idx = current_count
        end_idx = min(start_idx + batch_size, len(self.properties_data))
        
        if start_idx >= len(self.properties_data):
            print(f"⚠️ Reached end of property data ({len(self.properties_data)} properties)")
            return property_ids
        
        properties_to_add = self.properties_data[start_idx:end_idx]
        
        with self.connection.cursor() as cursor:
            for property_data in properties_to_add:
                # Extract data from the JSON structure
                address = property_data['address']
                lat = property_data['location']['lat']
                lon = property_data['location']['lon']
                price = property_data['price']
                area_sqm = property_data['area_sqm']
                property_type = property_data['property_type']
                number_of_rooms = property_data['number_of_rooms']
                
                cursor.execute("""
                    INSERT INTO properties (address, lat, lon, price, area_sqm, 
                                          property_type, number_of_rooms)
                    VALUES (%s, %s, %s, %s, %s, %s, %s)
                    RETURNING id
                """, (
                    address, lat, lon, price, area_sqm, property_type, number_of_rooms
                ))
                
                property_id = cursor.fetchone()[0]
                property_ids.append(property_id)
            
            self.connection.commit()
        
        return property_ids


class RecommendationScalabilityTester:
    def __init__(self, base_url: str = "http://localhost:8080", db_url: str = None, data_dir: str = "data"):
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
        self.results: List[ScalabilityResult] = []
        self.db_manager = DatabaseManager(db_url, data_dir) if db_url else None
        
    def test_health_endpoint(self) -> bool:
        """Test if the server is running."""
        try:
            response = self.session.get(f"{self.base_url}/health", timeout=5)
            return response.status_code == 200
        except requests.exceptions.RequestException:
            return False
    
    def test_recommendation_endpoint(self, contact_id: int, total_contacts: int, 
                                   total_properties: int) -> ScalabilityResult:
        """Test a single recommendation request and record scalability metrics."""
        url = f"{self.base_url}/recommendations/contact/{contact_id}"
        params = {"limit": 10}
        
        start_time = time.time()
        timestamp = datetime.now().isoformat()
        
        try:
            response = self.session.get(url, params=params, timeout=30)
            end_time = time.time()
            
            response_time_ms = (end_time - start_time) * 1000
            response_size = len(response.content)
            success = response.status_code == 200
            
            # Try to parse recommendations count
            recommendations_count = 0
            if success:
                try:
                    data = response.json()
                    if isinstance(data, dict) and 'recommendations' in data:
                        recommendations_count = len(data['recommendations'])
                    elif isinstance(data, list):
                        recommendations_count = len(data)
                except:
                    pass
            
            error_message = None if success else f"HTTP {response.status_code}: {response.text[:100]}"
            
            return ScalabilityResult(
                timestamp=timestamp,
                total_contacts=total_contacts,
                total_properties=total_properties,
                endpoint="single_contact",
                contact_id=contact_id,
                response_time_ms=response_time_ms,
                status_code=response.status_code,
                response_size_bytes=response_size,
                recommendations_count=recommendations_count,
                success=success,
                error_message=error_message
            )
        
        except requests.exceptions.RequestException as e:
            end_time = time.time()
            response_time_ms = (end_time - start_time) * 1000
            
            return ScalabilityResult(
                timestamp=timestamp,
                total_contacts=total_contacts,
                total_properties=total_properties,
                endpoint="single_contact",
                contact_id=contact_id,
                response_time_ms=response_time_ms,
                status_code=0,
                response_size_bytes=0,
                recommendations_count=0,
                success=False,
                error_message=str(e)
            )
    
    def test_bulk_recommendation_endpoint(self, contact_ids: List[int], total_contacts: int, 
                                        total_properties: int, batch_size: int = None) -> ScalabilityResult:
        """Test a bulk recommendation request and record scalability metrics."""
        url = f"{self.base_url}/recommendations/bulk"
        
        if batch_size:
            # Limit the contact IDs to the specified batch size
            contact_ids = contact_ids[:batch_size]
        
        payload = {
            "contact_ids": contact_ids,
            "limit_per_contact": 5  # Fewer per contact for bulk to manage response size
        }
        
        request_data = json.dumps(payload)
        request_size = len(request_data.encode('utf-8'))
        start_time = time.time()
        timestamp = datetime.now().isoformat()
        
        try:
            response = self.session.post(
                url,
                json=payload,
                headers={'Content-Type': 'application/json'},
                timeout=60  # Longer timeout for bulk operations
            )
            end_time = time.time()
            
            response_time_ms = (end_time - start_time) * 1000
            response_size = len(response.content)
            success = response.status_code == 200
            
            # Try to parse recommendations count
            recommendations_count = 0
            if success:
                try:
                    data = response.json()
                    if isinstance(data, dict):
                        if 'recommendations' in data:
                            # Handle direct recommendations array
                            recommendations_count = len(data['recommendations'])
                        elif 'total_recommendations' in data:
                            # Handle summary response
                            recommendations_count = data['total_recommendations']
                        elif any(key.endswith('recommendations') for key in data.keys()):
                            # Handle nested recommendations
                            for key, value in data.items():
                                if key.endswith('recommendations') and isinstance(value, list):
                                    for contact_recs in value:
                                        if isinstance(contact_recs, dict) and 'recommendations' in contact_recs:
                                            recommendations_count += len(contact_recs['recommendations'])
                except Exception as parse_error:
                    print(f"Warning: Could not parse bulk response: {parse_error}")
            
            error_message = None if success else f"HTTP {response.status_code}: {response.text[:100]}"
            
            return ScalabilityResult(
                timestamp=timestamp,
                total_contacts=total_contacts,
                total_properties=total_properties,
                endpoint=f"bulk_recommendations_{len(contact_ids)}",
                contact_id=None,  # Not applicable for bulk
                response_time_ms=response_time_ms,
                status_code=response.status_code,
                response_size_bytes=response_size,
                recommendations_count=recommendations_count,
                success=success,
                error_message=error_message
            )
        
        except requests.exceptions.RequestException as e:
            end_time = time.time()
            response_time_ms = (end_time - start_time) * 1000
            
            return ScalabilityResult(
                timestamp=timestamp,
                total_contacts=total_contacts,
                total_properties=total_properties,
                endpoint=f"bulk_recommendations_{len(contact_ids)}",
                contact_id=None,
                response_time_ms=response_time_ms,
                status_code=0,
                response_size_bytes=0,
                recommendations_count=0,
                success=False,
                error_message=str(e)
            )
    
    def run_scalability_test(self, 
                           max_contacts: int = 1000,
                           max_properties: int = 2000,
                           contact_batch_size: int = 50,
                           property_batch_size: int = 100,
                           tests_per_step: int = 10,
                           include_bulk_tests: bool = True,
                           bulk_batch_sizes: List[int] = None) -> None:
        """Run the complete scalability test."""
        
        if bulk_batch_sizes is None:
            bulk_batch_sizes = [2, 5, 10, 20]  # Different bulk sizes to test
        
        if not self.db_manager:
            print("ERROR: Database connection required for scalability testing.")
            sys.exit(1)
        
        if not self.db_manager.connect():
            print("ERROR: Failed to connect to database.")
            sys.exit(1)
        
        if not self.test_health_endpoint():
            print("ERROR: Server health check failed.")
            sys.exit(1)
        
        print(f"🚀 Starting scalability test...")
        print(f"Target: {max_contacts} contacts, {max_properties} properties")
        print(f"Available data: {len(self.db_manager.contacts_data)} contacts, {len(self.db_manager.properties_data)} properties")
        print(f"Batch sizes: {contact_batch_size} contacts, {property_batch_size} properties")
        print(f"Tests per step: {tests_per_step}")
        print(f"Bulk testing: {'Enabled' if include_bulk_tests else 'Disabled'}")
        if include_bulk_tests:
            print(f"Bulk batch sizes: {bulk_batch_sizes}")
        
        # Check if we have enough data
        if max_contacts > len(self.db_manager.contacts_data):
            print(f"⚠️ Warning: Requested {max_contacts} contacts but only {len(self.db_manager.contacts_data)} available")
            max_contacts = len(self.db_manager.contacts_data)
            print(f"   Limiting test to {max_contacts} contacts")
        
        if max_properties > len(self.db_manager.properties_data):
            print(f"⚠️ Warning: Requested {max_properties} properties but only {len(self.db_manager.properties_data)} available")
            max_properties = len(self.db_manager.properties_data)
            print(f"   Limiting test to {max_properties} properties")
        
        # Clear existing data
        print("🧹 Clearing existing test data...")
        self.db_manager.clear_tables()
        
        # Start with small dataset and gradually increase
        current_contacts = 0
        current_properties = 0
        step = 0
        all_contact_ids = []  # Keep track of all added contact IDs for bulk testing
        
        # Add initial properties (we want some properties available from the start)
        print("📊 Adding initial properties...")
        self.db_manager.add_properties_batch(property_batch_size)
        current_properties = property_batch_size
        
        while current_contacts < max_contacts:
            step += 1
            
            # Add batch of contacts
            print(f"\n📈 Step {step}: Adding {contact_batch_size} contacts...")
            new_contact_ids = self.db_manager.add_contacts_batch(contact_batch_size)
            if not new_contact_ids:  # No more contacts available
                break
                
            current_contacts += len(new_contact_ids)
            all_contact_ids.extend(new_contact_ids)
            
            # Add batch of properties (to maintain good ratio)
            if current_properties < max_properties:
                properties_to_add = min(property_batch_size, max_properties - current_properties)
                if properties_to_add > 0:
                    self.db_manager.add_properties_batch(properties_to_add)
                    current_properties += properties_to_add
            
            # Verify counts
            db_contacts, db_properties = self.db_manager.get_counts()
            print(f"   Database now has: {db_contacts} contacts, {db_properties} properties")
            
            # Test single contact recommendations for this dataset size
            print(f"   Testing {tests_per_step} single contact recommendations...")
            for test_num in range(tests_per_step):
                # Pick a random contact from the recently added ones for testing
                test_contact_id = random.choice(new_contact_ids)
                
                result = self.test_recommendation_endpoint(
                    test_contact_id, db_contacts, db_properties
                )
                self.results.append(result)
                
                if (test_num + 1) % 3 == 0:
                    print(f"     Completed {test_num + 1}/{tests_per_step} single tests")
            
            # Test bulk recommendations if enabled
            if include_bulk_tests and len(all_contact_ids) >= min(bulk_batch_sizes):
                print(f"   Testing bulk recommendations...")
                for bulk_size in bulk_batch_sizes:
                    if bulk_size <= len(all_contact_ids):
                        # Select random contacts for bulk testing
                        bulk_contact_ids = random.sample(all_contact_ids, bulk_size)
                        
                        bulk_result = self.test_bulk_recommendation_endpoint(
                            bulk_contact_ids, db_contacts, db_properties, bulk_size
                        )
                        self.results.append(bulk_result)
                        
                        print(f"     Completed bulk test with {bulk_size} contacts")
            
            # Print step summary
            step_results = [r for r in self.results 
                          if r.total_contacts == db_contacts and r.endpoint == "single_contact"]
            if step_results:
                avg_response_time = sum(r.response_time_ms for r in step_results) / len(step_results)
                success_rate = sum(1 for r in step_results if r.success) / len(step_results) * 100
                print(f"   Step {step} single contact summary: Avg response time: {avg_response_time:.2f}ms, "
                      f"Success rate: {success_rate:.1f}%")
            
            # Print bulk summary if applicable
            if include_bulk_tests:
                bulk_results = [r for r in self.results 
                              if r.total_contacts == db_contacts and r.endpoint.startswith("bulk_recommendations")]
                if bulk_results:
                    avg_bulk_time = sum(r.response_time_ms for r in bulk_results) / len(bulk_results)
                    bulk_success_rate = sum(1 for r in bulk_results if r.success) / len(bulk_results) * 100
                    print(f"   Step {step} bulk summary: Avg response time: {avg_bulk_time:.2f}ms, "
                          f"Success rate: {bulk_success_rate:.1f}%")
        
        print(f"\n✅ Scalability test complete! Total results: {len(self.results)}")
        self.db_manager.disconnect()
    
    def save_results_to_csv(self, filename: str = "scalability_results.csv") -> None:
        """Save the test results to a CSV file."""
        if not self.results:
            print("No results to save.")
            return
        
        with open(filename, 'w', newline='') as csvfile:
            fieldnames = [
                'timestamp', 'total_contacts', 'total_properties', 'endpoint', 
                'contact_id', 'response_time_ms', 'status_code', 'response_size_bytes',
                'recommendations_count', 'success', 'error_message'
            ]
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writeheader()
            
            for result in self.results:
                writer.writerow({
                    'timestamp': result.timestamp,
                    'total_contacts': result.total_contacts,
                    'total_properties': result.total_properties,
                    'endpoint': result.endpoint,
                    'contact_id': result.contact_id,
                    'response_time_ms': result.response_time_ms,
                    'status_code': result.status_code,
                    'response_size_bytes': result.response_size_bytes,
                    'recommendations_count': result.recommendations_count,
                    'success': result.success,
                    'error_message': result.error_message
                })
        
        print(f"✅ Scalability results saved to {filename}")
    
    def print_summary_stats(self) -> None:
        """Print summary statistics grouped by dataset size and endpoint type."""
        if not self.results:
            print("No results to analyze.")
            return
        
        print("\n" + "="*90)
        print("SCALABILITY TEST SUMMARY")
        print("="*90)
        
        # Group results by dataset size and endpoint
        size_groups = {}
        for result in self.results:
            key = (result.total_contacts, result.total_properties, result.endpoint)
            if key not in size_groups:
                size_groups[key] = []
            size_groups[key].append(result)
        
        # Print single contact results
        print("\nSINGLE CONTACT RECOMMENDATIONS:")
        print(f"{'Contacts':<10} {'Properties':<12} {'Tests':<8} {'Avg Time (ms)':<15} "
              f"{'Success Rate':<12} {'Avg Recommendations':<18}")
        print("-" * 80)
        
        single_contact_groups = {k: v for k, v in size_groups.items() if k[2] == "single_contact"}
        for (contacts, properties, endpoint), results in sorted(single_contact_groups.items()):
            successful_results = [r for r in results if r.success]
            
            if successful_results:
                avg_time = sum(r.response_time_ms for r in successful_results) / len(successful_results)
                avg_recommendations = sum(r.recommendations_count for r in successful_results) / len(successful_results)
            else:
                avg_time = 0
                avg_recommendations = 0
            
            success_rate = len(successful_results) / len(results) * 100
            
            print(f"{contacts:<10} {properties:<12} {len(results):<8} {avg_time:<15.2f} "
                  f"{success_rate:<12.1f}% {avg_recommendations:<18.1f}")
        
        # Print bulk results if any
        bulk_groups = {k: v for k, v in size_groups.items() if k[2].startswith("bulk_recommendations")}
        if bulk_groups:
            print("\nBULK RECOMMENDATIONS:")
            print(f"{'Contacts':<10} {'Properties':<12} {'Bulk Size':<10} {'Tests':<8} {'Avg Time (ms)':<15} "
                  f"{'Success Rate':<12} {'Avg Recommendations':<18}")
            print("-" * 95)
            
            for (contacts, properties, endpoint), results in sorted(bulk_groups.items()):
                # Extract bulk size from endpoint name
                bulk_size = endpoint.split('_')[-1] if '_' in endpoint else "Unknown"
                
                successful_results = [r for r in results if r.success]
                
                if successful_results:
                    avg_time = sum(r.response_time_ms for r in successful_results) / len(successful_results)
                    avg_recommendations = sum(r.recommendations_count for r in successful_results) / len(successful_results)
                else:
                    avg_time = 0
                    avg_recommendations = 0
                
                success_rate = len(successful_results) / len(results) * 100
                
                print(f"{contacts:<10} {properties:<12} {bulk_size:<10} {len(results):<8} {avg_time:<15.2f} "
                      f"{success_rate:<12.1f}% {avg_recommendations:<18.1f}")
        
        # Overall performance comparison
        if bulk_groups:
            print("\nPERFORMANCE COMPARISON:")
            print("-" * 50)
            
            # Get largest dataset results for comparison
            max_contacts = max(contacts for contacts, _, _ in size_groups.keys())
            
            single_results = []
            bulk_results = []
            
            for (contacts, properties, endpoint), results in size_groups.items():
                if contacts == max_contacts:
                    successful = [r for r in results if r.success]
                    if successful:
                        avg_time = sum(r.response_time_ms for r in successful) / len(successful)
                        if endpoint == "single_contact":
                            single_results.append(avg_time)
                        elif endpoint.startswith("bulk_recommendations"):
                            bulk_size = int(endpoint.split('_')[-1])
                            bulk_results.append((bulk_size, avg_time))
            
            if single_results and bulk_results:
                avg_single_time = sum(single_results) / len(single_results)
                print(f"Single contact average time: {avg_single_time:.2f} ms")
                
                for bulk_size, bulk_time in sorted(bulk_results):
                    efficiency = bulk_time / (avg_single_time * bulk_size)
                    print(f"Bulk {bulk_size} contacts: {bulk_time:.2f} ms "
                          f"(Efficiency: {efficiency:.2f}, {'Good' if efficiency < 0.8 else 'Poor'})")
                
                print("\nEfficiency < 0.8 indicates good bulk performance optimization")


def main():
    parser = argparse.ArgumentParser(description='Run scalability tests for recommendation endpoints')
    parser.add_argument('--url', default='http://localhost:8080', 
                       help='Base URL of the API server (default: http://localhost:8080)')
    parser.add_argument('--db-url', 
                       default='postgresql://username:password@localhost/real_estate_db',
                       help='Database connection URL')
    parser.add_argument('--data-dir', default='data',
                       help='Directory containing contacts.json and properties.json (default: data)')
    parser.add_argument('--max-contacts', type=int, default=1000,
                       help='Maximum number of contacts to test (default: 1000)')
    parser.add_argument('--max-properties', type=int, default=2000,
                       help='Maximum number of properties to test (default: 2000)')
    parser.add_argument('--contact-batch-size', type=int, default=50,
                       help='Number of contacts to add per batch (default: 50)')
    parser.add_argument('--property-batch-size', type=int, default=100,
                       help='Number of properties to add per batch (default: 100)')
    parser.add_argument('--tests-per-step', type=int, default=5,
                       help='Number of tests to run per dataset size (default: 5)')
    parser.add_argument('--include-bulk', action='store_true', default=True,
                       help='Include bulk recommendation tests (default: True)')
    parser.add_argument('--no-bulk', action='store_false', dest='include_bulk',
                       help='Disable bulk recommendation tests')
    parser.add_argument('--bulk-sizes', nargs='+', type=int, default=[2, 5, 10, 20],
                       help='Bulk batch sizes to test (default: 2 5 10 20)')
    parser.add_argument('--output', default='scalability_results.csv',
                       help='Output CSV file name (default: scalability_results.csv)')
    
    args = parser.parse_args()
    
    tester = RecommendationScalabilityTester(args.url, args.db_url, args.data_dir)
    tester.run_scalability_test(
        max_contacts=args.max_contacts,
        max_properties=args.max_properties,
        contact_batch_size=args.contact_batch_size,
        property_batch_size=args.property_batch_size,
        tests_per_step=args.tests_per_step,
        include_bulk_tests=args.include_bulk,
        bulk_batch_sizes=args.bulk_sizes
    )
    tester.save_results_to_csv(args.output)
    tester.print_summary_stats()


if __name__ == "__main__":
    main()
