#!/bin/bash
# OPERACJA "FORGE" - Setup Script
# 
# Automatyczna konfiguracja atomowej kuźni inteligencji
# TensorZero + AWS + Docker setup

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TENSORZERO_VERSION="1.4.0"
DOCKER_COMPOSE_VERSION="2.21.0"
AWS_CLI_VERSION="2.13.0"

echo -e "${BLUE}🔥 OPERACJA 'FORGE' - Setup Script${NC}"
echo -e "${BLUE}Atomowa kuźnia inteligencji - inicjalizacja${NC}"
echo "=" $(printf '=%.0s' {1..50})

# Function to print status
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Check if running as root
check_root() {
    if [[ $EUID -eq 0 ]]; then
        print_error "This script should not be run as root"
        exit 1
    fi
}

# Check system requirements
check_system() {
    print_info "Checking system requirements..."
    
    # Check OS
    if [[ "$OSTYPE" != "linux-gnu"* ]]; then
        print_error "This script is designed for Linux systems"
        exit 1
    fi
    
    # Check architecture
    ARCH=$(uname -m)
    if [[ "$ARCH" != "x86_64" ]]; then
        print_warning "Non-x86_64 architecture detected: $ARCH"
    fi
    
    # Check available memory
    MEMORY_GB=$(free -g | awk '/^Mem:/{print $2}')
    if [[ $MEMORY_GB -lt 8 ]]; then
        print_warning "Less than 8GB RAM detected. FORGE may require more memory."
    fi
    
    print_status "System requirements check completed"
}

# Install system dependencies
install_dependencies() {
    print_info "Installing system dependencies..."
    
    # Update package list
    sudo apt-get update -qq
    
    # Install essential packages
    sudo apt-get install -y \
        curl \
        wget \
        git \
        build-essential \
        pkg-config \
        libssl-dev \
        llvm-dev \
        libclang-dev \
        unzip \
        jq \
        htop \
        tree
    
    print_status "System dependencies installed"
}

# Install Rust
install_rust() {
    print_info "Installing Rust toolchain..."
    
    if command -v rustc &> /dev/null; then
        print_warning "Rust already installed: $(rustc --version)"
        return
    fi
    
    # Install rustup
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    
    # Install additional targets
    rustup target add x86_64-unknown-linux-gnu
    rustup target add aarch64-unknown-linux-gnu
    
    # Install useful cargo tools
    cargo install cargo-watch cargo-edit cargo-audit
    
    print_status "Rust toolchain installed: $(rustc --version)"
}

# Install Docker
install_docker() {
    print_info "Installing Docker..."
    
    if command -v docker &> /dev/null; then
        print_warning "Docker already installed: $(docker --version)"
        return
    fi
    
    # Install Docker
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    rm get-docker.sh
    
    # Add user to docker group
    sudo usermod -aG docker $USER
    
    # Install Docker Compose
    sudo curl -L "https://github.com/docker/compose/releases/download/v${DOCKER_COMPOSE_VERSION}/docker-compose-$(uname -s)-$(uname -m)" \
        -o /usr/local/bin/docker-compose
    sudo chmod +x /usr/local/bin/docker-compose
    
    print_status "Docker installed. Please log out and back in to use Docker without sudo."
}

# Install AWS CLI
install_aws_cli() {
    print_info "Installing AWS CLI..."
    
    if command -v aws &> /dev/null; then
        print_warning "AWS CLI already installed: $(aws --version)"
        return
    fi
    
    # Download and install AWS CLI
    curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
    unzip awscliv2.zip
    sudo ./aws/install
    rm -rf aws awscliv2.zip
    
    print_status "AWS CLI installed: $(aws --version)"
}

