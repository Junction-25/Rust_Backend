#!/bin/bash

echo "🏠 Real Estate Recommendation System Setup"
echo "=========================================="

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "✅ Rust is installed"

# Check if PostgreSQL is available
if ! command -v psql &> /dev/null; then
    echo "❌ PostgreSQL is not installed."
    echo ""
    echo "Installing PostgreSQL..."
    
    # Detect OS and install PostgreSQL
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v apt &> /dev/null; then
            echo "📦 Installing PostgreSQL using apt..."
            sudo apt update
            sudo apt install -y postgresql postgresql-contrib
        elif command -v dnf &> /dev/null; then
            echo "📦 Installing PostgreSQL using dnf..."
            sudo dnf install -y postgresql postgresql-server postgresql-contrib
            sudo postgresql-setup --initdb
        elif command -v yum &> /dev/null; then
            echo "📦 Installing PostgreSQL using yum..."
            sudo yum install -y postgresql postgresql-server postgresql-contrib
            sudo postgresql-setup initdb
        else
            echo "❌ Cannot detect package manager. Please install PostgreSQL manually."
            exit 1
        fi
        
        # Start PostgreSQL service
        sudo systemctl start postgresql
        sudo systemctl enable postgresql
        
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        if command -v brew &> /dev/null; then
            echo "📦 Installing PostgreSQL using Homebrew..."
            brew install postgresql
            brew services start postgresql
        else
            echo "❌ Please install Homebrew first, then run: brew install postgresql"
            exit 1
        fi
    else
        echo "❌ Unsupported operating system"
        exit 1
    fi
fi

echo "✅ PostgreSQL is available"

# Check if SQLx CLI is installed
if ! command -v sqlx &> /dev/null; then
    echo "📦 Installing SQLx CLI..."
    cargo install sqlx-cli --no-default-features --features native-tls,postgres
    
    if [ $? -ne 0 ]; then
        echo "❌ Failed to install SQLx CLI"
        exit 1
    fi
fi

echo "✅ SQLx CLI is available"

echo ""
echo "🗄️  Setting up database..."

# Create database and user with Unix socket authentication
echo "Setting up PostgreSQL user and database..."
echo "Current user: $(whoami)"

# Check if database already exists
if psql -lqt | cut -d \| -f 1 | grep -qw real_estate_db; then
    echo "✅ Database 'real_estate_db' already exists"
else
    echo "Creating database..."
    createdb real_estate_db
    if [ $? -eq 0 ]; then
        echo "✅ Database 'real_estate_db' created successfully"
    else
        echo "❌ Failed to create database"
        exit 1
    fi
fi

echo ""
echo "📝 Updating .env file..."

# Create or update .env file with Unix socket connection
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

echo "✅ Database URL: postgresql:///real_estate_db"

echo ""
echo "🔄 Running database migrations..."
export DATABASE_URL="postgresql:///real_estate_db"
sqlx migrate run

if [ $? -eq 0 ]; then
    echo "✅ Database migrations completed successfully"
else
    echo "❌ Database migrations failed"
    echo "   You may need to run 'sqlx migrate run' manually"
    exit 1
fi

echo ""
echo "🔨 Building project in release mode..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Project built successfully"
else
    echo "❌ Build failed"
    exit 1
fi

echo ""
echo "🎉 Setup completed successfully!"
echo ""
echo "Next steps:"
echo "  1. Run './test.sh' to run the test suite"
echo "  2. Run 'cargo run --release' to start the server"
echo "  3. Run './examples.sh' to test the API endpoints"
echo ""
echo "Server will be available at: http://127.0.0.1:8080"
echo "Health check: curl http://127.0.0.1:8080/health"
        fi
    else
        echo "❌ Unsupported OS. Please install PostgreSQL manually."
        exit 1
    fi
    
    sleep 3  # Give PostgreSQL time to start
fi

echo "✅ PostgreSQL is available"

# Install sqlx-cli if not present
if ! command -v sqlx &> /dev/null; then
    echo "📦 Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features rustls,postgres
fi

echo "✅ SQLx CLI is available"

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cp .env.example .env
    echo "⚠️  Please update the DATABASE_URL in .env with your PostgreSQL credentials"
fi

# Create database
echo "🗄️  Setting up database..."

# Try to set up PostgreSQL user if it doesn't exist
echo "Setting up PostgreSQL user and database..."

# Get current system user
current_user=$(whoami)
echo "Current user: $current_user"

# Setup PostgreSQL user and database
echo "Creating PostgreSQL user and database..."

# Method 1: Try with peer authentication (common on Ubuntu)
if sudo -u postgres psql -c "SELECT 1" &>/dev/null; then
    echo "Using peer authentication..."
    
    # Create user if it doesn't exist
    sudo -u postgres psql -c "SELECT 1 FROM pg_roles WHERE rolname='$current_user'" | grep -q 1 || \
    sudo -u postgres createuser --interactive --pwprompt $current_user
    
    # Create database
    sudo -u postgres createdb -O $current_user real_estate_db 2>/dev/null || echo "Database may already exist"
    
    # Set connection string
    DATABASE_URL="postgresql://$current_user@localhost/real_estate_db"
    
else
    # Method 2: Interactive setup
    read -p "Enter PostgreSQL username (default: postgres): " db_user
    db_user=${db_user:-postgres}

    read -p "Enter PostgreSQL password: " -s db_password
    echo

    read -p "Enter database name (default: real_estate_db): " db_name
    db_name=${db_name:-real_estate_db}

    # Create database
    echo "Creating database ${db_name}..."
    PGPASSWORD=$db_password createdb -U $db_user $db_name 2>/dev/null || echo "Database may already exist"
    
    DATABASE_URL="postgresql://${db_user}:${db_password}@localhost/${db_name}"
fi

# Update .env file with database URL
echo "📝 Updating .env file..."
sed -i.bak "s|DATABASE_URL=.*|DATABASE_URL=${DATABASE_URL}|" .env

echo "✅ Database URL: $DATABASE_URL"

# Run migrations
echo "🔄 Running database migrations..."
export DATABASE_URL=$DATABASE_URL
sqlx migrate run

if [ $? -eq 0 ]; then
    echo "✅ Database migrations completed successfully"
else
    echo "❌ Database migrations failed"
    exit 1
fi

# Build the project
echo "🔨 Building the project..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build completed successfully"
else
    echo "❌ Build failed"
    exit 1
fi

echo ""
echo "🎉 Setup completed successfully!"
echo ""
echo "To start the server:"
echo "  cargo run"
echo ""
echo "Or run the release version:"
echo "  ./target/release/real-estate-recommender"
echo ""
echo "The server will be available at: http://localhost:8080"
echo "Health check: curl http://localhost:8080/health"
echo ""
echo "📚 Check the README.md for API usage examples"
