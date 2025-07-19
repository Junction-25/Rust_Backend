# 📋 MY-RECOMMENDER API REFERENCE

## 📖 **Complete API Documentation**

This document provides comprehensive documentation for all API endpoints available in the MY-RECOMMENDER system.

**Base URL**: `http://localhost:8080`  
**Version**: 1.0.0  
**Content-Type**: `application/json`

---

## 🏥 **System Health & Monitoring**

### **System Health Check**
```http
GET /health
```

**Description**: Returns system health status and version information.

**Response**:
```json
{
  "status": "healthy",
  "timestamp": "2025-07-19T00:00:00Z",
  "version": "1.0.0"
}
```

---

## 🎯 **Traditional Recommendations API**

### **Get Recommendations for Contact**
```http
GET /recommendations/contact/{contact_id}
```

**Parameters**:
- `contact_id` (path, required): Contact ID
- `limit` (query, optional): Maximum number of recommendations (default: 10)
- `min_score` (query, optional): Minimum recommendation score (0.0-1.0)
- `top_percentile` (query, optional): Return only top percentile matches
- `top_k` (query, optional): Return top K matches

**Example**:
```bash
GET /recommendations/contact/1001?limit=5&min_score=0.7
```

**Response**:
```json
{
  "recommendations": [
    {
      "contact": {
        "id": 1001,
        "name": "Sarah Ruiz",
        "preferred_locations": [...],
        "min_budget": 11680000.0,
        "max_budget": 13840000.0
      },
      "property": {
        "id": 1,
        "address": "11458 Christopher Point, Bab",
        "location": {"lat": 36.7243, "lon": 3.21647},
        "price": 15540000.0,
        "area_sqm": 137,
        "property_type": "apartment",
        "number_of_rooms": 3
      },
      "score": 0.95,
      "explanation": {
        "overall_score": 0.95,
        "budget_match": {"score": 1.0, "is_within_budget": true},
        "location_match": {"score": 0.9, "distance_km": 2.5},
        "property_type_match": true,
        "size_match": {"score": 1.0, "rooms_match": true}
      },
      "created_at": "2025-07-19T00:00:00Z"
    }
  ]
}
```

### **Get Recommendations for Property**
```http
GET /recommendations/property/{property_id}
```

**Parameters**:
- `property_id` (path, required): Property ID
- `limit` (query, optional): Maximum number of recommendations
- `min_score` (query, optional): Minimum recommendation score

**Response**: Similar structure to contact recommendations.

### **Bulk Recommendations**
```http
POST /recommendations/bulk
```

**Request Body**:
```json
{
  "contact_ids": [1001, 1002, 1003],
  "property_ids": [1, 2, 3],
  "limit_per_property": 5,
  "limit_per_contact": 5,
  "min_score": 0.6,
  "top_percentile": 0.1,
  "top_k": 10
}
```

**Response**:
```json
{
  "recommendations": [
    {
      "property_id": 1,
      "property_address": "11458 Christopher Point, Bab",
      "recommendations": [...]
    },
    {
      "contact_id": 1001,
      "contact_name": "Sarah Ruiz",
      "recommendations": [...]
    }
  ]
}
```

---

## 🧠 **AI/ML Recommendations API**

### **Initialize AI Models**
```http
POST /ai/models/initialize
```

**Description**: Initialize or reinitialize all AI/ML models.

**Response**:
```json
{
  "status": "success",
  "message": "AI models initialized successfully"
}
```

### **Get AI Model Statistics**
```http
GET /ai/models/stats
```

**Response**:
```json
{
  "initialized": true,
  "model_version": "v1.0.0-hackathon",
  "last_updated": "2025-07-19T00:00:00Z",
  "features": {
    "collaborative_filtering": true,
    "market_trends": true,
    "predictive_matching": true,
    "price_prediction": true
  }
}
```

### **AI-Powered Recommendations**
```http
GET /ai/recommendations/contact/{contact_id}
```

**Parameters**:
- `contact_id` (path, required): Contact ID
- `limit` (query, optional): Maximum recommendations
- `enable_ml_scoring` (query, optional): Enable ML-enhanced scoring
- `enable_market_analysis` (query, optional): Include market trend analysis
- `enable_predictive_matching` (query, optional): Use predictive algorithms
- `include_price_predictions` (query, optional): Include future price forecasts
- `min_confidence` (query, optional): Minimum AI confidence score

**Example**:
```bash
GET /ai/recommendations/contact/1001?enable_ml_scoring=true&enable_market_analysis=true&min_confidence=0.7
```

