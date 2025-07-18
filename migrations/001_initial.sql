-- Create custom types
CREATE TYPE property_type AS ENUM (
    'apartment',
    'house',
    'condo',
    'townhouse',
    'villa',
    'studio',
    'commercial'
);

-- Create properties table
CREATE TABLE properties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR NOT NULL,
    description TEXT,
    property_type VARCHAR NOT NULL,
    price BIGINT NOT NULL, -- Price in cents
    location JSONB NOT NULL,
    area_sqm INTEGER NOT NULL,
    rooms INTEGER NOT NULL,
    bathrooms INTEGER NOT NULL,
    features TEXT[] DEFAULT '{}',
    images TEXT[] DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE
);

-- Create contacts table
CREATE TABLE contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    email VARCHAR UNIQUE NOT NULL,
    phone VARCHAR,
    budget_min BIGINT NOT NULL, -- Budget in cents
    budget_max BIGINT NOT NULL, -- Budget in cents
    preferred_locations JSONB DEFAULT '[]',
    preferred_property_types JSONB DEFAULT '[]',
    min_rooms INTEGER,
    max_rooms INTEGER,
    min_area INTEGER,
    max_area INTEGER,
    required_features TEXT[] DEFAULT '{}',
    preferred_features TEXT[] DEFAULT '{}',
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE
);

-- Create indexes for better performance
CREATE INDEX idx_properties_price ON properties(price);
CREATE INDEX idx_properties_area ON properties(area_sqm);
CREATE INDEX idx_properties_rooms ON properties(rooms);
CREATE INDEX idx_properties_is_active ON properties(is_active);
CREATE INDEX idx_properties_location_gin ON properties USING GIN(location);

CREATE INDEX idx_contacts_budget ON contacts(budget_min, budget_max);
CREATE INDEX idx_contacts_email ON contacts(email);
CREATE INDEX idx_contacts_is_active ON contacts(is_active);
CREATE INDEX idx_contacts_preferred_locations_gin ON contacts USING GIN(preferred_locations);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers to automatically update updated_at
CREATE TRIGGER update_properties_updated_at
    BEFORE UPDATE ON properties
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_contacts_updated_at
    BEFORE UPDATE ON contacts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
