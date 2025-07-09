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

# Create .infisical.json configuration with user's project ID
cat > .infisical.json << EOF
{
  "workspaceId": "73c2f3cb-c922-4a46-a333-7b96fbc6301a",
  "environments": [
    {
      "name": "development",
      "slug": "dev"
    },
    {
      "name": "staging",
      "slug": "staging"
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
echo "🔐 OPERACJA 'VAULT' - Next steps:"
echo "1. Add secrets to Infisical dashboard: https://app.infisical.com/project/73c2f3cb-c922-4a46-a333-7b96fbc6301a"
echo "2. Run migration script: ./migrate-secrets-to-infisical.sh"
echo "3. Test with: infisical run --env=dev -- cargo check"
echo "4. Start secure: ./start-overmind-secure.sh"
echo ""
echo "🎯 Project ID: 73c2f3cb-c922-4a46-a333-7b96fbc6301a"
echo "🌍 Environments: dev, staging, prod"
