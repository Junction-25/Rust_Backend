# 🚀 MY-RECOMMENDER SCRIPTS QUICK REFERENCE

## **One-Line Usage Guide**

### **🏗️ Setup (Run Once)**
```bash
./setup.sh                    # Complete system setup
./setup-database.sh          # Database only
./start.sh                    # Start server
```

### **🧪 Testing (Development)**
```bash
./quick_route_test.sh         # Fast validation (15 tests, ~30s)
./test_comprehensive.sh       # Full suite (80+ tests, ~5min)
```

### **📊 Performance (Optimization)**
```bash
./run_latency_test.sh         # Response time analysis
./run_scalability_test.sh     # Stress testing
```

### **📖 Exploration**
```bash
./examples.sh                 # Interactive API demos
./scripts_manager.sh          # Interactive menu system
```

## **🎯 Common Workflows**

### **First Time Setup:**
```bash
./scripts_manager.sh          # Choose option 9: "Run All Setup"
```

### **Daily Development:**
```bash
./quick_route_test.sh         # Quick health check
# Make changes
./quick_route_test.sh         # Validate changes
```

### **Pre-Deployment:**
```bash
./test_comprehensive.sh       # Full validation
./run_latency_test.sh         # Performance check
```

### **Performance Tuning:**
```bash
./scripts_manager.sh          # Choose option 11: "Performance Suite"
```

## **⚡ Quick Commands**

| Need | Command | Time |
|------|---------|------|
| **Start system** | `./start.sh` | 10s |
| **Quick test** | `./quick_route_test.sh` | 30s |
| **Full test** | `./test_comprehensive.sh` | 5min |
| **See examples** | `./examples.sh` | Interactive |
| **Menu system** | `./scripts_manager.sh` | Interactive |

*All scripts are executable and self-documenting*
