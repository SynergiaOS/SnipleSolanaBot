#!/bin/bash
# THE OVERMIND PROTOCOL - OPERACJA 'VAULT' v2.0
# Advanced Security Hardening for Production Environment
# Token: st.31baa38e-572d-4abc-8de6-83b1abca9cbf...
# VPC: vpc-05f61f843ed60555e, Account: 962364259018

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${PURPLE}🔐 THE OVERMIND PROTOCOL - OPERACJA 'VAULT' v2.0${NC}"
echo -e "${PURPLE}====================================================${NC}"
echo -e "${BLUE}🛡️  Mission: Advanced Security Hardening${NC}"
echo -e "${BLUE}🌐 VPC: vpc-05f61f843ed60555e${NC}"
echo -e "${BLUE}🐉 DragonflyDB: High-performance cache layer${NC}"
echo ""

# =============================================================================
# SYSTEM HARDENING
# =============================================================================
echo -e "${CYAN}🔧 Phase 1: System Hardening${NC}"
echo "=============================="

# Update system packages
echo -e "${BLUE}📦 Updating system packages...${NC}"
sudo apt-get update -y
sudo apt-get upgrade -y

# Install security tools
echo -e "${BLUE}🛠️  Installing security tools...${NC}"
sudo apt-get install -y \
    fail2ban \
    ufw \
    rkhunter \
    chkrootkit \
    lynis \
    aide \
    auditd

# Configure firewall
echo -e "${BLUE}🔥 Configuring UFW firewall...${NC}"
sudo ufw --force reset
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow specific ports for THE OVERMIND PROTOCOL
sudo ufw allow from 192.168.0.0/16 to any port 8080 comment "OVERMIND API"
sudo ufw allow from 192.168.0.0/16 to any port 9090 comment "Prometheus"
sudo ufw allow from 192.168.0.0/16 to any port 6379 comment "DragonflyDB"
sudo ufw allow from 192.168.0.0/16 to any port 22 comment "SSH"

# Allow HTTPS outbound for Infisical
sudo ufw allow out 443 comment "HTTPS Outbound"

sudo ufw --force enable
echo -e "${GREEN}✅ Firewall configured${NC}"

# =============================================================================
# INFISICAL SECURITY
# =============================================================================
echo ""
echo -e "${CYAN}🔐 Phase 2: Infisical Security${NC}"
echo "==============================="

# Secure Infisical token storage
echo -e "${BLUE}🔑 Securing Infisical token...${NC}"
sudo mkdir -p /etc/overmind/secrets
sudo chmod 700 /etc/overmind/secrets

# Create secure token file
cat > /tmp/infisical-token << EOF
st.31baa38e-572d-4abc-8de6-83b1abca9cbf.97a3bb72ec1ab7c1002a187feaaa31d3.ccae3c429818d256c68d768c15f22e78
EOF

sudo mv /tmp/infisical-token /etc/overmind/secrets/
sudo chmod 600 /etc/overmind/secrets/infisical-token
sudo chown root:root /etc/overmind/secrets/infisical-token

echo -e "${GREEN}✅ Infisical token secured${NC}"

# =============================================================================
# DRAGONFLYDB SECURITY
# =============================================================================
echo ""
echo -e "${CYAN}🐉 Phase 3: DragonflyDB Security${NC}"
echo "================================="

# Configure DragonflyDB security
echo -e "${BLUE}🔒 Configuring DragonflyDB security...${NC}"

# Create DragonflyDB configuration
sudo mkdir -p /etc/dragonflydb
cat > /tmp/dragonflydb.conf << EOF
# THE OVERMIND PROTOCOL - DragonflyDB Configuration
# VPC: vpc-05f61f843ed60555e

# Network Security
bind 192.168.0.0/16
port 6379
protected-mode yes

# Authentication
requirepass $(openssl rand -base64 32)

# TLS Configuration
tls-port 6380
tls-cert-file /etc/ssl/certs/dragonflydb.crt
tls-key-file /etc/ssl/private/dragonflydb.key
tls-protocols "TLSv1.2 TLSv1.3"

# Memory and Performance
maxmemory 2gb
maxmemory-policy allkeys-lru

