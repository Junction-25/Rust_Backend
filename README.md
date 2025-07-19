# Real Estate Recommendation System

A high-performance Rust-based real estate recommendation system that matches properties with contacts using advanced scoring algorithms. The system provides REST APIs for property recommendations, comparisons, and automated quote generation with PDF reports.

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- PostgreSQL 14+ 
- Git

### One-Command Setup
```bash
git clone <repository-url>
cd real-estate-recommender
docker-compose up -d
python migrate_data.py
```

### Manual Setup
```bash
# 1. Setup environment
cp .env.example .env
# Edit .env if needed (default settings work for local development)

# 2. Install dependencies and setup database
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx migrate run

# 3. Run the server
cargo run --release
```

### Testing the System
```bash
# Run comprehensive tests
./test.sh

# Test API endpoints with sample data
./examples.sh

# Start the server (production mode)
./start.sh
```

## ✨ Features

### Core Functionality
- **🎯 Smart Recommendations**: Advanced scoring algorithm that matches properties to contacts based on:
  - Budget compatibility (with intelligent scoring for over/under budget scenarios)
  - Location preferences with distance calculations
  - Property type matching (apartment, house, office, etc.)
  - Size requirements (rooms, area)
  - Preference matching for optimal contact-property pairing
- **📊 Property Comparisons**: Detailed side-by-side property analysis with similarity metrics
- **📄 PDF Generation**: Professional quotes, comparison reports, and recommendation summaries
- **⚡ High Performance**: Built with Rust and Actix-web for maximum performance
- **🔄 Caching**: In-memory caching with Moka for lightning-fast repeated queries
- **🔍 Parallel Processing**: CPU-intensive calculations optimized with Rayon

### API Endpoints

#### Health Check
- `GET /health` - Service health status and version

#### Recommendations
- `GET /recommendations/contact/{contact_id}?limit={n}&min_score={score}` - Get recommended properties for a contact
- `POST /recommendations/bulk` - Generate recommendations for multiple contacts

#### Comparisons  
- `GET /comparisons/properties?property1_id={id1}&property2_id={id2}` - Compare two properties

#### PDF Reports
- `POST /quotes/generate` - Generate a PDF quote for a property and contact
- `POST /quotes/comparison` - Generate a PDF comparison report

## 🏗️ Technology Stack

- **Backend**: Rust with Actix-web framework
- **Database**: PostgreSQL with SQLx for async database operations  
- **Caching**: Moka for high-performance in-memory caching
- **PDF Generation**: PrintPDF for professional document generation
- **Parallel Processing**: Rayon for CPU-intensive recommendation calculations
- **Serialization**: Serde for JSON handling
- **Logging**: env_logger for structured logging

## 📁 Architecture

```
src/
├── main.rs                 # Application entry point and server setup
├── config.rs               # Configuration management from environment
├── models/                 # Data models and structures
│   ├── mod.rs              # Module declarations
│   ├── property.rs         # Property, Location, PropertyType models
│   ├── contact.rs          # Contact and ContactPreferences models
│   └── recommendation.rs   # Recommendation and scoring models
├── services/               # Business logic layer
│   ├── mod.rs              # Service module declarations
│   ├── recommendation.rs   # Core recommendation engine with caching
│   ├── comparison.rs       # Property comparison logic
│   └── quote.rs            # PDF generation service
├── db/                     # Database layer
│   ├── mod.rs              # Database module declaration
│   └── repository.rs       # Database access layer with SQLx
├── api/                    # HTTP API layer
│   ├── mod.rs              # API module declarations
│   ├── recommendations.rs  # Recommendation endpoints
│   ├── comparisons.rs      # Comparison endpoints
│   └── quotes.rs           # Quote generation endpoints
└── utils/                  # Utility functions
    ├── mod.rs              # Utility module declarations
    ├── scoring.rs          # Scoring algorithm implementations
    └── pdf.rs              # PDF generation utilities
```

