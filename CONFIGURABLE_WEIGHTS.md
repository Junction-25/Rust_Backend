# Configurable Scoring Weights Feature

## Overview

The real estate recommendation system now supports configurable scoring weights, allowing you to customize how different factors (budget, location, property type, and size) are weighted in the overall recommendation score.

## Default Weights

If no weights are provided, the system uses these default values:
- **Budget Weight**: 0.3 (30%)
- **Location Weight**: 0.25 (25%) 
- **Property Type Weight**: 0.2 (20%)
- **Size Weight**: 0.25 (25%)

## Weight Parameters

All endpoints now accept the following optional weight parameters:

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `budget_weight` | float | Weight for budget matching (0.0-1.0) | 0.3 |
| `location_weight` | float | Weight for location proximity (0.0-1.0) | 0.25 |
| `property_type_weight` | float | Weight for property type matching (0.0-1.0) | 0.2 |
| `size_weight` | float | Weight for size requirements (0.0-1.0) | 0.25 |

## Validation Rules

1. **Sum Constraint**: If all four weights are provided, they **must sum to exactly 1.0**
2. **Non-negative**: All weights must be non-negative (≥ 0.0)
3. **Partial Weights**: You can provide partial weights - missing weights will use default values
4. **Tolerance**: Sum validation uses a tolerance of 0.001 for floating-point precision

## API Examples

### Property Recommendations with Custom Weights

```bash
# Default weights
GET /recommendations/property/123?limit=10

# Custom weights (emphasizing budget and location)
GET /recommendations/property/123?limit=10&budget_weight=0.4&location_weight=0.4&property_type_weight=0.1&size_weight=0.1

# Partial weights (only budget specified, others use defaults)
GET /recommendations/property/123?budget_weight=0.5
```

### Bulk Recommendations with Custom Weights

```bash
POST /recommendations/bulk
Content-Type: application/json

{
  "property_ids": [1, 2, 3],
  "limit_per_property": 5,
  "min_score": 0.3,
  "budget_weight": 0.5,
  "location_weight": 0.2,
  "property_type_weight": 0.2,
  "size_weight": 0.1
}
```

## Error Responses

### Invalid Weight Sum
```json
{
  "error": "Invalid weights",
  "message": "Weights must sum to 1.0, got 1.200"
}
```

### Negative Weight
```json
{
  "error": "Invalid weights", 
  "message": "All weights must be non-negative"
}
```

## Use Cases

### Budget-Focused Recommendations
For clients with strict budget constraints:
```
budget_weight: 0.6
location_weight: 0.2
property_type_weight: 0.1
size_weight: 0.1
```

### Location-Priority Recommendations
For clients who prioritize location above all:
```
budget_weight: 0.1
location_weight: 0.6
property_type_weight: 0.2
size_weight: 0.1
```

### Balanced Approach
Equal weighting across all factors:
```
budget_weight: 0.25
location_weight: 0.25
property_type_weight: 0.25
size_weight: 0.25
```

## Testing

Use the provided test script to verify the feature:
```bash
python3 test_weights_api.py
```

This script tests:
- Default weight behavior
- Custom weight configurations
- Weight validation (sum to 1.0)
- Error handling for invalid weights
- Both single property and bulk recommendation endpoints

## Performance Impact

- **Caching**: Different weight combinations create separate cache entries
- **Cache Keys**: Include weight values in cache key generation
- **Memory**: Minimal additional memory usage for weight parameters
- **Processing**: No significant performance impact on recommendation calculation

## Migration Notes

- **Backward Compatibility**: All existing API calls continue to work with default weights
- **Gradual Adoption**: You can introduce custom weights incrementally
- **Default Behavior**: No changes to default recommendation behavior when weights are not specified
