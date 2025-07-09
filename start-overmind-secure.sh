#!/bin/bash
# THE OVERMIND PROTOCOL - Secure Startup with Infisical
# Maximum profit mode with enterprise-grade security

set -e

echo "🧠 THE OVERMIND PROTOCOL v2.0 - VAULT-SECURED STARTUP"
echo "====================================================="
echo "🎯 MISSION: 28 SOL → 100 SOL"
echo "🔐 SECURITY: Infisical + DragonflyDB Cache"
echo "🌐 VPC: vpc-05f61f843ed60555e (192.168.0.0/16)"
echo "🐉 DragonflyDB: High-performance cache layer"
echo ""

# Check prerequisites
echo "🔍 Checking prerequisites..."

# Check if Infisical is installed and authenticated
if ! command -v infisical &> /dev/null; then
    echo "❌ Infisical CLI not found. Run ./infisical-setup.sh first"
    exit 1
fi

if ! infisical user &> /dev/null; then
    echo "❌ Not authenticated with Infisical. Run: infisical login"
    exit 1
fi

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Verify Rust compilation
echo "🦀 Verifying Rust compilation..."
if ! cargo check &> /dev/null; then
    echo "❌ Rust compilation failed. Please fix errors first."
    exit 1
fi

echo "✅ All prerequisites met!"
echo ""

# Start infrastructure
echo "🚀 Starting THE OVERMIND PROTOCOL infrastructure..."
docker-compose up -d

# Wait for services to be ready
echo "⏳ Waiting for services to initialize..."
sleep 10

# Load production configuration
echo "🔧 Loading production configuration..."
source config/production-vault.env

# Start THE OVERMIND PROTOCOL with full VAULT security
echo "🧠 Starting THE OVERMIND PROTOCOL v2.0 with VAULT security..."
echo "🎯 Target: 28 SOL → 100 SOL"
echo "⚡ Mode: ULTRA BLITZKRIEG + VAULT-SECURED"
echo "🔐 Token: st.31baa38e-572d-4abc-8de6-83b1abca9cbf..."
echo "🐉 DragonflyDB: Enabled"
echo ""

# Run with Infisical service token
INFISICAL_SERVICE_TOKEN=st.31baa38e-572d-4abc-8de6-83b1abca9cbf.97a3bb72ec1ab7c1002a187feaaa31d3.ccae3c429818d256c68d768c15f22e78 \
INFISICAL_PROJECT_ID=73c2f3cb-c922-4a46-a333-7b96fbc6301a \
INFISICAL_ENVIRONMENT=production \
DRAGONFLYDB_VPC_ID=vpc-05f61f843ed60555e \
DRAGONFLYDB_CIDR=192.168.0.0/16 \
DRAGONFLYDB_ACCOUNT_ID=962364259018 \
cargo run --profile contabo

echo "🎉 THE OVERMIND PROTOCOL v2.0 started successfully!"
echo "📊 Monitor at: http://localhost:8080"
echo "🧠 AI Brain at: http://localhost:8000"
echo "🐉 DragonflyDB Cache: Active"
echo "🔐 Infisical Vault: Secured"
echo "🌐 VPC Network: Isolated"
echo ""
echo "🚀 VAULT-SECURED TRADING SYSTEM ONLINE!"