## 🗄️ Database Schema

### Tables

#### Properties
```sql
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
```

#### Contacts  
```sql
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
```

### Key Features
- **Simplified Structure**: Integer IDs and direct field storage for optimal performance
- **JSONB columns** for flexible location preferences and property types
- **Comprehensive Indexes**: Optimized for price, location, area, and type queries
- **Sample Data**: Included 5 contacts and 10 properties for immediate testing

## 🧠 Recommendation Algorithm

The system uses a sophisticated scoring algorithm that matches properties to contacts based on:

### 1. Budget Compatibility (Weight: 35%)
- **Within Budget**: Perfect match with optimal utilization scoring
- **Under Budget**: Scored based on budget utilization (70-90% is optimal)
- **Over Budget**: Penalized based on excess amount

### 2. Location Preference (Weight: 30%)
- Distance calculation using Haversine formula
- Preferred locations get bonus scoring
- Proximity-based scoring with distance decay

### 3. Property Type Match (Weight: 20%)
- Exact match for preferred property types
- Boolean scoring (match/no match)

### 4. Size Requirements (Weight: 15%)
- Room count matching with tolerance
- Area requirements with flexible bounds
- Composite scoring for multiple criteria

### Score Calculation
- Each factor contributes to an overall score (0.0 to 1.0)
- Detailed explanations provided for each recommendation
- Configurable minimum score thresholds

## 🚀 Performance

- **Parallel Processing**: Recommendations calculated using Rayon for multi-core utilization
- **Caching**: Moka cache reduces database queries for repeated requests
- **Database Optimization**: Indexed queries and optimized joins
- **Async Operations**: Non-blocking I/O throughout the application

## 📊 API Examples

### Get Contact Recommendations
```bash
curl "http://localhost:8080/recommendations/contact/1?limit=5&min_score=0.3" | jq '.'
```

### Bulk Recommendations for Multiple Contacts
```bash
curl -X POST "http://localhost:8080/recommendations/bulk" \
  -H "Content-Type: application/json" \
  -d '{
    "limit_per_contact": 3,
    "min_score": 0.3,
    "contact_ids": [1, 2, 3]
  }' | jq '.'
```

### Compare Properties
```bash
curl "http://localhost:8080/comparisons/properties?property1_id=1&property2_id=2" | jq '.'
```

### Generate PDF Quote
```bash
curl -X POST "http://localhost:8080/quotes/generate" \
  -H "Content-Type: application/json" \
  -d '{
    "property_id": 1,
    "contact_id": 1,
    "additional_costs": [
      {"description": "Legal Fees", "amount": 150000}
    ],
    "custom_message": "Thank you for your interest!"
  }' \
  --output quote.pdf
```

## 🧪 Testing

### Automated Testing
```bash
./test.sh  # Runs all tests including linting, building, and functionality tests
```

### Manual Testing  
```bash
./examples.sh  # Interactive API testing with sample data
```

### Test Coverage
- Unit tests for scoring algorithms
- Integration tests for API endpoints
- Database migration testing
- Performance benchmarks

## 🔧 Configuration

### Environment Variables
```bash
# Database
DATABASE_URL=postgresql:///real_estate_db

# Server
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Cache
CACHE_TTL_SECONDS=3600
CACHE_MAX_CAPACITY=10000

# Recommendations
RECOMMENDATION_THRESHOLD=0.3
MAX_RECOMMENDATIONS=10
```

### Default Values
- Uses PostgreSQL peer authentication for local development
- Includes sample data for immediate testing
- Optimized for development and production environments

## 📁 Project Scripts

- **`./setup.sh`**: Complete environment setup with dependency installation
- **`./test.sh`**: Comprehensive testing suite
- **`./start.sh`**: Production server startup
- **`./examples.sh`**: Interactive API demonstration with sample data

## 🔍 Monitoring & Logging

