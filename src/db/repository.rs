use crate::models::{Contact, Property, Location, PropertyType};
use anyhow::Result;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct Repository {
    pool: PgPool,
}

impl Repository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Property operations
    pub async fn get_property_by_id(&self, id: Uuid) -> Result<Option<Property>> {
        let row = sqlx::query(
            "SELECT id, title, description, property_type, price, location, area_sqm, rooms, bathrooms, features, images, created_at, updated_at, is_active FROM properties WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let location: Location = serde_json::from_value(row.get::<serde_json::Value, _>("location"))?;
            let property_type_str: String = row.get("property_type");
            let property_type: PropertyType = serde_json::from_str(&format!("\"{}\"", property_type_str))?;
            
            Ok(Some(Property {
                id: row.get("id"),
                title: row.get("title"),
                description: row.get("description"),
                property_type,
                price: row.get("price"),
                location,
                area_sqm: row.get("area_sqm"),
                rooms: row.get("rooms"),
                bathrooms: row.get("bathrooms"),
                features: row.get("features"),
                images: row.get("images"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_active: row.get("is_active"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_active_properties(&self) -> Result<Vec<Property>> {
        let rows = sqlx::query(
            "SELECT id, title, description, property_type, price, location, area_sqm, rooms, bathrooms, features, images, created_at, updated_at, is_active FROM properties WHERE is_active = true"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut properties = Vec::new();
        for row in rows {
            let location: Location = serde_json::from_value(row.get::<serde_json::Value, _>("location"))?;
            let property_type_str: String = row.get("property_type");
            let property_type: PropertyType = serde_json::from_str(&format!("\"{}\"", property_type_str))?;
            
            properties.push(Property {
                id: row.get("id"),
                title: row.get("title"),
                description: row.get("description"),
                property_type,
                price: row.get("price"),
                location,
                area_sqm: row.get("area_sqm"),
                rooms: row.get("rooms"),
                bathrooms: row.get("bathrooms"),
                features: row.get("features"),
                images: row.get("images"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_active: row.get("is_active"),
            });
        }

        Ok(properties)
    }

    pub async fn get_properties_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Property>> {
        let rows = sqlx::query(
            "SELECT id, title, description, property_type, price, location, area_sqm, rooms, bathrooms, features, images, created_at, updated_at, is_active FROM properties WHERE id = ANY($1) AND is_active = true"
        )
        .bind(ids)
        .fetch_all(&self.pool)
        .await?;

        let mut properties = Vec::new();
        for row in rows {
            let location: Location = serde_json::from_value(row.get::<serde_json::Value, _>("location"))?;
            let property_type_str: String = row.get("property_type");
            let property_type: PropertyType = serde_json::from_str(&format!("\"{}\"", property_type_str))?;
            
            properties.push(Property {
                id: row.get("id"),
                title: row.get("title"),
                description: row.get("description"),
                property_type,
                price: row.get("price"),
                location,
                area_sqm: row.get("area_sqm"),
                rooms: row.get("rooms"),
                bathrooms: row.get("bathrooms"),
                features: row.get("features"),
                images: row.get("images"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_active: row.get("is_active"),
            });
        }

        Ok(properties)
    }

    // Contact operations
    pub async fn get_contact_by_id(&self, id: Uuid) -> Result<Option<Contact>> {
        let row = sqlx::query(
            "SELECT id, first_name, last_name, email, phone, budget_min, budget_max, preferred_locations, preferred_property_types, min_rooms, max_rooms, min_area, max_area, required_features, preferred_features, notes, created_at, updated_at, is_active FROM contacts WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let preferred_locations: Vec<Location> = serde_json::from_value(row.get::<serde_json::Value, _>("preferred_locations"))?;
            let preferred_property_types: Vec<PropertyType> = serde_json::from_value(row.get::<serde_json::Value, _>("preferred_property_types"))?;
            
            Ok(Some(Contact {
                id: row.get("id"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                email: row.get("email"),
                phone: row.get("phone"),
                budget_min: row.get("budget_min"),
                budget_max: row.get("budget_max"),
                preferred_locations,
                preferred_property_types,
                min_rooms: row.get("min_rooms"),
                max_rooms: row.get("max_rooms"),
                min_area: row.get("min_area"),
                max_area: row.get("max_area"),
                required_features: row.get("required_features"),
                preferred_features: row.get("preferred_features"),
                notes: row.get("notes"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_active: row.get("is_active"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_active_contacts(&self) -> Result<Vec<Contact>> {
        let rows = sqlx::query(
            "SELECT id, first_name, last_name, email, phone, budget_min, budget_max, preferred_locations, preferred_property_types, min_rooms, max_rooms, min_area, max_area, required_features, preferred_features, notes, created_at, updated_at, is_active FROM contacts WHERE is_active = true"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut contacts = Vec::new();
        for row in rows {
            let preferred_locations: Vec<Location> = serde_json::from_value(row.get::<serde_json::Value, _>("preferred_locations"))?;
            let preferred_property_types: Vec<PropertyType> = serde_json::from_value(row.get::<serde_json::Value, _>("preferred_property_types"))?;
            
            contacts.push(Contact {
                id: row.get("id"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                email: row.get("email"),
                phone: row.get("phone"),
                budget_min: row.get("budget_min"),
                budget_max: row.get("budget_max"),
                preferred_locations,
                preferred_property_types,
                min_rooms: row.get("min_rooms"),
                max_rooms: row.get("max_rooms"),
                min_area: row.get("min_area"),
                max_area: row.get("max_area"),
                required_features: row.get("required_features"),
                preferred_features: row.get("preferred_features"),
                notes: row.get("notes"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_active: row.get("is_active"),
            });
        }

        Ok(contacts)
    }

    pub async fn create_contact(&self, contact: &Contact) -> Result<Contact> {
        let preferred_locations_json = serde_json::to_value(&contact.preferred_locations)?;
        let preferred_property_types_json = serde_json::to_value(&contact.preferred_property_types)?;

        sqlx::query(
            r#"
            INSERT INTO contacts (id, first_name, last_name, email, phone, budget_min, budget_max, preferred_locations, preferred_property_types, min_rooms, max_rooms, min_area, max_area, required_features, preferred_features, notes, created_at, updated_at, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            "#
        )
        .bind(contact.id)
        .bind(&contact.first_name)
        .bind(&contact.last_name)
        .bind(&contact.email)
        .bind(&contact.phone)
        .bind(contact.budget_min)
        .bind(contact.budget_max)
        .bind(preferred_locations_json)
        .bind(preferred_property_types_json)
        .bind(contact.min_rooms)
        .bind(contact.max_rooms)
        .bind(contact.min_area)
        .bind(contact.max_area)
        .bind(&contact.required_features)
        .bind(&contact.preferred_features)
        .bind(&contact.notes)
        .bind(contact.created_at)
        .bind(contact.updated_at)
        .bind(contact.is_active)
        .execute(&self.pool)
        .await?;

        Ok(contact.clone())
    }

    pub async fn create_property(&self, property: &Property) -> Result<Property> {
        let location_json = serde_json::to_value(&property.location)?;
        let property_type_str = serde_json::to_string(&property.property_type)?;

        sqlx::query(
            r#"
            INSERT INTO properties (id, title, description, property_type, price, location, area_sqm, rooms, bathrooms, features, images, created_at, updated_at, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#
        )
        .bind(property.id)
        .bind(&property.title)
        .bind(&property.description)
        .bind(property_type_str)
        .bind(property.price)
        .bind(location_json)
        .bind(property.area_sqm)
        .bind(property.rooms)
        .bind(property.bathrooms)
        .bind(&property.features)
        .bind(&property.images)
        .bind(property.created_at)
        .bind(property.updated_at)
        .bind(property.is_active)
        .execute(&self.pool)
        .await?;

        Ok(property.clone())
    }
}
