# 🚀 DEPLOYMENT GUIDE

## 📋 **Production Deployment Guide**

This guide covers all aspects of deploying MY-RECOMMENDER system to production environments.

## 🎯 **Pre-Deployment Checklist**

### **System Requirements**
- **OS**: Linux (Ubuntu 20.04+, CentOS 8+, RHEL 8+)
- **CPU**: 4+ cores recommended (8+ for high load)
- **RAM**: 8GB minimum (16GB+ recommended)
- **Storage**: 50GB+ SSD storage
- **Network**: 1Gbps+ network connection

### **Dependencies**
- **Rust**: 1.70+ (install via rustup)
- **PostgreSQL**: 14+ 
- **System Libraries**: build-essential, pkg-config, libssl-dev

### **Port Requirements**
- **8080**: HTTP API (configurable)
- **5432**: PostgreSQL (can be external)
- **443/80**: HTTPS/HTTP (with reverse proxy)

---

## 🛠️ **Installation Methods**

### **Method 1: Direct Binary Deployment**

#### **Step 1: System Setup**
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install system dependencies
sudo apt install -y build-essential pkg-config libssl-dev curl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install PostgreSQL
sudo apt install -y postgresql postgresql-contrib
```

#### **Step 2: Database Setup**
```bash
# Start PostgreSQL
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database and user
sudo -u postgres psql -c "CREATE DATABASE my_recommender_db;"
sudo -u postgres psql -c "CREATE USER recommender_user WITH PASSWORD 'secure_password';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE my_recommender_db TO recommender_user;"
```

#### **Step 3: Application Deployment**
```bash
# Clone repository
git clone <repository-url>
cd my-recommender

# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features rustls,postgres

# Setup environment
cp .env.example .env
# Edit .env with production values

# Run migrations
sqlx migrate run

# Build release
cargo build --release

# Copy binary to system location
sudo cp target/release/my-recommender /usr/local/bin/

# Create system user
sudo useradd -r -s /bin/false recommender

# Create directories
sudo mkdir -p /var/log/my-recommender
sudo mkdir -p /var/lib/my-recommender
sudo chown -R recommender:recommender /var/log/my-recommender /var/lib/my-recommender
```

#### **Step 4: Systemd Service**
```bash
# Create systemd service
sudo tee /etc/systemd/system/my-recommender.service > /dev/null <<EOF
[Unit]
Description=MY-RECOMMENDER Real Estate Platform
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=recommender
Group=recommender
WorkingDirectory=/var/lib/my-recommender
ExecStart=/usr/local/bin/my-recommender
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=DATABASE_URL=postgresql://recommender_user:secure_password@localhost/my_recommender_db
Environment=SERVER_HOST=0.0.0.0
Environment=SERVER_PORT=8080
StandardOutput=journal
StandardError=journal
SyslogIdentifier=my-recommender

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable my-recommender
sudo systemctl start my-recommender

# Check status
sudo systemctl status my-recommender
```

### **Method 2: Docker Deployment**

#### **Step 1: Create Dockerfile**
```dockerfile
# Build stage
FROM rust:1.70-slim AS builder

WORKDIR /app
COPY . .

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install SQLx CLI
RUN cargo install sqlx-cli --no-default-features --features rustls,postgres

# Build application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Copy application binary
COPY --from=builder /app/target/release/my-recommender /usr/local/bin/my-recommender
COPY --from=builder /app/migrations /app/migrations

# Set ownership
RUN chown appuser:appuser /usr/local/bin/my-recommender

USER appuser

EXPOSE 8080

CMD ["/usr/local/bin/my-recommender"]
```

#### **Step 2: Docker Compose Setup**
```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: my_recommender_db
      POSTGRES_USER: recommender_user
      POSTGRES_PASSWORD: secure_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    restart: always

  my-recommender:
    build: .
    environment:
      DATABASE_URL: postgresql://recommender_user:secure_password@postgres:5432/my_recommender_db
      SERVER_HOST: 0.0.0.0
      SERVER_PORT: 8080
      RUST_LOG: info
    ports:
      - "8080:8080"
    depends_on:
      - postgres
    restart: always

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - my-recommender
    restart: always

volumes:
  postgres_data:
```

#### **Step 3: Deploy with Docker**
```bash
# Build and start services
docker-compose up -d

# Check logs
docker-compose logs -f my-recommender

# Run database migrations
docker-compose exec my-recommender sqlx migrate run
```

### **Method 3: Kubernetes Deployment**

#### **Step 1: Create Kubernetes Manifests**
```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: my-recommender

