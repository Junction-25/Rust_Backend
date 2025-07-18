use crate::models::{Contact, Property, Location, NamedLocation};
use anyhow::Result;
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct Repository {
    pool: PgPool,
}

impl Repository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Property operations
    pub async fn get_property_by_id(&self, id: i32) -> Result<Option<Property>> {
        let row = sqlx::query(
            "SELECT id, address, lat, lon, price, area_sqm, property_type, number_of_rooms FROM properties WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Property {
                id: row.get("id"),
                address: row.get("address"),
                location: Location {
                    lat: row.get("lat"),
                    lon: row.get("lon"),
                },
                price: row.get("price"),
                area_sqm: row.get("area_sqm"),
                property_type: row.get("property_type"),
                number_of_rooms: row.get("number_of_rooms"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_active_properties(&self) -> Result<Vec<Property>> {
        let rows = sqlx::query(
            "SELECT id, address, lat, lon, price, area_sqm, property_type, number_of_rooms FROM properties"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut properties = Vec::new();
        for row in rows {
            properties.push(Property {
                id: row.get("id"),
                address: row.get("address"),
                location: Location {
                    lat: row.get("lat"),
                    lon: row.get("lon"),
                },
                price: row.get("price"),
                area_sqm: row.get("area_sqm"),
                property_type: row.get("property_type"),
                number_of_rooms: row.get("number_of_rooms"),
            });
        }

        Ok(properties)
    }

    pub async fn get_properties_by_ids(&self, ids: &[i32]) -> Result<Vec<Property>> {
        let rows = sqlx::query(
            "SELECT id, address, lat, lon, price, area_sqm, property_type, number_of_rooms FROM properties WHERE id = ANY($1)"
        )
        .bind(ids)
        .fetch_all(&self.pool)
        .await?;

        let mut properties = Vec::new();
        for row in rows {
            properties.push(Property {
                id: row.get("id"),
                address: row.get("address"),
                location: Location {
                    lat: row.get("lat"),
                    lon: row.get("lon"),
                },
                price: row.get("price"),
                area_sqm: row.get("area_sqm"),
                property_type: row.get("property_type"),
                number_of_rooms: row.get("number_of_rooms"),
            });
        }

        Ok(properties)
    }

    // Contact operations
    pub async fn get_contact_by_id(&self, id: i32) -> Result<Option<Contact>> {
        let row = sqlx::query(
            "SELECT id, name, preferred_locations, min_budget, max_budget, min_area_sqm, max_area_sqm, property_types, min_rooms FROM contacts WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let preferred_locations: Vec<NamedLocation> = serde_json::from_value(row.get::<serde_json::Value, _>("preferred_locations"))?;
            let property_types: Vec<String> = serde_json::from_value(row.get::<serde_json::Value, _>("property_types"))?;
            
            Ok(Some(Contact {
                id: row.get("id"),
                name: row.get("name"),
                preferred_locations,
                min_budget: row.get("min_budget"),
                max_budget: row.get("max_budget"),
                min_area_sqm: row.get("min_area_sqm"),
                max_area_sqm: row.get("max_area_sqm"),
                property_types,
                min_rooms: row.get("min_rooms"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_active_contacts(&self) -> Result<Vec<Contact>> {
        let rows = sqlx::query(
            "SELECT id, name, preferred_locations, min_budget, max_budget, min_area_sqm, max_area_sqm, property_types, min_rooms FROM contacts"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut contacts = Vec::new();
        for row in rows {
            let preferred_locations: Vec<NamedLocation> = serde_json::from_value(row.get::<serde_json::Value, _>("preferred_locations"))?;
            let property_types: Vec<String> = serde_json::from_value(row.get::<serde_json::Value, _>("property_types"))?;
            
            contacts.push(Contact {
                id: row.get("id"),
                name: row.get("name"),
                preferred_locations,
                min_budget: row.get("min_budget"),
                max_budget: row.get("max_budget"),
                min_area_sqm: row.get("min_area_sqm"),
                max_area_sqm: row.get("max_area_sqm"),
                property_types,
                min_rooms: row.get("min_rooms"),
            });
        }

        Ok(contacts)
    }

    pub async fn create_contact(&self, contact: &Contact) -> Result<Contact> {
        let preferred_locations_json = serde_json::to_value(&contact.preferred_locations)?;
        let property_types_json = serde_json::to_value(&contact.property_types)?;

        let row = sqlx::query(
            "INSERT INTO contacts (name, preferred_locations, min_budget, max_budget, min_area_sqm, max_area_sqm, property_types, min_rooms) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
        )
        .bind(&contact.name)
        .bind(preferred_locations_json)
        .bind(contact.min_budget)
        .bind(contact.max_budget)
        .bind(contact.min_area_sqm)
        .bind(contact.max_area_sqm)
        .bind(property_types_json)
        .bind(contact.min_rooms)
        .fetch_one(&self.pool)
        .await?;

        let id: i32 = row.get("id");
        let mut new_contact = contact.clone();
        new_contact.id = id;
        Ok(new_contact)
    }

    pub async fn create_property(&self, property: &Property) -> Result<Property> {
        let row = sqlx::query(
            "INSERT INTO properties (address, lat, lon, price, area_sqm, property_type, number_of_rooms) 
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"
        )
        .bind(&property.address)
        .bind(property.location.lat)
        .bind(property.location.lon)
        .bind(property.price)
        .bind(property.area_sqm)
        .bind(&property.property_type)
        .bind(property.number_of_rooms)
        .fetch_one(&self.pool)
        .await?;

        let id: i32 = row.get("id");
        let mut new_property = property.clone();
        new_property.id = id;
        Ok(new_property)
    }

    // Distance calculation helper
    pub fn calculate_distance(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let r = 6371.0; // Earth's radius in kilometers
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();
        let a = (d_lat / 2.0).sin().powi(2)
            + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        r * c
    }
}
