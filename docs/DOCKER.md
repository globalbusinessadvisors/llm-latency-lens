# Docker Guide for LLM-Latency-Lens

This guide covers how to build, run, and deploy LLM-Latency-Lens using Docker and Docker Compose.

## Table of Contents

- [Quick Start](#quick-start)
- [Building the Image](#building-the-image)
- [Running the Container](#running-the-container)
- [Docker Compose](#docker-compose)
- [Configuration](#configuration)
- [Monitoring Stack](#monitoring-stack)
- [Production Deployment](#production-deployment)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Using Docker Compose (Recommended)

The easiest way to get started is with Docker Compose, which includes the full monitoring stack:

```bash
# Start all services (LLM-Latency-Lens + Prometheus + Grafana + AlertManager)
docker-compose up -d

# View logs
docker-compose logs -f llm-latency-lens

# Stop all services
docker-compose down
```

Access the services:
- **Grafana**: http://localhost:3000 (admin/admin)
- **Prometheus**: http://localhost:9091
- **AlertManager**: http://localhost:9093
- **LLM-Latency-Lens Metrics**: http://localhost:9090/metrics

### Using Docker Only

```bash
# Build the image
docker build -t llm-latency-lens .

# Run the container
docker run --rm llm-latency-lens --help
```

## Building the Image

### Standard Build

```bash
docker build -t llm-latency-lens:latest .
```

### Build for Specific Platform

```bash
# For AMD64
docker build --platform linux/amd64 -t llm-latency-lens:amd64 .

# For ARM64
docker build --platform linux/arm64 -t llm-latency-lens:arm64 .
```

### Multi-platform Build

```bash
docker buildx create --use
docker buildx build --platform linux/amd64,linux/arm64 -t llm-latency-lens:latest --push .
```

### Build Arguments

```bash
docker build \
  --build-arg RUST_VERSION=1.75 \
  -t llm-latency-lens:latest .
```

## Running the Container

### Basic Usage

```bash
# Display help
docker run --rm llm-latency-lens --help

# Run with specific provider
docker run --rm \
  -e OPENAI_API_KEY=your-key \
  llm-latency-lens benchmark --provider openai --model gpt-4
```

### With Volume Mounts

```bash
# Mount configuration and output directories
docker run --rm \
  -v $(pwd)/config:/app/config:ro \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/logs:/app/logs \
  -e OPENAI_API_KEY=your-key \
  llm-latency-lens benchmark --provider openai
```

### Environment Variables

```bash
docker run --rm \
  -e RUST_LOG=info \
  -e RUST_BACKTRACE=1 \
  -e OPENAI_API_KEY=your-key \
  -e ANTHROPIC_API_KEY=your-key \
  -e GOOGLE_API_KEY=your-key \
  llm-latency-lens benchmark
```

### Expose Metrics Port

```bash
docker run --rm \
  -p 9090:9090 \
  -e PROMETHEUS_HOST=0.0.0.0 \
  -e PROMETHEUS_PORT=9090 \
  llm-latency-lens serve
```

## Docker Compose

### Full Stack Deployment

The `docker-compose.yml` file includes:
- **LLM-Latency-Lens**: Main application
- **Prometheus**: Metrics collection
- **Grafana**: Visualization dashboards
- **AlertManager**: Alert routing and notification

### Starting the Stack

```bash
# Start in detached mode
docker-compose up -d

# Start with rebuild
docker-compose up -d --build

# Start specific services
docker-compose up -d prometheus grafana
```

### Viewing Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f llm-latency-lens

# Last 100 lines
docker-compose logs --tail=100 llm-latency-lens
```

### Managing Services

```bash
# Stop services
docker-compose stop

# Start services
docker-compose start

# Restart services
docker-compose restart llm-latency-lens

# Remove containers (keeps volumes)
docker-compose down

# Remove containers and volumes
docker-compose down -v
```

### Scaling Services

```bash
# Scale LLM-Latency-Lens to 3 instances
docker-compose up -d --scale llm-latency-lens=3
```

## Configuration

### Environment Configuration

Create a `.env` file in the project root:

```env
# API Keys
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GOOGLE_API_KEY=...

# Application Configuration
RUST_LOG=info
RUST_BACKTRACE=1

# Prometheus Configuration
PROMETHEUS_HOST=0.0.0.0
PROMETHEUS_PORT=9090

# Grafana Configuration
GF_SECURITY_ADMIN_USER=admin
GF_SECURITY_ADMIN_PASSWORD=change_me_in_production
```

### Volume Configuration

The Docker Compose setup uses the following volumes:

```yaml
volumes:
  - ./config:/app/config:ro        # Configuration files (read-only)
  - ./data:/app/data                # Output data
  - ./logs:/app/logs                # Application logs
```

### Monitoring Configuration

#### Prometheus

Edit `monitoring/prometheus/prometheus.yml` to configure scrape targets:

```yaml
scrape_configs:
  - job_name: 'llm-latency-lens'
    static_configs:
      - targets: ['llm-latency-lens:9090']
```

#### Grafana Dashboards

Pre-configured dashboards are located in:
```
monitoring/grafana/dashboards/
├── llm-latency-overview.json
```

#### AlertManager

Configure alert routing in `monitoring/alertmanager/alertmanager.yml`:

```yaml
receivers:
  - name: 'slack'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK'
        channel: '#alerts'
```

## Monitoring Stack

### Accessing Grafana

1. Open http://localhost:3000
2. Login with admin/admin (change on first login)
3. Navigate to Dashboards → LLM Monitoring → LLM Latency Lens - Overview

### Key Metrics

The default dashboard shows:
- Request latency percentiles (p50, p95, p99)
- Request rate by provider
- Error rate
- Token throughput
- Time to First Token (TTFT)
- Time per Output Token (TPOT)

### Setting Up Alerts

1. Edit `monitoring/prometheus/alerts.yml`
2. Add your alert rules:

```yaml
- alert: HighLatency
  expr: llm_request_duration_seconds{quantile="0.95"} > 5
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "High latency detected"
```

3. Restart Prometheus:

```bash
docker-compose restart prometheus
```

## Production Deployment

### Security Hardening

1. **Use secrets management**:

```yaml
secrets:
  openai_api_key:
    external: true

services:
  llm-latency-lens:
    secrets:
      - openai_api_key
```

2. **Enable TLS for Prometheus/Grafana**:

```yaml
# Use a reverse proxy like Traefik or nginx
services:
  traefik:
    image: traefik:v2.10
    ports:
      - "443:443"
    volumes:
      - ./traefik.yml:/etc/traefik/traefik.yml
```

3. **Run with read-only root filesystem**:

```yaml
services:
  llm-latency-lens:
    read_only: true
    tmpfs:
      - /tmp
```

### Resource Limits

Set appropriate resource limits:

```yaml
services:
  llm-latency-lens:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 256M
```

### Health Checks

```yaml
healthcheck:
  test: ["CMD", "wget", "--spider", "http://localhost:9090/metrics"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 40s
```

### Logging

Configure centralized logging:

```yaml
services:
  llm-latency-lens:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

Or use a logging driver:

```yaml
logging:
  driver: "gelf"
  options:
    gelf-address: "udp://logstash:12201"
    tag: "llm-latency-lens"
```

## Troubleshooting

### Image Size Too Large

The image should be under 50MB. If it's larger:

```bash
# Check image size
docker images llm-latency-lens

# Inspect layers
docker history llm-latency-lens
```

### Container Fails to Start

```bash
# Check logs
docker logs llm-latency-lens

# Inspect container
docker inspect llm-latency-lens

# Run in interactive mode
docker run -it --rm llm-latency-lens sh
```

### Permission Issues

The container runs as non-root user (UID 65532). Ensure volumes have correct permissions:

```bash
# Fix permissions
sudo chown -R 65532:65532 ./data ./logs
```

### Metrics Not Appearing in Prometheus

1. Check Prometheus targets: http://localhost:9091/targets
2. Verify LLM-Latency-Lens is exposing metrics:

```bash
docker exec llm-latency-lens wget -O- http://localhost:9090/metrics
```

3. Check Prometheus logs:

```bash
docker-compose logs prometheus
```

### Build Cache Issues

```bash
# Clear build cache
docker builder prune -a

# Build without cache
docker build --no-cache -t llm-latency-lens .
```

### Network Issues

```bash
# Check network connectivity
docker-compose exec llm-latency-lens ping prometheus

# Inspect network
docker network inspect llm-latency-lens_llm-monitoring
```

## Advanced Topics

### Custom Prometheus Rules

Add custom recording rules in `monitoring/prometheus/prometheus.yml`:

```yaml
rule_files:
  - /etc/prometheus/alerts.yml
  - /etc/prometheus/recording_rules.yml
```

### Grafana Provisioning

Add custom dashboards by placing JSON files in:
```
monitoring/grafana/dashboards/
```

### Multi-Stage Build Optimization

The Dockerfile uses cargo-chef for dependency caching. To disable:

```dockerfile
# Comment out chef stages and build directly
FROM rust:1.75-slim AS builder
COPY . .
RUN cargo build --release
```

### Container Registry

Push to Docker Hub:

```bash
docker tag llm-latency-lens:latest username/llm-latency-lens:latest
docker push username/llm-latency-lens:latest
```

Push to GitHub Container Registry:

```bash
docker tag llm-latency-lens:latest ghcr.io/username/llm-latency-lens:latest
docker push ghcr.io/username/llm-latency-lens:latest
```

## Additional Resources

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)
- [Best Practices for Writing Dockerfiles](https://docs.docker.com/develop/develop-images/dockerfile_best-practices/)
