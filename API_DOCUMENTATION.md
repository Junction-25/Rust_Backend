# Real Estate Recommendation API Documentation

## Overview

This API provides intelligent real estate contact recommendations, property comparisons, and quote generation services. The system uses a multi-factor scoring algorithm to match potential buyers (contacts) with properties based on their preferences and property characteristics.

**Base URL**: `http://localhost:8080` (configurable)

## Table of Contents

- [Authentication](#authentication)
- [Response Format](#response-format)
- [Error Handling](#error-handling)
- [Endpoints](#endpoints)
  - [Health Check](#health-check)
  - [Recommendations](#recommendations)
  - [Property Comparisons](#property-comparisons)
  - [Quotes](#quotes)
- [Data Models](#data-models)
- [Scoring Algorithm](#scoring-algorithm)

## Authentication

Currently, the API does not implement authentication. All endpoints are publicly accessible.

## Response Format

All API responses follow a consistent JSON format:

### Success Response
```json
{
  "data": {}, // Response data
  "status": "success"
}
```

### Error Response
```json
{
  "error": "Error type description",
  "message": "Detailed error message"
}
```

## Error Handling

The API returns standard HTTP status codes:

- `200` - Success
- `400` - Bad Request (invalid parameters)
- `404` - Not Found (resource doesn't exist)
- `500` - Internal Server Error

## Endpoints

### Health Check

Check if the API service is running and healthy.

**Endpoint**: `GET /health`

**Response**:
```json
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

**Weight Validation**: If all four weights are provided, they must sum to exactly 1.0.

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

### Generate Recommendation Quote

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