### Health Check
```bash
curl http://localhost:8080/health
```

Returns:
```json
{
  "status": "healthy",
  "timestamp": "2025-07-18T09:56:03.783879516Z",
  "version": "0.1.0"
}
```

### Logging
- Structured logging with env_logger
- Request/response logging via Actix middleware
- Configurable log levels (DEBUG, INFO, WARN, ERROR)

## 🚀 Deployment

### Development
```bash
cargo run  # Debug mode with hot reloading
```

### Production
```bash
cargo run --release  # Optimized build
```

### Docker (Optional)
```dockerfile
# Dockerfile available for containerized deployment
# Includes multi-stage build for optimized image size
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests (`./test.sh`)
4. Commit changes (`git commit -m 'Add amazing feature'`)
5. Push to branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- Check the [DEVELOPMENT.md](DEVELOPMENT.md) for detailed development guide
- Review [NEXT_STEPS.md](NEXT_STEPS.md) for planned features
- Open an issue for bugs or feature requests

## 🎯 Roadmap

- [ ] Machine learning-based recommendation improvements
- [ ] Real-time notifications for new matches
- [ ] Advanced search and filtering
- [ ] Multi-language support
- [ ] Mobile API optimizations
- [ ] WebSocket support for real-time updates
   
   # Or using psql
   psql -c "CREATE DATABASE real_estate_db;"
   ```

3. **Configure environment variables**
   ```bash
   cp .env.example .env
   # Edit .env with your database credentials
   ```

   Update `.env` file:
   ```
   DATABASE_URL=postgresql://username:password@localhost/real_estate_db
   SERVER_HOST=127.0.0.1
   SERVER_PORT=8080
   RECOMMENDATION_THRESHOLD=0.3
   MAX_RECOMMENDATIONS=10
   CACHE_TTL_SECONDS=3600
   CACHE_MAX_CAPACITY=10000
   ```

4. **Install dependencies and run migrations**
   ```bash
   # Install SQLx CLI for migrations
   cargo install sqlx-cli --no-default-features --features rustls,postgres
   
   # Run database migrations
   sqlx migrate run
   ```

5. **Build and run the application**
   ```bash
   # Development mode
   cargo run
   
   # Production build
   cargo build --release
   ./target/release/real-estate-recommender
   ```

The server will start at `http://localhost:8080`

### Verify Installation

Test the health endpoint:
```bash
curl http://localhost:8080/health
```

Expected response:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "0.1.0"
}
```

## API Usage Examples

### Get Property Recommendations

Get recommended contacts for a specific property:

```bash
curl "http://localhost:8080/recommendations/property/{property_id}?limit=5&min_score=0.3"
```

Response:
```json
{
  "recommendations": [
    {
      "contact": {
        "id": "uuid",
        "first_name": "John",
        "last_name": "Smith",
        "email": "john.smith@email.com",
        "budget_min": 300000000,
        "budget_max": 500000000
      },
      "property": { ... },
      "score": 0.85,
      "explanation": {
        "overall_score": 0.85,
        "budget_match": {
          "is_within_budget": true,
          "budget_utilization": 0.7,
          "score": 0.9
        },
        "location_match": {
          "distance_km": 2.5,
          "is_preferred_location": true,
          "score": 0.95
        },
        "reasons": ["Excellent budget match", "Perfect location match"]
      }
    }
  ],
  "total_count": 5,
  "processing_time_ms": 45
}
```

### Bulk Recommendations

Generate recommendations for multiple properties:

```bash
curl -X POST http://localhost:8080/recommendations/bulk \
  -H "Content-Type: application/json" \
  -d '{
    "limit_per_property": 3,
    "min_score": 0.4,
    "property_ids": ["uuid1", "uuid2", "uuid3"]
  }'
