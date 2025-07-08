#!/bin/bash
# THE OVERMIND PROTOCOL - Infisical Integration Setup
# Secure secrets management for production trading

set -e

echo "🔐 THE OVERMIND PROTOCOL - Infisical Setup"
echo "=========================================="

# Check if Infisical CLI is installed
if ! command -v infisical &> /dev/null; then
    echo "📦 Installing Infisical CLI..."
    curl -1sLf 'https://dl.cloudsmith.io/public/infisical/infisical-cli/setup.deb.sh' | sudo -E bash
    sudo apt-get update && sudo apt-get install infisical
fi

# Login to Infisical (if not already logged in)
echo "🔑 Checking Infisical authentication..."
if ! infisical user &> /dev/null; then
    echo "Please login to Infisical:"
    infisical login
fi

# Initialize project
echo "🚀 Initializing Infisical in project..."
infisical init

# Create .infisical.json configuration
cat > .infisical.json << EOF
{
  "workspaceId": "5ee5b660-e4dc-4676-8e1d-a2b69b72ce36",
  "environments": [
    {
      "name": "development",
      "slug": "dev"
    },
    {
      "name": "production", 
      "slug": "prod"
    }
  ]
}
EOF

echo "✅ Infisical setup complete!"
echo ""
echo "Next steps:"
echo "1. Add secrets to Infisical dashboard"
echo "2. Update .env to use: infisical run -- cargo run"
echo "3. Remove sensitive data from .env file"