**Response**:
```json
{
  "recommendations": [
    {
      "recommendation": {
        "contact": {...},
        "property": {...},
        "score": 0.92,
        "explanation": {...}
      },
      "ml_enhancement": {
        "collaborative_score": 0.85,
        "market_trend_factor": 1.2,
        "predictive_likelihood": 0.78,
        "price_prediction": {
          "current_price": 15540000.0,
          "predicted_price": 16200000.0,
          "confidence": 0.82,
          "time_horizon": "6 months"
        }
      },
      "ai_confidence": 0.89
    }
  ]
}
```

### **Submit User Feedback**
```http
POST /ai/feedback
```

**Request Body**:
```json
{
  "contact_id": 1001,
  "property_id": 1,
  "feedback_type": "view|contact|interest|purchase",
  "outcome": "positive|negative|neutral"
}
```

**Response**:
```json
{
  "status": "success",
  "message": "AI models updated with feedback"
}
```

### **Market Analysis**
```http
GET /ai/market/analysis
```

**Response**:
```json
{
  "generated_at": "2025-07-19T00:00:00Z",
  "market_insights": [
    "⚡ Hot market alert: High demand in Bab apartment sector",
    "📈 Price increase trend detected in luxury villa market",
    "📊 Supply shortage in downtown office spaces"
  ],
  "trend_summary": {
    "hot_markets": 15,
    "price_increases": 8,
    "new_inventory": 23
  }
}
```

---

## ⚖️ **Property Comparison API**

### **Compare Two Properties**
```http
GET /comparisons/properties?property1_id={id1}&property2_id={id2}
```

**Parameters**:
- `property1_id` (query, required): First property ID
- `property2_id` (query, required): Second property ID

**Response**:
```json
{
  "property1": {
    "id": 1,
    "address": "11458 Christopher Point, Bab",
    "location": {"lat": 36.7243, "lon": 3.21647},
    "price": 15540000.0,
    "area_sqm": 137,
    "property_type": "apartment",
    "number_of_rooms": 3
  },
  "property2": {
    "id": 2,
    "address": "22822 Leblanc Squares, Constantine",
    "location": {"lat": 36.37705, "lon": 6.59604},
    "price": 12340000.0,
    "area_sqm": 95,
    "property_type": "apartment",
    "number_of_rooms": 2
  },
  "comparison": {
    "price_difference": 3200000.0,
    "price_difference_percentage": 25.9,
    "area_difference": 42,
    "area_difference_percentage": 44.2,
    "room_difference": 1,
    "distance_between": 304.2,
    "price_per_sqm_property1": 113431.39,
    "price_per_sqm_property2": 129894.74,
    "better_value": "property1"
  },
  "analysis": {
    "property1_advantages": [
      "Larger living space",
      "Better price per square meter",
      "More rooms"
    ],
    "property2_advantages": [
      "Lower absolute price",
      "Different location option"
    ],
    "recommendation": "Property 1 offers better value for money with more space"
  }
}
```

---

## 📄 **PDF Generation API**

### **Generate Property Quote**
```http
POST /quotes/generate
```

**Request Body**:
```json
{
  "property_id": 1,
  "contact_id": 1001,
  "quote_type": "purchase|rental|investment",
  "additional_costs": [
    {
      "name": "Inspection Fee",
      "amount": 500.0
    },
    {
      "name": "Legal Fees",
      "amount": 2500.0
    }
  ]
}
```

**Response**: PDF file download

### **Generate Property Comparison Report**
```http
POST /quotes/comparison
```

**Request Body**:
```json
{
  "property1_id": 1,
  "property2_id": 2,
  "contact_id": 1001
}
```

**Response**: PDF file download

### **Generate Recommendations Summary**
```http
GET /quotes/recommendations?property_id={id}
```

**Parameters**:
- `property_id` (query, required): Property ID for recommendations summary

**Response**: PDF file download

---

## ⚡ **Real-time Features API**

### **Real-time System Health**
```http
GET /realtime/health
```

**Response**:
```json
{
  "status": "healthy",
  "timestamp": "2025-07-19T00:00:00Z",
  "websocket_server": "running",
  "notification_service": "active",
  "ai_engine": "operational",
  "features": {
    "real_time_recommendations": true,
    "live_property_updates": true,
    "market_alerts": true,
    "price_predictions": true
  }
}
```

### **WebSocket Connection Statistics**
```http
GET /realtime/stats
```

**Response**:
```json
{
  "connected_clients": 5,
  "total_messages_sent": 1247,
  "uptime_seconds": 3600,
  "active_subscriptions": [
    {
      "subscription_type": "recommendations",
      "subscriber_count": 3
    },
    {
      "subscription_type": "market_updates",
      "subscriber_count": 2
    }
  ]
}
```

### **Send Test Notifications**
```http
POST /realtime/test-notification
```

**Request Body**:
```json
{
  "notification_type": "recommendation|market_alert|price_change|price_prediction",
  "count": 3
}
```

**Response**:
```json
{
  "success": true,
  "message": "Sent 3 recommendation notifications",
  "timestamp": "2025-07-19T00:00:00Z"
}
```

