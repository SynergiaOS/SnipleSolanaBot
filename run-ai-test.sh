#!/bin/bash
# THE OVERMIND PROTOCOL - AI Test Runner
# Load environment and run live AI test

set -e

echo "🧠 THE OVERMIND PROTOCOL - AI Test Runner"
echo "========================================="

# Load environment variables
if [ -f .env ]; then
    echo "📁 Loading environment from .env..."
    export $(grep -v '^#' .env | xargs)
    echo "✅ Environment loaded"
else
    echo "❌ .env file not found"
    exit 1
fi

# Verify critical API keys
echo "🔍 Verifying API keys..."

if [[ -z "$DEEPSEEK_API_KEY" ]] || [[ "$DEEPSEEK_API_KEY" == "your-"* ]]; then
    echo "❌ DEEPSEEK_API_KEY not configured properly"
    exit 1
fi

if [[ -z "$JINA_API_KEY" ]] || [[ "$JINA_API_KEY" == "your-"* ]]; then
    echo "❌ JINA_API_KEY not configured properly"
    exit 1
fi

echo "✅ API keys verified"
echo "   - DeepSeek: ${DEEPSEEK_API_KEY:0:10}..."
echo "   - Jina AI: ${JINA_API_KEY:0:10}..."
echo ""

# Run the live AI test
echo "🚀 Running live AI test..."
echo "=========================="

cargo run --bin test_ai_live

echo ""
echo "🎉 AI test completed successfully!"
