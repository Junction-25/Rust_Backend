# Development Guide

## Quick Start for Developers

### 1. Prerequisites Setup
- Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Install PostgreSQL 12+
- Install Docker (optional, for containerized development)

### 2. Initial Setup
```bash
# Clone and setup
git clone <repository-url>
cd real-estate-recommender

# Quick setup
./setup.sh

# Or manual setup:
cp .env.example .env
# Edit .env with your database credentials
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx migrate run
```

### 3. Development Workflow

```bash
# Run tests and checks
./test.sh

# Start development server
cargo run

# In another terminal, test the API
./examples.sh
```

## Project Structure Explained

### Core Components

1. **Models** (`src/models/`)
   - Define data structures for Property, Contact, and Recommendation
   - Handle serialization/deserialization
   - Define request/response types

2. **Services** (`src/services/`)
   - **RecommendationService**: Core recommendation engine with caching
   - **ComparisonService**: Property comparison logic
   - **QuoteService**: PDF generation for quotes and reports

3. **API Layer** (`src/api/`)
   - REST endpoint handlers
   - Request validation
   - Response formatting

4. **Database** (`src/db/`)
   - Repository pattern for data access
   - SQLx async queries
   - Connection pool management

5. **Utilities** (`src/utils/`)
   - Scoring algorithms
   - PDF generation
   - Helper functions

### Algorithm Deep Dive

#### Recommendation Scoring

The recommendation engine uses a weighted scoring system:

```rust
// Weights (configurable)
const BUDGET_WEIGHT: f64 = 0.3;      // 30%
const LOCATION_WEIGHT: f64 = 0.25;   // 25% 
const PROPERTY_TYPE_WEIGHT: f64 = 0.2; // 20%
const SIZE_WEIGHT: f64 = 0.15;       // 15%
const FEATURE_WEIGHT: f64 = 0.1;     // 10%
```

#### Budget Scoring Logic
- **Perfect Range (60-90% of budget)**: Score = 1.0
- **Under-budget**: Slight penalty (might be suspiciously cheap)
- **Over-budget**: Heavy penalty (decreases rapidly)

#### Location Scoring
- **Distance-based**: Closer properties score higher
- **City match bonus**: Extra points for preferred cities
- **Multiple locations**: Takes the best match

#### Performance Optimizations

1. **Parallel Processing**: Uses Rayon for CPU-intensive calculations
2. **Caching**: Moka cache for frequent recommendation requests
3. **Database Indexes**: Optimized for common query patterns
4. **Connection Pooling**: Efficient database connection management

## Development Patterns

### Adding New Endpoints

1. **Define models** in `src/models/`
2. **Add service logic** in `src/services/`
3. **Create API handler** in `src/api/`
4. **Register route** in the service configuration

Example:
```rust
// In src/api/new_feature.rs
pub async fn new_handler(
    request: web::Json<NewRequest>,
    service: web::Data<SomeService>,
) -> Result<HttpResponse> {
    match service.process(request.into_inner()).await {
        Ok(result) => Ok(HttpResponse::Ok().json(result)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Processing failed".to_string(),
            message: e.to_string(),
        })),
    }
}

// Register in configure_routes
.route("/new-endpoint", web::post().to(new_handler))
```

### Database Changes

1. **Create migration**: `sqlx migrate add migration_name`
2. **Write SQL**: Add tables/columns/indexes in the migration file
3. **Update models**: Modify Rust structs to match schema
4. **Update repository**: Add/modify database queries
5. **Run migration**: `sqlx migrate run`

### Testing Strategy

1. **Unit Tests**: Test individual functions and algorithms
2. **Integration Tests**: Test API endpoints end-to-end
3. **Database Tests**: Test repository operations
4. **Performance Tests**: Benchmark recommendation algorithms

Example test:
```rust
#[tokio::test]
async fn test_recommendation_scoring() {
    let property = create_test_property();
    let contact = create_test_contact();
    
    let service = RecommendationService::new(/* ... */);
    let recommendation = service.calculate_recommendation(&property, &contact);
    
    assert!(recommendation.score > 0.5);
    assert!(recommendation.explanation.budget_match.is_within_budget);
}
```

## Performance Tuning

### Database Optimization
```sql
-- Essential indexes
CREATE INDEX idx_properties_price_range ON properties(price) WHERE is_active = true;
CREATE INDEX idx_contacts_budget_range ON contacts(budget_min, budget_max) WHERE is_active = true;
CREATE INDEX idx_location_gin ON properties USING GIN(location);
```

### Cache Configuration
```rust
// Adjust based on memory and data patterns
let cache = Cache::builder()
    .time_to_live(Duration::from_secs(3600))  // 1 hour
    .max_capacity(10_000)                     // 10k items
    .build();
```

### Parallel Processing
```rust
// Use rayon for CPU-intensive operations
let recommendations: Vec<_> = contacts
    .par_iter()
    .map(|contact| calculate_recommendation(property, contact))
    .filter(|rec| rec.score >= min_score)
    .collect();
```

## Monitoring and Observability

### Health Checks
- `/health` endpoint provides service status
- Database connectivity check
- Version information

### Logging
```rust
// Use structured logging
log::info!("Processing recommendations for property {}", property_id);
log::debug!("Cache hit rate: {:.2}%", cache_hit_rate);
log::error!("Database error: {}", error);
```

### Metrics (Future Enhancement)
- Recommendation generation time
- Cache hit/miss rates
- Database query performance
- API endpoint response times

## Common Development Tasks

### Adding a New Property Feature
1. Update `Property` model
2. Update database migration
3. Modify scoring algorithm in `utils/scoring.rs`
4. Update feature matching logic
5. Add tests

### Modifying Recommendation Algorithm
1. Update scoring weights in `utils/scoring.rs`
2. Modify individual scoring functions
3. Update explanation generation
4. Test with various scenarios
5. Update documentation

### Adding New PDF Templates
1. Create template function in `utils/pdf.rs`
2. Add new service method in `QuoteService`
3. Create API endpoint
4. Test PDF generation

## Debugging Tips

### Database Issues
```bash
# Check database connectivity
psql -U username -d real_estate_db -c "SELECT 1;"

# View migration status
sqlx migrate info

# Reset database (caution: destroys data)
sqlx database drop && sqlx database create && sqlx migrate run
```

### Performance Issues
```bash
# Profile the application
cargo build --release
perf record --call-graph=dwarf ./target/release/real-estate-recommender
perf report

# Memory usage
valgrind --tool=massif ./target/release/real-estate-recommender
```

### API Testing
```bash
# Test with sample data
curl -X GET "localhost:8080/health" | jq '.'

# Load testing
wrk -t12 -c400 -d30s http://localhost:8080/health
```

## Contributing Guidelines

1. **Code Style**: Run `cargo fmt` before committing
2. **Linting**: Ensure `cargo clippy` passes
3. **Tests**: Add tests for new functionality
4. **Documentation**: Update README and code comments
5. **Performance**: Profile performance-critical changes

## Deployment Checklist

- [ ] All tests pass
- [ ] Database migrations run successfully
- [ ] Configuration validated
- [ ] Performance benchmarks meet requirements
- [ ] Security review completed
- [ ] Documentation updated
- [ ] Monitoring configured
