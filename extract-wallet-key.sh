#!/bin/bash
# THE OVERMIND PROTOCOL - Secure Wallet Key Extraction
# Extract private key from wallet JSON for environment configuration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${PURPLE}🔐 THE OVERMIND PROTOCOL - Wallet Key Extraction${NC}"
echo -e "${PURPLE}===============================================${NC}"
echo ""

# Check if wallet file exists
WALLET_FILE="wallets/mainnet-trading-wallet.json"
if [ ! -f "$WALLET_FILE" ]; then
    echo -e "${RED}❌ Wallet file not found: $WALLET_FILE${NC}"
    exit 1
fi

echo -e "${BLUE}🔍 Found wallet file: $WALLET_FILE${NC}"

# Extract private key using Python
echo -e "${BLUE}🔑 Extracting private key...${NC}"

PRIVATE_KEY=$(python3 -c "
import json
import base58

# Read wallet file
with open('$WALLET_FILE', 'r') as f:
    wallet_data = json.load(f)

# Convert to private key
private_key_bytes = bytes(wallet_data)
private_key_base58 = base58.b58encode(private_key_bytes).decode('utf-8')
print(private_key_base58)
")

if [ -z "$PRIVATE_KEY" ]; then
    echo -e "${RED}❌ Failed to extract private key${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Private key extracted successfully${NC}"
echo ""

# Update .env file
echo -e "${BLUE}📝 Updating .env file...${NC}"

# Backup original .env
cp .env .env.backup.$(date +%Y%m%d_%H%M%S)

# Replace private key in .env
sed -i "s/SNIPER_WALLET_PRIVATE_KEY=YOUR_PRIVATE_KEY_HERE/SNIPER_WALLET_PRIVATE_KEY=$PRIVATE_KEY/" .env

echo -e "${GREEN}✅ .env file updated with private key${NC}"
echo -e "${YELLOW}⚠️  Backup created: .env.backup.$(date +%Y%m%d_%H%M%S)${NC}"

# Secure permissions
chmod 600 .env
echo -e "${GREEN}✅ .env file permissions secured (600)${NC}"

echo ""
echo -e "${PURPLE}🎯 Wallet Configuration Complete!${NC}"
echo -e "${CYAN}📊 Wallet Address: $(python3 -c "
import json
from solders.keypair import Keypair
import base58

with open('$WALLET_FILE', 'r') as f:
    wallet_data = json.load(f)

keypair = Keypair.from_bytes(bytes(wallet_data))
print(str(keypair.pubkey()))
")${NC}"

echo ""
echo -e "${YELLOW}⚠️  SECURITY REMINDER:${NC}"
echo -e "${YELLOW}   - Never share your private key${NC}"
echo -e "${YELLOW}   - Keep .env file secure (600 permissions)${NC}"
echo -e "${YELLOW}   - Never commit .env to version control${NC}"
echo -e "${YELLOW}   - Consider using hardware wallet for large amounts${NC}"
