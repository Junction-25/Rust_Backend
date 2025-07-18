# Real Estate Recommendation System

A high-performance Rust-based real estate recommendation system that matches properties with potential contacts using advanced scoring algorithms. The system provides REST APIs for property recommendations, comparisons, and automated quote generation.

## Features

### Core Functionality
- **Smart Recommendations**: Advanced scoring algorithm that matches contacts to properties based on budget, location preferences, property type, size requirements, and feature preferences
- **Property Comparisons**: Detailed side-by-side property analysis with similarity metrics
- **PDF Quote Generation**: Automated generation of professional quotes and comparison reports
- **Bulk Processing**: Efficient bulk recommendation processing for multiple properties
- **High Performance**: Built with Rust and Actix-web for maximum performance and concurrency

### API Endpoints

#### Recommendations
- `GET /recommendations/property/{property_id}` - Get recommended contacts for a specific property
- `POST /recommendations/bulk` - Generate recommendations for multiple properties

#### Comparisons
- `GET /comparisons/properties?property1_id={id1}&property2_id={id2}` - Compare two properties

#### Quotes & Reports
- `POST /quotes/generate` - Generate a PDF quote for a property and contact
- `POST /quotes/comparison` - Generate a PDF comparison report
- `GET /quotes/recommendations?property_id={id}` - Generate a PDF recommendation report

#### Health Check
- `GET /health` - Service health status

## Technology Stack

- **Backend**: Rust with Actix-web framework
- **Database**: PostgreSQL with SQLx for async database operations
- **Caching**: In-memory caching with Moka for high-performance recommendations
- **PDF Generation**: PrintPDF for professional document generation
- **Parallel Processing**: Rayon for CPU-intensive recommendation calculations

## Architecture

```
src/
├── main.rs                 # Application entry point and server setup
├── config.rs               # Configuration management
├── models/                 # Data models and structures
│   ├── property.rs         # Property-related models
│   ├── contact.rs          # Contact-related models
│   └── recommendation.rs   # Recommendation models and responses
├── services/               # Business logic layer
│   ├── recommendation.rs   # Core recommendation engine
│   ├── comparison.rs       # Property comparison logic
│   └── quote.rs           # Quote and PDF generation
├── api/                    # REST API endpoints
│   ├── recommendations.rs  # Recommendation endpoints
│   ├── comparisons.rs      # Comparison endpoints
│   └── quotes.rs          # Quote generation endpoints
├── db/                     # Database layer
│   └── repository.rs      # Database operations and queries
└── utils/                  # Utility functions
    ├── scoring.rs         # Recommendation scoring algorithms
    └── pdf.rs            # PDF generation utilities
```

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- PostgreSQL 12+ running
- Git

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd real-estate-recommender
   ```

2. **Set up PostgreSQL database**
   ```bash
   # Create database
   createdb real_estate_db
   
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