# Install TensorZero
install_tensorzero() {
    print_info "Installing TensorZero compiler..."
    
    # Create TensorZero directory
    mkdir -p ~/.tensorzero ~/.cargo/bin
    
    # Download TensorZero
    TENSORZERO_URL="https://github.com/tensorzero/tensorzero/releases/download/v${TENSORZERO_VERSION}/tensorzero-${TENSORZERO_VERSION}-x86_64-unknown-linux-gnu.tar.gz"
    
    curl -L "$TENSORZERO_URL" -o tensorzero.tar.gz
    tar -xzf tensorzero.tar.gz
    
    # Install compiler
    mv "tensorzero-${TENSORZERO_VERSION}/tzc" ~/.cargo/bin/
    mv "tensorzero-${TENSORZERO_VERSION}"/* ~/.tensorzero/
    chmod +x ~/.cargo/bin/tzc
    
    # Cleanup
    rm -rf tensorzero.tar.gz "tensorzero-${TENSORZERO_VERSION}"
    
    # Verify installation
    if ~/.cargo/bin/tzc --version &> /dev/null; then
        print_status "TensorZero compiler installed: $(~/.cargo/bin/tzc --version)"
    else
        print_error "TensorZero installation failed"
        exit 1
    fi
}

# Setup project structure
setup_project() {
    print_info "Setting up project structure..."
    
    # Create necessary directories
    mkdir -p {artifacts,logs,config,strategies,prompts}
    mkdir -p strategies/{templates,custom,generated}
    mkdir -p artifacts/{compiled,reports,cache}
    mkdir -p logs/{forge,tensorzero,compilation}
    
    # Create .gitignore for artifacts
    cat > artifacts/.gitignore << EOF
# Compiled artifacts
*.so
*.dll
*.dylib

# Compilation reports
*.json
*.log

# Cache files
cache/
EOF
    
    # Create logs .gitignore
    cat > logs/.gitignore << EOF
# Log files
*.log
*.log.*

# Rotated logs
*.gz
*.zip
EOF
    
    print_status "Project structure created"
}

# Setup environment variables
setup_environment() {
    print_info "Setting up environment variables..."
    
    # Create .env template
    cat > .env.template << EOF
# OPERACJA "FORGE" Environment Variables
# Copy to .env and fill in your values

# TensorZero Configuration
TENSORZERO_URL=http://localhost:3000
CLICKHOUSE_URL=http://localhost:8123/tensorzero

# AI Model API Keys
ANTHROPIC_API_KEY=your_anthropic_key_here
OPENAI_API_KEY=your_openai_key_here

# AWS Configuration
AWS_ACCESS_KEY_ID=your_aws_access_key
AWS_SECRET_ACCESS_KEY=your_aws_secret_key
AWS_REGION=us-east-1
ARTIFACT_BUCKET=overmind-forge-artifacts

# FORGE Configuration
FORGE_MODE=enabled
FORGE_ENVIRONMENT=development
FORGE_LOG_LEVEL=info

# Security
FORGE_API_KEY=generate_secure_key_here
FORGE_ENABLE_SIGNATURE_VERIFICATION=true

# Performance
FORGE_MAX_COMPILATION_JOBS=4
FORGE_COMPILATION_TIMEOUT=300
EOF
    
    if [[ ! -f .env ]]; then
        cp .env.template .env
        print_warning "Created .env file. Please edit it with your configuration."
    fi
    
    print_status "Environment configuration created"
}

# Setup Docker services
setup_docker_services() {
    print_info "Setting up Docker services..."
    
    # Create docker-compose.yml for FORGE services
    cat > docker-compose.forge.yml << EOF
version: '3.8'

services:
  # ClickHouse for TensorZero observability
  clickhouse:
    image: clickhouse/clickhouse-server:23.8
    container_name: forge-clickhouse
    ports:
      - "8123:8123"
      - "9000:9000"
    environment:
      CLICKHOUSE_DB: tensorzero
      CLICKHOUSE_USER: default
      CLICKHOUSE_PASSWORD: ""
    volumes:
      - clickhouse_data:/var/lib/clickhouse
      - ./config/clickhouse:/etc/clickhouse-server/config.d
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:8123/ping"]
      interval: 30s
      timeout: 10s
      retries: 3

  # TensorZero Gateway
  tensorzero:
    image: tensorzero/gateway:${TENSORZERO_VERSION:-1.4.0}
    container_name: forge-tensorzero
    ports:
      - "3000:3000"
    environment:
      CLICKHOUSE_URL: http://clickhouse:8123/tensorzero
      ANTHROPIC_API_KEY: \${ANTHROPIC_API_KEY}
      OPENAI_API_KEY: \${OPENAI_API_KEY}
    volumes:
      - ./config/tensorzero.toml:/app/config.toml:ro
    depends_on:
      clickhouse:
        condition: service_healthy
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  # Redis for PheromoneBus
  redis:
    image: redis:7-alpine
    container_name: forge-redis
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 30s
      timeout: 10s
      retries: 3

  # Prometheus for monitoring
  prometheus:
    image: prom/prometheus:latest
    container_name: forge-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./config/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--web.enable-lifecycle'
    restart: unless-stopped

  # Grafana for visualization
  grafana:
    image: grafana/grafana:latest
    container_name: forge-grafana
    ports:
      - "3001:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: overmind123
    volumes:
      - grafana_data:/var/lib/grafana
      - ./config/grafana:/etc/grafana/provisioning
    restart: unless-stopped

volumes:
  clickhouse_data:
  redis_data:
  prometheus_data:
  grafana_data:

networks:
  default:
    name: forge-network
EOF
    
    print_status "Docker services configuration created"
}

# Create monitoring configuration
setup_monitoring() {
    print_info "Setting up monitoring configuration..."
    
    mkdir -p config/{prometheus,grafana/dashboards,grafana/datasources}
    
    # Prometheus configuration
    cat > config/prometheus.yml << EOF
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  # - "first_rules.yml"
  # - "second_rules.yml"

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'tensorzero'
    static_configs:
      - targets: ['tensorzero:3000']
    metrics_path: '/metrics'

  - job_name: 'overmind-protocol'
    static_configs:
      - targets: ['host.docker.internal:8080']
    metrics_path: '/metrics'

  - job_name: 'forge-compiler'
    static_configs:
      - targets: ['host.docker.internal:8081']
    metrics_path: '/metrics'
EOF
    
    # Grafana datasource
    cat > config/grafana/datasources/prometheus.yml << EOF
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
EOF
    
    print_status "Monitoring configuration created"
}

# Verify installation
verify_installation() {
    print_info "Verifying installation..."
    
    # Check Rust
    if command -v rustc &> /dev/null; then
        print_status "Rust: $(rustc --version)"
    else
        print_error "Rust not found"
    fi
    
    # Check TensorZero
    if ~/.cargo/bin/tzc --version &> /dev/null; then
        print_status "TensorZero: $(~/.cargo/bin/tzc --version)"
    else
        print_error "TensorZero not found"
    fi
    
    # Check Docker
    if command -v docker &> /dev/null; then
        print_status "Docker: $(docker --version)"
    else
        print_error "Docker not found"
    fi
    
    # Check AWS CLI
    if command -v aws &> /dev/null; then
        print_status "AWS CLI: $(aws --version)"
    else
        print_error "AWS CLI not found"
    fi
    
    print_status "Installation verification completed"
}

# Main execution
main() {
    echo -e "${BLUE}Starting FORGE setup...${NC}"
    
    check_root
    check_system
    install_dependencies
    install_rust
    install_docker
    install_aws_cli
    install_tensorzero
    setup_project
    setup_environment
    setup_docker_services
    setup_monitoring
    verify_installation
    
    echo ""
    echo -e "${GREEN}🎉 OPERACJA 'FORGE' setup completed successfully!${NC}"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "1. Edit .env file with your API keys and configuration"
    echo "2. Start Docker services: docker-compose -f docker-compose.forge.yml up -d"
    echo "3. Build the project: cargo build --release"
    echo "4. Run tests: cargo test --workspace"
    echo "5. Start FORGE: cargo run --bin overmind-protocol"
    echo ""
    echo -e "${BLUE}🔥 The atomic forge of intelligence is ready!${NC}"
}

# Run main function
main "$@"
