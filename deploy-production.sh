#!/bin/bash
# THE OVERMIND PROTOCOL - Production Deployment Script
# Deploy THE OVERMIND PROTOCOL to live production environment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# ASCII Art Banner
echo -e "${PURPLE}"
cat << "EOF"
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║    ████████╗██╗  ██╗███████╗    ██████╗ ██╗   ██╗███████╗    ║
║    ╚══██╔══╝██║  ██║██╔════╝   ██╔═══██╗██║   ██║██╔════╝    ║
║       ██║   ███████║█████╗     ██║   ██║██║   ██║█████╗      ║
║       ██║   ██╔══██║██╔══╝     ██║   ██║╚██╗ ██╔╝██╔══╝      ║
║       ██║   ██║  ██║███████╗   ╚██████╔╝ ╚████╔╝ ███████╗    ║
║       ╚═╝   ╚═╝  ╚═╝╚══════╝    ╚═════╝   ╚═══╝  ╚══════╝    ║
║                                                               ║
║    ██████╗ ██████╗  ██████╗ ████████╗ ██████╗  ██████╗ ██╗   ║
║    ██╔══██╗██╔══██╗██╔═══██╗╚══██╔══╝██╔═══██╗██╔════╝██║   ║
║    ██████╔╝██████╔╝██║   ██║   ██║   ██║   ██║██║     ██║   ║
║    ██╔═══╝ ██╔══██╗██║   ██║   ██║   ██║   ██║██║     ██║   ║
║    ██║     ██║  ██║╚██████╔╝   ██║   ╚██████╔╝╚██████╗███████╗║
║    ╚═╝     ╚═╝  ╚═╝ ╚═════╝    ╚═╝    ╚═════╝  ╚═════╝╚══════╝║
║                                                               ║
║                  PRODUCTION DEPLOYMENT                       ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

echo -e "${CYAN}🚀 THE OVERMIND PROTOCOL - Production Deployment${NC}"
echo -e "${CYAN}=================================================${NC}"
echo -e "${YELLOW}⚠️  WARNING: This will deploy to LIVE TRADING environment!${NC}"
echo -e "${YELLOW}💰 Real money will be at risk!${NC}"
echo ""

# Confirmation prompt
read -p "Are you sure you want to deploy to PRODUCTION? (type 'YES' to continue): " confirm
if [ "$confirm" != "YES" ]; then
    echo -e "${RED}❌ Deployment cancelled.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Production deployment confirmed.${NC}"
echo ""

# =============================================================================
# PRE-DEPLOYMENT CHECKS
# =============================================================================
echo -e "${BLUE}🔍 Running pre-deployment checks...${NC}"

# Check if running as root (security risk)
if [ "$EUID" -eq 0 ]; then
    echo -e "${RED}❌ Do not run as root for security reasons!${NC}"
    exit 1
fi

# Check if .env file exists
if [ ! -f ".env" ]; then
    echo -e "${YELLOW}⚠️  .env file not found. Creating from template...${NC}"
    if [ -f "config/production.env.template" ]; then
        cp config/production.env.template .env
        echo -e "${YELLOW}📝 Please edit .env file with your actual API keys before continuing.${NC}"
        echo -e "${YELLOW}🔑 Required keys: HELIUS_API_KEY, JITO_API_KEY, DEEPSEEK_API_KEY, JINA_API_KEY${NC}"
        exit 1
    else
        echo -e "${RED}❌ Template file not found!${NC}"
        exit 1
    fi
fi

# Check for required API keys
echo -e "${BLUE}🔑 Checking API keys...${NC}"
source .env

required_keys=(
    "HELIUS_API_KEY"
    "JITO_API_KEY" 
    "DEEPSEEK_API_KEY"
    "JINA_API_KEY"
    "MAIN_WALLET_PRIVATE_KEY"
)

missing_keys=()
for key in "${required_keys[@]}"; do
    if [ -z "${!key}" ] || [ "${!key}" = "your_${key,,}_here" ]; then
        missing_keys+=("$key")
    fi
done

