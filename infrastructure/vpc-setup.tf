# THE OVERMIND PROTOCOL - VPC Infrastructure
# VPC: vpc-05f61f843ed60555e, Account: 962364259018, CIDR: 192.168.0.0/16

terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

# Configure AWS Provider
provider "aws" {
  region = var.aws_region
  
  default_tags {
    tags = {
      Project     = "THE-OVERMIND-PROTOCOL"
      Environment = var.environment
      ManagedBy   = "Terraform"
      Owner       = "OVERMIND-TEAM"
    }
  }
}

# Variables
variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "production"
}

variable "vpc_id" {
  description = "Existing VPC ID"
  type        = string
  default     = "vpc-05f61f843ed60555e"
}

variable "account_id" {
  description = "AWS Account ID"
  type        = string
  default     = "962364259018"
}

# Data sources for existing VPC
data "aws_vpc" "overmind_vpc" {
  id = var.vpc_id
}

data "aws_availability_zones" "available" {
  state = "available"
}

# Subnets for THE OVERMIND PROTOCOL
resource "aws_subnet" "overmind_private" {
  count             = 2
  vpc_id            = data.aws_vpc.overmind_vpc.id
  cidr_block        = "192.168.${count.index + 1}.0/24"
  availability_zone = data.aws_availability_zones.available.names[count.index]
  
  map_public_ip_on_launch = false
  
  tags = {
    Name = "overmind-private-${count.index + 1}"
    Type = "Private"
    Tier = "Application"
  }
}

resource "aws_subnet" "overmind_public" {
  count             = 2
  vpc_id            = data.aws_vpc.overmind_vpc.id
  cidr_block        = "192.168.${count.index + 10}.0/24"
  availability_zone = data.aws_availability_zones.available.names[count.index]
  
  map_public_ip_on_launch = true
  
  tags = {
    Name = "overmind-public-${count.index + 1}"
    Type = "Public"
    Tier = "LoadBalancer"
  }
}

resource "aws_subnet" "overmind_database" {
  count             = 2
  vpc_id            = data.aws_vpc.overmind_vpc.id
  cidr_block        = "192.168.${count.index + 20}.0/24"
  availability_zone = data.aws_availability_zones.available.names[count.index]
  
  map_public_ip_on_launch = false
  
  tags = {
    Name = "overmind-database-${count.index + 1}"
    Type = "Database"
    Tier = "Data"
  }
}

# Security Groups
resource "aws_security_group" "overmind_protocol" {
  name_prefix = "overmind-protocol-"
  vpc_id      = data.aws_vpc.overmind_vpc.id
  description = "Security group for THE OVERMIND PROTOCOL"
  
  # Inbound rules
  ingress {
    description = "API access from VPC"
    from_port   = 8080
    to_port     = 8080
    protocol    = "tcp"
    cidr_blocks = [data.aws_vpc.overmind_vpc.cidr_block]
  }
  
  ingress {
    description = "Prometheus metrics"
    from_port   = 9090
    to_port     = 9090
    protocol    = "tcp"
    cidr_blocks = [data.aws_vpc.overmind_vpc.cidr_block]
  }
  
  ingress {
    description = "SSH access"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = [data.aws_vpc.overmind_vpc.cidr_block]
  }
  
  # Outbound rules
  egress {
    description = "All outbound traffic"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
  
  tags = {
    Name = "overmind-protocol-sg"
  }
}

resource "aws_security_group" "dragonflydb_cache" {
  name_prefix = "dragonflydb-cache-"
  vpc_id      = data.aws_vpc.overmind_vpc.id
  description = "Security group for DragonflyDB cache"
  
  # Inbound rules
  ingress {
    description     = "Redis access from OVERMIND"
    from_port       = 6379
    to_port         = 6379
    protocol        = "tcp"
    security_groups = [aws_security_group.overmind_protocol.id]
  }
  
  ingress {
    description = "Redis access from VPC"
    from_port   = 6379
    to_port     = 6379
    protocol    = "tcp"
    cidr_blocks = [data.aws_vpc.overmind_vpc.cidr_block]
  }
  
  # Outbound rules
  egress {
    description = "All outbound traffic"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
  
  tags = {
    Name = "dragonflydb-cache-sg"
  }
}

# Network ACLs
resource "aws_network_acl" "overmind_private" {
  vpc_id     = data.aws_vpc.overmind_vpc.id
  subnet_ids = aws_subnet.overmind_private[*].id
  
  # Inbound rules
  ingress {
    rule_no    = 100
    protocol   = "tcp"
    action     = "allow"
    cidr_block = data.aws_vpc.overmind_vpc.cidr_block
    from_port  = 8080
    to_port    = 8080
  }
  
  ingress {
    rule_no    = 110
    protocol   = "tcp"
    action     = "allow"
    cidr_block = data.aws_vpc.overmind_vpc.cidr_block
    from_port  = 6379
    to_port    = 6379
  }
  
  ingress {
    rule_no    = 120
    protocol   = "tcp"
    action     = "allow"
    cidr_block = "0.0.0.0/0"
    from_port  = 443
    to_port    = 443
  }
  
  # Outbound rules
  egress {
    rule_no    = 100
    protocol   = "-1"
    action     = "allow"
    cidr_block = "0.0.0.0/0"
    from_port  = 0
    to_port    = 0
  }
  
  tags = {
    Name = "overmind-private-nacl"
  }
}

# Route Tables
resource "aws_route_table" "overmind_private" {
  vpc_id = data.aws_vpc.overmind_vpc.id
  
  tags = {
    Name = "overmind-private-rt"
  }
}

resource "aws_route_table_association" "overmind_private" {
  count          = length(aws_subnet.overmind_private)
  subnet_id      = aws_subnet.overmind_private[count.index].id
  route_table_id = aws_route_table.overmind_private.id
}

# VPC Endpoints for secure communication
resource "aws_vpc_endpoint" "s3" {
  vpc_id       = data.aws_vpc.overmind_vpc.id
  service_name = "com.amazonaws.${var.aws_region}.s3"
  
  tags = {
    Name = "overmind-s3-endpoint"
  }
}

resource "aws_vpc_endpoint" "ec2" {
  vpc_id              = data.aws_vpc.overmind_vpc.id
  service_name        = "com.amazonaws.${var.aws_region}.ec2"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = aws_subnet.overmind_private[*].id
  security_group_ids  = [aws_security_group.overmind_protocol.id]
  
  tags = {
    Name = "overmind-ec2-endpoint"
  }
}

# Outputs
output "vpc_id" {
  description = "VPC ID"
  value       = data.aws_vpc.overmind_vpc.id
}

output "private_subnet_ids" {
  description = "Private subnet IDs"
  value       = aws_subnet.overmind_private[*].id
}

output "public_subnet_ids" {
  description = "Public subnet IDs"
  value       = aws_subnet.overmind_public[*].id
}

output "database_subnet_ids" {
  description = "Database subnet IDs"
  value       = aws_subnet.overmind_database[*].id
}

output "overmind_security_group_id" {
  description = "OVERMIND Protocol security group ID"
  value       = aws_security_group.overmind_protocol.id
}

output "dragonflydb_security_group_id" {
  description = "DragonflyDB security group ID"
  value       = aws_security_group.dragonflydb_cache.id
}