---
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: my-recommender-config
  namespace: my-recommender
data:
  SERVER_HOST: "0.0.0.0"
  SERVER_PORT: "8080"
  RUST_LOG: "info"

---
# k8s/secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: my-recommender-secret
  namespace: my-recommender
type: Opaque
data:
  DATABASE_URL: cG9zdGdyZXNxbDovL3VzZXI6cGFzc0Bwb3N0Z3Jlcy9kYg== # base64 encoded

---
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-recommender
  namespace: my-recommender
spec:
  replicas: 3
  selector:
    matchLabels:
      app: my-recommender
  template:
    metadata:
      labels:
        app: my-recommender
    spec:
      containers:
      - name: my-recommender
        image: my-recommender:latest
        ports:
        - containerPort: 8080
        envFrom:
        - configMapRef:
            name: my-recommender-config
        - secretRef:
            name: my-recommender-secret
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5

---
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: my-recommender-service
  namespace: my-recommender
spec:
  selector:
    app: my-recommender
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
```

#### **Step 2: Deploy to Kubernetes**
```bash
# Apply all manifests
kubectl apply -f k8s/

# Check deployment status
kubectl get all -n my-recommender

# Check logs
kubectl logs -f deployment/my-recommender -n my-recommender
```

---

## 🔧 **Environment Configuration**

### **Production Environment Variables**
```bash
# Database Configuration
DATABASE_URL=postgresql://user:password@host:port/database
DATABASE_MAX_CONNECTIONS=32
DATABASE_MIN_CONNECTIONS=5
DATABASE_TIMEOUT_SECONDS=30

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
SERVER_WORKERS=4

# Security
CORS_ALLOWED_ORIGINS=https://yourdomain.com,https://www.yourdomain.com
API_KEY_REQUIRED=true
JWT_SECRET=your-jwt-secret-key

# Performance
CACHE_TTL_SECONDS=3600
CACHE_MAX_CAPACITY=10000
REQUEST_TIMEOUT_SECONDS=30

# AI/ML Configuration
AI_MODEL_UPDATE_INTERVAL=300
ML_CONFIDENCE_THRESHOLD=0.7
ENABLE_MODEL_TRAINING=true

# Real-time Features
WEBSOCKET_MAX_CONNECTIONS=1000
NOTIFICATION_BATCH_SIZE=50
HEARTBEAT_INTERVAL_SECONDS=30

# Logging
RUST_LOG=info
LOG_FORMAT=json
LOG_FILE=/var/log/my-recommender/app.log

# Monitoring
ENABLE_METRICS=true
METRICS_PORT=9090
HEALTH_CHECK_INTERVAL=60
```

### **Database Configuration**
```sql
-- Create production database with optimizations
CREATE DATABASE my_recommender_db
  WITH 
  ENCODING = 'UTF8'
  LC_COLLATE = 'en_US.UTF-8'
  LC_CTYPE = 'en_US.UTF-8'
  TEMPLATE = template0;

-- Create indexes for performance
CREATE INDEX CONCURRENTLY idx_properties_location ON properties USING GIST (ST_Point(lon, lat));
CREATE INDEX CONCURRENTLY idx_properties_price ON properties (price);
CREATE INDEX CONCURRENTLY idx_properties_type ON properties (property_type);
CREATE INDEX CONCURRENTLY idx_contacts_budget ON contacts (min_budget, max_budget);
CREATE INDEX CONCURRENTLY idx_contact_preferences_location ON contact_preferences (contact_id, lat, lon);

-- Configure PostgreSQL for performance
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '2GB';
ALTER SYSTEM SET random_page_cost = 1.1;
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
SELECT pg_reload_conf();
```

---

## 🔒 **Security Configuration**

### **Firewall Setup (UFW)**
```bash
# Enable UFW
sudo ufw enable

# Allow SSH
sudo ufw allow ssh

# Allow HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Allow application port (if directly exposed)
sudo ufw allow 8080/tcp

