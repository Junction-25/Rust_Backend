# Deployment Guide - Setting Up on Another PC

This guide explains how to replicate the Real Estate Recommendation System database and application on a new machine.

## 🚀 Quick Deployment (Recommended)

### Prerequisites
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y git curl postgresql postgresql-contrib

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

### One-Command Setup
```bash
# Clone and setup everything
git clone <repository-url>
cd real-estate-recommender
./setup.sh
```

The setup script will automatically:
- Install system dependencies
- Create PostgreSQL user and database
- Run database migrations with sample data
- Configure environment variables
- Test the installation

### Start the Application
```bash
# Development mode
cargo run

# Production mode
cargo run --release

# Or use the start script
./start.sh
```

## 🔧 Manual Step-by-Step Setup

### 1. System Prerequisites

#### Ubuntu/Debian:
```bash
sudo apt update
sudo apt install -y git curl build-essential pkg-config libssl-dev postgresql postgresql-contrib
```

#### CentOS/RHEL/Fedora:
```bash
# Fedora
sudo dnf install -y git curl gcc pkg-config openssl-devel postgresql postgresql-server postgresql-contrib

# CentOS/RHEL
sudo yum install -y git curl gcc pkg-config openssl-devel postgresql postgresql-server postgresql-contrib
```

#### macOS:
```bash
# Install Homebrew if not installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install git postgresql
```

### 2. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 3. PostgreSQL Setup

#### Start PostgreSQL Service
```bash
# Ubuntu/Debian
sudo systemctl start postgresql
sudo systemctl enable postgresql

# CentOS/RHEL (first time setup)
sudo postgresql-setup initdb
sudo systemctl start postgresql
sudo systemctl enable postgresql

# macOS
brew services start postgresql
```

#### Create Database User and Database
```bash
# Switch to postgres user
sudo -u postgres psql

# In PostgreSQL shell, create user and database:
CREATE USER your_username WITH SUPERUSER;
CREATE DATABASE real_estate_db OWNER your_username;
\q

# Or using command line (replace 'your_username' with your system username):
sudo -u postgres createuser -s $USER
sudo -u postgres createdb -O $USER real_estate_db
```

### 4. Install SQLx CLI
```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

### 5. Clone and Setup Project
```bash
# Clone the repository
git clone <repository-url>
cd real-estate-recommender

# Create environment file
cp .env.example .env

# Edit .env file if needed (default settings work for local setup)
nano .env
```

### 6. Database Migration
```bash
# Run migrations (creates tables and inserts sample data)
sqlx migrate run

# Verify migration
psql real_estate_db -c "SELECT COUNT(*) FROM properties; SELECT COUNT(*) FROM contacts;"
```

### 7. Build and Test
```bash
# Install dependencies and build
cargo build --release

# Run tests
cargo test

# Or use the comprehensive test script
./test.sh
```

### 8. Start the Application
```bash
# Development mode
cargo run

# Production mode  
cargo run --release

# Background mode
nohup cargo run --release > server.log 2>&1 &
```

## 🗄️ Database Configuration Details

### Environment Variables (.env file)
```bash
# Database Configuration
DATABASE_URL=postgresql:///real_estate_db

# For remote database or custom credentials:
# DATABASE_URL=postgresql://username:password@localhost:5432/real_estate_db

# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Cache Configuration
CACHE_TTL_SECONDS=3600
CACHE_MAX_CAPACITY=10000

# Recommendation Engine Settings
RECOMMENDATION_THRESHOLD=0.3
MAX_RECOMMENDATIONS=10
```

### Database Schema
The migration files create:
- **Properties table**: Store property listings with JSONB location data
- **Contacts table**: Store contact preferences with JSONB arrays
- **Indexes**: Optimized for performance
- **Sample data**: 3 properties and 4 contacts for testing

### Sample Data Included
- 3 Properties: Downtown apartment, suburban house, luxury penthouse
- 4 Contacts: Various preferences for testing recommendations
- All with realistic data for immediate testing

## 🔧 Different Database Setup Options

### Option 1: Local Development (Recommended)
```bash
# Uses Unix socket authentication (no password needed)
DATABASE_URL=postgresql:///real_estate_db
```

### Option 2: Password Authentication
```bash
# Set password for database user
sudo -u postgres psql -c "ALTER USER $USER PASSWORD 'your_password';"