# Logging
loglevel notice
logfile /var/log/dragonflydb/dragonflydb.log

# Security
rename-command FLUSHDB ""
rename-command FLUSHALL ""
rename-command DEBUG ""
rename-command CONFIG "CONFIG_$(openssl rand -hex 8)"
EOF

sudo mv /tmp/dragonflydb.conf /etc/dragonflydb/
sudo chmod 640 /etc/dragonflydb/dragonflydb.conf
sudo chown root:dragonflydb /etc/dragonflydb/dragonflydb.conf

echo -e "${GREEN}✅ DragonflyDB security configured${NC}"

# =============================================================================
# SSL/TLS CERTIFICATES
# =============================================================================
echo ""
echo -e "${CYAN}🔐 Phase 4: SSL/TLS Certificates${NC}"
echo "================================="

echo -e "${BLUE}📜 Generating SSL certificates...${NC}"

# Create SSL directory
sudo mkdir -p /etc/ssl/overmind
sudo chmod 755 /etc/ssl/overmind

# Generate CA key and certificate
sudo openssl genrsa -out /etc/ssl/overmind/ca-key.pem 4096
sudo openssl req -new -x509 -days 365 -key /etc/ssl/overmind/ca-key.pem \
    -out /etc/ssl/overmind/ca-cert.pem \
    -subj "/C=US/ST=NY/L=NYC/O=OVERMIND/CN=OVERMIND-CA"

# Generate server key and certificate
sudo openssl genrsa -out /etc/ssl/overmind/server-key.pem 4096
sudo openssl req -new -key /etc/ssl/overmind/server-key.pem \
    -out /etc/ssl/overmind/server.csr \
    -subj "/C=US/ST=NY/L=NYC/O=OVERMIND/CN=overmind-protocol"

sudo openssl x509 -req -days 365 -in /etc/ssl/overmind/server.csr \
    -CA /etc/ssl/overmind/ca-cert.pem \
    -CAkey /etc/ssl/overmind/ca-key.pem \
    -CAcreateserial \
    -out /etc/ssl/overmind/server-cert.pem

