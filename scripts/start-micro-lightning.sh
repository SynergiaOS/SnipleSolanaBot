#!/bin/bash

# MICRO-LIGHTNING TRADING SYSTEM STARTUP SCRIPT
# OPERACJA MIKRO-BŁYSKAWICA - Complete System Deployment

set -eo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LOG_FILE="$PROJECT_ROOT/logs/micro-lightning-startup.log"

# Create logs directory
mkdir -p "$PROJECT_ROOT/logs"

# Logging function
log() {
    echo -e "${1}" | tee -a "$LOG_FILE"
}

# Error handling
error_exit() {
    log "${RED}❌ ERROR: $1${NC}"
    exit 1
}

# Success message
success() {
    log "${GREEN}✅ $1${NC}"
}

# Warning message
warning() {
    log "${YELLOW}⚠️  $1${NC}"
}

# Info message
info() {
    log "${BLUE}ℹ️  $1${NC}"
}

# Header
header() {
    log "${PURPLE}$1${NC}"
}

# Print banner
print_banner() {
    header "
╔══════════════════════════════════════════════════════════════════╗
║                    OPERACJA MIKRO-BŁYSKAWICA                     ║
║                  MICRO-LIGHTNING TRADING SYSTEM                  ║
║                                                                  ║
║  🚀 THE OVERMIND PROTOCOL v4.1 'MONOLITH'                      ║
║  ⚡ $20/60min High-Frequency Meme Coin Trading                 ║
║  🛡️ 5 Commandments Enforcement System                          ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝
"
}

# Check prerequisites
check_prerequisites() {
    header "🔍 CHECKING PREREQUISITES"
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        error_exit "Docker is not installed"
    fi
    success "Docker found: $(docker --version)"
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        error_exit "Docker Compose is not installed"
    fi
    success "Docker Compose found: $(docker-compose --version)"
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        error_exit "Rust/Cargo is not installed"
    fi
    success "Rust found: $(rustc --version)"
    
    # Check required files
    local required_files=(
        "$PROJECT_ROOT/docker-compose.yml"
        "$PROJECT_ROOT/Dockerfile.micro-lightning"
        "$PROJECT_ROOT/.env.micro-lightning"
        "$PROJECT_ROOT/config/prometheus-micro-lightning.yml"
        "$PROJECT_ROOT/config/micro-lightning-alerts.yml"
    )
    
    for file in "${required_files[@]}"; do
        if [[ ! -f "$file" ]]; then
            error_exit "Required file not found: $file"
        fi
    done
    success "All required files found"
}

# Load environment configuration
load_environment() {
    header "🔧 LOADING ENVIRONMENT CONFIGURATION"
    
    # Copy micro-lightning environment
    if [[ -f "$PROJECT_ROOT/.env.micro-lightning" ]]; then
        cp "$PROJECT_ROOT/.env.micro-lightning" "$PROJECT_ROOT/.env"
        success "Micro-Lightning environment loaded"
    else
        error_exit "Micro-Lightning environment file not found"
    fi
    
    # Validate critical environment variables
    source "$PROJECT_ROOT/.env"
    
    local required_vars=(
        "MICRO_LIGHTNING_ENABLED"
        "MICRO_CAPITAL_ALLOCATION"
        "COMMANDMENT_LIFE_LIMIT"
        "COMMANDMENT_WALLET_ROTATION"
        "COMMANDMENT_MILITIA_COOLDOWN"
        "COMMANDMENT_PSYCHOLOGY_TAX"
        "COMMANDMENT_BATTLEFIELD_MIN"
        "COMMANDMENT_BATTLEFIELD_MAX"
    )
    
    for var in "${required_vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            error_exit "Required environment variable not set: $var"
        fi
    done
    success "Environment variables validated"
}

