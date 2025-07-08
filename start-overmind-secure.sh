#!/bin/bash
# THE OVERMIND PROTOCOL - Secure Startup with Infisical
# Maximum profit mode with enterprise-grade security

set -e

echo "🧠 THE OVERMIND PROTOCOL - SECURE STARTUP"
echo "========================================"
echo "🎯 MISSION: 28 SOL → 100 SOL"
echo "🔐 SECURITY: Infisical secrets management"
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

# Start THE OVERMIND PROTOCOL with Infisical secrets
echo "🧠 Starting THE OVERMIND PROTOCOL with secure secrets..."
echo "🎯 Target: 28 SOL → 100 SOL"
echo "⚡ Mode: ULTRA BLITZKRIEG"
echo ""

# Run with Infisical secrets injection
infisical run --env=prod -- cargo run --profile contabo

echo "🎉 THE OVERMIND PROTOCOL started successfully!"
echo "📊 Monitor at: http://localhost:8080"
echo "🧠 AI Brain at: http://localhost:8000"
