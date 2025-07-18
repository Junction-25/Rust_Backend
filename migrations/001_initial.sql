-- Create properties table with new simplified schema
CREATE TABLE properties (
    id SERIAL PRIMARY KEY,
    address VARCHAR NOT NULL,
    lat DOUBLE PRECISION NOT NULL,
    lon DOUBLE PRECISION NOT NULL,
    price DOUBLE PRECISION NOT NULL,
    area_sqm INTEGER NOT NULL,
    property_type VARCHAR NOT NULL,
    number_of_rooms INTEGER NOT NULL
);

-- Create contacts table with new simplified schema
CREATE TABLE contacts (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    preferred_locations JSONB DEFAULT '[]',
    min_budget DOUBLE PRECISION NOT NULL,
    max_budget DOUBLE PRECISION NOT NULL,
    min_area_sqm INTEGER NOT NULL,
    max_area_sqm INTEGER NOT NULL,
    property_types JSONB DEFAULT '[]',
    min_rooms INTEGER NOT NULL
);

-- Create indexes for better performance
CREATE INDEX idx_properties_price ON properties(price);
CREATE INDEX idx_properties_area ON properties(area_sqm);
CREATE INDEX idx_properties_rooms ON properties(number_of_rooms);
CREATE INDEX idx_properties_type ON properties(property_type);
CREATE INDEX idx_properties_location ON properties(lat, lon);

CREATE INDEX idx_contacts_budget ON contacts(min_budget, max_budget);
CREATE INDEX idx_contacts_area ON contacts(min_area_sqm, max_area_sqm);
CREATE INDEX idx_contacts_rooms ON contacts(min_rooms);
CREATE INDEX idx_contacts_preferred_locations_gin ON contacts USING GIN(preferred_locations);
CREATE INDEX idx_contacts_property_types_gin ON contacts USING GIN(property_types);
q