# Update .env file
DATABASE_URL=postgresql://username:password@localhost/real_estate_db
```

### Option 3: Remote Database
```bash
# For connecting to remote PostgreSQL
DATABASE_URL=postgresql://username:password@remote_host:5432/real_estate_db
```

### Option 4: Docker PostgreSQL
```bash
# Run PostgreSQL in Docker
docker run --name postgres-real-estate -e POSTGRES_PASSWORD=password -e POSTGRES_DB=real_estate_db -p 5432:5432 -d postgres:14

# Update .env
DATABASE_URL=postgresql://postgres:password@localhost:5432/real_estate_db
```

## 🚀 Production Deployment

### 1. System Service Setup
```bash
# Create systemd service file
sudo tee /etc/systemd/system/real-estate-recommender.service > /dev/null <<EOF
[Unit]
Description=Real Estate Recommendation System
After=network.target postgresql.service

[Service]
Type=simple
User=$USER
WorkingDirectory=/path/to/real-estate-recommender
ExecStart=/path/to/real-estate-recommender/target/release/real-estate-recommender
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable real-estate-recommender
sudo systemctl start real-estate-recommender
```

### 2. Nginx Reverse Proxy (Optional)
```bash
# Install Nginx
sudo apt install nginx

# Create Nginx configuration
sudo tee /etc/nginx/sites-available/real-estate-api > /dev/null <<EOF
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

# Enable site
sudo ln -s /etc/nginx/sites-available/real-estate-api /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

## 🧪 Verification Steps

### 1. Health Check
```bash
curl http://localhost:8080/health
```

Expected response:
```json
{
  "status": "healthy",
  "timestamp": "2025-07-18T10:00:00.000Z",
  "version": "0.1.0"
}
```

### 2. Test Recommendations
```bash
# Get sample property ID
PROPERTY_ID=$(psql real_estate_db -t -c "SELECT id FROM properties LIMIT 1;" | tr -d ' ')

# Test recommendation API
curl "http://localhost:8080/recommendations/property/$PROPERTY_ID?limit=2" | jq '.'
```

### 3. Run Test Suite
```bash
./test.sh
```

### 4. Interactive API Testing
```bash
./examples.sh
```

## 🔍 Troubleshooting

### Database Connection Issues
```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Test database connection
psql real_estate_db -c "SELECT version();"

# Check database exists
psql -l | grep real_estate_db
```

### Permission Issues
```bash
# Fix PostgreSQL user permissions
sudo -u postgres psql -c "ALTER USER $USER CREATEDB;"

# Check database ownership
psql -l
```

### Port Conflicts
```bash
# Check if port 8080 is in use
sudo netstat -tlnp | grep 8080

# Kill process using port 8080
sudo kill $(sudo lsof -t -i:8080)
```

### Migration Issues
```bash
# Reset database (WARNING: deletes all data)
dropdb real_estate_db
createdb real_estate_db
sqlx migrate run

# Check migration status
sqlx migrate info
```

## 📋 Deployment Checklist

### Pre-deployment
- [ ] System dependencies installed
- [ ] Rust installed and configured
- [ ] PostgreSQL running and accessible
- [ ] SQLx CLI installed
- [ ] Repository cloned
- [ ] Environment variables configured

### Database Setup
- [ ] Database user created
- [ ] Database created
- [ ] Migrations run successfully
- [ ] Sample data loaded
- [ ] Database connections working

### Application Setup
- [ ] Dependencies installed (`cargo build`)
- [ ] Tests passing (`cargo test`)
- [ ] Application starts (`cargo run`)
- [ ] Health endpoint responding
- [ ] API endpoints working (`./examples.sh`)

### Production Ready
- [ ] Application runs in release mode
- [ ] Service configuration (systemd)
- [ ] Reverse proxy setup (if needed)
- [ ] Monitoring configured
- [ ] Backup strategy in place

## 📞 Support

If you encounter issues during deployment:

1. Check the logs: `tail -f server.log`
2. Verify database connectivity: `psql real_estate_db`
3. Run the test suite: `./test.sh`
4. Check system resources: `htop` or `top`
5. Review configuration: `cat .env`

For additional help, refer to:
- [README.md](README.md) - General project information
- [DEVELOPMENT.md](DEVELOPMENT.md) - Development guide
- [NEXT_STEPS.md](NEXT_STEPS.md) - Future enhancements

## 🎯 Quick Commands Summary

```bash
# Complete setup on new machine
git clone <repo> && cd real-estate-recommender && ./setup.sh

# Start application
cargo run --release

# Test everything
./test.sh && ./examples.sh

# Check health
curl http://localhost:8080/health
```
