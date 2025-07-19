#!/bin/bash

# Database Replication Script for Real Estate Recommendation System
# This script sets up the database on a new machine

set -e  # Exit on any error

echo "🗄️  Real Estate Recommendation System - Database Setup"
echo "====================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if PostgreSQL is installed
print_status "Checking PostgreSQL installation..."
if ! command -v psql &> /dev/null; then
    print_error "PostgreSQL is not installed. Please install it first:"
    echo ""
    echo "Ubuntu/Debian: sudo apt install postgresql postgresql-contrib"
    echo "CentOS/RHEL:   sudo yum install postgresql postgresql-server postgresql-contrib"
    echo "macOS:         brew install postgresql"
    exit 1
fi
print_success "PostgreSQL is installed"

# Check if PostgreSQL service is running
print_status "Checking PostgreSQL service..."
if ! sudo systemctl is-active --quiet postgresql 2>/dev/null && ! brew services list | grep postgresql | grep started &>/dev/null; then
    print_warning "PostgreSQL service is not running. Starting it..."
    if command -v systemctl &> /dev/null; then
        sudo systemctl start postgresql
        sudo systemctl enable postgresql
    elif command -v brew &> /dev/null; then
        brew services start postgresql
    fi
    print_success "PostgreSQL service started"
else
    print_success "PostgreSQL service is running"
fi

# Get current user
CURRENT_USER=$(whoami)
print_status "Current user: $CURRENT_USER"

# Check if database user exists
print_status "Checking database user..."
if sudo -u postgres psql -tAc "SELECT 1 FROM pg_roles WHERE rolname='$CURRENT_USER'" | grep -q 1; then
    print_success "Database user '$CURRENT_USER' exists"
else
    print_status "Creating database user '$CURRENT_USER'..."
    sudo -u postgres createuser -s "$CURRENT_USER"
    print_success "Database user '$CURRENT_USER' created"
fi

# Check if database exists
print_status "Checking database..."
if psql -lqt | cut -d \| -f 1 | grep -qw real_estate_db; then
    print_warning "Database 'real_estate_db' already exists"
    echo ""
    read -p "Do you want to recreate the database? This will delete all existing data! (y/N): " -r
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_status "Dropping existing database..."
        dropdb real_estate_db
        print_status "Creating fresh database..."
        createdb real_estate_db
        print_success "Database 'real_estate_db' recreated"
    else
        print_status "Using existing database"
    fi
else
    print_status "Creating database 'real_estate_db'..."
    # createdb real_estate_db
    print_success "Database 'real_estate_db' created"
fi

# Check if SQLx CLI is installed
print_status "Checking SQLx CLI..."
if ! command -v sqlx &> /dev/null; then
    print_status "Installing SQLx CLI..."
    cargo install sqlx-cli --no-default-features --features rustls,postgres
    print_success "SQLx CLI installed"
else
    print_success "SQLx CLI is available"
fi

# Create .env file if it doesn't exist
print_status "Setting up environment configuration..."
if [ ! -f ".env" ]; then
    cat > .env << EOF
# Database configuration
DATABASE_URL=postgresql:///real_estate_db

# Server configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# API configuration
API_KEY=your_api_key_here

# Cache configuration
CACHE_TTL_SECONDS=3600
CACHE_MAX_CAPACITY=10000

# Recommendation engine settings
RECOMMENDATION_THRESHOLD=0.3
MAX_RECOMMENDATIONS=10
EOF
    print_success "Environment file (.env) created"
else
    print_success "Environment file (.env) already exists"
fi

# Run database migrations
print_status "Running database migrations..."
if sqlx migrate run; then
    print_success "Database migrations completed successfully"
else
    print_error "Database migrations failed"
    exit 1
fi

# Verify database setup
print_status "Verifying database setup..."
PROPERTY_COUNT=$(psql real_estate_db -t -c "SELECT COUNT(*) FROM properties;" 2>/dev/null | tr -d ' ') || PROPERTY_COUNT=0
CONTACT_COUNT=$(psql real_estate_db -t -c "SELECT COUNT(*) FROM contacts;" 2>/dev/null | tr -d ' ') || CONTACT_COUNT=0

if [ "$PROPERTY_COUNT" -gt 0 ] && [ "$CONTACT_COUNT" -gt 0 ]; then
    print_success "Database verification successful"
    echo "   - Properties: $PROPERTY_COUNT"
    echo "   - Contacts: $CONTACT_COUNT"
else
    print_error "Database verification failed"
    echo "   - Properties: $PROPERTY_COUNT"
    echo "   - Contacts: $CONTACT_COUNT"
    exit 1
fi

# Test database connection with application
print_status "Testing database connection..."
if DATABASE_URL="postgresql:///real_estate_db" timeout 10 psql real_estate_db -c "SELECT 1;" > /dev/null 2>&1; then
    print_success "Database connection test successful"
else
    print_error "Database connection test failed"
    exit 1
fi

echo ""
print_success "🎉 Database setup completed successfully!"
echo ""
echo "📋 Database Information:"
echo "   Database Name: real_estate_db"
echo "   Database URL:  postgresql:///real_estate_db"
echo "   Properties:    $PROPERTY_COUNT"
echo "   Contacts:      $CONTACT_COUNT"
echo ""
echo "🚀 Next Steps:"
echo "   1. Build the application: cargo build --release"
echo "   2. Run tests: ./test.sh"
echo "   3. Start the server: cargo run --release"
echo "   4. Test the API: ./examples.sh"
echo ""
echo "🔍 Quick Test Commands:"
echo "   psql real_estate_db -c 'SELECT id, address FROM properties;'"
echo "   psql real_estate_db -c 'SELECT id, name FROM contacts;'"
echo ""
echo "💡 For troubleshooting, see DEPLOYMENT.md"
