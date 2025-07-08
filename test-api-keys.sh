#!/bin/bash
# THE OVERMIND PROTOCOL - API Keys Validation Script
# Test all configured API keys for connectivity and functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${PURPLE}🔑 THE OVERMIND PROTOCOL - API Keys Validation${NC}"
echo -e "${PURPLE}==============================================${NC}"
echo ""

# Load environment variables
if [ -f ".env" ]; then
    source .env
else
    echo -e "${RED}❌ .env file not found!${NC}"
    exit 1
fi

# Test results
tests_passed=0
tests_total=0

test_api() {
    local service="$1"
    local test_command="$2"
    local critical="$3"
    
    tests_total=$((tests_total + 1))
    echo -n -e "${BLUE}🔍 Testing $service API... ${NC}"
    
    if eval "$test_command" &>/dev/null; then
        echo -e "${GREEN}✅ PASS${NC}"
        tests_passed=$((tests_passed + 1))
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
# HELIUS API TESTING
# =============================================================================
echo -e "${CYAN}🌐 Testing Helius API${NC}"
echo -e "${CYAN}===================${NC}"

if [ ! -z "$HELIUS_API_KEY" ] && [ "$HELIUS_API_KEY" != "your_helius_api_key_here" ]; then
    test_api "Helius RPC" "curl -s -f \"$HELIUS_MAINNET_RPC_URL\" -X POST -H 'Content-Type: application/json' -d '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getHealth\"}'" true
    
    test_api "Helius Enhanced APIs" "curl -s -f \"https://api.helius.xyz/v0/addresses/So11111111111111111111111111111111111111112/balances?api-key=$HELIUS_API_KEY\"" false
else
    echo -e "${YELLOW}⚠️  Helius API key not configured${NC}"
fi

echo ""

# =============================================================================
# DEEPSEEK API TESTING
# =============================================================================
echo -e "${CYAN}🧠 Testing DeepSeek V2 API${NC}"
echo -e "${CYAN}=========================${NC}"

if [ ! -z "$DEEPSEEK_API_KEY" ] && [ "$DEEPSEEK_API_KEY" != "your_deepseek_api_key_here" ]; then
    test_api "DeepSeek Chat API" "curl -s -f https://api.deepseek.com/v1/chat/completions -H 'Content-Type: application/json' -H \"Authorization: Bearer $DEEPSEEK_API_KEY\" -d '{\"model\":\"deepseek-chat\",\"messages\":[{\"role\":\"user\",\"content\":\"test\"}],\"max_tokens\":1}'" true
else
    echo -e "${YELLOW}⚠️  DeepSeek API key not configured${NC}"
fi

echo ""

# =============================================================================
# JINA AI API TESTING
# =============================================================================
echo -e "${CYAN}🔍 Testing Jina AI API${NC}"
echo -e "${CYAN}=====================${NC}"

if [ ! -z "$JINA_API_KEY" ] && [ "$JINA_API_KEY" != "your_jina_api_key_here" ]; then
    test_api "Jina Embeddings API" "curl -s -f https://api.jina.ai/v1/embeddings -H 'Content-Type: application/json' -H \"Authorization: Bearer $JINA_API_KEY\" -d '{\"input\":[\"test\"],\"model\":\"jina-embeddings-v2-base-en\"}'" true
    
    test_api "Jina Reranker API" "curl -s -f https://api.jina.ai/v1/rerank -H 'Content-Type: application/json' -H \"Authorization: Bearer $JINA_API_KEY\" -d '{\"model\":\"jina-reranker-v1-base-en\",\"query\":\"test\",\"documents\":[\"test doc\"]}'" false
else
    echo -e "${YELLOW}⚠️  Jina AI API key not configured${NC}"
fi

echo ""

# =============================================================================
# SOLANA NETWORK TESTING
# =============================================================================
echo -e "${CYAN}⚡ Testing Solana Network${NC}"
echo -e "${CYAN}========================${NC}"

test_api "Solana Mainnet RPC" "curl -s -f https://api.mainnet-beta.solana.com -X POST -H 'Content-Type: application/json' -d '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getHealth\"}'" true

test_api "QuickNode RPC" "curl -s -f \"$QUICKNODE_MAINNET_RPC_URL\" -X POST -H 'Content-Type: application/json' -d '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getHealth\"}'" false

echo ""

# =============================================================================
# JITO V2 API TESTING (if configured)
# =============================================================================
echo -e "${CYAN}⚡ Testing Jito v2 API${NC}"
echo -e "${CYAN}=====================${NC}"

if [ ! -z "$JITO_API_KEY" ] && [ "$JITO_API_KEY" != "your_jito_v2_api_key_here" ]; then
    test_api "Jito Block Engine" "curl -s -f \"$JITO_VALIDATOR_URL\" -H \"Authorization: Bearer $JITO_API_KEY\"" true
else
    echo -e "${YELLOW}⚠️  Jito v2 API key not configured - using direct RPC mode${NC}"
    echo -e "${YELLOW}   Apply for Jito API key at: https://www.jito.wtf/${NC}"
fi

echo ""

# =============================================================================
# WALLET VALIDATION
# =============================================================================
echo -e "${CYAN}💰 Testing Wallet Configuration${NC}"
echo -e "${CYAN}===============================${NC}"

if [ ! -z "$SNIPER_WALLET_PRIVATE_KEY" ] && [ "$SNIPER_WALLET_PRIVATE_KEY" != "YOUR_PRIVATE_KEY_HERE" ]; then
    echo -e "${GREEN}✅ Wallet private key configured${NC}"
    tests_passed=$((tests_passed + 1))
else
    echo -e "${RED}❌ Wallet private key not configured${NC}"
fi
tests_total=$((tests_total + 1))

# Check wallet file exists
if [ -f "wallets/mainnet-trading-wallet.json" ]; then
    echo -e "${GREEN}✅ Mainnet wallet file exists${NC}"
    tests_passed=$((tests_passed + 1))
else
    echo -e "${RED}❌ Mainnet wallet file missing${NC}"
fi
tests_total=$((tests_total + 1))

echo ""

# =============================================================================
# INTERNET CONNECTIVITY
# =============================================================================
echo -e "${CYAN}🌐 Testing Internet Connectivity${NC}"
echo -e "${CYAN}================================${NC}"

test_api "Google DNS" "ping -c 1 8.8.8.8" true
test_api "Cloudflare DNS" "ping -c 1 1.1.1.1" false

echo ""

# =============================================================================
# VALIDATION SUMMARY
# =============================================================================
echo -e "${PURPLE}📊 API KEYS VALIDATION SUMMARY${NC}"
echo -e "${PURPLE}==============================${NC}"

validation_percentage=$((tests_passed * 100 / tests_total))

echo -e "${CYAN}Total Tests: $tests_total${NC}"
echo -e "${GREEN}Passed: $tests_passed${NC}"
echo -e "${RED}Failed: $((tests_total - tests_passed))${NC}"
echo -e "${CYAN}Success Rate: $validation_percentage%${NC}"

echo ""

if [ $validation_percentage -ge 80 ]; then
    echo -e "${GREEN}🎉 API KEYS VALIDATION PASSED!${NC}"
    echo -e "${GREEN}✅ THE OVERMIND PROTOCOL is ready for deployment${NC}"
    
    echo ""
    echo -e "${CYAN}📋 Configured Services:${NC}"
    [ ! -z "$HELIUS_API_KEY" ] && [ "$HELIUS_API_KEY" != "your_helius_api_key_here" ] && echo -e "${GREEN}✅ Helius API${NC}"
    [ ! -z "$DEEPSEEK_API_KEY" ] && [ "$DEEPSEEK_API_KEY" != "your_deepseek_api_key_here" ] && echo -e "${GREEN}✅ DeepSeek V2 API${NC}"
    [ ! -z "$JINA_API_KEY" ] && [ "$JINA_API_KEY" != "your_jina_api_key_here" ] && echo -e "${GREEN}✅ Jina AI API${NC}"
    [ ! -z "$JITO_API_KEY" ] && [ "$JITO_API_KEY" != "your_jito_v2_api_key_here" ] && echo -e "${GREEN}✅ Jito v2 API${NC}" || echo -e "${YELLOW}⚠️  Jito v2 API (Direct RPC mode)${NC}"
    [ ! -z "$SNIPER_WALLET_PRIVATE_KEY" ] && [ "$SNIPER_WALLET_PRIVATE_KEY" != "YOUR_PRIVATE_KEY_HERE" ] && echo -e "${GREEN}✅ Trading Wallet${NC}"
    
    echo ""
    echo -e "${PURPLE}🚀 Ready for: ./validate-production.sh${NC}"
    exit 0
elif [ $validation_percentage -ge 60 ]; then
    echo -e "${YELLOW}⚠️  API KEYS VALIDATION PASSED WITH WARNINGS${NC}"
    echo -e "${YELLOW}Some services failed. Review and fix if needed.${NC}"
    echo ""
    echo -e "${YELLOW}You can proceed with caution.${NC}"
    exit 0
else
    echo -e "${RED}❌ API KEYS VALIDATION FAILED${NC}"
    echo -e "${RED}Critical services are not working. Please fix before deployment.${NC}"
    echo ""
    echo -e "${RED}🛑 DO NOT DEPLOY TO PRODUCTION${NC}"
    exit 1
fi
