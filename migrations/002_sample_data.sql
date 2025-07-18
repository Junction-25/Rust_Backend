-- Sample data for the simplified schema
-- This migration adds sample data to work with the new simplified schema

-- Insert sample contacts with simplified structure
INSERT INTO contacts (name, preferred_locations, min_budget, max_budget, min_area_sqm, max_area_sqm, property_types, min_rooms) VALUES
('Eileen Barnes', '[{"name": "Around Bab", "lat": 36.73435, "lon": 3.20663}]', 15250000.0, 22280000.0, 65, 100, '["office"]', 0),
('Jonathan Munoz', '[{"name": "Alger Centre", "lat": 36.7631, "lon": 3.0573}]', 16000000.0, 25000000.0, 80, 120, '["apartment", "house"]', 2),
('Michelle Williams', '[{"name": "Constantine", "lat": 36.365, "lon": 6.6147}]', 20000000.0, 35000000.0, 100, 200, '["house", "villa"]', 3),
('Robert Johnson', '[{"name": "Oran", "lat": 35.6969, "lon": -0.6331}]', 18000000.0, 28000000.0, 90, 150, '["apartment", "office"]', 1),
('Sarah Davis', '[{"name": "Around Alger", "lat": 36.7538, "lon": 3.0588}]', 12000000.0, 20000000.0, 60, 90, '["apartment", "studio"]', 1);

-- Insert sample properties with simplified structure
INSERT INTO properties (address, lat, lon, price, area_sqm, property_type, number_of_rooms) VALUES
('11458 Christopher Point, Bab', 36.7243, 3.21647, 15540000.0, 137, 'apartment', 3),
('22822 Leblanc Squares, Constantine', 36.37705, 6.59604, 24980000.0, 386, 'land', 0),
('280 Woods Oval Apt. 572, Constantine', 36.37352, 6.61371, 30810000.0, 906, 'land', 0),
('45 Rue de la République, Alger', 36.7538, 3.0588, 18500000.0, 95, 'apartment', 2),
('123 Boulevard Mohamed V, Oran', 35.6969, -0.6331, 22000000.0, 120, 'house', 4),
('78 Avenue Pasteur, Constantine', 36.365, 6.6147, 16800000.0, 85, 'apartment', 2),
('156 Rue Didouche Mourad, Alger', 36.7631, 3.0573, 35000000.0, 150, 'office', 0),
('234 Avenue de l''Indépendance, Oran', 35.7022, -0.6412, 19500000.0, 110, 'apartment', 3),
('67 Rue Ben Badis, Bab El Oued', 36.7832, 3.0456, 14200000.0, 75, 'apartment', 1),
('189 Boulevard Zighoud Youcef, Constantine', 36.3711, 6.6042, 28500000.0, 200, 'house', 5);
