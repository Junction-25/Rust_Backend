-- Insert sample properties
INSERT INTO properties (id, title, description, property_type, price, location, area_sqm, rooms, bathrooms, features, images) VALUES
(
    gen_random_uuid(),
    'Modern Downtown Apartment',
    'Beautiful 2-bedroom apartment in the heart of downtown with city views',
    'apartment',
    350000000, -- $3,500,000 in cents
    '{"address": "123 Main St", "city": "New York", "state": "NY", "country": "USA", "postal_code": "10001", "latitude": 40.7589, "longitude": -73.9851}',
    95,
    2,
    2,
    ARRAY['parking', 'gym', 'pool', 'doorman', 'balcony']::text[],
    ARRAY['https://example.com/image1.jpg', 'https://example.com/image2.jpg']::text[]
),
(
    gen_random_uuid(),
    'Suburban Family House',
    'Spacious 4-bedroom house perfect for families with large backyard',
    'house',
    450000000, -- $4,500,000 in cents
    '{"address": "456 Oak Ave", "city": "Brooklyn", "state": "NY", "country": "USA", "postal_code": "11201", "latitude": 40.6892, "longitude": -73.9442}',
    180,
    4,
    3,
    ARRAY['garage', 'garden', 'fireplace', 'basement']::text[],
    ARRAY['https://example.com/house1.jpg', 'https://example.com/house2.jpg']::text[]
),
(
    gen_random_uuid(),
    'Luxury Penthouse',
    'Exclusive penthouse with panoramic views and premium amenities',
    'apartment',
    800000000, -- $8,000,000 in cents
    '{"address": "789 Park Ave", "city": "New York", "state": "NY", "country": "USA", "postal_code": "10021", "latitude": 40.7736, "longitude": -73.9566}',
    250,
    3,
    3,
    ARRAY['concierge', 'rooftop', 'gym', 'pool', 'spa', 'parking']::text[],
    ARRAY['https://example.com/penthouse1.jpg']::text[]
),
(
    gen_random_uuid(),
    'Cozy Studio in Williamsburg',
    'Charming studio apartment in trendy Williamsburg neighborhood',
    'studio',
    280000000, -- $2,800,000 in cents
    '{"address": "321 Berry St", "city": "Brooklyn", "state": "NY", "country": "USA", "postal_code": "11249", "latitude": 40.7208, "longitude": -73.9538}',
    45,
    1,
    1,
    ARRAY['laundry', 'bike_storage', 'rooftop']::text[],
    ARRAY['https://example.com/studio1.jpg']::text[]
);

-- Insert sample contacts
INSERT INTO contacts (id, first_name, last_name, email, phone, budget_min, budget_max, preferred_locations, preferred_property_types, min_rooms, max_rooms, min_area, max_area, required_features, preferred_features, notes) VALUES
(
    gen_random_uuid(),
    'John',
    'Smith',
    'john.smith@email.com',
    '+1-555-0101',
    300000000, -- $3,000,000 in cents
    500000000, -- $5,000,000 in cents
    '[
        {"address": "", "city": "New York", "state": "NY", "country": "USA", "postal_code": "", "latitude": 40.7589, "longitude": -73.9851},
        {"address": "", "city": "Brooklyn", "state": "NY", "country": "USA", "postal_code": "", "latitude": 40.6892, "longitude": -73.9442}
    ]',
    '["apartment", "condo"]',
    2,
    3,
    80,
    150,
    ARRAY['parking']::text[],
    ARRAY['gym', 'doorman', 'balcony']::text[],
    'Looking for a modern apartment with good amenities'
),
(
    gen_random_uuid(),
    'Sarah',
    'Johnson',
    'sarah.johnson@email.com',
    '+1-555-0102',
    400000000, -- $4,000,000 in cents
    600000000, -- $6,000,000 in cents
    '[
        {"address": "", "city": "Brooklyn", "state": "NY", "country": "USA", "postal_code": "", "latitude": 40.6892, "longitude": -73.9442}
    ]',
    '["house", "townhouse"]',
    3,
    5,
    150,
    250,
    ARRAY['garage', 'garden']::text[],
    ARRAY['fireplace', 'basement']::text[],
    'Family looking for a house with outdoor space'
),
(
    gen_random_uuid(),
    'Michael',
    'Chen',
    'michael.chen@email.com',
    '+1-555-0103',
    250000000, -- $2,500,000 in cents
    350000000, -- $3,500,000 in cents
    '[
        {"address": "", "city": "Brooklyn", "state": "NY", "country": "USA", "postal_code": "", "latitude": 40.7208, "longitude": -73.9538}
    ]',
    '["studio", "apartment"]',
    1,
    2,
    40,
    100,
    ARRAY[]::text[],
    ARRAY['laundry', 'bike_storage', 'rooftop']::text[],
    'Young professional looking for trendy neighborhood'
),
(
    gen_random_uuid(),
    'Emily',
    'Rodriguez',
    'emily.rodriguez@email.com',
    '+1-555-0104',
    700000000, -- $7,000,000 in cents
    1000000000, -- $10,000,000 in cents
    '[
        {"address": "", "city": "New York", "state": "NY", "country": "USA", "postal_code": "", "latitude": 40.7736, "longitude": -73.9566}
    ]',
    '["apartment", "condo"]',
    2,
    4,
    200,
    400,
    ARRAY['concierge', 'parking']::text[],
    ARRAY['gym', 'pool', 'spa', 'rooftop']::text[],
    'Looking for luxury living with premium amenities'
);
