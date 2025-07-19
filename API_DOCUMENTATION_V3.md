# MY-RECOMMENDER API Documentation

## Enterprise-Grade Real Estate Recommendation System

### Overview
MY-RECOMMENDER is a sophisticated AI-powered recommendation system for real estate, featuring three phases of advanced ML capabilities:

- **Phase 1**: Neural network-based recommendations with collaborative filtering
- **Phase 2**: Two-stage retrieval with feature store and advanced embedding pipelines  
- **Phase 3**: Real-time learning, A/B testing, drift detection, and advanced analytics

---

## API Endpoints

### 1. Basic Recommendations

#### GET `/recommendations/property/{property_id}`
Get property recommendations based on a specific property.

**Parameters:**
- `property_id` (path): Target property ID
- `limit` (query, optional): Number of recommendations (default: 10)
- `min_score` (query, optional): Minimum similarity score
- `top_k` (query, optional): Top-K filtering
- `top_percentile` (query, optional): Top percentile filtering
- `score_threshold_percentile` (query, optional): Score threshold percentile
- `neural_scoring` (query, optional): Enable neural network scoring

**Example:**
```bash
GET /recommendations/property/123?limit=5&neural_scoring=true
```

#### GET `/recommendations/contact/{contact_id}`
Get recommendations for a specific contact/user.

**Parameters:**
- `contact_id` (path): Target contact ID
- Same query parameters as property recommendations

**Example:**
```bash
GET /recommendations/contact/456?limit=10&min_score=0.7
```

#### POST `/recommendations/bulk`
Get bulk recommendations for multiple entities.

**Request Body:**
```json
{
  "property_ids": [123, 456, 789],
  "contact_ids": [101, 102, 103],
  "limit": 5,
  "neural_scoring": true
}
```

---

### 2. Advanced Recommendations (Phase 2)

#### GET `/advanced/recommendations/fast`
Ultra-fast recommendations using optimized retrieval.

**Parameters:**
- `user_id` (query): User identifier
- `limit` (query, optional): Number of recommendations (default: 10)
- `diversify` (query, optional): Enable result diversification

**Performance:** Sub-100ms response time

#### GET `/advanced/recommendations/accurate`
High-accuracy recommendations using full ML pipeline.

**Parameters:**
- `user_id` (query): User identifier  
- `limit` (query, optional): Number of recommendations (default: 10)
- `explain` (query, optional): Include explanation scores

**Features:** Neural embeddings, feature store integration, two-stage retrieval

#### POST `/advanced/recommendations/batch`
Batch processing for multiple recommendation requests.

**Request Body:**
```json
{
  "requests": [
    {
      "user_id": "user1",
      "limit": 5,
      "type": "fast"
    },
    {
      "user_id": "user2", 
      "limit": 10,
      "type": "accurate"
    }
  ]
}
```

#### GET `/advanced/stats`
Service performance statistics and metrics.

**Response:**
```json
{
  "requests_processed": 15420,
  "average_response_time": 45,
  "cache_hit_rate": 0.85,
  "model_accuracy": 0.94,
  "system_load": 0.67
}
```

#### GET `/advanced/health`
Advanced health check with ML model status.

#### POST `/advanced/benchmark`
Run comprehensive performance benchmark.

**Request Body:**
```json
{
  "test_duration_seconds": 30,
  "concurrent_users": 100,
  "test_scenarios": ["fast", "accurate", "batch"]
}
```

---

### 3. AI & Machine Learning

#### GET `/ai/recommendations`
AI-powered recommendations using advanced ML models.

**Parameters:**
- `user_id` (query): User identifier
- `limit` (query, optional): Number of recommendations
- `algorithm` (query, optional): ML algorithm selection
- `personalization_level` (query, optional): Personalization intensity

#### POST `/ai/initialize`
Initialize AI models and load training data.

**Request Body:**
```json
{
  "model_type": "collaborative_filtering",
  "training_data_size": 10000,
  "validation_split": 0.2
}
```

#### GET `/ai/stats`
AI model performance statistics.

**Response:**
```json
{
  "model_accuracy": 0.92,
  "training_samples": 50000,
  "last_training": "2025-07-19T10:30:00Z",
  "prediction_latency": 12.5,
  "memory_usage_mb": 256
}
```

#### POST `/ai/feedback`
Provide feedback to improve AI models.

**Request Body:**
```json
{
  "user_id": "user123",
  "property_id": 789,
  "feedback_type": "like",
  "engagement_score": 0.85,
  "context": {
    "session_id": "sess_456",
    "interaction_type": "click"
  }
}
```

