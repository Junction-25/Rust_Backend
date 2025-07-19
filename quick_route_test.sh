#!/bin/bash

# Quick Route Verification Script
# Tests if the server can start and basic routes respond

set -e

BASE_URL="http://localhost:8080"
SERVER_PID=""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🔧 MY-RECOMMENDER Route Verification${NC}"
echo -e "${BLUE}====================================${NC}"

# Start server
echo -e "${YELLOW}Starting server...${NC}"
cargo build --release --quiet

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

./target/release/my-recommender > /dev/null 2>&1 &
SERVER_PID=$!

echo -e "Server PID: $SERVER_PID"

# Wait for server to start
echo -e "Waiting for server..."
sleep 5

# Test basic routes
echo -e "\n${YELLOW}Testing basic routes:${NC}"

# Health check
if curl -s "$BASE_URL/health" > /dev/null; then
    echo -e "${GREEN}✅ Health check: OK${NC}"
else
    echo -e "${RED}❌ Health check: FAILED${NC}"
fi

# Basic recommendation
if curl -s "$BASE_URL/recommendations/property/1?limit=3" > /dev/null; then
    echo -e "${GREEN}✅ Basic recommendations: OK${NC}"
else
    echo -e "${RED}❌ Basic recommendations: FAILED${NC}"
fi

# Advanced recommendations
if curl -s "$BASE_URL/advanced/stats" > /dev/null; then
    echo -e "${GREEN}✅ Advanced stats: OK${NC}"
else
    echo -e "${RED}❌ Advanced stats: FAILED${NC}"
fi

# AI recommendations
if curl -s "$BASE_URL/ai/recommendations?user_id=1&limit=3" > /dev/null; then
    echo -e "${GREEN}✅ AI recommendations: OK${NC}"
else
    echo -e "${RED}❌ AI recommendations: FAILED${NC}"
fi

# Cleanup
echo -e "\n${YELLOW}Stopping server...${NC}"
kill $SERVER_PID 2>/dev/null || true

echo -e "${GREEN}✅ Route verification completed${NC}"
