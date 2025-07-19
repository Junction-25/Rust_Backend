#!/usr/bin/env python3
"""
Batch Recommendation Script for Real Estate System
Fetches recommendations for all properties from the Rust API and saves to JSON file
"""

import json
import requests
import psycopg2
import sys
import os
from datetime import datetime
from typing import List, Dict, Any, Optional
import time

class BatchRecommendationProcessor:
    def __init__(self, api_base_url: str = "http://localhost:8080", db_url: str = None):
        self.api_base_url = api_base_url.rstrip('/')
        self.db_url = db_url or os.getenv('DATABASE_URL', 'postgresql:///real_estate_db')
        self.session = requests.Session()
        self.session.headers.update({
            'Content-Type': 'application/json',
            'Accept': 'application/json'
        })
    
    def connect_to_database(self) -> psycopg2.extensions.connection:
        """Connect to PostgreSQL database"""
        try:
            conn = psycopg2.connect(self.db_url)
            return conn
        except Exception as e:
            print(f"❌ Database connection failed: {e}")
            sys.exit(1)
    
    def get_all_property_ids(self) -> List[int]:
        """Get all property IDs from the database"""
        conn = self.connect_to_database()
        cursor = conn.cursor()
        
        try:
            print("📋 Fetching all property IDs from database...")
            cursor.execute("SELECT id FROM properties ORDER BY id;")
            property_ids = [row[0] for row in cursor.fetchall()]
            print(f"✅ Found {len(property_ids)} properties")
            return property_ids
        except Exception as e:
            print(f"❌ Error fetching property IDs: {e}")
            sys.exit(1)
        finally:
            cursor.close()
            conn.close()
    
    def get_database_stats(self) -> Dict[str, int]:
        """Get database statistics"""
        conn = self.connect_to_database()
        cursor = conn.cursor()
        
        try:
            cursor.execute("SELECT COUNT(*) FROM properties;")
            property_count = cursor.fetchone()[0]
            
            cursor.execute("SELECT COUNT(*) FROM contacts;")
            contact_count = cursor.fetchone()[0]
            
            return {
                'properties': property_count,
                'contacts': contact_count
            }
        except Exception as e:
            print(f"❌ Error getting database stats: {e}")
            return {'properties': 0, 'contacts': 0}
        finally:
            cursor.close()
            conn.close()
    
    def test_api_connection(self) -> bool:
        """Test if the API is accessible"""
        try:
            print(f"🔌 Testing API connection to {self.api_base_url}...")
            response = self.session.get(f"{self.api_base_url}/health", timeout=5)
            if response.status_code == 200:
                print("✅ API connection successful")
                return True
            else:
                print(f"⚠️  API responded with status {response.status_code}")
                return False
        except requests.exceptions.RequestException as e:
            print(f"❌ API connection failed: {e}")
            return False
    
    def call_bulk_recommendations_api(
        self, 
        property_ids: Optional[List[int]] = None,
        limit_per_property: int = 10,
        min_score: float = 0.1,
        top_k: Optional[int] = None,
        top_percentile: Optional[float] = None,
        score_threshold_percentile: Optional[float] = None,
        budget_weight: float = 0.3,
        location_weight: float = 0.25,
        property_type_weight: float = 0.2,
        size_weight: float = 0.25
    ) -> Dict[str, Any]:
        """Call the bulk recommendations API"""
        
        url = f"{self.api_base_url}/recommendations/bulk"
        
        payload = {
            "limit_per_property": limit_per_property,
            "min_score": min_score,
            "budget_weight": budget_weight,
            "location_weight": location_weight,
            "property_type_weight": property_type_weight,
            "size_weight": size_weight
        }
        
        # Add optional parameters
        if property_ids:
            payload["property_ids"] = property_ids
        if top_k:
            payload["top_k"] = top_k
        if top_percentile:
            payload["top_percentile"] = top_percentile
        if score_threshold_percentile:
            payload["score_threshold_percentile"] = score_threshold_percentile
        
        try:
            print(f"🚀 Calling bulk recommendations API...")
            print(f"   📊 Parameters: {json.dumps(payload, indent=2)}")
            
            response = self.session.post(url, json=payload, timeout=120)
            
            if response.status_code == 200:
                result = response.json()
                print(f"✅ API call successful!")
                print(f"   📈 Processing time: {result.get('processing_time_ms', 0)}ms")
                print(f"   🏢 Properties processed: {result.get('total_properties', 0)}")
                print(f"   🎯 Total recommendations: {result.get('total_recommendations', 0)}")
                return result
            else:
                print(f"❌ API call failed with status {response.status_code}")
                print(f"   Response: {response.text}")
                return None
                
        except requests.exceptions.RequestException as e:
            print(f"❌ API request failed: {e}")
            return None
    
    def process_batch_recommendations(
        self,
        batch_size: int = 100,
        limit_per_property: int = 10,
        min_score: float = 0.1,
        **kwargs
    ) -> List[Dict[str, Any]]:
        """Process recommendations in batches"""
        
        all_property_ids = self.get_all_property_ids()
        all_results = []
        
        print(f"📦 Processing {len(all_property_ids)} properties in batches of {batch_size}")
        
        for i in range(0, len(all_property_ids), batch_size):
            batch_ids = all_property_ids[i:i + batch_size]
            batch_num = (i // batch_size) + 1
            total_batches = (len(all_property_ids) + batch_size - 1) // batch_size
            
            print(f"\n🔄 Processing batch {batch_num}/{total_batches} (Properties {i+1}-{min(i+batch_size, len(all_property_ids))})")
            
            result = self.call_bulk_recommendations_api(
                property_ids=batch_ids,
                limit_per_property=limit_per_property,
                min_score=min_score,
                **kwargs
            )
            
            if result:
                all_results.append({
                    'batch_number': batch_num,
                    'property_ids_range': f"{i+1}-{min(i+batch_size, len(all_property_ids))}",
                    'batch_size': len(batch_ids),
                    'result': result
                })
                print(f"✅ Batch {batch_num} completed successfully")
            else:
                print(f"❌ Batch {batch_num} failed")
                break
            
            # Small delay between batches to avoid overwhelming the API
            if i + batch_size < len(all_property_ids):
                time.sleep(0.5)
        
        return all_results
    
    def save_results_to_json(self, results: List[Dict[str, Any]], output_file: str):
        """Save results to JSON file with metadata"""
        
        # Calculate summary statistics
        total_properties = 0
        total_recommendations = 0
        total_processing_time = 0
        
        for batch in results:
            if 'result' in batch:
                batch_result = batch['result']
                total_properties += batch_result.get('total_properties', 0)
                total_recommendations += batch_result.get('total_recommendations', 0)
                total_processing_time += batch_result.get('processing_time_ms', 0)
        
        # Create final output structure
        output_data = {
            'metadata': {
                'generated_at': datetime.now().isoformat(),
                'api_base_url': self.api_base_url,
                'database_url': self.db_url.split('@')[-1] if '@' in self.db_url else self.db_url,  # Hide credentials
                'summary': {
                    'total_batches': len(results),
                    'total_properties_processed': total_properties,
                    'total_recommendations_generated': total_recommendations,
                    'total_processing_time_ms': total_processing_time,
                    'average_recommendations_per_property': round(total_recommendations / total_properties, 2) if total_properties > 0 else 0
                }
            },
            'batch_results': results
        }
        
        try:
            print(f"💾 Saving results to {output_file}...")
            with open(output_file, 'w', encoding='utf-8') as f:
                json.dump(output_data, f, indent=2, ensure_ascii=False)
            
            file_size = os.path.getsize(output_file) / (1024 * 1024)  # MB
            print(f"✅ Results saved successfully!")
            print(f"   📁 File: {output_file}")
            print(f"   📊 Size: {file_size:.2f} MB")
            print(f"   🏢 Properties: {total_properties:,}")
            print(f"   🎯 Recommendations: {total_recommendations:,}")
            
        except Exception as e:
            print(f"❌ Error saving results: {e}")
    
    def run_full_batch_process(
        self,
        output_file: str = None,
        batch_size: int = 100,
        limit_per_property: int = 10,
        min_score: float = 0.1,
        single_batch: bool = True,
        **kwargs
    ):
        """Run the complete batch recommendation process"""
        
        if not output_file:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            output_file = f"batch_recommendations_{timestamp}.json"
        
        print("🚀 Real Estate Batch Recommendation Process")
        print("=" * 60)
        
        # Get database stats
        stats = self.get_database_stats()
        print(f"📊 Database Statistics:")
        print(f"   Properties: {stats['properties']:,}")
        print(f"   Contacts: {stats['contacts']:,}")
        
        # Test API connection
        if not self.test_api_connection():
            print("❌ Cannot proceed without API connection")
            sys.exit(1)
        
        # Process recommendations
        if single_batch:
            print(f"\n🔄 Processing ALL properties in a single batch...")
            print(f"   Recommendations per property: {limit_per_property}")
            print(f"   Minimum score threshold: {min_score}")
            
            # Get all property IDs
            all_property_ids = self.get_all_property_ids()
            
            # Call API with all properties at once
            result = self.call_bulk_recommendations_api(
                property_ids=all_property_ids,
                limit_per_property=limit_per_property,
                min_score=min_score,
                **kwargs
            )
            
            if result:
                # Wrap single result in batch format for consistency
                results = [{
                    'batch_number': 1,
                    'property_ids_range': f"1-{len(all_property_ids)}",
                    'batch_size': len(all_property_ids),
                    'result': result
                }]
            else:
                results = []
        else:
            print(f"\n🔄 Starting batch processing...")
            print(f"   Batch size: {batch_size}")
            print(f"   Recommendations per property: {limit_per_property}")
            print(f"   Minimum score threshold: {min_score}")
            
            results = self.process_batch_recommendations(
                batch_size=batch_size,
                limit_per_property=limit_per_property,
                min_score=min_score,
                **kwargs
            )
        
        if results:
            self.save_results_to_json(results, output_file)
            print("\n" + "=" * 60)
            print("🎉 Batch recommendation process completed successfully!")
            print(f"📁 Results saved to: {output_file}")
        else:
            print("❌ Batch process failed - no results to save")

def main():
    """Main function with configurable parameters"""
    
    # Configuration
    config = {
        'api_base_url': os.getenv('API_BASE_URL', 'http://localhost:8080'),
        'batch_size': int(os.getenv('BATCH_SIZE', '50')),  # Only used if single_batch=False
        'limit_per_property': int(os.getenv('LIMIT_PER_PROPERTY', '10')),
        'min_score': float(os.getenv('MIN_SCORE', '0.1')),
        'single_batch': os.getenv('SINGLE_BATCH', 'true').lower() == 'true',  # Default to single batch
        'top_k': int(os.getenv('TOP_K')) if os.getenv('TOP_K') else None,
        'top_percentile': float(os.getenv('TOP_PERCENTILE')) if os.getenv('TOP_PERCENTILE') else None,
        'budget_weight': float(os.getenv('BUDGET_WEIGHT', '0.3')),
        'location_weight': float(os.getenv('LOCATION_WEIGHT', '0.25')),
        'property_type_weight': float(os.getenv('PROPERTY_TYPE_WEIGHT', '0.2')),
        'size_weight': float(os.getenv('SIZE_WEIGHT', '0.25')),
        'output_file': os.getenv('OUTPUT_FILE')
    }
    
    print("🔧 Configuration:")
    for key, value in config.items():
        if value is not None:
            print(f"   {key}: {value}")
    print()
    
    # Create processor and run
    processor = BatchRecommendationProcessor(api_base_url=config['api_base_url'])
    
    # Remove None values and output_file from kwargs
    kwargs = {k: v for k, v in config.items() if v is not None and k not in ['api_base_url', 'output_file']}
    output_file = kwargs.pop('output_file', None)
    
    processor.run_full_batch_process(
        output_file=output_file,
        **kwargs
    )

if __name__ == "__main__":
    main()