### **Send Custom Notification**
```http
POST /realtime/send-notification
```

**Request Body**:
```json
{
  "contact_id": 1001,
  "notification_type": "recommendation|market_alert|price_change",
  "message": "New luxury property matches your criteria!",
  "data": {
    "property_id": 123,
    "urgency": "high"
  }
}
```

### **Start Real-time Monitoring**
```http
POST /realtime/monitor/{contact_id}
```

**Parameters**:
- `contact_id` (path, required): Contact ID to monitor

**Response**:
```json
{
  "success": true,
  "message": "Real-time monitoring started for contact 1001"
}
```

---

## 🌐 **WebSocket API**

### **WebSocket Connection**
```
WS ws://localhost:8080/ws
```

### **WebSocket Message Types**

#### **Subscribe to Notifications**
```json
{
  "type": "subscribe",
  "contact_id": 1001,
  "subscription_types": [
    "new_properties",
    "price_changes", 
    "market_updates",
    "recommendations",
    "price_predictions"
  ]
}
```

#### **Unsubscribe from Notifications**
```json
{
  "type": "unsubscribe",
  "contact_id": 1001,
  "subscription_types": ["price_changes"]
}
```

#### **Heartbeat**
```json
{
  "type": "heartbeat",
  "timestamp": "2025-07-19T00:00:00Z"
}
```

### **Incoming WebSocket Messages**

#### **New Recommendation**
```json
{
  "type": "new_recommendation",
  "contact_id": 1001,
  "property_id": 123,
  "score": 0.92,
  "reason": "Perfect match for your budget and location preferences",
  "timestamp": "2025-07-19T00:00:00Z"
}
```

#### **Property Update**
```json
{
  "type": "property_update",
  "property_id": 123,
  "update_type": "price_change",
  "details": {
    "old_value": 15540000.0,
    "new_value": 15200000.0,
    "change_percentage": -2.2
  },
  "timestamp": "2025-07-19T00:00:00Z"
}
```

#### **Market Alert**
```json
{
  "type": "market_alert",
  "location": "Bab",
  "property_type": "apartment",
  "alert_type": "hot_market",
  "message": "High demand detected in Bab apartment sector",
  "timestamp": "2025-07-19T00:00:00Z"
}
```

#### **Price Prediction**
```json
{
  "type": "price_prediction",
  "property_id": 123,
  "current_price": 15540000.0,
  "predicted_price": 16200000.0,
  "confidence": 0.85,
  "time_horizon": "6 months",
  "timestamp": "2025-07-19T00:00:00Z"
}
```

---

## 🔧 **Error Handling**

### **Standard Error Response**
```json
{
  "error": "Error category",
  "message": "Detailed error description",
  "code": "ERROR_CODE",
  "timestamp": "2025-07-19T00:00:00Z"
}
```

### **HTTP Status Codes**
- `200` - OK: Request successful
- `400` - Bad Request: Invalid request parameters
- `404` - Not Found: Resource not found
- `500` - Internal Server Error: Server error occurred

### **Common Error Codes**
- `CONTACT_NOT_FOUND` - Contact ID does not exist
- `PROPERTY_NOT_FOUND` - Property ID does not exist  
- `INVALID_PARAMETERS` - Request parameters are invalid
- `AI_MODEL_NOT_READY` - AI models are not initialized
- `WEBSOCKET_CONNECTION_FAILED` - WebSocket connection error

---

## 📊 **Rate Limiting & Performance**

### **Rate Limits**
- **General API**: 100 requests/minute per IP
- **Bulk Operations**: 10 requests/minute per IP
- **WebSocket Connections**: 10 connections per IP
- **PDF Generation**: 20 requests/minute per IP

### **Response Times**
- **Health Check**: < 10ms
- **Simple Recommendations**: < 50ms
- **AI Recommendations**: < 200ms
- **PDF Generation**: < 300ms
- **WebSocket Connection**: < 5ms

### **Pagination**
Most list endpoints support pagination via:
- `limit`: Maximum results per page (default: 10, max: 100)
- `offset`: Number of results to skip
- `page`: Page number (alternative to offset)

---

## 🔐 **Authentication & Authorization**

**Note**: Current version uses open API for development. Production deployment should implement:
- API key authentication
- Rate limiting per user
- Role-based access control
- JWT token validation

---

## 📚 **Additional Resources**

- **Architecture Guide**: See [ARCHITECTURE.md](./ARCHITECTURE.md)
- **System Summary**: See [SYSTEM_SUMMARY.md](./SYSTEM_SUMMARY.md)
- **Deployment Guide**: See [docs/DEPLOYMENT.md](./docs/DEPLOYMENT.md)
- **Testing Suite**: Run `./test-comprehensive.sh`

---

*Last Updated: July 19, 2025*  
*API Version: 1.0.0*
