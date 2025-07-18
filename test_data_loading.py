#!/usr/bin/env python3
"""
Test script to verify that the scalability test can load data correctly.
"""

import json
import sys
import os

def test_data_loading():
    """Test that we can load the contact and property data."""
    
    # Check if data files exist
    contacts_file = "data/contacts.json"
    properties_file = "data/properties.json"
    
    if not os.path.exists(contacts_file):
        print(f"❌ Contact data file not found: {contacts_file}")
        return False
    
    if not os.path.exists(properties_file):
        print(f"❌ Property data file not found: {properties_file}")
        return False
    
    # Try to load and validate the data
    try:
        with open(contacts_file, 'r') as f:
            contacts_data = json.load(f)
        
        print(f"✓ Loaded {len(contacts_data)} contacts")
        
        # Check first contact structure
        if contacts_data:
            first_contact = contacts_data[0]
            required_fields = ['name', 'preferred_locations', 'min_budget', 'max_budget', 
                             'min_area_sqm', 'max_area_sqm', 'property_types', 'min_rooms']
            
            for field in required_fields:
                if field not in first_contact:
                    print(f"❌ Missing field '{field}' in contact data")
                    return False
            
            print("✓ Contact data structure is valid")
        
    except Exception as e:
        print(f"❌ Error loading contact data: {e}")
        return False
    
    try:
        with open(properties_file, 'r') as f:
            properties_data = json.load(f)
        
        print(f"✓ Loaded {len(properties_data)} properties")
        
        # Check first property structure
        if properties_data:
            first_property = properties_data[0]
            required_fields = ['address', 'location', 'price', 'area_sqm', 
                             'property_type', 'number_of_rooms']
            
            for field in required_fields:
                if field not in first_property:
                    print(f"❌ Missing field '{field}' in property data")
                    return False
            
            # Check location structure
            location = first_property['location']
            if 'lat' not in location or 'lon' not in location:
                print("❌ Invalid location structure in property data")
                return False
            
            print("✓ Property data structure is valid")
        
    except Exception as e:
        print(f"❌ Error loading property data: {e}")
        return False
    
    print(f"\n📊 Data Summary:")
    print(f"   Contacts: {len(contacts_data):,}")
    print(f"   Properties: {len(properties_data):,}")
    print(f"   Contact to Property Ratio: 1:{len(properties_data)/len(contacts_data):.1f}")
    
    return True

if __name__ == "__main__":
    print("🧪 Testing data loading for scalability test...")
    
    if test_data_loading():
        print("\n✅ All data loading tests passed!")
        print("The scalability test should work correctly with this data.")
    else:
        print("\n❌ Data loading tests failed!")
        print("Please check the data files and structure.")
        sys.exit(1)
