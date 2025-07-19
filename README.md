# Real Estate Recommendation API

## 🏠 Overview

Advanced real estate recommendation system built with Rust, providing intelligent property-to-contact matching, comprehensive property comparisons, and detailed quote generation. The system features configurable scoring algorithms, multi-dimensional analysis, and rich JSON APIs.

## 🚀 Key Features

### ✨ **Latest Enhancements (v2.0)**
- **🎛️ Configurable Scoring Weights**: Customize recommendation algorithms via API
- **📄 JSON Quote System**: Comprehensive financial analysis with structured responses
- **🔍 Enhanced Comparisons**: Multi-dimensional property analysis with intelligent recommendations
- **⚡ Advanced Filtering**: Top-K, percentile-based, and threshold filtering
- **🧠 AI-Powered Insights**: Smart reasoning and confidence scoring

### 🛠️ Core Capabilities
- **Property-to-Contact Recommendations**: Find potential buyers for properties
- **Multi-Factor Scoring**: Budget, location, property type, and size matching
- **Real-Time Analysis**: Parallel processing with caching optimization
- **Comprehensive Comparisons**: Side-by-side property analysis with detailed insights
- **Financial Calculations**: Mortgage analysis, affordability scoring, ROI projections
- **RESTful JSON APIs**: Modern, integration-friendly endpoints

## 🚀 Quick Start

### One-Command Setup
```bash
git clone <repository-url>
cd Rust_Backend
./setup.sh  # Sets up database and loads sample data
cargo run --release
```

### Alternative Setup
```bash
# With Docker
docker-compose up -d
python3 migrate_data.py

# Manual setup
./setup-database.sh
cargo run --release
```

**Default URL**: `http://localhost:8080`

## 📚 Table of Contents

