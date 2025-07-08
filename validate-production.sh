#!/bin/bash
# THE OVERMIND PROTOCOL - Production Validation Script
# Final validation before live trading deployment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${PURPLE}🔍 THE OVERMIND PROTOCOL - Production Validation${NC}"
echo -e "${PURPLE}===============================================${NC}"
echo ""

# =============================================================================
# VALIDATION CHECKLIST
# =============================================================================
validation_passed=0
validation_total=0

check_item() {
    local description="$1"
    local command="$2"
    local critical="$3"
    
    validation_total=$((validation_total + 1))
    echo -n -e "${BLUE}🔍 Checking: $description... ${NC}"
    
    if eval "$command" &>/dev/null; then
        echo -e "${GREEN}✅ PASS${NC}"
        validation_passed=$((validation_passed + 1))
        return 0
    else
        if [ "$critical" = "true" ]; then
            echo -e "${RED}❌ FAIL (CRITICAL)${NC}"
        else
            echo -e "${YELLOW}⚠️  FAIL (WARNING)${NC}"
        fi
        return 1
    fi
}

# =============================================================================
# ENVIRONMENT VALIDATION
# =============================================================================
echo -e "${CYAN}📋 Environment Validation${NC}"
echo -e "${CYAN}=========================${NC}"

# Check .env file exists
check_item "Environment file exists" "[ -f .env ]" true

# Load environment variables
if [ -f ".env" ]; then
    source .env
fi

# Check critical API keys
check_item "Helius API key configured" "[ ! -z \"$HELIUS_API_KEY\" ] && [ \"$HELIUS_API_KEY\" != \"your_helius_api_key_here\" ]" true
check_item "Jito API key configured" "[ ! -z \"$JITO_API_KEY\" ] && [ \"$JITO_API_KEY\" != \"your_jito_api_key_here\" ]" true
check_item "DeepSeek API key configured" "[ ! -z \"$DEEPSEEK_API_KEY\" ] && [ \"$DEEPSEEK_API_KEY\" != \"your_deepseek_api_key_here\" ]" true
check_item "Jina AI API key configured" "[ ! -z \"$JINA_API_KEY\" ] && [ \"$JINA_API_KEY\" != \"your_jina_api_key_here\" ]" true
check_item "Main wallet private key configured" "[ ! -z \"$MAIN_WALLET_PRIVATE_KEY\" ] && [ \"$MAIN_WALLET_PRIVATE_KEY\" != \"your_main_wallet_private_key_here\" ]" true

# Check trading mode
check_item "Trading mode set to live" "[ \"$SNIPER_TRADING_MODE\" = \"live\" ]" true
check_item "AI mode enabled" "[ \"$OVERMIND_AI_MODE\" = \"enabled\" ]" false

echo ""

# =============================================================================
# SYSTEM REQUIREMENTS
# =============================================================================
echo -e "${CYAN}🖥️  System Requirements${NC}"
echo -e "${CYAN}======================${NC}"

check_item "Docker installed" "command -v docker" true
check_item "Docker Compose installed" "command -v docker-compose" true
check_item "Docker daemon running" "docker info" true
check_item "Rust toolchain available" "command -v cargo" true
check_item "Git available" "command -v git" false

echo ""

# =============================================================================
# CODE VALIDATION
# =============================================================================
echo -e "${CYAN}🦀 Code Validation${NC}"
echo -e "${CYAN}=================${NC}"

check_item "Rust code compiles" "cargo check --profile contabo" true
check_item "Unit tests pass" "cargo test --lib --profile contabo" true
check_item "No critical warnings" "cargo clippy --profile contabo -- -D warnings" false

echo ""

# =============================================================================
# CONFIGURATION VALIDATION
# =============================================================================
echo -e "${CYAN}⚙️  Configuration Validation${NC}"
echo -e "${CYAN}============================${NC}"

check_item "Risk management config exists" "[ -f config/risk-management.yml ]" true
check_item "Monitoring config exists" "[ -f config/monitoring.yml ]" true
check_item "Prometheus config exists" "[ -f config/prometheus.yml ]" true
check_item "AlertManager config exists" "[ -f config/alertmanager.yml ]" true
check_item "Alert rules exist" "[ -f config/alert_rules.yml ]" true

echo ""

# =============================================================================
# SECURITY VALIDATION
# =============================================================================
echo -e "${CYAN}🛡️  Security Validation${NC}"
echo -e "${CYAN}======================${NC}"

