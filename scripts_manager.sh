#!/bin/bash

# MY-RECOMMENDER SCRIPT MANAGER
# Interactive script launcher and organizer

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

show_header() {
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                🚀 DAR.AI SCRIPT MANAGER               ║${NC}"
    echo -e "${CYAN}║              Interactive Script Launcher v2.0                ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

show_menu() {
    echo -e "${BLUE}📋 Available Script Categories:${NC}"
    echo ""
    echo -e "${GREEN}🏗️  SETUP & DEPLOYMENT${NC}"
    echo "   1) setup.sh                  - Complete system setup"
    echo "   2) setup-database.sh         - Database-only setup" 
    echo "   3) start.sh                  - Start application server"
    echo ""
    echo -e "${GREEN}🧪 TESTING & VALIDATION${NC}"
    echo "   4) test_comprehensive.sh     - Full system test suite (80+ tests)"
    echo "   5) quick_route_test.sh       - Quick API endpoint validation"
    echo ""
    echo -e "${GREEN}📊 PERFORMANCE & BENCHMARKING${NC}"
    echo "   6) run_latency_test.sh       - API latency benchmarking"
    echo "   7) run_scalability_test.sh   - Scalability stress testing"
    echo ""
    echo -e "${GREEN}📖 EXAMPLES & DEMOS${NC}"
    echo "   8) examples.sh               - Interactive API demonstrations"
    echo ""
    echo -e "${GREEN}🎯 SPECIALIZED OPERATIONS${NC}"
    echo "   9) Run All Setup             - Complete setup sequence"
    echo "  10) Run All Tests             - Complete testing sequence"
    echo "  11) Performance Suite        - All performance tests"
    echo ""
    echo -e "${PURPLE}0) Exit${NC}"
    echo ""
}

run_script() {
    local script_name="$1"
    local description="$2"
    
    if [ -f "$script_name" ]; then
        echo -e "${YELLOW}🚀 Executing: $description${NC}"
        echo -e "${BLUE}Script: $script_name${NC}"
        echo ""
        chmod +x "$script_name"
        ./"$script_name"
        echo ""
        echo -e "${GREEN}✅ Script completed: $description${NC}"
    else
        echo -e "${RED}❌ Script not found: $script_name${NC}"
    fi
}

run_setup_sequence() {
    echo -e "${CYAN}🏗️ Running Complete Setup Sequence${NC}"
    echo ""
    
    echo -e "${YELLOW}Step 1/3: System Setup${NC}"
    run_script "setup.sh" "Complete System Setup"
    
    echo -e "${YELLOW}Step 2/3: Database Setup${NC}" 
    run_script "setup-database.sh" "Database Setup"
    
    echo -e "${YELLOW}Step 3/3: Starting Server${NC}"
    run_script "start.sh" "Application Server"
    
    echo -e "${GREEN}🎉 Complete setup sequence finished!${NC}"
}

run_test_sequence() {
    echo -e "${CYAN}🧪 Running Complete Testing Sequence${NC}"
    echo ""
    
    echo -e "${YELLOW}Step 1/2: Quick Validation${NC}"
    run_script "quick_route_test.sh" "Quick Route Validation"
    
    echo -e "${YELLOW}Step 2/2: Comprehensive Testing${NC}"
    run_script "test_comprehensive.sh" "Full System Test Suite"
    
    echo -e "${GREEN}🎉 Complete testing sequence finished!${NC}"
}

run_performance_suite() {
    echo -e "${CYAN}📊 Running Performance Test Suite${NC}"
    echo ""
    
    echo -e "${YELLOW}Step 1/2: Latency Testing${NC}"
    run_script "run_latency_test.sh" "API Latency Benchmarking"
    
    echo -e "${YELLOW}Step 2/2: Scalability Testing${NC}" 
    run_script "run_scalability_test.sh" "Scalability Stress Testing"
    
    echo -e "${GREEN}🎉 Performance test suite finished!${NC}"
}

main() {
    while true; do
        clear
        show_header
        show_menu
        
        echo -n -e "${CYAN}Enter your choice (0-11): ${NC}"
        read choice
        
        case $choice in
            1)
                run_script "setup.sh" "Complete System Setup"
                ;;
            2) 
                run_script "setup-database.sh" "Database Setup"
                ;;
            3)
                run_script "start.sh" "Application Server"
                ;;
            4)
                run_script "test_comprehensive.sh" "Full System Test Suite"
                ;;
            5)
                run_script "quick_route_test.sh" "Quick Route Validation"
                ;;
            6)
                run_script "run_latency_test.sh" "API Latency Benchmarking"
                ;;
            7)
                run_script "run_scalability_test.sh" "Scalability Stress Testing"
                ;;
            8)
                run_script "examples.sh" "API Examples and Demonstrations"
                ;;
            9)
                run_setup_sequence
                ;;
            10)
                run_test_sequence
                ;;
            11)
                run_performance_suite
                ;;
            0)
                echo -e "${GREEN}👋 Goodbye! Thanks for using MY-RECOMMENDER Script Manager${NC}"
                exit 0
                ;;
            *)
                echo -e "${RED}❌ Invalid choice. Please select 0-11.${NC}"
                ;;
        esac
        
        echo ""
        echo -e "${YELLOW}Press Enter to return to main menu...${NC}"
        read
    done
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Please run this script from the project root directory${NC}"
    exit 1
fi

# Run main menu
main