# Build Docker images
build_images() {
    header "🏗️  BUILDING DOCKER IMAGES"
    
    cd "$PROJECT_ROOT"
    
    # Build main trading executor
    info "Building main trading executor..."
    docker build -t overmind-trading-executor . || error_exit "Failed to build trading executor"
    success "Trading executor built"
    
    # Build micro-lightning monitor
    info "Building micro-lightning monitor..."
    docker build -f Dockerfile.micro-lightning -t overmind-micro-lightning-monitor . || error_exit "Failed to build micro-lightning monitor"
    success "Micro-lightning monitor built"
}

# Start infrastructure services
start_infrastructure() {
    header "🚀 STARTING INFRASTRUCTURE SERVICES"
    
    cd "$PROJECT_ROOT"
    
    # Start DragonflyDB (Redis replacement)
    info "Starting DragonflyDB..."
    docker-compose up -d dragonfly || error_exit "Failed to start DragonflyDB"
    success "DragonflyDB started"
    
    # Start Prometheus
    info "Starting Prometheus..."
    docker-compose up -d prometheus || error_exit "Failed to start Prometheus"
    success "Prometheus started"
    
    # Start Node Exporter
    info "Starting Node Exporter..."
    docker-compose up -d node-exporter || error_exit "Failed to start Node Exporter"
    success "Node Exporter started"
    
    # Wait for services to be ready
    info "Waiting for infrastructure services to be ready..."
    sleep 10
}

# Start AI services
start_ai_services() {
    header "🧠 STARTING AI SERVICES"
    
    # Start TensorZero
    info "Starting TensorZero..."
    docker-compose up -d tensorzero || error_exit "Failed to start TensorZero"
    success "TensorZero started"
    
    # Start AI Brain
    info "Starting AI Brain..."
    docker-compose up -d ai-brain || error_exit "Failed to start AI Brain"
    success "AI Brain started"
    
    # Wait for AI services
    info "Waiting for AI services to initialize..."
    sleep 15
}

# Start micro-lightning services
start_micro_lightning() {
    header "⚡ STARTING MICRO-LIGHTNING SERVICES"
    
    # Start micro-lightning monitor
    info "Starting Micro-Lightning Monitor..."
    docker-compose up -d micro-lightning-monitor || error_exit "Failed to start Micro-Lightning Monitor"
    success "Micro-Lightning Monitor started"
    
    # Start trading executor with micro-lightning support
    info "Starting Trading Executor with Micro-Lightning support..."
    docker-compose up -d trading-executor || error_exit "Failed to start Trading Executor"
    success "Trading Executor started"
    
    # Wait for services to be ready
    info "Waiting for micro-lightning services to initialize..."
    sleep 20
}

# Verify system health
verify_system_health() {
    header "🏥 VERIFYING SYSTEM HEALTH"
    
    local services=(
        "dragonfly:6379"
        "prometheus:9090"
        "trading-executor:8080"
        "micro-lightning-monitor:8081"
    )
    
    for service in "${services[@]}"; do
        local host=$(echo "$service" | cut -d':' -f1)
        local port=$(echo "$service" | cut -d':' -f2)
        
        info "Checking $host:$port..."
        if timeout 10 bash -c "</dev/tcp/$host/$port"; then
            success "$host:$port is responding"
        else
            warning "$host:$port is not responding (may still be starting up)"
        fi
    done
    
    # Check micro-lightning specific health
    info "Checking Micro-Lightning Monitor health..."
    if curl -f http://localhost:8081/health &>/dev/null; then
        success "Micro-Lightning Monitor health check passed"
    else
        warning "Micro-Lightning Monitor health check failed (may still be starting)"
    fi
    
    # Check trading executor health
    info "Checking Trading Executor health..."
    if curl -f http://localhost:8080/health &>/dev/null; then
        success "Trading Executor health check passed"
    else
        warning "Trading Executor health check failed (may still be starting)"
    fi
}