# Deny all other incoming
sudo ufw default deny incoming
sudo ufw default allow outgoing
```

### **SSL/TLS Configuration (Nginx)**
```nginx
# /etc/nginx/sites-available/my-recommender
server {
    listen 80;
    server_name yourdomain.com www.yourdomain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name yourdomain.com www.yourdomain.com;

    ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;
    
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket proxy
    location /ws {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

---

## 📊 **Monitoring & Observability**

### **Health Checks**
```bash
# Basic health check
curl -f http://localhost:8080/health || exit 1

# Comprehensive health check script
#!/bin/bash
# health-check.sh

# Check main API
if ! curl -f -s http://localhost:8080/health > /dev/null; then
    echo "API health check failed"
    exit 1
fi

# Check AI models
if ! curl -f -s http://localhost:8080/ai/models/stats > /dev/null; then
    echo "AI models health check failed"
    exit 1
fi

# Check real-time service
if ! curl -f -s http://localhost:8080/realtime/health > /dev/null; then
    echo "Real-time service health check failed"
    exit 1
fi

echo "All health checks passed"
```

### **Logging Configuration**
```bash
# Logrotate configuration
sudo tee /etc/logrotate.d/my-recommender > /dev/null <<EOF
/var/log/my-recommender/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    create 0644 recommender recommender
    postrotate
        systemctl reload my-recommender
    endscript
}
EOF
```

### **Monitoring with Prometheus**
```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'my-recommender'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: /metrics
    scrape_interval: 30s
```

---

## 🔄 **Backup & Recovery**

### **Database Backup**
```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/var/backups/my-recommender"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/db_backup_$DATE.sql"

# Create backup directory
mkdir -p $BACKUP_DIR

# Perform backup
pg_dump -h localhost -U recommender_user my_recommender_db > $BACKUP_FILE

# Compress backup
gzip $BACKUP_FILE

# Remove old backups (keep 30 days)
find $BACKUP_DIR -name "*.sql.gz" -mtime +30 -delete

echo "Backup completed: $BACKUP_FILE.gz"
```

### **Application State Backup**
```bash
# Backup application data
tar -czf /var/backups/my-recommender/app_backup_$(date +%Y%m%d).tar.gz \
  /var/lib/my-recommender \
  /etc/systemd/system/my-recommender.service \
  /var/log/my-recommender
```

---

## 📈 **Performance Optimization**

### **System Tuning**
```bash
# System limits
echo "* soft nofile 65536" >> /etc/security/limits.conf
echo "* hard nofile 65536" >> /etc/security/limits.conf

# TCP tuning
echo "net.core.somaxconn = 4096" >> /etc/sysctl.conf
echo "net.ipv4.tcp_max_syn_backlog = 4096" >> /etc/sysctl.conf
sysctl -p
```

### **Database Optimization**
```sql
-- Analyze and vacuum regularly
ANALYZE;
VACUUM (ANALYZE);

-- Monitor query performance
SELECT query, mean_time, calls FROM pg_stat_statements ORDER BY mean_time DESC LIMIT 10;
```

---

## 🔧 **Troubleshooting**

### **Common Issues**

#### **Service Won't Start**
```bash
# Check service status
sudo systemctl status my-recommender

# Check logs
sudo journalctl -u my-recommender -f

# Check configuration
my-recommender --check-config
```

#### **Database Connection Issues**
```bash
# Test database connection
psql -h localhost -U recommender_user -d my_recommender_db -c "SELECT version();"

# Check PostgreSQL status
sudo systemctl status postgresql
```

#### **High Memory Usage**
```bash
# Monitor memory usage
sudo systemctl show my-recommender --property=MemoryCurrent

# Adjust cache settings
export CACHE_MAX_CAPACITY=5000
sudo systemctl restart my-recommender
```

### **Performance Issues**
```bash
# Check system resources
htop

# Monitor database performance
sudo -u postgres psql -c "SELECT * FROM pg_stat_activity WHERE state = 'active';"

# Check application metrics
curl http://localhost:9090/metrics
```

---

## 🎯 **Post-Deployment Checklist**

- [ ] Application starts successfully
- [ ] Health checks pass
- [ ] Database connection works
- [ ] All API endpoints respond
- [ ] WebSocket connections work
- [ ] SSL certificate valid
- [ ] Monitoring configured
- [ ] Backups scheduled
- [ ] Firewall configured
- [ ] Log rotation setup

---

## 📞 **Support & Maintenance**

### **Maintenance Schedule**
- **Daily**: Automated backups
- **Weekly**: Log rotation and cleanup
- **Monthly**: Security updates
- **Quarterly**: Full system review

### **Update Process**
```bash
# 1. Backup current version
./backup.sh

# 2. Download new version
git pull origin main

# 3. Build new version
cargo build --release

# 4. Stop service
sudo systemctl stop my-recommender

# 5. Deploy new binary
sudo cp target/release/my-recommender /usr/local/bin/

# 6. Run migrations (if any)
sqlx migrate run

# 7. Start service
sudo systemctl start my-recommender

# 8. Verify deployment
curl http://localhost:8080/health
```

---

*Last Updated: July 19, 2025*  
*Version: 1.0.0*
