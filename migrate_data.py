#!/usr/bin/env python3
"""
Data Migration Script for Real Estate Recommendation System
Loads contacts and properties from JSON files into PostgreSQL database
"""

import json
import psycopg2
import sys
from typing import List, Dict, Any
import os

def load_json_file(filepath: str) -> List[Dict[str, Any]]:
    """Load and parse JSON file"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            return json.load(f)
    except Exception as e:
        print(f"❌ Error loading {filepath}: {e}")
        sys.exit(1)

def connect_to_database() -> psycopg2.extensions.connection:
    """Connect to PostgreSQL database"""
    try:
        # Use DATABASE_URL from environment or default
        database_url = os.getenv('DATABASE_URL', 'postgresql:///real_estate_db')
        conn = psycopg2.connect(database_url)
        conn.autocommit = False
        return conn
    except Exception as e:
        print(f"❌ Database connection failed: {e}")
        sys.exit(1)

def clear_existing_data(conn: psycopg2.extensions.connection):
    """Clear existing data from tables"""
    cursor = conn.cursor()
    try:
        print("🗑️  Clearing existing data...")
        cursor.execute("DELETE FROM contacts;")
        cursor.execute("DELETE FROM properties;")
        cursor.execute("ALTER SEQUENCE contacts_id_seq RESTART WITH 1;")
        cursor.execute("ALTER SEQUENCE properties_id_seq RESTART WITH 1;")
        conn.commit()
        print("✅ Existing data cleared")
    except Exception as e:
        conn.rollback()
        print(f"❌ Error clearing data: {e}")
        sys.exit(1)
    finally:
        cursor.close()

def insert_properties(conn: psycopg2.extensions.connection, properties: List[Dict[str, Any]]):
    """Insert properties data"""
    cursor = conn.cursor()
    try:
        print(f"📦 Inserting {len(properties)} properties...")
        
        # Prepare batch insert
        insert_query = """
        INSERT INTO properties (address, lat, lon, price, area_sqm, property_type, number_of_rooms)
        VALUES (%s, %s, %s, %s, %s, %s, %s)
        """
        
        # Prepare data tuples
        property_data = []
        for prop in properties:
            property_data.append((
                prop['address'],
                prop['location']['lat'],
                prop['location']['lon'],
                float(prop['price']),
                int(prop['area_sqm']),
                prop['property_type'],
                int(prop['number_of_rooms'])
            ))
        
        # Execute batch insert
        cursor.executemany(insert_query, property_data)
        conn.commit()
        print(f"✅ {len(properties)} properties inserted successfully")
        
    except Exception as e:
        conn.rollback()
        print(f"❌ Error inserting properties: {e}")
        sys.exit(1)
    finally:
        cursor.close()

def insert_contacts(conn: psycopg2.extensions.connection, contacts: List[Dict[str, Any]]):
    """Insert contacts data"""
    cursor = conn.cursor()
    try:
        print(f"👥 Inserting {len(contacts)} contacts...")
        
        # Prepare batch insert
        insert_query = """
        INSERT INTO contacts (name, preferred_locations, min_budget, max_budget, 
                            min_area_sqm, max_area_sqm, property_types, min_rooms)
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
        """
        
        # Prepare data tuples
        contact_data = []
        for contact in contacts:
            contact_data.append((
                contact['name'],
                json.dumps(contact['preferred_locations']),  # Convert to JSON string
                float(contact['min_budget']),
                float(contact['max_budget']),
                int(contact['min_area_sqm']),
                int(contact['max_area_sqm']),
                json.dumps(contact['property_types']),  # Convert to JSON string
                int(contact['min_rooms'])
            ))
        
        # Execute batch insert in chunks for better performance
        chunk_size = 1000
        for i in range(0, len(contact_data), chunk_size):
            chunk = contact_data[i:i + chunk_size]
            cursor.executemany(insert_query, chunk)
            conn.commit()
            print(f"   📝 Inserted contacts {i + 1} to {min(i + chunk_size, len(contact_data))}")
        
        print(f"✅ {len(contacts)} contacts inserted successfully")
        
    except Exception as e:
        conn.rollback()
        print(f"❌ Error inserting contacts: {e}")
        sys.exit(1)
    finally:
        cursor.close()

def verify_data(conn: psycopg2.extensions.connection):
    """Verify the inserted data"""
    cursor = conn.cursor()
    try:
        cursor.execute("SELECT COUNT(*) FROM properties;")
        property_count = cursor.fetchone()[0]
        
        cursor.execute("SELECT COUNT(*) FROM contacts;")
        contact_count = cursor.fetchone()[0]
        
        print(f"📊 Database verification:")
        print(f"   Properties: {property_count:,}")
        print(f"   Contacts: {contact_count:,}")
        
        # Show sample data
        cursor.execute("SELECT address, price, property_type FROM properties LIMIT 3;")
        sample_properties = cursor.fetchall()
        print(f"   Sample properties:")
        for prop in sample_properties:
            print(f"     - {prop[0]} | {prop[1]:,.0f} DZD | {prop[2]}")
            
        cursor.execute("SELECT name, min_budget, max_budget FROM contacts LIMIT 3;")
        sample_contacts = cursor.fetchall()
        print(f"   Sample contacts:")
        for contact in sample_contacts:
            print(f"     - {contact[0]} | Budget: {contact[1]:,.0f} - {contact[2]:,.0f} DZD")
        
    except Exception as e:
        print(f"❌ Error verifying data: {e}")
    finally:
        cursor.close()

def main():
    """Main migration function"""
    print("🚀 Real Estate Data Migration Starting...")
    print("=" * 50)
    
    # Load JSON data
    print("📂 Loading JSON files...")
    properties = load_json_file('data/properties.json')
    contacts = load_json_file('data/contacts.json')
    
    print(f"📊 Data loaded:")
    print(f"   Properties: {len(properties):,}")
    print(f"   Contacts: {len(contacts):,}")
    
    # Connect to database
    print("🔌 Connecting to database...")
    conn = connect_to_database()
    
    try:
        # Clear existing data
        clear_existing_data(conn)
        
        # Insert new data
        insert_properties(conn, properties)
        insert_contacts(conn, contacts)
        
        # Verify data
        verify_data(conn)
        
        print("=" * 50)
        print("🎉 Data migration completed successfully!")
        print("🚀 Ready for production with full dataset!")
        
    finally:
        conn.close()

if __name__ == "__main__":
    main()