```

### Property Comparison

Compare two properties:

```bash
curl "http://localhost:8080/comparisons/properties?property1_id={id1}&property2_id={id2}"
```

### Generate Quote PDF

Generate a professional quote PDF:

```bash
curl -X POST http://localhost:8080/quotes/generate \
  -H "Content-Type: application/json" \
  -d '{
    "property_id": "uuid",
    "contact_id": "uuid",
    "additional_costs": [
      {"description": "Legal Fees", "amount": 150000},
      {"description": "Inspection", "amount": 50000}
    ]
  }' \
  --output quote.pdf
```

## Recommendation Algorithm

The system uses a sophisticated weighted scoring algorithm that evaluates:

### Scoring Factors

1. **Budget Match (30% weight)**
   - Exact budget fit: Higher scores for properties using 60-90% of budget
   - Over-budget penalty: Heavily penalized
   - Under-budget consideration: Slightly penalized (might be suspiciously cheap)

2. **Location Match (25% weight)**
   - Distance-based scoring: Closer properties score higher
   - City match bonus: Extra points for exact city matches
   - Preferred location alignment

3. **Property Type Match (20% weight)**
   - Exact type match: Perfect score
   - No match: Zero score

4. **Size Requirements (15% weight)**
   - Room count matching: Penalties for too few/many rooms
   - Area matching: Size preference alignment

5. **Feature Matching (10% weight)**
   - Required features: Must be met (zero score if missing)
   - Preferred features: Bonus points for matches

### Algorithm Benefits

- **Transparent Scoring**: Each recommendation includes detailed explanation
- **Configurable Thresholds**: Adjustable minimum scores and limits
- **Performance Optimized**: Parallel processing for bulk operations
- **Cached Results**: Intelligent caching for frequently requested recommendations

## Performance Characteristics

- **Concurrent Processing**: Handles multiple recommendation requests simultaneously
- **Parallel Algorithms**: CPU-intensive calculations use all available cores
- **Memory Caching**: Frequently accessed recommendations cached in memory
- **Database Optimization**: Efficient queries with proper indexing
- **Sub-second Response**: Typical recommendation response < 100ms

## Database Schema

### Properties Table
- Property details, location, pricing, features
- JSONB for flexible location and feature storage
- Indexes on price, area, rooms for fast filtering

### Contacts Table
- Contact information and preferences
- Budget ranges, location preferences, feature requirements
- Flexible preference storage with JSONB

### Key Indexes
- Price and budget range indexes for fast matching
- GIN indexes on JSONB location data for spatial queries
- Composite indexes for common query patterns

## Development

### Running Tests
```bash
cargo test
```

### Database Migrations
```bash
# Create new migration
sqlx migrate add migration_name

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `SERVER_HOST` | Server bind address | `127.0.0.1` |
| `SERVER_PORT` | Server port | `8080` |
| `RECOMMENDATION_THRESHOLD` | Minimum recommendation score | `0.3` |
| `MAX_RECOMMENDATIONS` | Default max recommendations per request | `10` |
| `CACHE_TTL_SECONDS` | Cache time-to-live | `3600` |
| `CACHE_MAX_CAPACITY` | Maximum cached items | `10000` |

### Performance Tuning

For high-load environments:

1. **Database Connection Pool**: Adjust `max_connections` in config
2. **Cache Settings**: Increase cache capacity and TTL for stable data
3. **Worker Threads**: Configure Actix-web workers based on CPU cores
4. **Database Indexes**: Add custom indexes for specific query patterns

## Deployment

### Docker Deployment
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/real-estate-recommender /usr/local/bin/
EXPOSE 8080
CMD ["real-estate-recommender"]
```

### Production Considerations

1. **Database**: Use connection pooling and read replicas
2. **Caching**: Consider Redis for distributed caching
3. **Monitoring**: Add metrics and health checks
4. **Security**: Implement authentication and rate limiting
5. **Scaling**: Use load balancers for horizontal scaling

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For issues and questions:
- Create an issue in the repository
- Check existing documentation
- Review API examples above