if [ ${#missing_keys[@]} -ne 0 ]; then
    echo -e "${RED}❌ Missing required API keys:${NC}"
    for key in "${missing_keys[@]}"; do
        echo -e "${RED}   - $key${NC}"
    done
    echo -e "${YELLOW}Please update your .env file with actual API keys.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ All required API keys present.${NC}"

# Check Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker not found. Please install Docker first.${NC}"
    exit 1
fi

if ! docker info &> /dev/null; then
    echo -e "${RED}❌ Docker daemon not running. Please start Docker.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Docker is running.${NC}"

# Check Rust compilation
echo -e "${BLUE}🦀 Checking Rust compilation...${NC}"
if ! cargo check --profile contabo &> /dev/null; then
    echo -e "${RED}❌ Rust compilation failed. Please fix errors first.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Rust compilation successful.${NC}"

# Run tests
echo -e "${BLUE}🧪 Running final tests...${NC}"
if ! cargo test --lib --profile contabo &> /dev/null; then
    echo -e "${RED}❌ Tests failed. Please fix issues first.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ All tests passed.${NC}"

# =============================================================================
# SECURITY CHECKS
# =============================================================================
echo -e "${BLUE}🛡️  Running security checks...${NC}"

# Check file permissions
chmod 600 .env
echo -e "${GREEN}✅ Environment file permissions secured.${NC}"

# Check wallet file permissions
if [ -d "wallets" ]; then
    chmod 700 wallets
    chmod 600 wallets/*.json 2>/dev/null || true
    echo -e "${GREEN}✅ Wallet file permissions secured.${NC}"
fi

# =============================================================================
# INFRASTRUCTURE DEPLOYMENT
# =============================================================================
echo -e "${BLUE}🏗️  Deploying infrastructure...${NC}"

# Stop any existing containers
echo -e "${YELLOW}🛑 Stopping existing containers...${NC}"
docker-compose down 2>/dev/null || true

# Pull latest images
echo -e "${BLUE}📥 Pulling latest Docker images...${NC}"
docker-compose pull

# Build custom images
echo -e "${BLUE}🔨 Building custom images...${NC}"
docker-compose build --no-cache

# Start infrastructure services
echo -e "${BLUE}🚀 Starting infrastructure services...${NC}"
docker-compose up -d dragonfly chroma

# Wait for services to be ready
echo -e "${YELLOW}⏳ Waiting for infrastructure to initialize...${NC}"
sleep 15

# Health check for DragonflyDB
echo -e "${BLUE}🔍 Checking DragonflyDB health...${NC}"
if ! docker exec overmind-dragonfly redis-cli ping &> /dev/null; then
    echo -e "${RED}❌ DragonflyDB health check failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ DragonflyDB is healthy.${NC}"

# Health check for Chroma
echo -e "${BLUE}🔍 Checking Chroma health...${NC}"
if ! curl -s http://localhost:8000/api/v1/heartbeat &> /dev/null; then
    echo -e "${RED}❌ Chroma health check failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Chroma is healthy.${NC}"

# =============================================================================
# APPLICATION DEPLOYMENT
# =============================================================================
echo -e "${BLUE}🎯 Deploying THE OVERMIND PROTOCOL...${NC}"

# Create logs directory
mkdir -p logs
chmod 755 logs

# Start AI Brain
echo -e "${BLUE}🧠 Starting AI Brain...${NC}"
docker-compose up -d ai-brain

# Wait for AI Brain
sleep 10

# Final confirmation before starting trading
echo ""
echo -e "${YELLOW}⚠️  FINAL WARNING: About to start LIVE TRADING!${NC}"
echo -e "${YELLOW}💰 Real SOL will be used for trading!${NC}"
echo -e "${YELLOW}📊 Current configuration:${NC}"
echo -e "${YELLOW}   - Trading Mode: ${SNIPER_TRADING_MODE}${NC}"
echo -e "${YELLOW}   - Max Daily Loss: ${SNIPER_MAX_DAILY_LOSS} SOL${NC}"
echo -e "${YELLOW}   - Max Position Size: ${SNIPER_MAX_POSITION_SIZE} SOL${NC}"
echo ""

read -p "Start live trading? (type 'START TRADING' to continue): " trading_confirm
if [ "$trading_confirm" != "START TRADING" ]; then
    echo -e "${RED}❌ Trading start cancelled.${NC}"
    exit 1
fi

# Start THE OVERMIND PROTOCOL
echo -e "${GREEN}🚀 Starting THE OVERMIND PROTOCOL...${NC}"
echo -e "${PURPLE}⚡ ULTRA BLITZKRIEG MODE ACTIVATED!${NC}"
echo ""

# Start the main trading application
RUST_LOG=info cargo run --profile contabo &
TRADING_PID=$!

# =============================================================================
# POST-DEPLOYMENT MONITORING
# =============================================================================
echo -e "${GREEN}✅ THE OVERMIND PROTOCOL deployed successfully!${NC}"
echo ""
echo -e "${CYAN}📊 MONITORING ENDPOINTS:${NC}"
echo -e "${CYAN}   - Trading API: http://localhost:8080${NC}"
echo -e "${CYAN}   - AI Brain: http://localhost:8000${NC}"
echo -e "${CYAN}   - DragonflyDB: localhost:6379${NC}"
echo -e "${CYAN}   - Metrics: http://localhost:9090${NC}"
echo ""
echo -e "${YELLOW}📝 IMPORTANT NOTES:${NC}"
echo -e "${YELLOW}   - Monitor logs: tail -f logs/trading.log${NC}"
echo -e "${YELLOW}   - Stop trading: kill $TRADING_PID${NC}"
echo -e "${YELLOW}   - Emergency stop: docker-compose down${NC}"
echo ""
echo -e "${GREEN}🎯 THE OVERMIND PROTOCOL is now LIVE and hunting for MEV opportunities!${NC}"
echo -e "${PURPLE}💰 Target: 28 SOL → 100 SOL${NC}"
echo -e "${PURPLE}⚡ May the profits be with you!${NC}"

# Keep script running to monitor
wait $TRADING_PID