check_item "Environment file permissions secure" "[ \$(stat -c %a .env 2>/dev/null || echo 644) = '600' ]" true
check_item "Wallet directory permissions secure" "[ ! -d wallets ] || [ \$(stat -c %a wallets) = '700' ]" true
check_item ".env in .gitignore" "grep -q '\.env' .gitignore" true
check_item "No hardcoded secrets in code" "! grep -r 'sk-\|api_key.*=' src/ --include='*.rs' | grep -v placeholder" true

echo ""

# =============================================================================
# NETWORK CONNECTIVITY
# =============================================================================
echo -e "${CYAN}🌐 Network Connectivity${NC}"
echo -e "${CYAN}======================${NC}"

if [ ! -z "$HELIUS_API_KEY" ] && [ "$HELIUS_API_KEY" != "your_helius_api_key_here" ]; then
    check_item "Helius API connectivity" "curl -s -f \"https://mainnet.helius-rpc.com/?api-key=$HELIUS_API_KEY\" -X POST -H 'Content-Type: application/json' -d '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getHealth\"}'" true
else
    echo -e "${YELLOW}⚠️  Skipping Helius connectivity test (API key not configured)${NC}"
fi

check_item "Solana mainnet connectivity" "curl -s -f https://api.mainnet-beta.solana.com -X POST -H 'Content-Type: application/json' -d '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getHealth\"}'" true
check_item "Internet connectivity" "ping -c 1 8.8.8.8" true

echo ""

# =============================================================================
# INFRASTRUCTURE VALIDATION
# =============================================================================
echo -e "${CYAN}🏗️  Infrastructure Validation${NC}"
echo -e "${CYAN}=============================${NC}"

# Start infrastructure for testing
echo -e "${BLUE}🚀 Starting infrastructure for validation...${NC}"
docker-compose up -d dragonfly chroma prometheus grafana alertmanager node-exporter

# Wait for services
sleep 15

check_item "DragonflyDB health" "docker exec overmind-dragonfly redis-cli ping" true
check_item "Chroma health" "curl -s -f http://localhost:8000/api/v1/heartbeat" true
check_item "Prometheus health" "curl -s -f http://localhost:9090/-/healthy" true
check_item "Grafana health" "curl -s -f http://localhost:3000/api/health" true
check_item "AlertManager health" "curl -s -f http://localhost:9093/-/healthy" true

echo ""

# =============================================================================
# PERFORMANCE VALIDATION
# =============================================================================
echo -e "${CYAN}⚡ Performance Validation${NC}"
echo -e "${CYAN}========================${NC}"

# Run performance tests
check_item "Performance tests pass" "cargo test --profile contabo test_memory_usage_stability test_transaction_processing_latency" false

echo ""

# =============================================================================
# FINAL VALIDATION SUMMARY
# =============================================================================
echo -e "${PURPLE}📊 VALIDATION SUMMARY${NC}"
echo -e "${PURPLE}===================${NC}"

validation_percentage=$((validation_passed * 100 / validation_total))

echo -e "${CYAN}Total Checks: $validation_total${NC}"
echo -e "${GREEN}Passed: $validation_passed${NC}"
echo -e "${RED}Failed: $((validation_total - validation_passed))${NC}"
echo -e "${CYAN}Success Rate: $validation_percentage%${NC}"

echo ""

if [ $validation_percentage -ge 90 ]; then
    echo -e "${GREEN}🎉 VALIDATION PASSED - READY FOR PRODUCTION!${NC}"
    echo -e "${GREEN}✅ THE OVERMIND PROTOCOL is ready for live trading${NC}"
    echo ""
    echo -e "${CYAN}📋 Next Steps:${NC}"
    echo -e "${CYAN}1. Run: ./deploy-production.sh${NC}"
    echo -e "${CYAN}2. Monitor: http://localhost:3000 (Grafana)${NC}"
    echo -e "${CYAN}3. API: http://localhost:8080${NC}"
    echo -e "${CYAN}4. Alerts: http://localhost:9093${NC}"
    echo ""
    echo -e "${PURPLE}⚡ Ready to dominate the MEV space!${NC}"
    exit 0
elif [ $validation_percentage -ge 80 ]; then
    echo -e "${YELLOW}⚠️  VALIDATION PASSED WITH WARNINGS${NC}"
    echo -e "${YELLOW}Some non-critical checks failed. Review and fix if needed.${NC}"
    echo ""
    echo -e "${YELLOW}You can proceed with deployment, but monitor closely.${NC}"
    exit 0
else
    echo -e "${RED}❌ VALIDATION FAILED${NC}"
    echo -e "${RED}Critical issues found. Please fix before deployment.${NC}"
    echo ""
    echo -e "${RED}🛑 DO NOT DEPLOY TO PRODUCTION${NC}"
    exit 1
fi
