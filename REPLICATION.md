# Complete System Replication Guide

This guide explains how to replicate the Real Estate Recommendation System on a new computer from scratch.

## 📋 Prerequisites

Before starting, ensure you have:

1. **Operating System**: Linux (Ubuntu/Debian/CentOS/RHEL) or macOS
2. **User Account**: Administrator/sudo privileges
3. **Internet Connection**: For downloading dependencies

## 🚀 Quick Setup (Recommended)

For a completely fresh machine, run these commands:

```bash
# 1. Clone the repository
git clone <your-repo-url>
cd my-recommender

# 2. Run the complete setup (installs everything)
./setup.sh

# 3. Run tests to verify everything works
./test.sh

# 4. Start the application
cargo run --release
```

## 🔧 Manual Step-by-Step Setup

If you prefer manual control or the quick setup fails:

### Step 1: Install System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y curl build-essential pkg-config libssl-dev postgresql postgresql-contrib
```

**CentOS/RHEL:**
```bash
sudo yum update
sudo yum groupinstall -y "Development Tools"
sudo yum install -y curl openssl-devel postgresql postgresql-server postgresql-contrib
sudo postgresql-setup initdb
```

**macOS:**
```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install postgresql rust
```

### Step 2: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup update
```

### Step 3: Setup Database

```bash
# Run the database setup script
./setup-database.sh
```

Or manually:

```bash
# Start PostgreSQL service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database user (using your username)
sudo -u postgres createuser -s $(whoami)

# Create database
createdb real_estate_db

# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features rustls,postgres

# Run migrations
sqlx migrate run
```

### Step 4: Build and Test

```bash
# Build the application
cargo build --release

# Run tests
cargo test

# Start the server
cargo run --release
```

## 🗄️ Database-Only Setup

If you only need to replicate the database:

```bash
./setup-database.sh
```

This script will:
- Install PostgreSQL if needed
- Create the database user
- Create the database
- Run all migrations
- Insert sample data
- Verify the setup

## 🌐 Network Setup (Multi-Machine)

To access the application from other machines:

1. **Configure the server to bind to all interfaces:**
   ```bash
   # Edit .env file
   SERVER_HOST=0.0.0.0
   SERVER_PORT=8080
   ```

2. **Open firewall port (if needed):**
   ```bash
   # Ubuntu/Debian
   sudo ufw allow 8080
   
   # CentOS/RHEL
   sudo firewall-cmd --permanent --add-port=8080/tcp
   sudo firewall-cmd --reload
   ```

3. **Access from other machines:**
   ```
   http://YOUR_SERVER_IP:8080/health
   ```

## 📦 Docker Deployment (Alternative)

For containerized deployment:

```bash
# Build Docker image
docker build -t real-estate-recommender .

# Run with Docker Compose
docker-compose up -d
```

## 🔍 Verification Steps

After setup, verify everything works:

```bash
# 1. Check database
psql real_estate_db -c "SELECT COUNT(*) FROM properties;"
psql real_estate_db -c "SELECT COUNT(*) FROM contacts;"

# 2. Check application
cargo run --release &
sleep 5

# 3. Test API endpoints
curl http://localhost:8080/health
curl http://localhost:8080/api/recommendations?contact_id=1

# 4. Run example scripts
./examples.sh
```

## 🛠️ Troubleshooting

### Common Issues and Solutions

**1. PostgreSQL Connection Failed**
```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Check if database exists
psql -l | grep real_estate_db

# Check connection
psql real_estate_db -c "SELECT 1;"
```

**2. Rust Compilation Errors**
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

**3. Permission Denied Errors**
```bash
# Ensure scripts are executable
chmod +x *.sh

# Check database permissions
sudo -u postgres psql -c "\du"
```

**4. Port Already in Use**
```bash
# Check what's using port 8080
sudo netstat -tlnp | grep 8080

# Kill process if needed
sudo kill -9 <PID>
```

**5. Migration Errors**
```bash
# Reset database
dropdb real_estate_db
createdb real_estate_db
sqlx migrate run
```

## 📊 Performance Tuning

For production deployment:

**1. PostgreSQL Configuration:**
```bash
# Edit postgresql.conf
sudo nano /etc/postgresql/*/main/postgresql.conf

# Recommended settings:
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 4MB
maintenance_work_mem = 64MB
```

**2. Application Configuration:**
```bash
# Edit .env for production
RUST_LOG=info
CACHE_TTL_SECONDS=7200
CACHE_MAX_CAPACITY=50000
```

**3. System Limits:**
```bash
# Increase file descriptor limits
echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf
```

## 🔐 Security Configuration

For production deployment:

**1. Database Security:**
```bash
# Create dedicated database user
sudo -u postgres psql -c "CREATE USER realestate WITH PASSWORD 'secure_password';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE real_estate_db TO realestate;"

# Update DATABASE_URL in .env
DATABASE_URL=postgresql://realestate:secure_password@localhost/real_estate_db
```

**2. Application Security:**
```bash
# Generate secure API key
API_KEY=$(openssl rand -hex 32)
echo "API_KEY=$API_KEY" >> .env
```

**3. Firewall Configuration:**
```bash
# Only allow necessary ports
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 8080
sudo ufw enable
```

## 📱 Client Access

Once the system is running, clients can access:

**Web Interface:**
- Health Check: `http://your-server:8080/health`
- API Documentation: See README.md for complete API reference

**API Examples:**
```bash
# Get recommendations
curl "http://your-server:8080/api/recommendations?contact_id=1&max_results=5"

# Compare properties
curl -X POST "http://your-server:8080/api/compare" \
  -H "Content-Type: application/json" \
  -d '{"property_ids": [1, 2, 3]}'

# Generate PDF report
curl "http://your-server:8080/api/pdf/recommendation/1" --output recommendation.pdf
```

## 🆘 Support

If you encounter issues:

1. Check the logs: `tail -f logs/application.log`
2. Verify database connectivity: `psql real_estate_db`
3. Check service status: `systemctl status postgresql`
4. Review the troubleshooting section in DEPLOYMENT.md
5. Run diagnostics: `./test.sh --verbose`

## 📚 Additional Resources

- **README.md**: Complete API documentation and features
- **DEVELOPMENT.md**: Developer guide and technical details
- **DEPLOYMENT.md**: Production deployment guide
- **examples.sh**: API usage examples

---

*For production deployment with high availability, load balancing, and monitoring, refer to the DEPLOYMENT.md file.*