- [🏠 Overview](#-overview)
- [🚀 Quick Start](#-quick-start)
- [🎯 API Endpoints](#-api-endpoints)
  - [Recommendations](#recommendations)
  - [Comparisons](#comparisons)
  - [Quotes](#quotes)
- [⚙️ Configuration](#️-configuration)
- [🧪 Testing](#-testing)
- [📊 Performance](#-performance)
- [🔧 Development](#-development)

## 🎯 API Endpoints

### Health Check
```http
GET /health
```
Returns service status and version information.

---

## Recommendations

### 🎯 Get Property Recommendations
Find potential buyers for a specific property with configurable scoring.

**Endpoint**: `GET /recommendations/property/{property_id}`

**Query Parameters**:
- `limit` (optional): Maximum recommendations to return
- `min_score` (optional): Minimum score threshold (0.0-1.0)
- `top_k` (optional): Return only top K highest-scoring contacts
- `top_percentile` (optional): Return top X% of contacts (e.g., 0.1 for 10%)
- `score_threshold_percentile` (optional): Filter by score percentile

**🎛️ Configurable Scoring Weights**:
- `budget_weight` (default: 0.3): Budget matching importance
- `location_weight` (default: 0.25): Location proximity importance  
- `property_type_weight` (default: 0.2): Property type matching importance
- `size_weight` (default: 0.25): Size requirements importance

**Example Requests**:
```http
# Default weights
GET /recommendations/property/123?limit=10&min_score=0.6

# Budget-focused recommendations
GET /recommendations/property/123?budget_weight=0.5&location_weight=0.2&property_type_weight=0.2&size_weight=0.1

# Advanced filtering
GET /recommendations/property/123?top_k=10&top_percentile=0.2&score_threshold_percentile=0.8
```

### 📊 Bulk Recommendations
Get recommendations for multiple properties simultaneously.

**Endpoint**: `POST /recommendations/bulk`

**Request Body**:
```json
{
  "property_ids": [1, 2, 3],
  "limit_per_property": 5,
  "min_score": 0.6,
  "top_k": 10,
  "budget_weight": 0.4,
  "location_weight": 0.3,
  "property_type_weight": 0.2,
  "size_weight": 0.1
}
```

---

## Comparisons

### 🔍 Enhanced Property Comparison
Compare two properties with comprehensive multi-dimensional analysis.

**Endpoint**: `GET /comparisons/properties`

**Query Parameters**:
- `property1_id` (required): First property ID
- `property2_id` (required): Second property ID

**Example Request**:
```http
GET /comparisons/properties?property1_id=1&property2_id=2
```

**📊 Response Includes**:
- **Basic Metrics**: Price/area differences, similarity scores
- **Price Analysis**: Affordability ratings, cost per sqm
- **Space Analysis**: Room comparisons, efficiency ratios
- **Location Analysis**: Distance, accessibility insights
- **Feature Analysis**: Type matching, unique advantages
- **Value Analysis**: Investment potential, ROI projections
- **Smart Recommendation**: AI-powered choice with confidence scoring

---

## Quotes

### 💰 Property Quote Generation
Generate comprehensive financial quotes in structured JSON format.

**Endpoint**: `POST /quotes/generate`

**Request Body**:
```json
{
  "property_id": 1,
  "contact_id": 1,
  "additional_costs": [
    {
      "description": "Legal Fees",
      "amount": 150000
    }
  ]
}
```

**📈 Response Includes**:
- **Property & Contact Details**: Complete information
- **Financial Analysis**: Down payments, monthly costs, closing costs
- **Financing Options**: Multiple loan scenarios (30-year, 15-year, FHA)
- **Affordability Scoring**: Budget compatibility analysis
- **Recommendations**: Next steps and considerations
- **Quote Validity**: Creation and expiration dates

### 🏘️ Property Comparison Quotes
**Endpoint**: `POST /quotes/comparison`

**Request Body**:
```json
{
  "property1_id": 1,
  "property2_id": 2,
  "contact_id": 1
}
```

---

## ⚙️ Configuration

### Scoring Algorithm Customization

#### Default Weights:
- **Budget**: 30% - How well property price fits contact's budget
- **Location**: 25% - Proximity to preferred locations  
- **Property Type**: 20% - Matching preferred property types
- **Size**: 25% - Room count and area requirements

#### Custom Weight Examples:

**Budget-Focused (Conservative Buyers)**:
```json
{
  "budget_weight": 0.6,
  "location_weight": 0.2,
  "property_type_weight": 0.1,
  "size_weight": 0.1
}
```

**Location-Priority (Commute-Focused)**:
```json
{
  "budget_weight": 0.1,
  "location_weight": 0.6,
  "property_type_weight": 0.2,
  "size_weight": 0.1
}
```

**Balanced Approach**:
```json
{
  "budget_weight": 0.25,
  "location_weight": 0.25,
  "property_type_weight": 0.25,
  "size_weight": 0.25
}
```

### Environment Configuration
```bash
# Database
DATABASE_URL=postgresql://user:pass@localhost/realestate

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Cache
CACHE_TTL_SECONDS=300
CACHE_CAPACITY=1000
```

---

## 🧪 Testing

### Comprehensive Test Suite

```bash
# Test configurable weights
python3 test_weights_api.py

# Test JSON quotes  
python3 test_json_quotes.py

# Test enhanced comparisons
python3 test_enhanced_comparisons.py

# Performance testing
python3 analysis/latency_test.py

# All tests
./test.sh
```

### 📊 Performance Testing

**K-Value Scenarios**:
```bash
# Test different recommendation sizes
./run_latency_test.sh  # Tests K=5,10,50,100

# Scalability testing  
./run_scalability_test.sh

# Custom latency analysis
python3 analysis/plot_latency.py
```

---

## 📊 Performance

### 🚀 Optimizations
- **Parallel Processing**: Rayon-based concurrent calculations
- **Smart Caching**: Moka cache with configurable TTL
- **Database Pooling**: Connection pooling for high throughput
- **Advanced Filtering**: Efficient percentile-based filtering

### 📈 Benchmarks
- **Single Recommendation**: ~5-15ms (10K contacts)
- **Bulk Operations**: ~50-200ms (10 properties, 10K contacts)
- **Property Comparison**: ~10-30ms
- **Quote Generation**: ~20-50ms

### 📊 Scalability Results
- **Throughput**: 500+ requests/second
- **Memory Usage**: ~100MB baseline
- **Cache Hit Rate**: 85-95% (typical workloads)

---

## 🔧 Development

### Project Structure
```
src/
├── main.rs              # Application entry point
├── config.rs            # Configuration management
├── api/                 # REST API endpoints
│   ├── recommendations.rs
│   ├── comparisons.rs
│   └── quotes.rs
├── services/            # Business logic
│   ├── recommendation.rs
│   ├── comparison.rs
│   └── quote.rs
├── models/              # Data structures
│   ├── property.rs
│   ├── contact.rs
│   └── recommendation.rs
├── db/                  # Database layer
│   └── repository.rs
└── utils/               # Utilities
    ├── scoring.rs
    └── pdf.rs
```

### 🛠️ Tech Stack
- **Backend**: Rust with Actix-Web
- **Database**: PostgreSQL with SQLx
- **Caching**: Moka (in-memory)
- **Concurrency**: Rayon (parallel processing)
- **Serialization**: Serde JSON
- **Testing**: Python integration tests

### 🚀 Deployment
```bash
# Docker deployment
docker-compose up -d

# Production build
cargo build --release

# Database migrations
sqlx migrate run
```

---

## 📖 Documentation

### 📚 Additional Resources
- [API_DOCUMENTATION.md](API_DOCUMENTATION.md) - Complete API reference
- [CONFIGURABLE_WEIGHTS.md](CONFIGURABLE_WEIGHTS.md) - Scoring customization guide
- [JSON_QUOTES_DOCUMENTATION.md](JSON_QUOTES_DOCUMENTATION.md) - Quote system details  
- [ENHANCED_COMPARISONS_DOCUMENTATION.md](ENHANCED_COMPARISONS_DOCUMENTATION.md) - Comparison features
- [LATENCY_TESTING.md](LATENCY_TESTING.md) - Performance analysis
- [SCALABILITY_TESTING.md](SCALABILITY_TESTING.md) - Load testing guide

### 🎯 Example Responses

**Recommendation Response**:
```json
{
  "recommendations": [
    {
      "contact": { /* Contact details */ },
      "property": { /* Property details */ },
      "score": 0.87,
      "explanation": {
        "budget_match": { "score": 0.95 },
        "location_match": { "score": 0.82 },
        "reasons": ["Excellent budget match", "Good location fit"]
      }
    }
  ],
  "total_count": 15,
  "processing_time_ms": 23
}
```

**Comparison Response**:
```json
{
  "property1": { /* Property details */ },
  "property2": { /* Property details */ },
  "detailed_analysis": {
    "price_analysis": { /* Affordability insights */ },
    "value_analysis": { /* Investment potential */ }
  },
  "recommendation": {
    "recommended_property": 1,
    "confidence_score": 0.85,
    "summary": "Property 1 offers better value for money"
  }
}
```

**Quote Response**:
```json
{
  "property": { /* Property details */ },
  "financial_details": {
    "estimated_monthly_payment": 2850.50,
    "financing_options": [
      {
        "loan_type": "Conventional 30-year",
        "interest_rate": 6.5,
        "monthly_payment": 2850.50
      }
    ]
  },
  "quote_summary": {
    "affordability_score": 0.92,
    "recommendation_level": "Highly Recommended"
  }
}
```

---

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🏆 Acknowledgments

- Built with ❤️ using Rust and modern web technologies
- Performance optimized for high-throughput real estate applications
- Designed for scalability and extensibility
{
  "status": "healthy",
  "timestamp": "2025-07-18T10:30:00.000Z",
  "version": "1.0.0"
}
```

---

## Recommendations

### Get Property Recommendations

Get contact recommendations for a specific property - find potential buyers who might be interested in the property.

**Endpoint**: `GET /recommendations/property/{property_id}`

**Parameters**:
- `property_id` (path, required): Integer - The ID of the property
- `limit` (query, optional): Integer - Maximum number of contact recommendations to return
- `min_score` (query, optional): Float - Minimum recommendation score (0.0-1.0)
- `top_k` (query, optional): Integer - Return only the top K highest-scoring contacts
- `top_percentile` (query, optional): Float - Return only the top X% of contacts (e.g., 0.1 for top 10%)
- `score_threshold_percentile` (query, optional): Float - Only return contacts above the Xth percentile threshold (e.g., 0.8 for top 20%)
- `budget_weight` (query, optional): Float - Weight for budget matching (0.0-1.0), default: 0.3
- `location_weight` (query, optional): Float - Weight for location proximity (0.0-1.0), default: 0.25
- `property_type_weight` (query, optional): Float - Weight for property type matching (0.0-1.0), default: 0.2
- `size_weight` (query, optional): Float - Weight for size requirements (0.0-1.0), default: 0.25

**Weight Validation**: If all four weights are provided, they must sum to exactly 1.0.

**Example Request**:
```http
GET /recommendations/property/456?limit=10&min_score=0.6&top_k=5&budget_weight=0.4&location_weight=0.3&property_type_weight=0.2&size_weight=0.1
```

**Response**:
```json
{
  "recommendations": [
    {
      "contact": {
        "id": 123,
        "name": "John Doe",
        "preferred_locations": [
          {
            "name": "Downtown",
            "lat": 40.7128,
            "lon": -74.0060
          }
        ],
        "min_budget": 500000.0,
        "max_budget": 800000.0,
        "min_area_sqm": 80,
        "max_area_sqm": 150,
        "property_types": ["apartment", "condo"],
        "min_rooms": 2
      },
      "property": {
        "id": 456,
        "address": "123 Main St, New York, NY",
        "location": {
          "lat": 40.7129,
          "lon": -74.0059
        },
        "price": 750000.0,
        "area_sqm": 95,
        "property_type": "apartment",
        "number_of_rooms": 3
      },
      "score": 0.85,
      "explanation": {
        "overall_score": 0.85,
        "budget_match": {
          "is_within_budget": true,
          "budget_utilization": 0.83,
          "score": 0.9
        },
        "location_match": {
          "distance_km": 0.1,
          "is_preferred_location": true,
          "score": 1.0
        },
        "property_type_match": true,
        "size_match": {
          "rooms_match": true,
          "area_match": true,
          "score": 0.8
        },
        "reasons": [
          "Excellent budget match",
          "Perfect location match",
          "Preferred property type",
          "Ideal size requirements"
        ]
      },
      "created_at": "2025-07-18T10:30:00.000Z"
    }
  ],
  "total_count": 1,
  "processing_time_ms": 45
}
```

### Get Bulk Recommendations

Get contact recommendations for multiple properties in a single request.

**Endpoint**: `POST /recommendations/bulk`

**Request Body**:
```json
{
  "property_ids": [456, 789, 123],    // Optional: specific property IDs, omit for all properties
  "limit_per_property": 5,            // Optional: max contact recommendations per property
  "min_score": 0.6,                   // Optional: minimum score threshold
  "top_k": 3,                         // Optional: return only top K contacts per property
  "top_percentile": 0.2,              // Optional: return only top 20% of contacts per property
  "score_threshold_percentile": 0.8,  // Optional: only return contacts above 80th percentile
  "budget_weight": 0.4,               // Optional: weight for budget matching (default: 0.3)
  "location_weight": 0.3,             // Optional: weight for location proximity (default: 0.25)
  "property_type_weight": 0.2,        // Optional: weight for property type matching (default: 0.2)
  "size_weight": 0.1                  // Optional: weight for size requirements (default: 0.25)
}
```

**Weight Validation**: Weights do not necessarely need to sum to 1.0, In that case Scores are larger than 1.

**Response**:
```json
{
  "recommendations": [
    {
      "property_id": 456,
      "property_address": "123 Main St, New York, NY",
      "recommendation_count": 3,
      "recommendations": [
        // Array of recommendation objects (same structure as single property)
      ]
    }
  ],
  "total_properties": 3,
  "total_recommendations": 12,
  "processing_time_ms": 156
}
```

---

## Property Comparisons

### Compare Properties

Compare two properties side by side with detailed analysis.

**Endpoint**: `GET /comparisons/properties`

**Parameters**:
- `property1_id` (query, required): Integer - ID of first property
- `property2_id` (query, required): Integer - ID of second property

**Example Request**:
```http
GET /comparisons/properties?property1_id=123&property2_id=456
```

**Response**:
```json
{
  "property1": {
    "id": 123,
    "address": "123 Main St",
    "price": 750000.0,
    "area_sqm": 95,
    "property_type": "apartment",
    "number_of_rooms": 3,
    "location": {
      "lat": 40.7128,
      "lon": -74.0060
    }
  },
  "property2": {
    "id": 456,
    "address": "456 Oak Ave",
    "price": 820000.0,
    "area_sqm": 110,
    "property_type": "condo",
    "number_of_rooms": 3,
    "location": {
      "lat": 40.7150,
      "lon": -74.0080
    }
  },
  "comparison": {
    "price_difference": 70000.0,
    "area_difference": 15,
    "price_per_sqm_difference": 125.5,
    "distance_between_km": 2.1,
    "winner": {
      "better_value": "property1",
      "larger_space": "property2",
      "lower_price": "property1"
    }
  }
}
```

---

## Quotes

### Generate Property Quote

Generate a PDF quote for a specific property and contact.

**Endpoint**: `POST /quotes/generate`

**Request Body**:
```json
{
  "property_id": 123,
  "contact_id": 456,
  "additional_costs": [          // Optional
    {
      "description": "Legal fees",
      "amount": 500000           // Amount in cents
    },
    {
      "description": "Inspection",
      "amount": 75000
    }
  ],
  "custom_message": "Special offer for valued client"  // Optional
}
```

**Response**: 
- Content-Type: `application/pdf`
- Content-Disposition: `attachment; filename="quote_123.pdf"`
- Body: PDF file data

### Generate Comparison Quote

Generate a PDF quote comparing two properties.

**Endpoint**: `POST /quotes/comparison`

**Request Body**:
```json
{
  "property1_id": 123,
  "property2_id": 456,
  "contact_id": 789,
  "custom_message": "Comparison analysis for your consideration"  // Optional
}
```

**Response**:
- Content-Type: `application/pdf`
- Content-Disposition: `attachment; filename="property_comparison.pdf"`
- Body: PDF file data

<!-- ### Generate Recommendation Quote -->

Generate a quote based on property recommendations (Not yet implemented).

**Endpoint**: `GET /quotes/recommendations`

**Parameters**:
- `property_id` (query, required): Integer - Property ID

**Response**:
```json
{
  "message": "Recommendation quote generation not yet implemented for new schema",
  "property_id": 123
}
```

---

## Data Models

### Contact
```json
{
  "id": 123,
  "name": "John Doe",
  "preferred_locations": [
    {
      "name": "Downtown",
      "lat": 40.7128,
      "lon": -74.0060
    }
  ],
  "min_budget": 500000.0,
  "max_budget": 800000.0,
  "min_area_sqm": 80,
  "max_area_sqm": 150,
  "property_types": ["apartment", "condo"],
  "min_rooms": 2
}
```

### Property
```json
{
  "id": 456,
  "address": "123 Main St, New York, NY",
  "location": {
    "lat": 40.7128,
    "lon": -74.0060
  },
  "price": 750000.0,
  "area_sqm": 95,
  "property_type": "apartment",
  "number_of_rooms": 3
}
```

### Recommendation
```json
{
  "contact": {}, // Contact object
  "property": {}, // Property object
  "score": 0.85,
  "explanation": {
    "overall_score": 0.85,
    "budget_match": {
      "is_within_budget": true,
      "budget_utilization": 0.83,
      "score": 0.9
    },
    "location_match": {
      "distance_km": 0.1,
      "is_preferred_location": true,
      "score": 1.0
    },
    "property_type_match": true,
    "size_match": {
      "rooms_match": true,
      "area_match": true,
      "score": 0.8
    },
    "reasons": ["Excellent budget match", "Perfect location match"]
  },
  "created_at": "2025-07-18T10:30:00.000Z"
}
```

## Scoring Algorithm

The recommendation system uses a weighted multi-factor scoring algorithm to find the best potential buyers (contacts) for a given property:

### Core Algorithm Flow

1. **Input**: Property ID
2. **Data Retrieval**: Get property details and all active contacts  
3. **Parallel Processing**: Use Rayon for concurrent calculations across contacts
4. **Multi-factor Scoring**: Calculate how well each contact matches the property
5. **Weighted Aggregation**: Combine scores with predefined weights
6. **Ranking & Filtering**: Sort contacts by score and apply limits/thresholds
7. **Output**: Ranked list of potential buyers with explanations

### Scoring Components

The algorithm evaluates how well each contact's preferences match the property across **4 main factors**:

1. **Budget Score (30% weight)**
   - Perfect match: Property price within contact's budget range
   - Optimal: Properties using 60-90% of the contact's budget
   - Penalties for properties outside budget range
   - Range: 0.0 - 1.0

2. **Location Score (25% weight)**
   - Distance-based scoring using Haversine formula
   - Measures distance from property to contact's preferred locations
   - ≤5km: 1.0, 5-15km: 0.5-1.0, 15-50km: 0.1-0.5, >50km: 0.1
   - Takes best score among multiple preferred locations

3. **Property Type Score (20% weight)**
   - Binary match: 1.0 if property type matches contact's preferences, 0.0 otherwise
   - 0.5 neutral score if contact has no type preferences specified

4. **Size Score (25% weight)**
   - Room requirements: Contact's minimum room needs vs property rooms
   - Area requirements: Property area vs contact's preferred area range
   - Heavy penalties for properties that don't meet minimum requirements
   - Combines room and area scores

### Overall Score Calculation

```
Overall Score = (Budget × 0.3) + (Location × 0.25) + (Type × 0.2) + (Size × 0.25)
```

### Performance Features

- **Parallel Processing**: Uses Rayon for concurrent calculations
- **Caching**: Moka cache with configurable TTL and capacity
- **Explanation**: Detailed scoring breakdown for transparency

## Rate Limits

Currently, no rate limiting is implemented.

## SDK & Examples

### cURL Examples

**Get contact recommendations for a property:**
```bash
curl -X GET "http://localhost:8080/recommendations/property/456?limit=5&min_score=0.7"
```

**Get bulk recommendations:**
```bash
curl -X POST "http://localhost:8080/recommendations/bulk" \
  -H "Content-Type: application/json" \
  -d '{"property_ids": [456, 789], "limit_per_property": 3}'
```

**Compare properties:**
```bash
curl -X GET "http://localhost:8080/comparisons/properties?property1_id=123&property2_id=456"
```

**Generate quote:**
```bash
curl -X POST "http://localhost:8080/quotes/generate" \
  -H "Content-Type: application/json" \
  -d '{"property_id": 123, "contact_id": 456}' \
  --output quote.pdf
```

## Support

For API support or questions, please refer to the project documentation or create an issue in the project repository.
