#!/bin/bash
# THE OVERMIND PROTOCOL - OPERACJA 'VAULT'
# Safe cleanup of .env files after Infisical migration
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
echo -e "${BLUE}🗂️  Mission: Safe cleanup of .env files${NC}"
echo -e "${BLUE}📋 Project: 73c2f3cb-c922-4a46-a333-7b96fbc6301a${NC}"
echo ""

# Check if Infisical is working
echo -e "${BLUE}🔍 Verifying Infisical integration...${NC}"

if ! command -v infisical &> /dev/null; then
    echo -e "${RED}❌ Infisical CLI not found. Cannot proceed safely.${NC}"
    exit 1
fi

if ! infisical user &> /dev/null; then
    echo -e "${RED}❌ Not authenticated with Infisical. Cannot proceed safely.${NC}"
    exit 1
fi

# Test if we can retrieve secrets
echo -e "${BLUE}🧪 Testing secret retrieval...${NC}"
if ! infisical secrets get OPENAI_API_KEY --env=dev &> /dev/null; then
    echo -e "${RED}❌ Cannot retrieve secrets from Infisical. Migration may not be complete.${NC}"
    echo -e "${YELLOW}⚠️  Please run ./migrate-secrets-to-infisical.sh first${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Infisical integration verified!${NC}"
echo ""

# Create backup directory
backup_dir="env-backups/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$backup_dir"

echo -e "${BLUE}💾 Creating secure backups...${NC}"

# Backup all .env files
for env_file in .env .env.local .env.development .env.production .env.staging; do
    if [ -f "$env_file" ]; then
        echo -e "${BLUE}📋 Backing up $env_file...${NC}"
        cp "$env_file" "$backup_dir/"
        echo -e "${GREEN}✅ Backed up $env_file${NC}"
    fi
done

# Backup any .env.* files
find . -maxdepth 1 -name ".env.*" -type f | while read -r file; do
    if [ -f "$file" ]; then
        echo -e "${BLUE}📋 Backing up $file...${NC}"
        cp "$file" "$backup_dir/"
        echo -e "${GREEN}✅ Backed up $file${NC}"
    fi
done

echo ""
echo -e "${BLUE}🧪 Testing system with Infisical...${NC}"

# Test cargo check with Infisical
if infisical run --env=dev -- cargo check &> /dev/null; then
    echo -e "${GREEN}✅ System works with Infisical!${NC}"
else
    echo -e "${RED}❌ System test failed with Infisical${NC}"
    echo -e "${YELLOW}⚠️  Keeping .env files for safety${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}🗑️  Removing .env files...${NC}"

# Remove .env files
for env_file in .env .env.local .env.development .env.production .env.staging; do
    if [ -f "$env_file" ]; then
        echo -e "${BLUE}🗑️  Removing $env_file...${NC}"
        rm "$env_file"
        echo -e "${GREEN}✅ Removed $env_file${NC}"
    fi
done

# Remove any other .env.* files (except templates)
find . -maxdepth 1 -name ".env.*" -type f ! -name "*.template" ! -name "*.example" | while read -r file; do
    if [ -f "$file" ]; then
        echo -e "${BLUE}🗑️  Removing $file...${NC}"
        rm "$file"
        echo -e "${GREEN}✅ Removed $file${NC}"
    fi
done

echo ""
echo -e "${BLUE}📝 Creating secure templates...${NC}"

# Create .env.template
cat > .env.template << 'EOF'
# THE OVERMIND PROTOCOL - Environment Template
# This file contains only template values for reference
# All actual secrets are managed through Infisical
# Project: 73c2f3cb-c922-4a46-a333-7b96fbc6301a

# =============================================================================
# INFISICAL CONFIGURATION
# =============================================================================
INFISICAL_PROJECT_ID=73c2f3cb-c922-4a46-a333-7b96fbc6301a
INFISICAL_ENVIRONMENT=development
INFISICAL_CLIENT_ID=your_client_id_here
INFISICAL_CLIENT_SECRET=your_client_secret_here

# =============================================================================
# USAGE INSTRUCTIONS
# =============================================================================
# To run THE OVERMIND PROTOCOL with secure secrets:
# 1. Development: infisical run --env=dev -- cargo run
# 2. Staging:     infisical run --env=staging -- cargo run --profile staging
# 3. Production:  infisical run --env=prod -- cargo run --profile contabo
#
# To add new secrets:
# infisical secrets set SECRET_NAME "secret_value" --env=dev
#
# To view secrets:
# infisical secrets get SECRET_NAME --env=dev

# =============================================================================
# TEMPLATE VALUES (DO NOT USE IN PRODUCTION)
# =============================================================================
OPENAI_API_KEY=sk-your_openai_key_here
HELIUS_API_KEY=your_helius_key_here
QUICKNODE_API_KEY=your_quicknode_key_here
JINA_API_KEY=jina_your_jina_key_here
DEEPSEEK_API_KEY=sk-your_deepseek_key_here

SNIPER_WALLET_PRIVATE_KEY=your_wallet_private_key_here
WALLET_ADDRESS=your_wallet_address_here

SNIPER_TRADING_MODE=paper
OVERMIND_AI_MODE=enabled
SNIPER_MAX_POSITION_SIZE=1000
SNIPER_MAX_DAILY_LOSS=500

SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_WS_URL=wss://api.devnet.solana.com
EOF

echo -e "${GREEN}✅ Created .env.template${NC}"

# Update .gitignore to ensure .env files are ignored
if [ -f ".gitignore" ]; then
    if ! grep -q "^\.env$" .gitignore; then
        echo ".env" >> .gitignore
        echo -e "${GREEN}✅ Added .env to .gitignore${NC}"
    fi
    if ! grep -q "^\.env\.local$" .gitignore; then
        echo ".env.local" >> .gitignore
        echo -e "${GREEN}✅ Added .env.local to .gitignore${NC}"
    fi
else
    cat > .gitignore << 'EOF'
# Environment files
.env
.env.local
.env.development
.env.production
.env.staging

# Backup directories
env-backups/
EOF
    echo -e "${GREEN}✅ Created .gitignore${NC}"
fi

echo ""
echo -e "${GREEN}🎉 OPERACJA 'VAULT' cleanup completed successfully!${NC}"
echo ""
echo -e "${BLUE}📋 Summary:${NC}"
echo -e "${BLUE}✅ All .env files backed up to: $backup_dir${NC}"
echo -e "${BLUE}✅ All .env files removed from working directory${NC}"
echo -e "${BLUE}✅ .env.template created for reference${NC}"
echo -e "${BLUE}✅ .gitignore updated to prevent future .env commits${NC}"
echo ""
echo -e "${BLUE}🚀 Next steps:${NC}"
echo -e "${BLUE}1. Test system: infisical run --env=dev -- cargo check${NC}"
echo -e "${BLUE}2. Start secure: ./start-overmind-secure.sh${NC}"
echo -e "${BLUE}3. Monitor logs for any missing secrets${NC}"
echo ""
echo -e "${PURPLE}🔐 THE OVERMIND PROTOCOL is now fully VAULT-SECURED!${NC}"
