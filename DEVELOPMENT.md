# Development Guide

## 🚀 Quick Start for Developers

### Prerequisites Setup
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install PostgreSQL (Ubuntu/Debian)
sudo apt update && sudo apt install postgresql postgresql-contrib

# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

### One-Command Setup
```bash
git clone <repository-url>
cd real-estate-recommender
./setup.sh
```

### Manual Development Setup
```bash
# 1. Clone and enter directory
git clone <repository-url>
cd real-estate-recommender

# 2. Setup environment variables
cp .env.example .env
# Edit .env if needed (defaults work for local development)

# 3. Setup database
sudo -u postgres createuser -s $USER
createdb real_estate_db
sqlx migrate run

# 4. Install dependencies and build
cargo build

# 5. Run tests
cargo test

# 6. Start development server
cargo run
```

### Development Workflow

```bash
# Quick test everything
./test.sh

# Run development server with hot reloading
cargo watch -x run

# Test API endpoints
./examples.sh

# Run specific tests
cargo test test_name

# Check code formatting
cargo fmt --check

# Run clippy for linting
cargo clippy

# Build optimized release
cargo build --release
```

## 🏗️ Project Structure Explained

### Core Components

#### 1. **Models** (`src/models/`)
```
models/
├── mod.rs              # Module exports
├── property.rs         # Property, Location, PropertyType
├── contact.rs          # Contact, ContactPreferences  
└── recommendation.rs   # Recommendation, scoring structs
```

**Key Features:**
- Serde serialization for JSON API responses
- Integer primary keys for optimal performance
- JSONB fields for flexible preference storage
- Simplified schema for better maintainability

#### 2. **Services** (`src/services/`)
```
services/
├── mod.rs              # Service exports
├── recommendation.rs   # Core recommendation engine
├── comparison.rs       # Property comparison logic
└── quote.rs           # PDF generation service
```

**RecommendationService Features:**
- In-memory caching with Moka (configurable TTL and capacity)
- Parallel processing with Rayon for CPU-intensive calculations
- Sophisticated scoring algorithm with multiple weighted factors
- Bulk processing support for multiple properties

**ComparisonService Features:**
- Side-by-side property analysis
- Distance calculations using Haversine formula
- Feature similarity scoring
- Price and area difference analysis

**QuoteService Features:**
- Professional PDF generation with PrintPDF
- Customizable templates for different report types
- Support for additional costs and custom messages
- Automatic formatting and styling

#### 3. **Database Layer** (`src/db/`)
```
db/
├── mod.rs              # Database module exports
└── repository.rs       # SQLx-based database operations
```

**Repository Pattern Features:**
- Async database operations with SQLx
- Type-safe SQL queries
- Connection pooling for performance
- Proper error handling and transaction support
- JSON field parsing with explicit type conversion

#### 4. **API Layer** (`src/api/`)
```
api/
├── mod.rs              # API module exports
├── recommendations.rs  # Recommendation endpoints
├── comparisons.rs      # Comparison endpoints
└── quotes.rs          # Quote generation endpoints
```

**API Features:**
- RESTful design with proper HTTP status codes
- JSON request/response handling
- Query parameter validation
- Error response standardization
- CORS support for web applications

#### 5. **Utilities** (`src/utils/`)
```
utils/
├── mod.rs              # Utility exports
├── scoring.rs          # Recommendation algorithms
└── pdf.rs             # PDF generation helpers
```

**Scoring Algorithm Components:**
- Budget compatibility scoring with intelligent over/under budget handling
- Location scoring with distance calculations and preference matching
- Feature matching with required vs. preferred feature distinction
- Size requirement scoring with flexible bounds
- Composite scoring with configurable weights

## 🧠 Algorithm Deep Dive

### Recommendation Scoring Formula

The overall recommendation score is calculated as a weighted average:

```
Overall Score = (Budget Score × 0.30) + 
                (Location Score × 0.25) + 
                (Property Type Score × 0.20) + 
                (Size Score × 0.15) + 
                (Feature Score × 0.10)
```