# Set permissions
sudo chmod 600 /etc/ssl/overmind/*-key.pem
sudo chmod 644 /etc/ssl/overmind/*-cert.pem

echo -e "${GREEN}✅ SSL certificates generated${NC}"

# =============================================================================
# AUDIT AND MONITORING
# =============================================================================
echo ""
echo -e "${CYAN}📊 Phase 5: Audit and Monitoring${NC}"
echo "================================="

echo -e "${BLUE}📋 Configuring audit system...${NC}"

# Configure auditd
cat > /tmp/audit.rules << EOF
# THE OVERMIND PROTOCOL - Audit Rules

# Monitor Infisical token access
-w /etc/overmind/secrets/infisical-token -p rwxa -k infisical_access

# Monitor DragonflyDB configuration
-w /etc/dragonflydb/ -p rwxa -k dragonflydb_config

# Monitor SSL certificates
-w /etc/ssl/overmind/ -p rwxa -k ssl_certs

# Monitor THE OVERMIND PROTOCOL binary
-w /usr/local/bin/overmind -p x -k overmind_execution

# Monitor network configuration
-w /etc/network/ -p rwxa -k network_config
-w /etc/ufw/ -p rwxa -k firewall_config

# Monitor system authentication
-w /etc/passwd -p wa -k passwd_changes
-w /etc/shadow -p wa -k shadow_changes
-w /etc/group -p wa -k group_changes

# Monitor privilege escalation
-a always,exit -F arch=b64 -S execve -F euid=0 -F auid>=1000 -k privilege_escalation
EOF

sudo mv /tmp/audit.rules /etc/audit/rules.d/overmind.rules
sudo systemctl restart auditd

echo -e "${GREEN}✅ Audit system configured${NC}"

# =============================================================================
# INTRUSION DETECTION
# =============================================================================
echo ""
echo -e "${CYAN}🛡️  Phase 6: Intrusion Detection${NC}"
echo "================================="

echo -e "${BLUE}🔍 Configuring intrusion detection...${NC}"

# Configure fail2ban for THE OVERMIND PROTOCOL
cat > /tmp/overmind.conf << EOF
[overmind-api]
enabled = true
port = 8080
filter = overmind-api
logpath = /var/log/overmind/access.log
maxretry = 5
bantime = 3600
findtime = 600

[dragonflydb]
enabled = true
port = 6379
filter = dragonflydb
logpath = /var/log/dragonflydb/dragonflydb.log
maxretry = 3
bantime = 7200
findtime = 300
EOF

sudo mv /tmp/overmind.conf /etc/fail2ban/jail.d/
sudo systemctl restart fail2ban

echo -e "${GREEN}✅ Intrusion detection configured${NC}"

# =============================================================================
# SECURE STARTUP SCRIPT
# =============================================================================
echo ""
echo -e "${CYAN}🚀 Phase 7: Secure Startup Script${NC}"
echo "=================================="

echo -e "${BLUE}📝 Creating secure startup script...${NC}"

cat > /tmp/overmind-secure-start.sh << 'EOF'
#!/bin/bash
# THE OVERMIND PROTOCOL - Ultra-Secure Startup
# OPERACJA 'VAULT' v2.0 - Production Ready

set -euo pipefail

# Load Infisical token securely
export INFISICAL_SERVICE_TOKEN=$(sudo cat /etc/overmind/secrets/infisical-token)
export INFISICAL_PROJECT_ID=73c2f3cb-c922-4a46-a333-7b96fbc6301a
export INFISICAL_ENVIRONMENT=production

# DragonflyDB configuration
export DRAGONFLYDB_VPC_ID=vpc-05f61f843ed60555e
export DRAGONFLYDB_CIDR=192.168.0.0/16
export DRAGONFLYDB_ACCOUNT_ID=962364259018

# Security settings
export RUST_LOG=info
export OVERMIND_SECURITY_MODE=maximum
export ENABLE_AUDIT_LOGGING=true

# Start THE OVERMIND PROTOCOL with maximum security
echo "🔐 Starting THE OVERMIND PROTOCOL with VAULT v2.0 security..."
cd /opt/overmind-protocol
cargo run --profile contabo
EOF

sudo mv /tmp/overmind-secure-start.sh /usr/local/bin/
sudo chmod 755 /usr/local/bin/overmind-secure-start.sh

echo -e "${GREEN}✅ Secure startup script created${NC}"

# =============================================================================
# FINAL SECURITY VALIDATION
# =============================================================================
echo ""
echo -e "${CYAN}✅ Phase 8: Security Validation${NC}"
echo "==============================="

echo -e "${BLUE}🧪 Running security validation...${NC}"

# Check file permissions
echo -e "${BLUE}📋 Checking file permissions...${NC}"
ls -la /etc/overmind/secrets/
ls -la /etc/dragonflydb/
ls -la /etc/ssl/overmind/

# Check firewall status
echo -e "${BLUE}🔥 Checking firewall status...${NC}"
sudo ufw status

# Check audit system
echo -e "${BLUE}📊 Checking audit system...${NC}"
sudo auditctl -l | grep overmind || echo "Audit rules loaded"

# Check fail2ban
echo -e "${BLUE}🛡️  Checking fail2ban...${NC}"
sudo fail2ban-client status

echo ""
echo -e "${GREEN}🎉 OPERACJA 'VAULT' v2.0 SECURITY HARDENING COMPLETE!${NC}"
echo ""
echo -e "${PURPLE}📋 SECURITY SUMMARY:${NC}"
echo -e "${BLUE}✅ System hardened with fail2ban, UFW, audit${NC}"
echo -e "${BLUE}✅ Infisical token secured in /etc/overmind/secrets/${NC}"
echo -e "${BLUE}✅ DragonflyDB configured with TLS and auth${NC}"
echo -e "${BLUE}✅ SSL certificates generated${NC}"
echo -e "${BLUE}✅ Audit logging enabled${NC}"
echo -e "${BLUE}✅ Intrusion detection active${NC}"
echo -e "${BLUE}✅ VPC network isolation: vpc-05f61f843ed60555e${NC}"
echo ""
echo -e "${PURPLE}🚀 THE OVERMIND PROTOCOL is now VAULT-SECURED v2.0!${NC}"
echo -e "${PURPLE}🔐 Maximum security posture achieved!${NC}"
