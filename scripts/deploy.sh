#!/usr/bin/env bash
# Production deployment script for LLM-Latency-Lens

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.yml}"
COMPOSE_PROD_FILE="${COMPOSE_PROD_FILE:-docker-compose.prod.yml}"
ENV_FILE="${ENV_FILE:-.env}"
BACKUP_DIR="${BACKUP_DIR:-./backups}"
DATA_DIR="${DATA_DIR:-./data}"

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        exit 1
    fi

    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed"
        exit 1
    fi

    # Check environment file
    if [ ! -f "$ENV_FILE" ]; then
        log_error "Environment file $ENV_FILE not found"
        log_info "Create it from .env.example: cp .env.example $ENV_FILE"
        exit 1
    fi

    log_info "Prerequisites check passed"
}

# Create directories
create_directories() {
    log_info "Creating directories..."

    mkdir -p "$BACKUP_DIR"
    mkdir -p "$DATA_DIR/prometheus"
    mkdir -p "$DATA_DIR/grafana"
    mkdir -p "$DATA_DIR/alertmanager"

    # Set permissions
    chmod 755 "$DATA_DIR"
    chmod 755 "$DATA_DIR/prometheus"
    chmod 755 "$DATA_DIR/grafana"
    chmod 755 "$DATA_DIR/alertmanager"

    log_info "Directories created"
}

# Backup existing data
backup_data() {
    log_info "Backing up existing data..."

    if [ -d "$DATA_DIR" ]; then
        TIMESTAMP=$(date +%Y%m%d_%H%M%S)
        BACKUP_FILE="$BACKUP_DIR/backup_$TIMESTAMP.tar.gz"

        tar -czf "$BACKUP_FILE" "$DATA_DIR" 2>/dev/null || true

        if [ -f "$BACKUP_FILE" ]; then
            log_info "Backup created: $BACKUP_FILE"
        else
            log_warn "No data to backup"
        fi
    else
        log_warn "No existing data directory"
    fi
}

# Pull latest images
pull_images() {
    log_info "Pulling latest Docker images..."

    docker-compose -f "$COMPOSE_FILE" -f "$COMPOSE_PROD_FILE" pull

    log_info "Images pulled successfully"
}

# Stop existing services
stop_services() {
    log_info "Stopping existing services..."

    docker-compose -f "$COMPOSE_FILE" -f "$COMPOSE_PROD_FILE" down || true

    log_info "Services stopped"
}

# Start services
start_services() {
    log_info "Starting services..."

    docker-compose -f "$COMPOSE_FILE" -f "$COMPOSE_PROD_FILE" up -d

    log_info "Services started"
}

# Health check
health_check() {
    log_info "Performing health checks..."

    # Wait for services to be ready
    sleep 10

    # Check each service
    SERVICES=("llm-latency-lens" "prometheus" "grafana" "alertmanager")

    for SERVICE in "${SERVICES[@]}"; do
        if docker-compose -f "$COMPOSE_FILE" -f "$COMPOSE_PROD_FILE" ps | grep -q "$SERVICE.*Up"; then
            log_info "$SERVICE is healthy"
        else
            log_error "$SERVICE is not healthy"
            docker-compose -f "$COMPOSE_FILE" -f "$COMPOSE_PROD_FILE" logs "$SERVICE"
            exit 1
        fi
    done

    log_info "All health checks passed"
}

# Show service URLs
show_urls() {
    log_info "Service URLs:"
    echo ""
    echo "  Grafana:        http://localhost:3000"
    echo "  Prometheus:     http://localhost:9091"
    echo "  AlertManager:   http://localhost:9093"
    echo "  Metrics:        http://localhost:9090/metrics"
    echo ""
}

# Main deployment
deploy() {
    log_info "Starting deployment..."

    check_prerequisites
    create_directories
    backup_data
    pull_images
    stop_services
    start_services
    health_check
    show_urls

    log_info "Deployment completed successfully!"
}

# Rollback
rollback() {
    log_warn "Rolling back to previous version..."

    # Find latest backup
    LATEST_BACKUP=$(ls -t "$BACKUP_DIR"/backup_*.tar.gz 2>/dev/null | head -n 1)

    if [ -z "$LATEST_BACKUP" ]; then
        log_error "No backup found for rollback"
        exit 1
    fi

    log_info "Restoring from: $LATEST_BACKUP"

    # Stop services
    stop_services

    # Restore backup
    rm -rf "$DATA_DIR"
    tar -xzf "$LATEST_BACKUP"

    # Start services
    start_services
    health_check

    log_info "Rollback completed"
}

# Usage
usage() {
    cat << EOF
Usage: $0 [COMMAND]

Commands:
    deploy      Deploy or update the application (default)
    rollback    Rollback to previous version
    stop        Stop all services
    start       Start all services
    restart     Restart all services
    logs        Show logs
    status      Show service status
    backup      Create backup
    help        Show this help message

Environment Variables:
    COMPOSE_FILE       Docker Compose file (default: docker-compose.yml)
    COMPOSE_PROD_FILE  Production override file (default: docker-compose.prod.yml)
    ENV_FILE           Environment file (default: .env)
    BACKUP_DIR         Backup directory (default: ./backups)
    DATA_DIR           Data directory (default: ./data)

Examples:
    $0 deploy
    $0 rollback
    BACKUP_DIR=/mnt/backups $0 deploy

EOF
}

# Main
main() {
    COMMAND="${1:-deploy}"

    case "$COMMAND" in
        deploy)
            deploy
            ;;
        rollback)
            rollback
            ;;
        stop)
            stop_services
            ;;
        start)
            start_services
            health_check
            show_urls
            ;;
        restart)
            stop_services
            start_services
            health_check
            show_urls
            ;;
        logs)
            docker-compose -f "$COMPOSE_FILE" -f "$COMPOSE_PROD_FILE" logs -f
            ;;
        status)
            docker-compose -f "$COMPOSE_FILE" -f "$COMPOSE_PROD_FILE" ps
            ;;
        backup)
            backup_data
            ;;
        help|--help|-h)
            usage
            ;;
        *)
            log_error "Unknown command: $COMMAND"
            usage
            exit 1
            ;;
    esac
}

main "$@"