### 1. Budget Scoring Algorithm
```rust
fn calculate_budget_score(property_price: i64, budget_min: i64, budget_max: i64) -> f64 {
    if property_price < budget_min {
        // Under budget - potential suspicion discount
        let diff_ratio = (budget_min - property_price) as f64 / budget_min as f64;
        (1.0 - diff_ratio * 0.5).max(0.1)
    } else if property_price <= budget_max {
        // Within budget - optimal utilization scoring
        let budget_utilization = (property_price - budget_min) as f64 / (budget_max - budget_min) as f64;
        if budget_utilization >= 0.6 && budget_utilization <= 0.9 {
            1.0  // Sweet spot
        } else if budget_utilization < 0.6 {
            0.8 + budget_utilization * 0.2
        } else {
            1.0 - (budget_utilization - 0.9) * 2.0
        }
    } else {
        // Over budget - heavily penalized
        let over_budget_ratio = (property_price - budget_max) as f64 / budget_max as f64;
        (1.0 - over_budget_ratio * 2.0).max(0.0)
    }
}
```

### 2. Location Scoring with Distance
```rust
fn calculate_distance_km(loc1: &Location, loc2: &Location) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;
    
    // Haversine formula implementation
    let lat1_rad = loc1.latitude.to_radians();
    let lat2_rad = loc2.latitude.to_radians();
    let delta_lat = (loc2.latitude - loc1.latitude).to_radians();
    let delta_lng = (loc2.longitude - loc1.longitude).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2) + 
            lat1_rad.cos() * lat2_rad.cos() * (delta_lng / 2.0).sin().powi(2);
    
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    
    EARTH_RADIUS_KM * c
}
```

## 🗄️ Database Design

### Schema Overview
```sql
-- Properties table with JSONB location
CREATE TABLE properties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR NOT NULL,
    description TEXT,
    property_type VARCHAR NOT NULL,  -- 'apartment', 'house', etc.
    price BIGINT NOT NULL,          -- Price in cents
    location JSONB NOT NULL,        -- Flexible location data
    area_sqm INTEGER NOT NULL,
    rooms INTEGER NOT NULL,
    bathrooms INTEGER NOT NULL,
    features TEXT[] DEFAULT '{}',
    images TEXT[] DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE
);

-- Contacts table with JSONB preferences
CREATE TABLE contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    phone VARCHAR,
    budget_min BIGINT NOT NULL,
    budget_max BIGINT NOT NULL,
    preferred_locations JSONB DEFAULT '[]'::jsonb,
    preferred_property_types JSONB DEFAULT '[]'::jsonb,
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
```

### Performance Optimizations
```sql
-- Critical indexes for performance
CREATE INDEX idx_properties_price ON properties(price) WHERE is_active = true;
CREATE INDEX idx_properties_type ON properties(property_type) WHERE is_active = true;
CREATE INDEX idx_properties_location_gin ON properties USING gin(location);

CREATE INDEX idx_contacts_budget ON contacts(budget_min, budget_max) WHERE is_active = true;
CREATE INDEX idx_contacts_email ON contacts(email);
CREATE INDEX idx_contacts_preferred_locations_gin ON contacts USING gin(preferred_locations);
```

## 🧪 Testing Strategy

### Test Structure
```
tests/
├── unit/               # Unit tests for individual functions
│   ├── scoring_tests.rs
│   ├── model_tests.rs
│   └── service_tests.rs
├── integration/        # API endpoint testing
│   ├── recommendation_api_tests.rs
│   ├── comparison_api_tests.rs
│   └── quote_api_tests.rs
└── performance/        # Load and performance tests
    ├── benchmark_tests.rs
    └── load_tests.rs
```

### Running Tests
```bash
# All tests
cargo test

# Specific test module
cargo test scoring

# Integration tests only
cargo test --test integration

# With output
cargo test -- --nocapture

# Performance tests
cargo test --release performance
```

### Test Database Setup
```bash
# Create test database
createdb real_estate_test_db

# Run migrations for test
DATABASE_URL=postgresql:///real_estate_test_db sqlx migrate run

# Run tests with test database
DATABASE_URL=postgresql:///real_estate_test_db cargo test
```

## 🔧 Configuration Management

