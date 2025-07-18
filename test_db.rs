use real_estate_recommender::db::Repository;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = "postgresql://lyes:password123@localhost/real_estate_db";
    let pool = PgPool::connect(database_url).await?;
    let repo = Repository::new(pool);
    
    println!("Testing property fetch...");
    let property_id = uuid::Uuid::parse_str("5271018a-8b81-4fed-b17f-8a7e829f35ab")?;
    match repo.get_property_by_id(property_id).await {
        Ok(Some(property)) => println!("Property fetched successfully: {}", property.title),
        Ok(None) => println!("Property not found"),
        Err(e) => println!("Error fetching property: {}", e),
    }
    
    println!("Testing contacts fetch...");
    match repo.get_all_active_contacts().await {
        Ok(contacts) => println!("Fetched {} contacts successfully", contacts.len()),
        Err(e) => println!("Error fetching contacts: {}", e),
    }
    
    Ok(())
}
