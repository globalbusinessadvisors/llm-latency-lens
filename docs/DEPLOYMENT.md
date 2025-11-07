# Deployment Guide for LLM-Latency-Lens

This comprehensive guide covers deploying LLM-Latency-Lens to various environments.

## Table of Contents

- [Quick Start](#quick-start)
- [Prerequisites](#prerequisites)
- [Local Development](#local-development)
- [Production Deployment](#production-deployment)
- [Cloud Deployment](#cloud-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Configuration Management](#configuration-management)
- [Monitoring Setup](#monitoring-setup)
- [Backup and Recovery](#backup-and-recovery)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Development

```bash
# Clone repository
git clone https://github.com/llm-devops/llm-latency-lens.git
cd llm-latency-lens

# Create environment file
cp .env.example .env
# Edit .env with your API keys

# Start with Docker Compose
make up

# Or manually
docker-compose up -d
```

### Production

```bash
# Deploy with production configuration
./scripts/deploy.sh deploy

# Check status
./scripts/deploy.sh status
```

## Prerequisites

### Required Software

- **Docker**: >= 20.10
- **Docker Compose**: >= 2.0
- **Git**: >= 2.30
- **Make**: >= 4.3 (optional, for convenience)

### API Keys

You'll need API keys from:
- [OpenAI](https://platform.openai.com/api-keys)
- [Anthropic](https://console.anthropic.com/settings/keys)
- [Google Cloud (Vertex AI)](https://console.cloud.google.com/apis/credentials)

### System Requirements

#### Minimum
- **CPU**: 2 cores
- **Memory**: 2GB RAM
- **Disk**: 10GB

#### Recommended
- **CPU**: 4+ cores
- **Memory**: 4GB+ RAM
- **Disk**: 50GB+ (for metrics storage)

## Local Development

### Using Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f llm-latency-lens

# Stop services
docker-compose down
```

### Using Make

```bash
# Show available commands
make help

# Start development environment
make up

# View logs
make logs

# Run tests
make test

# Stop everything
make down
```

### Without Docker

```bash
# Build from source
cargo build --release

# Run
./target/release/llm-latency-lens --help

# Start Prometheus separately
docker run -p 9090:9090 -v $(pwd)/monitoring/prometheus:/etc/prometheus prom/prometheus
```

## Production Deployment

### Using Deployment Script

The deployment script handles:
- ✅ Prerequisites checking
- ✅ Directory creation
- ✅ Automatic backups
- ✅ Image pulling
- ✅ Service orchestration
- ✅ Health checks

```bash
# Full deployment
./scripts/deploy.sh deploy

# Check status
./scripts/deploy.sh status

# View logs
./scripts/deploy.sh logs

# Stop services
./scripts/deploy.sh stop

# Rollback to previous version
./scripts/deploy.sh rollback
```

### Manual Production Deployment

```bash
# 1. Create environment file
cp .env.example .env
# Edit with production values

# 2. Create data directories
mkdir -p data/{prometheus,grafana,alertmanager}
chmod -R 755 data/

# 3. Pull images
docker-compose -f docker-compose.yml -f docker-compose.prod.yml pull

# 4. Start services
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# 5. Verify health
docker-compose ps
curl http://localhost:9090/metrics
```

### Production Configuration

Edit `docker-compose.prod.yml` for:

```yaml
# Resource limits
deploy:
  resources:
    limits:
      cpus: '4'
      memory: 2G

# Restart policy
restart_policy:
  condition: on-failure
  max_attempts: 3

# Security
security_opt:
  - no-new-privileges:true
read_only: true
```

## Cloud Deployment

### AWS EC2

```bash
# 1. Launch EC2 instance (t3.medium or larger)
aws ec2 run-instances \
  --image-id ami-xxxxx \
  --instance-type t3.medium \
  --key-name your-key \
  --security-groups llm-latency-lens-sg

# 2. SSH to instance
ssh -i your-key.pem ubuntu@ec2-instance

# 3. Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# 4. Clone and deploy
git clone https://github.com/llm-devops/llm-latency-lens.git
cd llm-latency-lens
./scripts/deploy.sh deploy
```

### AWS ECS

```yaml
# task-definition.json
{
  "family": "llm-latency-lens",
  "containerDefinitions": [
    {
      "name": "app",
      "image": "llm-devops/llm-latency-lens:latest",
      "memory": 2048,
      "cpu": 1024,
      "essential": true,
      "portMappings": [
        {
          "containerPort": 9090,
          "protocol": "tcp"
        }
      ],
      "secrets": [
        {
          "name": "OPENAI_API_KEY",
          "valueFrom": "arn:aws:secretsmanager:region:account:secret:openai-key"
        }
      ]
    }
  ]
}
```

### Google Cloud Run

```bash
# Build and push to GCR
docker build -t gcr.io/PROJECT-ID/llm-latency-lens .
docker push gcr.io/PROJECT-ID/llm-latency-lens

# Deploy to Cloud Run
gcloud run deploy llm-latency-lens \
  --image gcr.io/PROJECT-ID/llm-latency-lens \
  --platform managed \
  --region us-central1 \
  --memory 2Gi \
  --cpu 2 \
  --set-secrets OPENAI_API_KEY=openai-key:latest
```

### Azure Container Instances

```bash
# Create resource group
az group create --name llm-latency-lens-rg --location eastus

# Deploy container
az container create \
  --resource-group llm-latency-lens-rg \
  --name llm-latency-lens \
  --image llm-devops/llm-latency-lens:latest \
  --cpu 2 \
  --memory 2 \
  --ports 9090 \
  --environment-variables \
    RUST_LOG=info \
  --secure-environment-variables \
    OPENAI_API_KEY=$OPENAI_API_KEY
```

### DigitalOcean App Platform

```yaml
# .do/app.yaml
name: llm-latency-lens
services:
  - name: app
    image:
      registry_type: DOCKER_HUB
      registry: llm-devops
      repository: llm-latency-lens
      tag: latest
    instance_count: 1
    instance_size_slug: professional-xs
    routes:
      - path: /
    envs:
      - key: RUST_LOG
        value: info
      - key: OPENAI_API_KEY
        type: SECRET
        value: ${OPENAI_API_KEY}
```

## Kubernetes Deployment

### Using Helm

```bash
# Add Helm repository
helm repo add llm-latency-lens https://charts.llm-latency-lens.io
helm repo update

# Install
helm install llm-latency-lens llm-latency-lens/llm-latency-lens \
  --set secrets.openaiApiKey=$OPENAI_API_KEY \
  --set resources.limits.memory=2Gi \
  --set resources.limits.cpu=2
```

### Manual Kubernetes Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-latency-lens
spec:
  replicas: 3
  selector:
    matchLabels:
      app: llm-latency-lens
  template:
    metadata:
      labels:
        app: llm-latency-lens
    spec:
      containers:
      - name: llm-latency-lens
        image: llm-devops/llm-latency-lens:latest
        ports:
        - containerPort: 9090
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2"
        env:
        - name: RUST_LOG
          value: "info"
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: llm-secrets
              key: openai-api-key
---
apiVersion: v1
kind: Service
metadata:
  name: llm-latency-lens
spec:
  selector:
    app: llm-latency-lens
  ports:
  - port: 9090
    targetPort: 9090
  type: LoadBalancer
```

Apply:

```bash
kubectl apply -f deployment.yaml
```

## Configuration Management

### Using Docker Secrets

```bash
# Create secrets
echo "sk-..." | docker secret create openai_api_key -
echo "sk-ant-..." | docker secret create anthropic_api_key -

# Use in docker-compose.yml
services:
  llm-latency-lens:
    secrets:
      - openai_api_key
      - anthropic_api_key

secrets:
  openai_api_key:
    external: true
  anthropic_api_key:
    external: true
```

### Using Vault

```bash
# Store in Vault
vault kv put secret/llm-latency-lens \
  openai_api_key="sk-..." \
  anthropic_api_key="sk-ant-..."

# Retrieve in application
export OPENAI_API_KEY=$(vault kv get -field=openai_api_key secret/llm-latency-lens)
```

### Using AWS Secrets Manager

```bash
# Store secret
aws secretsmanager create-secret \
  --name llm-latency-lens/openai-key \
  --secret-string "sk-..."

# Retrieve in ECS task definition
{
  "secrets": [
    {
      "name": "OPENAI_API_KEY",
      "valueFrom": "arn:aws:secretsmanager:region:account:secret:llm-latency-lens/openai-key"
    }
  ]
}
```

## Monitoring Setup

### Grafana Configuration

```bash
# Access Grafana
open http://localhost:3000

# Login
Username: admin
Password: admin (change on first login)

# Add dashboards
1. Navigate to Dashboards
2. Import dashboard
3. Use file: monitoring/grafana/dashboards/llm-latency-overview.json
```

### Prometheus Configuration

```yaml
# monitoring/prometheus/prometheus.yml
scrape_configs:
  - job_name: 'llm-latency-lens'
    static_configs:
      - targets: ['llm-latency-lens:9090']
```

### Alert Configuration

```yaml
# monitoring/prometheus/alerts.yml
- alert: HighLatency
  expr: llm_request_duration_seconds{quantile="0.95"} > 5
  for: 5m
  annotations:
    summary: "High latency detected"
```

### External Monitoring

#### Datadog

```yaml
# datadog-agent configuration
docker_labels_as_tags:
  "com.docker.compose.service": "service"

env:
  - DD_API_KEY=${DATADOG_API_KEY}
  - DD_PROMETHEUS_SCRAPE_ENABLED=true
  - DD_PROMETHEUS_SCRAPE_SERVICE_ENDPOINTS=true
```

#### New Relic

```yaml
environment:
  - NEW_RELIC_LICENSE_KEY=${NEW_RELIC_KEY}
  - NEW_RELIC_APP_NAME=llm-latency-lens
```

## Backup and Recovery

### Automated Backups

```bash
# Create backup script
cat > /usr/local/bin/backup-llm-latency-lens.sh << 'EOF'
#!/bin/bash
BACKUP_DIR=/backups
DATE=$(date +%Y%m%d_%H%M%S)
tar -czf $BACKUP_DIR/backup_$DATE.tar.gz /data
find $BACKUP_DIR -mtime +7 -delete
EOF

chmod +x /usr/local/bin/backup-llm-latency-lens.sh

# Add to crontab
echo "0 2 * * * /usr/local/bin/backup-llm-latency-lens.sh" | crontab -
```

### Manual Backup

```bash
# Backup data
./scripts/deploy.sh backup

# Or manually
tar -czf backup.tar.gz data/
```

### Recovery

```bash
# Stop services
./scripts/deploy.sh stop

# Restore backup
tar -xzf backup.tar.gz

# Start services
./scripts/deploy.sh start
```

## Troubleshooting

### Services Won't Start

```bash
# Check logs
docker-compose logs

# Check resource usage
docker stats

# Check disk space
df -h
```

### High Memory Usage

```bash
# Adjust resource limits in docker-compose.prod.yml
deploy:
  resources:
    limits:
      memory: 4G
```

### Metrics Not Appearing

```bash
# Check Prometheus targets
curl http://localhost:9091/targets

# Check metrics endpoint
curl http://localhost:9090/metrics

# Restart Prometheus
docker-compose restart prometheus
```

### Connection Issues

```bash
# Check network
docker network inspect llm-latency-lens_llm-monitoring

# Test connectivity
docker exec llm-latency-lens ping prometheus
```

## Additional Resources

- [Docker Documentation](docs/DOCKER.md)
- [CI/CD Documentation](docs/CI-CD.md)
- [Main README](../README.md)
- [GitHub Repository](https://github.com/llm-devops/llm-latency-lens)