### Environment Variables
```bash
# Database Configuration
DATABASE_URL=postgresql:///real_estate_db  # Uses Unix socket for local dev
DATABASE_MAX_CONNECTIONS=32
DATABASE_MIN_CONNECTIONS=5

# Server Configuration  
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Cache Configuration
CACHE_TTL_SECONDS=3600
CACHE_MAX_CAPACITY=10000

# Recommendation Engine
RECOMMENDATION_THRESHOLD=0.3
MAX_RECOMMENDATIONS=10

# Logging
RUST_LOG=info  # debug, info, warn, error
```

### Configuration Loading
```rust
// config.rs structure
#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub cache: CacheConfig,
    pub recommendation: RecommendationConfig,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Load from environment with sensible defaults
    }
}
```

## 🚀 Performance Optimization

### Caching Strategy
```rust
// Moka cache configuration
let cache = Cache::builder()
    .time_to_live(Duration::from_secs(config.cache.ttl_seconds))
    .max_capacity(config.cache.max_capacity)
    .build();

// Cache key strategy
let cache_key = format!("property_{}_{:?}_{:?}", property_id, limit, min_score);
```

### Parallel Processing
```rust
// Rayon for CPU-intensive calculations
let recommendations: Vec<Recommendation> = contacts
    .par_iter()  // Parallel iterator
    .filter_map(|contact| {
        let recommendation = self.calculate_recommendation(&property, contact);
        if recommendation.score >= min_score.unwrap_or(0.0) {
            Some(recommendation)
        } else {
            None
        }
    })
    .collect();
```

### Database Optimization
- Connection pooling with SQLx
- Prepared statements for common queries
- JSONB indexes for location and preference queries
- Selective field retrieval to minimize data transfer

## 🔍 Debugging and Monitoring

### Logging Configuration
```bash
# Debug level logging
RUST_LOG=debug cargo run

# Module-specific logging
RUST_LOG=real_estate_recommender::services::recommendation=debug cargo run
```

### Common Debugging Commands
```bash
# Check database connections
psql real_estate_db -c "SELECT COUNT(*) FROM properties;"

# Verify API health
curl http://localhost:8080/health

# Test specific endpoint
curl -v "http://localhost:8080/recommendations/contact/1?limit=1"

# Check server logs
tail -f logs/server.log
```

### Performance Monitoring
```rust
// Built-in timing for recommendations
let start_time = std::time::Instant::now();
// ... processing ...
let processing_time_ms = start_time.elapsed().as_millis() as u64;
```

## 📋 Development Checklist

### Before Committing
- [ ] Run `cargo fmt` for formatting
- [ ] Run `cargo clippy` for linting
- [ ] Run `cargo test` for all tests
- [ ] Run `./test.sh` for comprehensive testing
- [ ] Update documentation if adding features
- [ ] Test API endpoints manually with `./examples.sh`

### Performance Checklist
- [ ] Check query performance with `EXPLAIN ANALYZE`
- [ ] Monitor cache hit rates
- [ ] Test with realistic data volumes
- [ ] Profile CPU usage for scoring algorithms
- [ ] Verify memory usage under load

### Production Readiness
- [ ] Security review of SQL queries
- [ ] Error handling for all failure modes
- [ ] Comprehensive logging for debugging
- [ ] Configuration validation
- [ ] Performance benchmarks meet requirements
- [ ] Documentation up to date

## 🤝 Contributing Guidelines

### Code Style
- Follow Rust standard formatting (`cargo fmt`)
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Include error handling for all fallible operations

### Git Workflow
```bash
# Feature development
git checkout -b feature/recommendation-improvements
# ... make changes ...
cargo test && ./test.sh
git commit -m "feat: improve recommendation scoring algorithm"
git push origin feature/recommendation-improvements
# Create Pull Request
```

### Pull Request Template
- [ ] Tests pass locally
- [ ] Documentation updated
- [ ] Performance impact considered
- [ ] Breaking changes documented
- [ ] Examples updated if needed

## 📚 Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Rust language fundamentals
- [Actix-web Documentation](https://actix.rs/) - Web framework documentation
- [SQLx Documentation](https://docs.rs/sqlx/) - Database toolkit
- [PostgreSQL Documentation](https://www.postgresql.org/docs/) - Database reference
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