#### GET `/ai/market-analysis`
AI-powered market trend analysis.

**Parameters:**
- `region` (query, optional): Geographic region
- `property_type` (query, optional): Property type filter
- `time_range` (query, optional): Analysis time range

---

### 4. Property Comparisons

#### POST `/comparisons/compare`
Compare multiple properties side-by-side.

**Request Body:**
```json
{
  "property_ids": [123, 456, 789],
  "comparison_criteria": [
    "price", "location", "size", "amenities"
  ],
  "include_market_data": true
}
```

**Response:**
```json
{
  "comparison_matrix": {
    "123": {
      "price": 450000,
      "score": 0.85,
      "advantages": ["location", "size"],
      "disadvantages": ["price"]
    }
  },
  "recommendation": "Property 456 offers best value"
}
```

---

### 5. Real-time & WebSocket

#### WebSocket: `/ws`
Real-time notifications and updates.

**Connection:**
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Real-time update:', data);
};
```

#### GET `/realtime/stats`
WebSocket connection statistics.

#### POST `/realtime/notify`
Send test notification through WebSocket.

**Request Body:**
```json
{
  "user_id": "user123",
  "message": "New property matches your criteria!",
  "type": "property_alert"
}
```

#### POST `/realtime/custom-notify`
Send custom notification with advanced options.

#### POST `/realtime/monitoring/start`
Start real-time monitoring for a user.

#### GET `/realtime/health`
Real-time system health check.

---

### 6. Phase 3 Advanced ML Features

#### Real-time Learning Engine
- **Endpoint**: `/ml/online-learning/feedback`
- **Method**: POST
- **Description**: Process user feedback for real-time model updates

#### Concept Drift Detection
- **Endpoint**: `/ml/drift-detection/status`
- **Method**: GET
- **Description**: Check model drift status and recommendations

#### A/B Testing Framework
- **Endpoint**: `/ml/experiments/create`
- **Method**: POST
- **Description**: Create new A/B test experiments

- **Endpoint**: `/ml/experiments/{id}/results`
- **Method**: GET
- **Description**: Get experiment results and statistical analysis

#### Advanced Analytics
- **Endpoint**: `/ml/analytics/dashboard`
- **Method**: GET
- **Description**: Comprehensive analytics dashboard

- **Endpoint**: `/ml/analytics/user-segments`
- **Method**: GET
- **Description**: User segmentation analysis

---

## Authentication & Security

### API Key Authentication
All endpoints require API key authentication:

```bash
curl -H "Authorization: Bearer YOUR_API_KEY" \
     https://api.my-recommender.com/recommendations/property/123
```

### Rate Limiting
- **Standard endpoints**: 1000 requests/hour
- **Advanced ML endpoints**: 500 requests/hour
- **Real-time endpoints**: 100 connections/user

---

## Response Formats

### Success Response
```json
{
  "status": "success",
  "data": { ... },
  "metadata": {
    "processing_time_ms": 45,
    "algorithm_used": "neural_collaborative_filtering",
    "confidence_score": 0.94
  }
}
```

### Error Response
```json
{
  "status": "error",
  "error": "Invalid parameter",
  "message": "Property ID must be a positive integer",
  "code": "INVALID_PARAMETER",
  "timestamp": "2025-07-19T10:30:00Z"
}
```

---

## Performance Benchmarks

| Endpoint Type | Avg Response Time | P95 Response Time | Throughput |
|---------------|------------------|------------------|------------|
| Basic Recommendations | 50ms | 120ms | 2000 req/s |
| Advanced Fast | 25ms | 60ms | 4000 req/s |
| Advanced Accurate | 150ms | 300ms | 800 req/s |
| AI Recommendations | 75ms | 180ms | 1200 req/s |
| Real-time Updates | 5ms | 15ms | 10000 msg/s |

---

## SDK & Client Libraries

### JavaScript/TypeScript
```bash
npm install @my-recommender/sdk
```

### Python
```bash
pip install my-recommender-python
```

### Rust
```toml
[dependencies]
my-recommender = "0.3.0"
```

---

## Support & Contact

- **Documentation**: https://docs.my-recommender.com
- **API Status**: https://status.my-recommender.com
- **Support**: support@my-recommender.com
- **GitHub**: https://github.com/my-recommender/api

---

*Last Updated: July 19, 2025*
*API Version: 3.0 (Phase 3 - Enterprise ML)*