# Display system status
display_system_status() {
    header "📊 SYSTEM STATUS"
    
    info "System URLs:"
    echo "  🎯 Trading Executor:        http://localhost:8080"
    echo "  ⚡ Micro-Lightning Monitor: http://localhost:8081"
    echo "  📊 Prometheus:              http://localhost:9090"
    echo "  🧠 AI Brain:                http://localhost:3000"
    echo ""
    
    info "Key Endpoints:"
    echo "  📈 Trading Status:          http://localhost:8080/status"
    echo "  ⚡ Micro-Lightning Status:  http://localhost:8081/status"
    echo "  📊 Metrics:                 http://localhost:8081/metrics"
    echo "  🛡️ Commandments Status:     http://localhost:8081/commandments"
    echo "  🚨 Alerts:                  http://localhost:8081/alerts"
    echo ""
    
    info "Configuration:"
    echo "  💰 Capital Allocation:      $MICRO_CAPITAL_ALLOCATION"
    echo "  ⏰ Life Limit:              $COMMANDMENT_LIFE_LIMIT minutes"
    echo "  🔄 Wallet Rotation:         $COMMANDMENT_WALLET_ROTATION operations"
    echo "  ❄️ Militia Cooldown:        $COMMANDMENT_MILITIA_COOLDOWN minutes"
    echo "  🧠 Psychology Tax:          $(echo "$COMMANDMENT_PSYCHOLOGY_TAX * 100" | bc)%"
    echo "  🎯 Battlefield Range:       \$${COMMANDMENT_BATTLEFIELD_MIN} - \$${COMMANDMENT_BATTLEFIELD_MAX}"
    echo ""
}

# Display operational commands
display_commands() {
    header "🎮 OPERATIONAL COMMANDS"
    
    echo "System Control:"
    echo "  🛑 Stop System:             docker-compose down"
    echo "  🔄 Restart System:          docker-compose restart"
    echo "  📋 View Logs:               docker-compose logs -f"
    echo ""
    
    echo "Micro-Lightning Specific:"
    echo "  ⚡ Monitor Logs:             docker-compose logs -f micro-lightning-monitor"
    echo "  📊 View Metrics:            curl http://localhost:8081/metrics"
    echo "  🛡️ Check Commandments:      curl http://localhost:8081/commandments"
    echo "  🚨 Trigger Emergency:       curl -X POST http://localhost:8081/emergency"
    echo ""
    
    echo "Monitoring:"
    echo "  📊 Prometheus Targets:      http://localhost:9090/targets"
    echo "  📈 Prometheus Metrics:      http://localhost:9090/graph"
    echo "  🚨 Alert Rules:             http://localhost:9090/alerts"
    echo ""
}

# Main execution
main() {
    print_banner
    
    log "$(date): Starting Micro-Lightning Trading System deployment..."
    
    check_prerequisites
    load_environment
    build_images
    start_infrastructure
    start_ai_services
    start_micro_lightning
    verify_system_health
    display_system_status
    display_commands
    
    success "🎉 MICRO-LIGHTNING TRADING SYSTEM DEPLOYMENT COMPLETE!"
    
    header "
╔══════════════════════════════════════════════════════════════════╗
║                        SYSTEM READY                             ║
║                                                                  ║
║  🟢 MODUŁ MIKRO-BŁYSKAWICA - AKTYWNY                           ║
║  ⚡ Ready for $20/60min operations                             ║
║  🛡️ 5 Commandments enforcement active                          ║
║  📊 Real-time monitoring enabled                                ║
║                                                                  ║
║  \"W królestwie memcoinów ślimaki są pożywieniem,               ║
║   nie handlującymi.\"                                           ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝
"
    
    info "System is now running. Press Ctrl+C to stop monitoring, or run 'docker-compose down' to stop all services."
    
    # Follow logs
    if [[ "${1:-}" != "--no-follow" ]]; then
        info "Following system logs (Ctrl+C to exit)..."
        docker-compose logs -f micro-lightning-monitor trading-executor
    fi
}

# Handle script interruption
trap 'echo -e "\n${YELLOW}⚠️  Script interrupted. System may still be running.${NC}"; exit 130' INT

# Run main function
main "$@"
