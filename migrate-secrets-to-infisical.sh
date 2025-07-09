#!/bin/bash
# THE OVERMIND PROTOCOL - OPERACJA 'VAULT'
# Secure migration from .env to Infisical
# Project: 73c2f3cb-c922-4a46-a333-7b96fbc6301a

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${PURPLE}🔐 THE OVERMIND PROTOCOL - OPERACJA 'VAULT'${NC}"
echo -e "${PURPLE}================================================${NC}"
echo -e "${BLUE}🎯 Mission: Secure migration from .env to Infisical${NC}"
echo -e "${BLUE}📋 Project: 73c2f3cb-c922-4a46-a333-7b96fbc6301a${NC}"
echo ""

# Check prerequisites
echo -e "${BLUE}🔍 Checking prerequisites...${NC}"

if ! command -v infisical &> /dev/null; then
    echo -e "${RED}❌ Infisical CLI not found. Run ./infisical-setup.sh first${NC}"
    exit 1
fi

if ! infisical user &> /dev/null; then
    echo -e "${RED}❌ Not authenticated with Infisical. Run: infisical login${NC}"
    exit 1
fi

if [ ! -f ".env" ]; then
    echo -e "${RED}❌ .env file not found${NC}"
    exit 1
fi

echo -e "${GREEN}✅ All prerequisites met!${NC}"
echo ""

# Function to add secret to Infisical
add_secret() {
    local key=$1
    local value=$2
    local env=$3
    
    if [ -n "$value" ] && [ "$value" != "your_*_here" ]; then
        echo -e "${BLUE}📋 Adding $key to $env environment...${NC}"
        echo "$value" | infisical secrets set "$key" --env="$env" --stdin
        echo -e "${GREEN}✅ Added $key${NC}"
    else
        echo -e "${YELLOW}⚠️  Skipping $key (empty or placeholder value)${NC}"
    fi
}

# Read .env file and extract secrets
echo -e "${BLUE}📖 Reading secrets from .env file...${NC}"

# Create temporary file for processing
temp_file=$(mktemp)
grep -E '^[A-Z_]+=.*' .env | grep -v '^#' > "$temp_file" || true

# API Keys
echo -e "${PURPLE}🔑 Migrating API Keys...${NC}"
while IFS='=' read -r key value; do
    case $key in
        OPENAI_API_KEY|DEEPSEEK_API_KEY|JINA_API_KEY|HELIUS_API_KEY|QUICKNODE_API_KEY|ANTHROPIC_API_KEY)
            # Remove quotes if present
            value=$(echo "$value" | sed 's/^"//;s/"$//')
            add_secret "$key" "$value" "dev"
            add_secret "$key" "$value" "staging"
            add_secret "$key" "$value" "prod"
            ;;
    esac
done < "$temp_file"

# Wallet Secrets
echo -e "${PURPLE}💰 Migrating Wallet Secrets...${NC}"
while IFS='=' read -r key value; do
    case $key in
        WALLET_PRIVATE_KEY|WALLET_ADDRESS|SNIPER_WALLET_*|OVERMIND_WALLET_*)
            value=$(echo "$value" | sed 's/^"//;s/"$//')
            add_secret "$key" "$value" "dev"
            add_secret "$key" "$value" "staging"
            add_secret "$key" "$value" "prod"
            ;;
    esac
done < "$temp_file"

# Database Secrets
echo -e "${PURPLE}🗄️  Migrating Database Secrets...${NC}"
while IFS='=' read -r key value; do
    case $key in
        DATABASE_URL|REDIS_URL|POSTGRES_*|MONGODB_*)
            value=$(echo "$value" | sed 's/^"//;s/"$//')
            add_secret "$key" "$value" "dev"
            add_secret "$key" "$value" "staging"
            add_secret "$key" "$value" "prod"
            ;;
    esac
done < "$temp_file"

# RPC URLs
echo -e "${PURPLE}🌐 Migrating RPC URLs...${NC}"
while IFS='=' read -r key value; do
    case $key in
        *_RPC_URL|*_WS_URL|SOLANA_*)
            value=$(echo "$value" | sed 's/^"//;s/"$//')
            add_secret "$key" "$value" "dev"
            add_secret "$key" "$value" "staging"
            add_secret "$key" "$value" "prod"
            ;;
    esac
done < "$temp_file"

# Trading Configuration
echo -e "${PURPLE}📊 Migrating Trading Configuration...${NC}"
while IFS='=' read -r key value; do
    case $key in
        SNIPER_*|OVERMIND_*|TRADING_*|MAX_*|MIN_*)
            value=$(echo "$value" | sed 's/^"//;s/"$//')
            add_secret "$key" "$value" "dev"
            add_secret "$key" "$value" "staging"
            add_secret "$key" "$value" "prod"
            ;;
    esac
done < "$temp_file"

# Cleanup
rm "$temp_file"

echo ""
echo -e "${GREEN}🎉 Migration completed successfully!${NC}"
echo ""
echo -e "${BLUE}📋 Next steps:${NC}"
echo -e "${BLUE}1. Verify secrets in Infisical dashboard${NC}"
echo -e "${BLUE}2. Test with: infisical run --env=dev -- cargo check${NC}"
echo -e "${BLUE}3. Backup .env: cp .env .env.backup.$(date +%Y%m%d_%H%M%S)${NC}"
echo -e "${BLUE}4. Remove .env: rm .env${NC}"
echo -e "${BLUE}5. Start secure: ./start-overmind-secure.sh${NC}"
echo ""
echo -e "${PURPLE}🔐 THE OVERMIND PROTOCOL is now VAULT-SECURED!${NC}"
