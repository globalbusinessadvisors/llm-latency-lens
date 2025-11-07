# LLM-Latency-Lens: Comprehensive Deployment Strategy

## Executive Summary

This document defines the deployment topologies, operational modes, and integration patterns for LLM-Latency-Lens, a high-performance latency profiling tool for Large Language Models. The profiler supports five primary deployment modes, each optimized for different use cases and operational requirements.

---

## Table of Contents

1. [Deployment Mode 1: Standalone CLI](#1-standalone-cli)
2. [Deployment Mode 2: CI/CD Integration](#2-cicd-integration)
3. [Deployment Mode 3: Embedded Library Mode](#3-embedded-library-mode)
4. [Deployment Mode 4: Distributed Execution](#4-distributed-execution)
5. [Deployment Mode 5: Observatory Integration](#5-observatory-integration)
6. [Cross-Cutting Concerns](#6-cross-cutting-concerns)
7. [Migration Paths](#7-migration-paths)
8. [Reference Architectures](#8-reference-architectures)

---

## 1. Standalone CLI

### 1.1 Overview

The standalone CLI mode provides a self-contained, zero-dependency executable for local latency profiling and analysis.

### 1.2 Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Local Environment                         │
│                                                              │
│  ┌──────────────┐         ┌──────────────┐                 │
│  │   Terminal   │────────▶│  llm-lens    │                 │
│  │              │         │     CLI      │                 │
│  └──────────────┘         └──────┬───────┘                 │
│                                   │                          │
│                           ┌───────▼────────┐                │
│                           │  Config Loader │                │
│                           │  (YAML/TOML)   │                │
│                           └───────┬────────┘                │
│                                   │                          │
│               ┌───────────────────┼───────────────────┐     │
│               │                   │                   │     │
│         ┌─────▼─────┐      ┌─────▼─────┐      ┌─────▼─────┐│
│         │ LLM APIs  │      │ Metrics   │      │  Report   ││
│         │ (OpenAI,  │      │ Collector │      │ Generator ││
│         │ Anthropic)│      └─────┬─────┘      └─────┬─────┘│
│         └───────────┘            │                   │      │
│                                  │                   │      │
│                        ┌─────────▼───────────────────▼─────┐│
│                        │      Output Destinations          ││
│                        │  • stdout/stderr                  ││
│                        │  • JSON/CSV files                 ││
│                        │  • SQLite database                ││
│                        │  • Prometheus metrics             ││
│                        └───────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### 1.3 Installation Methods

#### 1.3.1 Cargo Install (Preferred for Rust Users)

```bash
# Install from crates.io
cargo install llm-latency-lens

# Install from source
git clone https://github.com/your-org/llm-latency-lens
cd llm-latency-lens
cargo install --path .

# Install with specific features
cargo install llm-latency-lens --features "distributed,prometheus"
```

#### 1.3.2 Binary Releases

```bash
# Linux x86_64
curl -L https://github.com/your-org/llm-latency-lens/releases/latest/download/llm-lens-linux-x64 -o llm-lens
chmod +x llm-lens
sudo mv llm-lens /usr/local/bin/

# macOS ARM64
curl -L https://github.com/your-org/llm-latency-lens/releases/latest/download/llm-lens-darwin-arm64 -o llm-lens
chmod +x llm-lens
sudo mv llm-lens /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/your-org/llm-latency-lens/releases/latest/download/llm-lens-windows.exe" -OutFile "llm-lens.exe"
```

#### 1.3.3 Package Managers

```bash
# Homebrew (macOS/Linux)
brew install llm-latency-lens

# Scoop (Windows)
scoop bucket add llm-tools https://github.com/your-org/scoop-bucket
scoop install llm-latency-lens

# APT (Debian/Ubuntu)
sudo add-apt-repository ppa:llm-tools/stable
sudo apt update
sudo apt install llm-latency-lens

# Docker
docker pull ghcr.io/your-org/llm-latency-lens:latest
```

#### 1.3.4 Container Image

```dockerfile
# Dockerfile for standalone usage
FROM rust:1.75-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin llm-lens

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder /app/target/release/llm-lens /usr/local/bin/
ENTRYPOINT ["llm-lens"]
```

### 1.4 Configuration File Formats

#### 1.4.1 YAML Configuration (llm-lens.yaml)

```yaml
# llm-lens.yaml - Primary configuration format
version: "1.0"

# Profiler settings
profiler:
  name: "production-benchmark"
  mode: "latency"  # latency, throughput, cost, comprehensive
  duration: 300s
  warmup: 30s
  cooldown: 10s

# Target LLM providers
providers:
  - name: openai
    enabled: true
    api_key: "${OPENAI_API_KEY}"  # Environment variable substitution
    endpoint: "https://api.openai.com/v1"
    models:
      - id: "gpt-4-turbo-preview"
        max_tokens: 1024
        temperature: 0.7
      - id: "gpt-3.5-turbo"
        max_tokens: 1024
        temperature: 0.7

  - name: anthropic
    enabled: true
    api_key: "${ANTHROPIC_API_KEY}"
    endpoint: "https://api.anthropic.com/v1"
    models:
      - id: "claude-3-opus-20240229"
        max_tokens: 1024
        temperature: 0.7
      - id: "claude-3-sonnet-20240229"
        max_tokens: 1024
        temperature: 0.7

# Request patterns
workload:
  type: "mixed"  # fixed, random, poisson, replay

  # Fixed rate configuration
  fixed:
    requests_per_second: 10

  # Poisson distribution
  poisson:
    lambda: 15

  # Request replay from logs
  replay:
    source: "/path/to/production-logs.json"
    time_scale: 1.0  # 1.0 = real-time, 2.0 = 2x speed

  # Prompt templates
  prompts:
    - template: "Summarize the following text: {{text}}"
      weight: 0.4
      variables:
        text:
          type: "file"
          source: "prompts/summaries.txt"

    - template: "Translate to {{language}}: {{text}}"
      weight: 0.3
      variables:
        language: ["French", "Spanish", "German"]
        text:
          type: "generator"
          generator: "lorem"

    - template: "{{question}}"
      weight: 0.3
      variables:
        question:
          type: "file"
          source: "prompts/questions.json"

# Metrics collection
metrics:
  collect:
    - latency  # TTFT, inter-token, total
    - throughput  # tokens/sec, requests/sec
    - cost  # per request, per token
    - errors  # rate, types, retries
    - token_usage  # input, output, total

  percentiles: [50, 75, 90, 95, 99, 99.9]

  # Time series granularity
  resolution: 1s

  # Statistical aggregation
  aggregation:
    window: 60s
    method: "sliding"  # sliding, tumbling, session

# Output destinations
output:
  # Console output
  console:
    enabled: true
    format: "table"  # table, json, compact
    colors: true
    live_updates: true

  # File outputs
  files:
    - type: "json"
      path: "results/benchmark-{{timestamp}}.json"
      pretty: true

    - type: "csv"
      path: "results/metrics-{{timestamp}}.csv"
      delimiter: ","

    - type: "markdown"
      path: "results/report-{{timestamp}}.md"
      template: "templates/report.md.tmpl"

  # Database storage
  database:
    enabled: false
    type: "sqlite"  # sqlite, postgres, mysql
    connection: "benchmark-results.db"
    table_prefix: "llm_lens_"

  # Metrics export
  prometheus:
    enabled: false
    port: 9090
    path: "/metrics"

  # Remote storage
  remote:
    - type: "s3"
      enabled: false
      bucket: "llm-benchmarks"
      prefix: "results/"
      region: "us-east-1"

    - type: "gcs"
      enabled: false
      bucket: "llm-benchmarks"
      prefix: "results/"

# Retry and error handling
resilience:
  retries:
    max_attempts: 3
    backoff: "exponential"  # fixed, exponential, fibonacci
    initial_delay: 1s
    max_delay: 30s

  timeouts:
    connect: 10s
    request: 120s
    idle: 30s

  circuit_breaker:
    enabled: true
    failure_threshold: 5
    success_threshold: 2
    timeout: 60s

# Logging configuration
logging:
  level: "info"  # trace, debug, info, warn, error
  format: "json"  # json, pretty, compact
  output: "stderr"

  # Log file rotation
  file:
    enabled: true
    path: "logs/llm-lens.log"
    max_size: "100MB"
    max_backups: 5
    compress: true
```

#### 1.4.2 TOML Configuration (llm-lens.toml)

```toml
# llm-lens.toml - Alternative configuration format
version = "1.0"

[profiler]
name = "production-benchmark"
mode = "latency"
duration = "300s"
warmup = "30s"
cooldown = "10s"

[[providers]]
name = "openai"
enabled = true
api_key = "${OPENAI_API_KEY}"
endpoint = "https://api.openai.com/v1"

[[providers.models]]
id = "gpt-4-turbo-preview"
max_tokens = 1024
temperature = 0.7

[[providers]]
name = "anthropic"
enabled = true
api_key = "${ANTHROPIC_API_KEY}"
endpoint = "https://api.anthropic.com/v1"

[[providers.models]]
id = "claude-3-opus-20240229"
max_tokens = 1024
temperature = 0.7

[workload]
type = "mixed"

[workload.fixed]
requests_per_second = 10

[workload.poisson]
lambda = 15

[[workload.prompts]]
template = "Summarize the following text: {{text}}"
weight = 0.4

[workload.prompts.variables.text]
type = "file"
source = "prompts/summaries.txt"

[metrics]
collect = ["latency", "throughput", "cost", "errors", "token_usage"]
percentiles = [50, 75, 90, 95, 99, 99.9]
resolution = "1s"

[metrics.aggregation]
window = "60s"
method = "sliding"

[output.console]
enabled = true
format = "table"
colors = true
live_updates = true

[[output.files]]
type = "json"
path = "results/benchmark-{{timestamp}}.json"
pretty = true

[output.database]
enabled = false
type = "sqlite"
connection = "benchmark-results.db"

[output.prometheus]
enabled = false
port = 9090
path = "/metrics"

[resilience.retries]
max_attempts = 3
backoff = "exponential"
initial_delay = "1s"
max_delay = "30s"

[resilience.timeouts]
connect = "10s"
request = "120s"
idle = "30s"

[resilience.circuit_breaker]
enabled = true
failure_threshold = 5
success_threshold = 2
timeout = "60s"

[logging]
level = "info"
format = "json"
output = "stderr"

[logging.file]
enabled = true
path = "logs/llm-lens.log"
max_size = "100MB"
max_backups = 5
compress = true
```

### 1.5 Command-Line Interface

#### 1.5.1 Basic Usage

```bash
# Run with default configuration
llm-lens run

# Run with custom config
llm-lens run --config llm-lens.yaml

# Quick benchmark with CLI options
llm-lens run \
  --provider openai \
  --model gpt-4-turbo-preview \
  --duration 60s \
  --rps 10 \
  --output results.json

# Compare multiple models
llm-lens compare \
  --models gpt-4-turbo-preview,claude-3-opus-20240229,gemini-pro \
  --duration 300s \
  --output comparison.json

# Analyze existing results
llm-lens analyze results/benchmark-*.json \
  --output report.html \
  --format html

# Generate configuration template
llm-lens init --format yaml > llm-lens.yaml
llm-lens init --format toml > llm-lens.toml
```

#### 1.5.2 Advanced CLI Commands

```bash
# Validate configuration
llm-lens validate --config llm-lens.yaml

# Dry run (validate without executing)
llm-lens run --config llm-lens.yaml --dry-run

# Profile specific endpoint
llm-lens profile \
  --url https://api.custom-llm.com/v1/chat \
  --auth "Bearer ${API_KEY}" \
  --payload-template request.json \
  --duration 120s

# Export metrics to Prometheus
llm-lens export prometheus \
  --input results.json \
  --output metrics.prom

# Replay production traffic
llm-lens replay \
  --log production-requests.jsonl \
  --time-scale 2.0 \
  --output replay-results.json

# Cost analysis
llm-lens cost \
  --input results.json \
  --pricing config/pricing.yaml \
  --currency USD

# Generate report
llm-lens report \
  --input results/*.json \
  --template templates/executive-summary.html \
  --output report.html
```

### 1.6 Output Formats

#### 1.6.1 JSON Output

```json
{
  "benchmark": {
    "id": "bench_1234567890",
    "name": "production-benchmark",
    "start_time": "2024-01-15T10:00:00Z",
    "end_time": "2024-01-15T10:05:00Z",
    "duration_seconds": 300,
    "config": { /* configuration snapshot */ }
  },
  "providers": [
    {
      "name": "openai",
      "models": [
        {
          "id": "gpt-4-turbo-preview",
          "requests": {
            "total": 3000,
            "successful": 2985,
            "failed": 15,
            "retried": 23
          },
          "latency": {
            "ttft": {
              "min_ms": 245,
              "max_ms": 1823,
              "mean_ms": 456,
              "median_ms": 423,
              "p95_ms": 789,
              "p99_ms": 1234,
              "stddev_ms": 156
            },
            "total": {
              "min_ms": 1234,
              "max_ms": 15678,
              "mean_ms": 3456,
              "median_ms": 3234,
              "p95_ms": 6789,
              "p99_ms": 9876,
              "stddev_ms": 876
            },
            "inter_token": {
              "mean_ms": 45,
              "median_ms": 42,
              "p95_ms": 78,
              "p99_ms": 123
            }
          },
          "throughput": {
            "requests_per_second": 9.95,
            "tokens_per_second": 234.5,
            "input_tokens_per_second": 89.3,
            "output_tokens_per_second": 145.2
          },
          "tokens": {
            "total": 70350,
            "input": 26790,
            "output": 43560,
            "average_per_request": 23.45
          },
          "cost": {
            "total_usd": 4.56,
            "input_usd": 0.89,
            "output_usd": 3.67,
            "average_per_request_usd": 0.00152
          },
          "errors": [
            {
              "type": "rate_limit",
              "count": 8,
              "percentage": 0.27
            },
            {
              "type": "timeout",
              "count": 5,
              "percentage": 0.17
            },
            {
              "type": "api_error",
              "count": 2,
              "percentage": 0.07
            }
          ],
          "time_series": [
            {
              "timestamp": "2024-01-15T10:00:00Z",
              "rps": 10.2,
              "tokens_per_second": 235.6,
              "mean_latency_ms": 445,
              "p95_latency_ms": 756,
              "error_rate": 0.0
            }
            // ... more data points
          ]
        }
      ]
    }
  ],
  "summary": {
    "total_requests": 6000,
    "total_tokens": 140700,
    "total_cost_usd": 9.12,
    "average_latency_ms": 3567,
    "fastest_model": "gpt-3.5-turbo",
    "most_cost_effective": "claude-3-haiku-20240307"
  }
}
```

#### 1.6.2 CSV Output

```csv
timestamp,provider,model,request_id,ttft_ms,total_latency_ms,input_tokens,output_tokens,cost_usd,status,error_type
2024-01-15T10:00:00.123Z,openai,gpt-4-turbo-preview,req_001,456,3234,89,145,0.00152,success,
2024-01-15T10:00:01.234Z,openai,gpt-4-turbo-preview,req_002,423,3156,92,142,0.00148,success,
2024-01-15T10:00:02.345Z,anthropic,claude-3-opus-20240229,req_003,389,2987,87,138,0.00134,success,
2024-01-15T10:00:03.456Z,openai,gpt-4-turbo-preview,req_004,,,89,0,0,failed,rate_limit
```

#### 1.6.3 Markdown Report

```markdown
# LLM Benchmark Report
**Generated:** 2024-01-15 10:05:00 UTC
**Duration:** 5 minutes

## Executive Summary
- Total Requests: 6,000
- Success Rate: 99.5%
- Average Latency: 3.57s
- Total Cost: $9.12

## Model Comparison

| Model | Avg Latency | P95 Latency | Tokens/s | Cost/1K Tokens | Success Rate |
|-------|-------------|-------------|----------|----------------|--------------|
| GPT-4 Turbo | 3.46s | 6.79s | 234.5 | $0.065 | 99.5% |
| Claude 3 Opus | 3.23s | 6.12s | 245.8 | $0.058 | 99.7% |
| Gemini Pro | 2.98s | 5.67s | 267.3 | $0.042 | 99.3% |

## Latency Distribution
```

### 1.7 Best Practices

#### 1.7.1 Configuration Management

1. **Environment-Specific Configs**: Maintain separate configs for dev, staging, production
2. **Secret Management**: Use environment variables or secret managers (never commit API keys)
3. **Version Control**: Track configuration files in git with sensitive data templated
4. **Validation**: Always validate configs before running benchmarks
5. **Defaults**: Provide sensible defaults for common use cases

#### 1.7.2 Local Development Workflow

```bash
# 1. Initialize configuration
llm-lens init --format yaml > llm-lens.dev.yaml

# 2. Edit configuration
vim llm-lens.dev.yaml

# 3. Validate configuration
llm-lens validate --config llm-lens.dev.yaml

# 4. Run dry run
llm-lens run --config llm-lens.dev.yaml --dry-run

# 5. Execute benchmark
llm-lens run --config llm-lens.dev.yaml

# 6. Analyze results
llm-lens analyze results/benchmark-*.json --output report.html
```

#### 1.7.3 Performance Tuning

1. **Request Parallelism**: Adjust based on target RPS and response latency
2. **Connection Pooling**: Reuse HTTP connections for better performance
3. **Batch Processing**: Group requests when supported by provider
4. **Resource Limits**: Monitor CPU, memory, network bandwidth
5. **Output Optimization**: Disable live updates for high-RPS scenarios

---

## 2. CI/CD Integration

### 2.1 Overview

Integrate LLM-Latency-Lens into continuous integration pipelines for automated performance testing, regression detection, and quality gates.

### 2.2 Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         CI/CD Pipeline                          │
│                                                                 │
│  ┌────────────┐      ┌────────────┐      ┌────────────┐      │
│  │   Code     │─────▶│   Build    │─────▶│   Test     │      │
│  │   Push     │      │   Stage    │      │   Stage    │      │
│  └────────────┘      └────────────┘      └─────┬──────┘      │
│                                                 │              │
│                                     ┌───────────▼──────────┐  │
│                                     │  Benchmark Stage     │  │
│                                     │                      │  │
│                                     │  ┌────────────────┐ │  │
│                                     │  │  llm-lens run  │ │  │
│                                     │  └────────┬───────┘ │  │
│                                     │           │         │  │
│                                     │  ┌────────▼───────┐ │  │
│                                     │  │ Result Parser  │ │  │
│                                     │  └────────┬───────┘ │  │
│                                     └───────────┼─────────┘  │
│                                                 │              │
│                          ┌──────────────────────┼─────┐       │
│                          │                      │     │       │
│                    ┌─────▼─────┐         ┌─────▼─────▼────┐ │
│                    │ Regression│         │  Performance   │ │
│                    │  Analysis │         │  Quality Gate  │ │
│                    └─────┬─────┘         └─────┬──────────┘ │
│                          │                      │             │
│                          │         ┌────────────▼─────┐      │
│                          │         │   Pass/Fail      │      │
│                          │         │   Decision       │      │
│                          │         └────────┬─────────┘      │
│                          │                  │                 │
│              ┌───────────▼──────────────────▼──────────┐     │
│              │         Reporting & Artifacts           │     │
│              │  • GitHub PR Comments                   │     │
│              │  • S3 Result Storage                    │     │
│              │  • Slack/Teams Notifications            │     │
│              │  • Dashboard Updates                    │     │
│              └─────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 GitHub Actions Integration

#### 2.3.1 Basic Workflow

```yaml
# .github/workflows/llm-benchmark.yml
name: LLM Performance Benchmark

on:
  pull_request:
    branches: [main, develop]
    paths:
      - 'src/**'
      - 'models/**'
      - '.github/workflows/llm-benchmark.yml'

  push:
    branches: [main]

  schedule:
    # Run daily at 2 AM UTC
    - cron: '0 2 * * *'

  workflow_dispatch:
    inputs:
      duration:
        description: 'Benchmark duration (seconds)'
        required: false
        default: '300'
      models:
        description: 'Comma-separated model list'
        required: false
        default: 'gpt-4-turbo-preview,claude-3-opus-20240229'

jobs:
  benchmark:
    name: Run LLM Benchmark
    runs-on: ubuntu-latest
    timeout-minutes: 30

    permissions:
      pull-requests: write
      contents: read

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal
          override: true

      - name: Cache Cargo dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install llm-latency-lens
        run: |
          cargo install llm-latency-lens --version 0.1.0

      - name: Download baseline results
        uses: actions/download-artifact@v3
        with:
          name: baseline-results
          path: baseline/
        continue-on-error: true

      - name: Run benchmark
        id: benchmark
        env:
          OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
          GOOGLE_API_KEY: ${{ secrets.GOOGLE_API_KEY }}
        run: |
          llm-lens run \
            --config .github/benchmark-config.yaml \
            --duration ${{ github.event.inputs.duration || '60' }}s \
            --output results/benchmark-${{ github.run_id }}.json \
            --format json

      - name: Analyze results
        id: analysis
        run: |
          llm-lens analyze results/benchmark-${{ github.run_id }}.json \
            --baseline baseline/baseline.json \
            --threshold 10 \
            --output results/analysis.json

          # Extract key metrics for PR comment
          MEAN_LATENCY=$(jq -r '.summary.average_latency_ms' results/analysis.json)
          P95_LATENCY=$(jq -r '.summary.p95_latency_ms' results/analysis.json)
          REGRESSION=$(jq -r '.regression.detected' results/analysis.json)

          echo "mean_latency=${MEAN_LATENCY}" >> $GITHUB_OUTPUT
          echo "p95_latency=${P95_LATENCY}" >> $GITHUB_OUTPUT
          echo "regression=${REGRESSION}" >> $GITHUB_OUTPUT

      - name: Generate report
        run: |
          llm-lens report \
            --input results/benchmark-${{ github.run_id }}.json \
            --format markdown \
            --output results/report.md

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results-${{ github.run_id }}
          path: results/
          retention-days: 30

      - name: Store in S3
        if: github.ref == 'refs/heads/main'
        run: |
          aws s3 cp results/benchmark-${{ github.run_id }}.json \
            s3://llm-benchmarks/results/${{ github.sha }}.json
        env:
          AWS_ACCESS_KEY_ID: ${{ secrets.AWS_ACCESS_KEY_ID }}
          AWS_SECRET_ACCESS_KEY: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          AWS_REGION: us-east-1

      - name: Comment PR with results
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('results/report.md', 'utf8');

            const comment = `## LLM Performance Benchmark Results

            **Commit:** ${{ github.sha }}
            **Duration:** ${{ github.event.inputs.duration || '60' }}s

            ### Key Metrics
            - Mean Latency: ${{ steps.analysis.outputs.mean_latency }}ms
            - P95 Latency: ${{ steps.analysis.outputs.p95_latency }}ms
            - Regression Detected: ${{ steps.analysis.outputs.regression }}

            ${report}

            <details>
            <summary>View full results</summary>

            Download artifact: benchmark-results-${{ github.run_id }}
            </details>
            `;

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: comment
            });

      - name: Performance quality gate
        if: steps.analysis.outputs.regression == 'true'
        run: |
          echo "::error::Performance regression detected!"
          echo "::error::Mean latency increased beyond threshold"
          exit 1

      - name: Send Slack notification
        if: always()
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {
              "text": "LLM Benchmark ${{ job.status }}",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": "*LLM Benchmark Results*\n*Status:* ${{ job.status }}\n*Branch:* ${{ github.ref }}\n*Commit:* ${{ github.sha }}"
                  }
                },
                {
                  "type": "section",
                  "fields": [
                    {
                      "type": "mrkdwn",
                      "text": "*Mean Latency:*\n${{ steps.analysis.outputs.mean_latency }}ms"
                    },
                    {
                      "type": "mrkdwn",
                      "text": "*P95 Latency:*\n${{ steps.analysis.outputs.p95_latency }}ms"
                    }
                  ]
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
```

#### 2.3.2 Baseline Management Workflow

```yaml
# .github/workflows/baseline-update.yml
name: Update Performance Baseline

on:
  workflow_dispatch:
  schedule:
    # Update baseline weekly
    - cron: '0 3 * * 0'

jobs:
  update-baseline:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install llm-latency-lens
        run: cargo install llm-latency-lens

      - name: Run comprehensive benchmark
        env:
          OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
        run: |
          llm-lens run \
            --config .github/baseline-config.yaml \
            --duration 600s \
            --output baseline/baseline.json

      - name: Validate baseline
        run: |
          llm-lens validate baseline \
            --input baseline/baseline.json \
            --min-requests 1000 \
            --max-error-rate 1.0

      - name: Upload baseline artifact
        uses: actions/upload-artifact@v3
        with:
          name: baseline-results
          path: baseline/baseline.json
          retention-days: 90

      - name: Commit baseline
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add baseline/baseline.json
          git commit -m "Update performance baseline [skip ci]"
          git push
```

### 2.4 GitLab CI Integration

#### 2.4.1 GitLab CI Pipeline

```yaml
# .gitlab-ci.yml
variables:
  BENCHMARK_DURATION: "300"
  CARGO_HOME: "${CI_PROJECT_DIR}/.cargo"

stages:
  - build
  - test
  - benchmark
  - report
  - deploy

cache:
  key: ${CI_COMMIT_REF_SLUG}
  paths:
    - .cargo/
    - target/

.benchmark_template: &benchmark_template
  stage: benchmark
  image: rust:1.75
  timeout: 30 minutes
  before_script:
    - cargo install llm-latency-lens --version 0.1.0
  artifacts:
    paths:
      - results/
    reports:
      junit: results/junit.xml
    expire_in: 30 days

benchmark:development:
  <<: *benchmark_template
  script:
    - |
      llm-lens run \
        --config ci/benchmark-dev.yaml \
        --duration ${BENCHMARK_DURATION}s \
        --output results/benchmark-${CI_COMMIT_SHA}.json

    - llm-lens report --input results/*.json --format junit --output results/junit.xml
  only:
    - merge_requests
    - develop
  except:
    variables:
      - $CI_COMMIT_MESSAGE =~ /\[skip benchmark\]/

benchmark:production:
  <<: *benchmark_template
  script:
    - |
      llm-lens run \
        --config ci/benchmark-prod.yaml \
        --duration 600s \
        --output results/benchmark-${CI_COMMIT_SHA}.json

    - |
      llm-lens analyze results/benchmark-${CI_COMMIT_SHA}.json \
        --baseline results/baseline.json \
        --threshold 5 \
        --output results/analysis.json

    - |
      REGRESSION=$(jq -r '.regression.detected' results/analysis.json)
      if [ "$REGRESSION" = "true" ]; then
        echo "Performance regression detected!"
        exit 1
      fi
  only:
    - main
    - tags
  environment:
    name: production
    url: https://benchmarks.example.com

benchmark:scheduled:
  <<: *benchmark_template
  script:
    - |
      llm-lens run \
        --config ci/benchmark-comprehensive.yaml \
        --duration 1800s \
        --output results/benchmark-scheduled-$(date +%Y%m%d).json

    - |
      aws s3 cp results/benchmark-scheduled-$(date +%Y%m%d).json \
        s3://llm-benchmarks/scheduled/
  only:
    - schedules

report:generate:
  stage: report
  image: node:20
  script:
    - npm install -g @llm-tools/report-generator
    - |
      llm-report generate \
        --input results/*.json \
        --output public/index.html \
        --template templates/gitlab-pages.html
  artifacts:
    paths:
      - public
  only:
    - main

pages:
  stage: deploy
  script:
    - echo "Deploying benchmark results to GitLab Pages"
  artifacts:
    paths:
      - public
  only:
    - main
```

### 2.5 Regression Detection

#### 2.5.1 Statistical Regression Analysis

```yaml
# regression-config.yaml
regression:
  # Statistical methods
  methods:
    - type: "threshold"
      metric: "mean_latency"
      threshold_percent: 10.0  # Fail if >10% increase

    - type: "threshold"
      metric: "p95_latency"
      threshold_percent: 15.0

    - type: "threshold"
      metric: "error_rate"
      threshold_percent: 50.0  # Fail if >50% increase in errors

    - type: "statistical"
      method: "t-test"  # t-test, mann-whitney, kolmogorov-smirnov
      confidence: 0.95
      metric: "latency_distribution"

    - type: "trend"
      method: "linear_regression"
      window: 10  # Last 10 benchmarks
      max_slope: 0.05  # Max 5% increase per benchmark

  # Baseline comparison
  baseline:
    source: "s3://llm-benchmarks/baseline/main.json"
    update_frequency: "weekly"
    min_sample_size: 1000

  # Quality gates
  quality_gates:
    - name: "latency_gate"
      condition: "mean_latency < 5000"
      severity: "error"

    - name: "cost_gate"
      condition: "total_cost_usd < 100.0"
      severity: "warning"

    - name: "success_rate_gate"
      condition: "success_rate > 99.0"
      severity: "error"

  # Notification rules
  notifications:
    - type: "slack"
      webhook: "${SLACK_WEBHOOK_URL}"
      conditions:
        - "regression_detected == true"
        - "severity == 'error'"

    - type: "pagerduty"
      integration_key: "${PAGERDUTY_KEY}"
      conditions:
        - "regression_detected == true"
        - "severity == 'error'"
        - "branch == 'main'"
```

#### 2.5.2 CLI Usage for Regression Detection

```bash
# Compare against baseline
llm-lens analyze results/current.json \
  --baseline baseline/main.json \
  --threshold 10 \
  --output analysis.json

# Trend analysis
llm-lens trend \
  --input results/benchmark-*.json \
  --metric mean_latency \
  --method linear_regression \
  --output trend.json

# Quality gate evaluation
llm-lens gate \
  --input results/current.json \
  --config regression-config.yaml \
  --fail-on-error
```

### 2.6 Result Storage Strategies

#### 2.6.1 S3 Storage Structure

```
s3://llm-benchmarks/
├── baselines/
│   ├── main.json
│   ├── develop.json
│   └── release-v1.0.json
├── results/
│   ├── 2024/
│   │   ├── 01/
│   │   │   ├── 15/
│   │   │   │   ├── commit-abc123.json
│   │   │   │   ├── commit-def456.json
│   │   │   │   └── scheduled-20240115.json
│   │   │   └── 16/
│   │   └── 02/
│   └── index.json  # Metadata index
├── reports/
│   └── 2024/
│       └── week-03/
│           ├── summary.html
│           └── detailed.pdf
└── trends/
    ├── latency-trend.json
    ├── cost-trend.json
    └── error-rate-trend.json
```

#### 2.6.2 Database Schema (PostgreSQL)

```sql
-- benchmarks table
CREATE TABLE benchmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    commit_sha VARCHAR(40),
    branch VARCHAR(255),
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    duration_seconds INTEGER NOT NULL,
    config JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- results table
CREATE TABLE results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    benchmark_id UUID REFERENCES benchmarks(id),
    provider VARCHAR(100) NOT NULL,
    model VARCHAR(255) NOT NULL,
    request_count INTEGER NOT NULL,
    success_count INTEGER NOT NULL,
    error_count INTEGER NOT NULL,
    mean_latency_ms DECIMAL(10,2),
    median_latency_ms DECIMAL(10,2),
    p95_latency_ms DECIMAL(10,2),
    p99_latency_ms DECIMAL(10,2),
    ttft_mean_ms DECIMAL(10,2),
    ttft_p95_ms DECIMAL(10,2),
    tokens_per_second DECIMAL(10,2),
    total_tokens BIGINT,
    total_cost_usd DECIMAL(10,4),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- time_series table
CREATE TABLE time_series (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    result_id UUID REFERENCES results(id),
    timestamp TIMESTAMPTZ NOT NULL,
    rps DECIMAL(10,2),
    tokens_per_second DECIMAL(10,2),
    mean_latency_ms DECIMAL(10,2),
    p95_latency_ms DECIMAL(10,2),
    error_rate DECIMAL(5,4),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_benchmarks_commit ON benchmarks(commit_sha);
CREATE INDEX idx_benchmarks_branch ON benchmarks(branch);
CREATE INDEX idx_benchmarks_start_time ON benchmarks(start_time);
CREATE INDEX idx_results_benchmark ON results(benchmark_id);
CREATE INDEX idx_results_model ON results(model);
CREATE INDEX idx_time_series_result ON time_series(result_id);
CREATE INDEX idx_time_series_timestamp ON time_series(timestamp);

-- Query examples
-- Get latest benchmark for branch
SELECT * FROM benchmarks
WHERE branch = 'main'
ORDER BY start_time DESC
LIMIT 1;

-- Compare model performance
SELECT
    model,
    AVG(mean_latency_ms) as avg_latency,
    AVG(p95_latency_ms) as avg_p95,
    AVG(tokens_per_second) as avg_throughput
FROM results
WHERE benchmark_id IN (
    SELECT id FROM benchmarks
    WHERE start_time > NOW() - INTERVAL '7 days'
)
GROUP BY model
ORDER BY avg_latency ASC;

-- Trend analysis
SELECT
    DATE(b.start_time) as date,
    r.model,
    AVG(r.mean_latency_ms) as daily_avg_latency
FROM benchmarks b
JOIN results r ON b.id = r.benchmark_id
WHERE b.start_time > NOW() - INTERVAL '30 days'
GROUP BY DATE(b.start_time), r.model
ORDER BY date DESC, r.model;
```

### 2.7 Best Practices

#### 2.7.1 CI/CD Integration Guidelines

1. **Run on Relevant Changes**: Only trigger benchmarks when code changes affect performance
2. **Parallel Execution**: Run benchmarks for different models in parallel when possible
3. **Fail Fast**: Set reasonable timeouts and early exit conditions
4. **Baseline Management**: Regularly update baselines on main/release branches
5. **Cost Control**: Limit benchmark duration for PRs, full runs for main branch only
6. **Result Retention**: Store results for trend analysis and debugging
7. **Clear Communication**: Provide actionable feedback in PR comments

#### 2.7.2 Performance Budget Example

```yaml
# performance-budget.yaml
budgets:
  - metric: "mean_latency"
    budget: 3000  # ms
    baseline: "main"
    variance: 10  # percent

  - metric: "p95_latency"
    budget: 6000  # ms
    baseline: "main"
    variance: 15  # percent

  - metric: "cost_per_1k_tokens"
    budget: 0.10  # USD
    baseline: "main"
    variance: 5  # percent

  - metric: "error_rate"
    budget: 1.0  # percent
    baseline: "main"
    variance: 50  # percent increase allowed
```

---

## 3. Embedded Library Mode

### 3.1 Overview

Embed LLM-Latency-Lens as a Rust library within existing services for in-process profiling and metrics collection.

### 3.2 Architecture Diagram

```
┌───────────────────────────────────────────────────────────┐
│               Your Application (Rust)                     │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │           Application Code                      │    │
│  │                                                  │    │
│  │  ┌────────────────────────────────────────┐    │    │
│  │  │      LLM Client Integration            │    │    │
│  │  │                                         │    │    │
│  │  │  use llm_latency_lens::Profiler;      │    │    │
│  │  │                                         │    │    │
│  │  │  let profiler = Profiler::new();      │    │    │
│  │  │  let response = profiler              │    │    │
│  │  │    .profile(llm_request)              │    │    │
│  │  │    .await?;                           │    │    │
│  │  └────────────┬───────────────────────────┘    │    │
│  └───────────────┼────────────────────────────────┘    │
│                  │                                       │
│  ┌───────────────▼────────────────────────────────┐    │
│  │       llm-latency-lens Library                 │    │
│  │                                                 │    │
│  │  ┌──────────────┐  ┌──────────────┐          │    │
│  │  │  Interceptor │  │   Metrics    │          │    │
│  │  │   Middleware │  │  Collector   │          │    │
│  │  └──────┬───────┘  └──────┬───────┘          │    │
│  │         │                  │                   │    │
│  │  ┌──────▼──────────────────▼───────┐         │    │
│  │  │      Instrumentation             │         │    │
│  │  │  • Request/Response tracking     │         │    │
│  │  │  • Latency measurement           │         │    │
│  │  │  • Token counting                │         │    │
│  │  └──────┬───────────────────────────┘         │    │
│  │         │                                       │    │
│  │  ┌──────▼───────────────────────────┐         │    │
│  │  │       Export Interfaces          │         │    │
│  │  │  • Prometheus metrics            │         │    │
│  │  │  • OpenTelemetry traces          │         │    │
│  │  │  • Structured logging            │         │    │
│  │  │  • Custom callbacks              │         │    │
│  │  └──────────────────────────────────┘         │    │
│  └─────────────────────────────────────────────────┘    │
└───────────────────────────────────────────────────────────┘
                            │
                ┌───────────┼───────────┐
                │           │           │
        ┌───────▼────┐ ┌───▼─────┐ ┌──▼──────────┐
        │ Prometheus │ │  Jaeger │ │ Application │
        │            │ │  Tracing│ │    Logs     │
        └────────────┘ └─────────┘ └─────────────┘
```

### 3.3 Rust Library API

#### 3.3.1 Cargo.toml Dependencies

```toml
[package]
name = "my-llm-service"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core library
llm-latency-lens = "0.1.0"

# Optional features
llm-latency-lens = { version = "0.1.0", features = [
    "prometheus",      # Prometheus metrics export
    "opentelemetry",   # OpenTelemetry integration
    "async-runtime",   # Async runtime support
    "distributed",     # Distributed tracing
] }

# If using specific integrations
tokio = { version = "1.35", features = ["full"] }
openai_api = "0.4"
anthropic_api = "0.2"
```

#### 3.3.2 Basic Usage

```rust
// src/main.rs
use llm_latency_lens::{
    Profiler, ProfilerConfig, ProfilerBuilder,
    MetricsCollector, MetricsConfig,
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize profiler with default config
    let profiler = Profiler::new()?;

    // Or use builder pattern for custom configuration
    let profiler = ProfilerBuilder::new()
        .with_provider("openai")
        .with_metrics_enabled(true)
        .with_prometheus_export(9090)
        .with_sampling_rate(1.0)  // 100% sampling
        .build()?;

    // Profile a single LLM request
    let request = openai::ChatRequest {
        model: "gpt-4-turbo-preview".to_string(),
        messages: vec![/* ... */],
        max_tokens: 1024,
        temperature: 0.7,
    };

    let response = profiler.profile(request).await?;

    // Access metrics
    let metrics = profiler.get_metrics();
    println!("TTFT: {}ms", metrics.ttft_ms);
    println!("Total latency: {}ms", metrics.total_latency_ms);
    println!("Tokens: {}", metrics.output_tokens);

    Ok(())
}
```

#### 3.3.3 Advanced Integration

```rust
// src/llm_service.rs
use llm_latency_lens::{
    Profiler, ProfilerConfig, Middleware, MetricsExporter,
    ProfileResult, ProfileContext,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct LLMService {
    profiler: Arc<Profiler>,
    client: OpenAIClient,
}

impl LLMService {
    pub async fn new() -> Result<Self> {
        let config = ProfilerConfig::from_file("profiler.yaml")?;

        let profiler = ProfilerBuilder::new()
            .with_config(config)
            // Add custom middleware
            .with_middleware(Box::new(LoggingMiddleware))
            .with_middleware(Box::new(CostTrackingMiddleware))
            // Configure exports
            .with_prometheus_export(9090)
            .with_opentelemetry_export("http://jaeger:4317")
            // Set sampling for production
            .with_sampling_rate(0.1)  // 10% sampling
            .build()?;

        Ok(Self {
            profiler: Arc::new(profiler),
            client: OpenAIClient::new()?,
        })
    }

    pub async fn generate(&self, prompt: &str) -> Result<String> {
        // Create profile context
        let ctx = ProfileContext::new()
            .with_tag("service", "llm-api")
            .with_tag("endpoint", "generate")
            .with_metadata("user_id", "user123");

        // Profile the request
        let result = self.profiler
            .profile_with_context(ctx, async {
                self.client
                    .chat_completion(prompt)
                    .await
            })
            .await?;

        // Access profiling results
        log::info!(
            "LLM request completed: TTFT={}ms, Total={}ms, Tokens={}",
            result.metrics.ttft_ms,
            result.metrics.total_latency_ms,
            result.metrics.output_tokens
        );

        Ok(result.response)
    }

    pub async fn batch_generate(&self, prompts: Vec<String>) -> Result<Vec<String>> {
        // Profile batch operations
        let batch_ctx = ProfileContext::new()
            .with_tag("operation", "batch")
            .with_metadata("batch_size", prompts.len());

        let results = self.profiler
            .profile_batch(batch_ctx, prompts, |prompt| async move {
                self.client.chat_completion(&prompt).await
            })
            .await?;

        // Aggregate metrics
        let aggregated = results.aggregate_metrics();
        log::info!("Batch completed: avg_latency={}ms", aggregated.mean_latency_ms);

        Ok(results.into_iter().map(|r| r.response).collect())
    }

    pub fn get_metrics_snapshot(&self) -> MetricsSnapshot {
        self.profiler.metrics().snapshot()
    }
}

// Custom middleware example
struct LoggingMiddleware;

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn before_request(&self, ctx: &ProfileContext) -> Result<()> {
        log::info!("Starting LLM request: {:?}", ctx.tags);
        Ok(())
    }

    async fn after_response(&self, ctx: &ProfileContext, result: &ProfileResult) -> Result<()> {
        log::info!(
            "Completed LLM request: latency={}ms, tokens={}",
            result.metrics.total_latency_ms,
            result.metrics.output_tokens
        );
        Ok(())
    }

    async fn on_error(&self, ctx: &ProfileContext, error: &Error) -> Result<()> {
        log::error!("LLM request failed: {:?}", error);
        Ok(())
    }
}

// Cost tracking middleware
struct CostTrackingMiddleware {
    pricing: PricingTable,
}

#[async_trait::async_trait]
impl Middleware for CostTrackingMiddleware {
    async fn after_response(&self, ctx: &ProfileContext, result: &ProfileResult) -> Result<()> {
        let cost = self.pricing.calculate_cost(
            &result.model,
            result.metrics.input_tokens,
            result.metrics.output_tokens
        );

        // Store cost in context for later retrieval
        ctx.set_metadata("cost_usd", cost);

        Ok(())
    }
}
```

#### 3.3.4 Metrics Export Interfaces

```rust
// Prometheus metrics export
use llm_latency_lens::exports::prometheus::{PrometheusExporter, PrometheusConfig};

let prometheus_config = PrometheusConfig {
    port: 9090,
    path: "/metrics".to_string(),
    namespace: "llm".to_string(),
    subsystem: "latency_lens".to_string(),
};

let exporter = PrometheusExporter::new(prometheus_config)?;
profiler.add_exporter(Box::new(exporter));

// The following metrics are exported:
// llm_latency_lens_request_duration_seconds{provider, model, status}
// llm_latency_lens_ttft_duration_seconds{provider, model}
// llm_latency_lens_tokens_total{provider, model, type}
// llm_latency_lens_cost_usd_total{provider, model}
// llm_latency_lens_errors_total{provider, model, error_type}

// OpenTelemetry export
use llm_latency_lens::exports::opentelemetry::{OtelExporter, OtelConfig};

let otel_config = OtelConfig {
    endpoint: "http://jaeger:4317".to_string(),
    service_name: "llm-service".to_string(),
    service_version: "0.1.0".to_string(),
};

let exporter = OtelExporter::new(otel_config)?;
profiler.add_exporter(Box::new(exporter));

// Custom callback export
use llm_latency_lens::exports::callback::CallbackExporter;

let callback_exporter = CallbackExporter::new(|result: &ProfileResult| {
    // Custom handling of profiling results
    if result.metrics.total_latency_ms > 5000 {
        alert_slow_request(result);
    }

    // Store in custom database
    store_metrics(result)?;

    Ok(())
});

profiler.add_exporter(Box::new(callback_exporter));

// Structured logging export
use llm_latency_lens::exports::logging::{LogExporter, LogConfig};
use tracing::Level;

let log_config = LogConfig {
    level: Level::INFO,
    format: LogFormat::Json,
    include_context: true,
};

let log_exporter = LogExporter::new(log_config);
profiler.add_exporter(Box::new(log_exporter));
```

### 3.4 Integration Patterns

#### 3.4.1 Actix Web Integration

```rust
// src/web.rs
use actix_web::{web, App, HttpServer, HttpResponse, Error};
use llm_latency_lens::Profiler;
use std::sync::Arc;

struct AppState {
    profiler: Arc<Profiler>,
    llm_client: OpenAIClient,
}

async fn generate_handler(
    state: web::Data<AppState>,
    req: web::Json<GenerateRequest>,
) -> Result<HttpResponse, Error> {
    let ctx = ProfileContext::new()
        .with_tag("endpoint", "api/generate")
        .with_tag("model", &req.model);

    let result = state.profiler
        .profile_with_context(ctx, async {
            state.llm_client
                .generate(&req.prompt, &req.model)
                .await
        })
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(GenerateResponse {
        text: result.response,
        metrics: result.metrics.into(),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let profiler = Arc::new(Profiler::new().unwrap());
    let llm_client = OpenAIClient::new().unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                profiler: Arc::clone(&profiler),
                llm_client: llm_client.clone(),
            }))
            .route("/api/generate", web::post().to(generate_handler))
            .route("/metrics", web::get().to(metrics_handler))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

#### 3.4.2 Tokio Service Integration

```rust
// src/service.rs
use llm_latency_lens::{Profiler, ProfilerMiddleware};
use tower::{Service, ServiceBuilder};
use std::task::{Context, Poll};

pub struct LLMServiceMiddleware<S> {
    inner: S,
    profiler: Arc<Profiler>,
}

impl<S, Request> Service<Request> for LLMServiceMiddleware<S>
where
    S: Service<Request>,
    Request: LLMRequest,
{
    type Response = ProfiledResponse<S::Response>;
    type Error = S::Error;
    type Future = ProfiledFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let profiler = Arc::clone(&self.profiler);
        let start = Instant::now();

        let fut = self.inner.call(req);

        ProfiledFuture {
            inner: fut,
            profiler,
            start,
        }
    }
}

// Usage with Tower service stack
let llm_service = ServiceBuilder::new()
    .layer(ProfilerMiddleware::new(profiler))
    .layer(RateLimitLayer::new(100))
    .layer(TimeoutLayer::new(Duration::from_secs(30)))
    .service(OpenAIService::new());
```

#### 3.4.3 Async Stream Processing

```rust
// src/streaming.rs
use llm_latency_lens::StreamProfiler;
use futures::StreamExt;

pub async fn process_streaming_response(
    profiler: &Profiler,
    prompt: String,
) -> Result<String> {
    let mut stream = client.stream_completion(prompt).await?;

    let mut stream_profiler = profiler.start_stream_profile();
    let mut result = String::new();

    // Track TTFT
    let first_token = stream.next().await;
    stream_profiler.record_first_token();

    if let Some(token) = first_token {
        result.push_str(&token?);
    }

    // Track inter-token latency
    while let Some(token) = stream.next().await {
        stream_profiler.record_token();
        result.push_str(&token?);
    }

    // Finalize profiling
    let metrics = stream_profiler.finish();

    log::info!(
        "Stream completed: TTFT={}ms, avg_inter_token={}ms, tokens={}",
        metrics.ttft_ms,
        metrics.avg_inter_token_ms,
        metrics.token_count
    );

    Ok(result)
}
```

### 3.5 Configuration Management

#### 3.5.1 Programmatic Configuration

```rust
use llm_latency_lens::{
    ProfilerConfig, MetricsConfig, ExportConfig,
    SamplingStrategy, AggregationMethod,
};

let config = ProfilerConfig {
    // Sampling configuration
    sampling: SamplingStrategy::Probabilistic {
        rate: 0.1,  // 10% sampling
        seed: Some(42),
    },

    // Metrics configuration
    metrics: MetricsConfig {
        enabled: true,
        percentiles: vec![50, 75, 90, 95, 99, 99.9],
        histogram_buckets: vec![10, 25, 50, 100, 250, 500, 1000, 2500, 5000],
        aggregation_window: Duration::from_secs(60),
        aggregation_method: AggregationMethod::Sliding,
    },

    // Export configuration
    exports: vec![
        ExportConfig::Prometheus {
            port: 9090,
            path: "/metrics".to_string(),
        },
        ExportConfig::OpenTelemetry {
            endpoint: "http://jaeger:4317".to_string(),
        },
    ],

    // Provider-specific settings
    providers: HashMap::from([
        ("openai".to_string(), ProviderConfig {
            timeout: Duration::from_secs(120),
            retry_policy: RetryPolicy::ExponentialBackoff {
                max_attempts: 3,
                initial_delay: Duration::from_secs(1),
                max_delay: Duration::from_secs(30),
            },
        }),
    ]),
};

let profiler = Profiler::with_config(config)?;
```

### 3.6 Best Practices

#### 3.6.1 Performance Overhead

1. **Sampling**: Use probabilistic sampling (1-10%) in production
2. **Async Operations**: All profiling operations are non-blocking
3. **Buffer Management**: Use bounded queues for metrics collection
4. **Lazy Initialization**: Defer expensive operations until needed
5. **Zero-Copy**: Minimize data copying in hot paths

#### 3.6.2 Memory Management

```rust
// Configure bounded metrics buffer
let config = ProfilerConfig {
    metrics: MetricsConfig {
        buffer_size: 10_000,  // Max metrics in memory
        flush_interval: Duration::from_secs(10),
        overflow_strategy: OverflowStrategy::DropOldest,
    },
    // ...
};
```

#### 3.6.3 Error Handling

```rust
// Graceful degradation
let profiler = ProfilerBuilder::new()
    .with_error_policy(ErrorPolicy::LogAndContinue)  // Don't fail application on profiler errors
    .build()?;

// Result handling
match profiler.profile(request).await {
    Ok(result) => {
        // Handle successful profiling
        process_response(result.response);
    }
    Err(ProfilerError::Timeout) => {
        // Handle timeout
        log::warn!("Profiler timeout, request may have completed");
    }
    Err(e) => {
        // Handle other errors
        log::error!("Profiler error: {}", e);
    }
}
```

---

## 4. Distributed Execution

### 4.1 Overview

Distribute LLM benchmarking across multiple nodes for high-scale load testing, geographic distribution, and coordinated profiling.

### 4.2 Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                     Coordinator Node                             │
│                                                                  │
│  ┌────────────────────────────────────────────────────────┐    │
│  │            Distributed Coordinator                     │    │
│  │                                                         │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │    │
│  │  │ Work Splitter│  │Task Scheduler│  │  Aggregator │ │    │
│  │  └──────┬───────┘  └──────┬───────┘  └─────▲───────┘ │    │
│  └─────────┼──────────────────┼──────────────────┼────────┘    │
└────────────┼──────────────────┼──────────────────┼─────────────┘
             │                  │                  │
        ┌────▼──────────────────▼──────────────────┼─────────┐
        │                                           │         │
        │        Message Queue / Coordinator        │         │
        │              (Redis / NATS)               │         │
        │                                           │         │
        └────┬──────────────┬───────────────┬──────▲─────────┘
             │              │               │      │
    ┌────────▼────┐  ┌─────▼──────┐  ┌────▼──────▼───┐
    │   Worker 1  │  │  Worker 2  │  │   Worker N     │
    │   (US-East) │  │ (EU-West)  │  │  (AP-South)    │
    │             │  │            │  │                │
    │  ┌────────┐ │  │ ┌────────┐ │  │  ┌────────┐   │
    │  │Executor│ │  │ │Executor│ │  │  │Executor│   │
    │  └───┬────┘ │  │ └───┬────┘ │  │  └───┬────┘   │
    │      │      │  │     │      │  │      │        │
    │  ┌───▼────┐ │  │ ┌───▼────┐ │  │  ┌───▼────┐   │
    │  │Metrics │ │  │ │Metrics │ │  │  │Metrics │   │
    │  │Collect │ │  │ │Collect │ │  │  │Collect │   │
    │  └───┬────┘ │  │ └───┬────┘ │  │  └───┬────┘   │
    └──────┼──────┘  └─────┼──────┘  └──────┼────────┘
           │               │                 │
           └───────────────┴─────────────────┘
                           │
                   ┌───────▼──────────┐
                   │  Result Storage   │
                   │  (S3 / Database)  │
                   └───────────────────┘
```

### 4.3 Deployment Modes

#### 4.3.1 Kubernetes Deployment

```yaml
# k8s/coordinator-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-lens-coordinator
  namespace: benchmarking
spec:
  replicas: 1
  selector:
    matchLabels:
      app: llm-lens-coordinator
  template:
    metadata:
      labels:
        app: llm-lens-coordinator
    spec:
      containers:
      - name: coordinator
        image: ghcr.io/your-org/llm-latency-lens:latest
        command: ["llm-lens"]
        args:
          - "distributed"
          - "coordinator"
          - "--config"
          - "/config/coordinator.yaml"
        env:
        - name: REDIS_URL
          value: "redis://redis-service:6379"
        - name: RUST_LOG
          value: "info"
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        volumeMounts:
        - name: config
          mountPath: /config
        resources:
          requests:
            cpu: "500m"
            memory: "512Mi"
          limits:
            cpu: "2000m"
            memory: "2Gi"
      volumes:
      - name: config
        configMap:
          name: llm-lens-config

---
# k8s/worker-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-lens-worker
  namespace: benchmarking
spec:
  replicas: 10
  selector:
    matchLabels:
      app: llm-lens-worker
  template:
    metadata:
      labels:
        app: llm-lens-worker
    spec:
      containers:
      - name: worker
        image: ghcr.io/your-org/llm-latency-lens:latest
        command: ["llm-lens"]
        args:
          - "distributed"
          - "worker"
          - "--coordinator"
          - "http://llm-lens-coordinator:8080"
          - "--config"
          - "/config/worker.yaml"
        env:
        - name: WORKER_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: WORKER_REGION
          value: "us-east-1"
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: llm-api-keys
              key: openai
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: llm-api-keys
              key: anthropic
        ports:
        - containerPort: 8081
          name: http
        - containerPort: 9091
          name: metrics
        volumeMounts:
        - name: config
          mountPath: /config
        resources:
          requests:
            cpu: "1000m"
            memory: "1Gi"
          limits:
            cpu: "4000m"
            memory: "4Gi"
      volumes:
      - name: config
        configMap:
          name: llm-lens-config

---
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: llm-lens-coordinator
  namespace: benchmarking
spec:
  selector:
    app: llm-lens-coordinator
  ports:
  - name: http
    port: 8080
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090

---
# k8s/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: llm-lens-worker-hpa
  namespace: benchmarking
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: llm-lens-worker
  minReplicas: 5
  maxReplicas: 100
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Pods
    pods:
      metric:
        name: llm_lens_worker_queue_depth
      target:
        type: AverageValue
        averageValue: "10"
```

#### 4.3.2 Docker Compose Deployment

```yaml
# docker-compose.yml
version: '3.8'

services:
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    command: redis-server --appendonly yes

  coordinator:
    image: ghcr.io/your-org/llm-latency-lens:latest
    command: >
      llm-lens distributed coordinator
      --config /config/coordinator.yaml
      --redis-url redis://redis:6379
    ports:
      - "8080:8080"
      - "9090:9090"
    volumes:
      - ./config:/config
      - ./results:/results
    environment:
      - RUST_LOG=info
    depends_on:
      - redis

  worker:
    image: ghcr.io/your-org/llm-latency-lens:latest
    command: >
      llm-lens distributed worker
      --coordinator http://coordinator:8080
      --config /config/worker.yaml
    volumes:
      - ./config:/config
    environment:
      - RUST_LOG=info
      - WORKER_REGION=us-east-1
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
    depends_on:
      - coordinator
      - redis
    deploy:
      replicas: 5

  postgres:
    image: postgres:16-alpine
    environment:
      - POSTGRES_DB=llm_benchmarks
      - POSTGRES_USER=benchmark
      - POSTGRES_PASSWORD=secure_password
    ports:
      - "5432:5432"
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./schema.sql:/docker-entrypoint-initdb.d/schema.sql

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9093:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    depends_on:
      - prometheus

volumes:
  redis-data:
  postgres-data:
  prometheus-data:
  grafana-data:
```

### 4.4 Configuration

#### 4.4.1 Coordinator Configuration

```yaml
# config/coordinator.yaml
coordinator:
  mode: "coordinator"
  bind_address: "0.0.0.0:8080"

  # Work distribution strategy
  distribution:
    strategy: "round_robin"  # round_robin, least_loaded, geographic, custom

    # Work splitting
    splitting:
      method: "time_based"  # time_based, request_count, adaptive
      chunk_size: 1000  # requests per worker

    # Worker management
    workers:
      min_workers: 5
      max_workers: 100
      health_check_interval: 10s
      heartbeat_timeout: 30s

  # Coordination backend
  backend:
    type: "redis"  # redis, nats, etcd
    url: "redis://localhost:6379"
    pool_size: 50

  # Result aggregation
  aggregation:
    method: "streaming"  # streaming, batch
    buffer_size: 10000
    flush_interval: 10s

  # Benchmark configuration
  benchmark:
    name: "distributed-benchmark"
    duration: 600s
    rps: 1000  # Total across all workers

    # Workload distribution
    workload:
      type: "uniform"  # uniform, weighted, geographic

    # Provider configuration
    providers:
      - name: "openai"
        models: ["gpt-4-turbo-preview", "gpt-3.5-turbo"]
        weight: 0.5

      - name: "anthropic"
        models: ["claude-3-opus-20240229"]
        weight: 0.5

  # Storage
  storage:
    type: "s3"
    bucket: "llm-benchmarks"
    prefix: "distributed/"

  # Monitoring
  monitoring:
    prometheus:
      enabled: true
      port: 9090

    logging:
      level: "info"
      format: "json"
```

#### 4.4.2 Worker Configuration

```yaml
# config/worker.yaml
worker:
  mode: "worker"
  coordinator_url: "http://coordinator:8080"

  # Worker identity
  id: "${WORKER_ID}"
  region: "${WORKER_REGION}"
  tags:
    environment: "production"
    tier: "standard"

  # Capacity configuration
  capacity:
    max_concurrent_requests: 100
    max_rps: 50
    memory_limit: "4GB"

  # Task polling
  polling:
    interval: 1s
    batch_size: 10
    timeout: 5s

  # Provider configuration
  providers:
    openai:
      api_key: "${OPENAI_API_KEY}"
      endpoint: "https://api.openai.com/v1"
      timeout: 120s

    anthropic:
      api_key: "${ANTHROPIC_API_KEY}"
      endpoint: "https://api.anthropic.com/v1"
      timeout: 120s

  # Local metrics
  metrics:
    enabled: true
    export_interval: 10s

  # Result reporting
  reporting:
    method: "streaming"  # streaming, batch
    batch_size: 100
    flush_interval: 5s

  # Health check
  health:
    port: 8081
    endpoint: "/health"

  # Monitoring
  monitoring:
    prometheus:
      enabled: true
      port: 9091
```

### 4.5 CLI Usage

#### 4.5.1 Start Coordinator

```bash
# Start coordinator with Redis backend
llm-lens distributed coordinator \
  --config coordinator.yaml \
  --redis-url redis://localhost:6379 \
  --bind 0.0.0.0:8080

# Start coordinator with NATS backend
llm-lens distributed coordinator \
  --config coordinator.yaml \
  --nats-url nats://localhost:4222 \
  --bind 0.0.0.0:8080
```

#### 4.5.2 Start Workers

```bash
# Start worker connecting to coordinator
llm-lens distributed worker \
  --coordinator http://coordinator:8080 \
  --config worker.yaml \
  --region us-east-1

# Start multiple workers
for i in {1..10}; do
  llm-lens distributed worker \
    --coordinator http://coordinator:8080 \
    --config worker.yaml \
    --worker-id worker-$i \
    --region us-east-1 &
done
```

#### 4.5.3 Submit Benchmark Job

```bash
# Submit distributed benchmark
llm-lens distributed submit \
  --coordinator http://coordinator:8080 \
  --config benchmark.yaml \
  --output results/

# Monitor job progress
llm-lens distributed status \
  --coordinator http://coordinator:8080 \
  --job-id job-123

# Cancel running job
llm-lens distributed cancel \
  --coordinator http://coordinator:8080 \
  --job-id job-123
```

### 4.6 Geographic Distribution

#### 4.6.1 Multi-Region Deployment

```yaml
# terraform/main.tf
module "benchmark_cluster" {
  source = "./modules/benchmark-cluster"

  regions = {
    "us-east-1" = {
      worker_count = 20
      instance_type = "c5.2xlarge"
    }
    "eu-west-1" = {
      worker_count = 15
      instance_type = "c5.2xlarge"
    }
    "ap-southeast-1" = {
      worker_count = 10
      instance_type = "c5.2xlarge"
    }
  }

  coordinator_region = "us-east-1"
}
```

#### 4.6.2 Geographic Workload Distribution

```yaml
# geographic-config.yaml
distribution:
  strategy: "geographic"

  regions:
    - name: "us-east-1"
      weight: 0.4
      providers:
        - "openai"
        - "anthropic"
      latency_target_ms: 200

    - name: "eu-west-1"
      weight: 0.35
      providers:
        - "openai"
        - "anthropic"
      latency_target_ms: 250

    - name: "ap-southeast-1"
      weight: 0.25
      providers:
        - "openai"
        - "google"
      latency_target_ms: 300
```

### 4.7 Result Aggregation

#### 4.7.1 Streaming Aggregation

```rust
// Coordinator aggregation logic
use llm_latency_lens::distributed::{Coordinator, AggregationStrategy};

let coordinator = Coordinator::new(config)?;

// Start streaming aggregation
let mut aggregator = coordinator.start_aggregation(
    AggregationStrategy::Streaming {
        window: Duration::from_secs(60),
        method: AggregationMethod::TimeWeighted,
    }
)?;

// Process results as they arrive
while let Some(worker_result) = aggregator.next().await {
    // Update running statistics
    aggregator.update_stats(&worker_result)?;

    // Check for anomalies
    if worker_result.mean_latency_ms > threshold {
        alert_slow_worker(&worker_result.worker_id)?;
    }
}

// Get final aggregated results
let final_results = aggregator.finalize().await?;
```

#### 4.7.2 Aggregated Metrics Format

```json
{
  "job_id": "job-123",
  "start_time": "2024-01-15T10:00:00Z",
  "end_time": "2024-01-15T10:10:00Z",
  "duration_seconds": 600,
  "workers": {
    "total": 50,
    "active": 50,
    "failed": 0
  },
  "requests": {
    "total": 50000,
    "successful": 49750,
    "failed": 250,
    "retried": 380
  },
  "latency": {
    "mean_ms": 2345,
    "median_ms": 2123,
    "p95_ms": 4567,
    "p99_ms": 6789,
    "min_ms": 456,
    "max_ms": 15678
  },
  "throughput": {
    "rps": 83.3,
    "tokens_per_second": 1950.5
  },
  "by_region": {
    "us-east-1": {
      "requests": 20000,
      "mean_latency_ms": 2234,
      "workers": 20
    },
    "eu-west-1": {
      "requests": 17500,
      "mean_latency_ms": 2456,
      "workers": 15
    },
    "ap-southeast-1": {
      "requests": 12500,
      "mean_latency_ms": 2678,
      "workers": 15
    }
  },
  "by_model": {
    "gpt-4-turbo-preview": {
      "requests": 25000,
      "mean_latency_ms": 3456,
      "cost_usd": 34.56
    },
    "claude-3-opus-20240229": {
      "requests": 25000,
      "mean_latency_ms": 3123,
      "cost_usd": 29.87
    }
  }
}
```

### 4.8 Best Practices

#### 4.8.1 Fault Tolerance

1. **Worker Failures**: Automatically redistribute work from failed workers
2. **Network Partitions**: Handle coordinator disconnections gracefully
3. **Result Persistence**: Stream results to durable storage continuously
4. **Checkpointing**: Enable resuming from last checkpoint
5. **Idempotency**: Ensure safe retry of failed tasks

#### 4.8.2 Performance Optimization

1. **Connection Pooling**: Reuse HTTP connections across requests
2. **Batch Processing**: Group small tasks for efficiency
3. **Work Stealing**: Allow idle workers to steal tasks
4. **Adaptive Scaling**: Scale workers based on queue depth
5. **Result Compression**: Compress results before transmission

#### 4.8.3 Cost Optimization

```yaml
# Cost-optimized configuration
cost_optimization:
  # Use spot instances for workers
  worker_instance_type: "spot"

  # Scale down during idle periods
  auto_scaling:
    scale_down_delay: 300s
    min_idle_time: 60s

  # Request deduplication
  deduplication:
    enabled: true
    cache_ttl: 3600s
```

---

## 5. Observatory Integration

### 5.1 Overview

Real-time monitoring, alerting, and historical analysis through integrated observability platforms.

### 5.2 Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    LLM-Latency-Lens                             │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │   Profiler   │  │    Metrics   │  │    Traces    │        │
│  │              │  │  Collector   │  │   Exporter   │        │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘        │
│         │                  │                  │                 │
└─────────┼──────────────────┼──────────────────┼─────────────────┘
          │                  │                  │
          │                  │                  │
    ┌─────▼──────────────────▼──────────────────▼─────┐
    │            Export Layer                          │
    │  • Prometheus metrics                            │
    │  • OpenTelemetry traces                          │
    │  • Structured logs                               │
    └─────┬──────────┬───────────────┬─────────────────┘
          │          │               │
    ┌─────▼───┐ ┌────▼──────┐ ┌─────▼────────┐
    │Promethe-│ │  Jaeger   │ │Elasticsearch │
    │   us    │ │  Tracing  │ │   Logging    │
    └─────┬───┘ └────┬──────┘ └─────┬────────┘
          │          │               │
          └──────────┼───────────────┘
                     │
          ┌──────────▼──────────────┐
          │      Grafana            │
          │    Dashboards           │
          │                         │
          │  ┌──────────────────┐  │
          │  │  Latency Trends  │  │
          │  ├──────────────────┤  │
          │  │ Model Comparison │  │
          │  ├──────────────────┤  │
          │  │ Cost Analysis    │  │
          │  ├──────────────────┤  │
          │  │ Error Tracking   │  │
          │  └──────────────────┘  │
          └─────────────────────────┘
                     │
          ┌──────────▼──────────────┐
          │    Alert Manager        │
          │                         │
          │  • Slack notifications  │
          │  • PagerDuty incidents  │
          │  • Email alerts         │
          └─────────────────────────┘
```

### 5.3 Prometheus Integration

#### 5.3.1 Metrics Export Configuration

```yaml
# prometheus-config.yaml
prometheus:
  enabled: true
  port: 9090
  path: "/metrics"

  # Metric configuration
  metrics:
    # Latency histograms
    - name: "llm_request_duration_seconds"
      type: "histogram"
      help: "LLM request duration in seconds"
      buckets: [0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0]
      labels:
        - provider
        - model
        - status

    - name: "llm_ttft_duration_seconds"
      type: "histogram"
      help: "Time to first token in seconds"
      buckets: [0.1, 0.2, 0.5, 1.0, 2.0, 5.0]
      labels:
        - provider
        - model

    # Counters
    - name: "llm_requests_total"
      type: "counter"
      help: "Total number of LLM requests"
      labels:
        - provider
        - model
        - status

    - name: "llm_tokens_total"
      type: "counter"
      help: "Total number of tokens processed"
      labels:
        - provider
        - model
        - type  # input/output

    - name: "llm_cost_usd_total"
      type: "counter"
      help: "Total cost in USD"
      labels:
        - provider
        - model

    - name: "llm_errors_total"
      type: "counter"
      help: "Total number of errors"
      labels:
        - provider
        - model
        - error_type

    # Gauges
    - name: "llm_concurrent_requests"
      type: "gauge"
      help: "Number of concurrent requests"
      labels:
        - provider

    - name: "llm_queue_depth"
      type: "gauge"
      help: "Request queue depth"
      labels:
        - provider
```

#### 5.3.2 Prometheus Scrape Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'llm-latency-lens'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'llm-workers'
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - benchmarking
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        action: keep
        regex: llm-lens-worker
      - source_labels: [__meta_kubernetes_pod_name]
        target_label: instance
      - source_labels: [__meta_kubernetes_pod_label_region]
        target_label: region

# Alert rules
rule_files:
  - 'alerts.yml'

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']
```

#### 5.3.3 Alert Rules

```yaml
# alerts.yml
groups:
  - name: llm_latency_alerts
    interval: 30s
    rules:
      - alert: HighLatency
        expr: |
          histogram_quantile(0.95,
            rate(llm_request_duration_seconds_bucket[5m])
          ) > 10
        for: 5m
        labels:
          severity: warning
          component: llm-profiler
        annotations:
          summary: "High LLM latency detected"
          description: "P95 latency is {{ $value }}s for {{ $labels.provider }}/{{ $labels.model }}"

      - alert: HighErrorRate
        expr: |
          rate(llm_errors_total[5m]) /
          rate(llm_requests_total[5m]) > 0.05
        for: 2m
        labels:
          severity: critical
          component: llm-profiler
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }} for {{ $labels.provider }}/{{ $labels.model }}"

      - alert: CostSpike
        expr: |
          rate(llm_cost_usd_total[1h]) > 100
        for: 5m
        labels:
          severity: warning
          component: llm-profiler
        annotations:
          summary: "LLM cost spike detected"
          description: "Cost rate is ${{ $value }}/hour"

      - alert: TTFTRegression
        expr: |
          histogram_quantile(0.95,
            rate(llm_ttft_duration_seconds_bucket[10m])
          ) >
          histogram_quantile(0.95,
            rate(llm_ttft_duration_seconds_bucket[1h] offset 1d)
          ) * 1.2
        for: 10m
        labels:
          severity: warning
          component: llm-profiler
        annotations:
          summary: "TTFT regression detected"
          description: "P95 TTFT increased by >20% compared to yesterday"
```

### 5.4 Grafana Dashboards

#### 5.4.1 Dashboard Configuration

```json
{
  "dashboard": {
    "title": "LLM Performance Observatory",
    "tags": ["llm", "performance", "latency"],
    "timezone": "browser",
    "refresh": "30s",
    "time": {
      "from": "now-6h",
      "to": "now"
    },
    "panels": [
      {
        "id": 1,
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(rate(llm_requests_total[5m])) by (provider, model)",
            "legendFormat": "{{provider}}/{{model}}"
          }
        ],
        "gridPos": {"x": 0, "y": 0, "w": 12, "h": 8}
      },
      {
        "id": 2,
        "title": "Latency Distribution",
        "type": "heatmap",
        "targets": [
          {
            "expr": "sum(rate(llm_request_duration_seconds_bucket[5m])) by (le)",
            "format": "heatmap"
          }
        ],
        "gridPos": {"x": 12, "y": 0, "w": 12, "h": 8}
      },
      {
        "id": 3,
        "title": "P50/P95/P99 Latency",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, sum(rate(llm_request_duration_seconds_bucket[5m])) by (le, model))",
            "legendFormat": "p50 - {{model}}"
          },
          {
            "expr": "histogram_quantile(0.95, sum(rate(llm_request_duration_seconds_bucket[5m])) by (le, model))",
            "legendFormat": "p95 - {{model}}"
          },
          {
            "expr": "histogram_quantile(0.99, sum(rate(llm_request_duration_seconds_bucket[5m])) by (le, model))",
            "legendFormat": "p99 - {{model}}"
          }
        ],
        "gridPos": {"x": 0, "y": 8, "w": 12, "h": 8}
      },
      {
        "id": 4,
        "title": "Time to First Token",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(llm_ttft_duration_seconds_bucket[5m])) by (le, model))",
            "legendFormat": "{{model}}"
          }
        ],
        "gridPos": {"x": 12, "y": 8, "w": 12, "h": 8}
      },
      {
        "id": 5,
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(llm_errors_total[5m]) / rate(llm_requests_total[5m])",
            "legendFormat": "{{provider}}/{{model}}"
          }
        ],
        "gridPos": {"x": 0, "y": 16, "w": 12, "h": 8},
        "alert": {
          "conditions": [
            {
              "evaluator": {"params": [0.05], "type": "gt"},
              "operator": {"type": "and"},
              "query": {"params": ["A", "5m", "now"]},
              "type": "query"
            }
          ]
        }
      },
      {
        "id": 6,
        "title": "Cost per Hour",
        "type": "stat",
        "targets": [
          {
            "expr": "sum(rate(llm_cost_usd_total[1h])) * 3600",
            "legendFormat": "Total"
          }
        ],
        "gridPos": {"x": 12, "y": 16, "w": 6, "h": 4}
      },
      {
        "id": 7,
        "title": "Throughput (tokens/s)",
        "type": "stat",
        "targets": [
          {
            "expr": "sum(rate(llm_tokens_total[5m]))",
            "legendFormat": "Total"
          }
        ],
        "gridPos": {"x": 18, "y": 16, "w": 6, "h": 4}
      }
    ]
  }
}
```

#### 5.4.2 Dashboard Provisioning

```yaml
# grafana/provisioning/dashboards/llm-dashboards.yaml
apiVersion: 1

providers:
  - name: 'LLM Dashboards'
    orgId: 1
    folder: 'LLM Performance'
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    allowUiUpdates: true
    options:
      path: /etc/grafana/provisioning/dashboards/json

---
# grafana/provisioning/datasources/prometheus.yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
```

### 5.5 OpenTelemetry Integration

#### 5.5.1 Tracing Configuration

```yaml
# otel-config.yaml
opentelemetry:
  enabled: true

  # OTLP exporter
  exporter:
    endpoint: "http://jaeger:4317"
    protocol: "grpc"  # grpc, http/protobuf

  # Sampling
  sampling:
    type: "probabilistic"  # always_on, always_off, probabilistic, parent_based
    rate: 0.1  # 10% sampling

  # Resource attributes
  resource:
    service.name: "llm-latency-lens"
    service.version: "0.1.0"
    deployment.environment: "production"

  # Span configuration
  spans:
    # Trace entire benchmark
    - name: "benchmark"
      kind: "internal"
      attributes:
        - benchmark.name
        - benchmark.duration
        - benchmark.config

    # Trace individual requests
    - name: "llm.request"
      kind: "client"
      attributes:
        - llm.provider
        - llm.model
        - llm.prompt_tokens
        - llm.completion_tokens
        - llm.cost_usd

    # Trace TTFT
    - name: "llm.ttft"
      kind: "internal"
      attributes:
        - llm.ttft_ms
```

#### 5.5.2 Rust Implementation

```rust
use opentelemetry::{
    global, trace::{Span, Tracer, TracerProvider},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;

// Initialize OpenTelemetry
let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(
        opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint("http://jaeger:4317")
    )
    .with_trace_config(
        opentelemetry::sdk::trace::config()
            .with_sampler(opentelemetry::sdk::trace::Sampler::TraceIdRatioBased(0.1))
            .with_resource(opentelemetry::sdk::Resource::new(vec![
                KeyValue::new("service.name", "llm-latency-lens"),
            ]))
    )
    .install_batch(opentelemetry::runtime::Tokio)?;

// Instrument LLM requests
async fn profile_request(prompt: &str) -> Result<String> {
    let tracer = global::tracer("llm-profiler");
    let mut span = tracer.start("llm.request");

    span.set_attribute(KeyValue::new("llm.provider", "openai"));
    span.set_attribute(KeyValue::new("llm.model", "gpt-4-turbo-preview"));

    // Trace TTFT
    let ttft_span = tracer.start_with_context("llm.ttft", &span.context());
    let start = Instant::now();

    // Make request
    let response = client.request(prompt).await?;

    let ttft_ms = start.elapsed().as_millis();
    ttft_span.set_attribute(KeyValue::new("llm.ttft_ms", ttft_ms as i64));
    ttft_span.end();

    // Set final attributes
    span.set_attribute(KeyValue::new("llm.prompt_tokens", response.usage.prompt_tokens as i64));
    span.set_attribute(KeyValue::new("llm.completion_tokens", response.usage.completion_tokens as i64));
    span.set_attribute(KeyValue::new("llm.total_latency_ms", start.elapsed().as_millis() as i64));

    span.end();

    Ok(response.text)
}
```

### 5.6 Alerting Configuration

#### 5.6.1 Alert Manager Configuration

```yaml
# alertmanager.yml
global:
  slack_api_url: '${SLACK_WEBHOOK_URL}'
  pagerduty_url: 'https://events.pagerduty.com/v2/enqueue'

route:
  group_by: ['alertname', 'provider', 'model']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'default'

  routes:
    - match:
        severity: critical
      receiver: 'pagerduty'
      continue: true

    - match:
        severity: warning
      receiver: 'slack'

    - match:
        component: llm-profiler
      receiver: 'llm-team'

receivers:
  - name: 'default'
    slack_configs:
      - channel: '#alerts'
        title: 'LLM Alert: {{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

  - name: 'pagerduty'
    pagerduty_configs:
      - routing_key: '${PAGERDUTY_INTEGRATION_KEY}'
        severity: '{{ .CommonLabels.severity }}'
        description: '{{ .CommonAnnotations.summary }}'
        details:
          provider: '{{ .CommonLabels.provider }}'
          model: '{{ .CommonLabels.model }}'

  - name: 'slack'
    slack_configs:
      - channel: '#llm-alerts'
        color: '{{ if eq .Status "firing" }}danger{{ else }}good{{ end }}'
        title: '{{ .GroupLabels.alertname }}'
        text: |
          {{ range .Alerts }}
          *Alert:* {{ .Annotations.summary }}
          *Description:* {{ .Annotations.description }}
          *Provider:* {{ .Labels.provider }}
          *Model:* {{ .Labels.model }}
          {{ end }}

  - name: 'llm-team'
    email_configs:
      - to: 'llm-team@company.com'
        from: 'alerts@company.com'
        subject: 'LLM Performance Alert: {{ .GroupLabels.alertname }}'
        html: |
          <h2>{{ .GroupLabels.alertname }}</h2>
          {{ range .Alerts }}
          <p><strong>Description:</strong> {{ .Annotations.description }}</p>
          <p><strong>Provider:</strong> {{ .Labels.provider }}</p>
          <p><strong>Model:</strong> {{ .Labels.model }}</p>
          {{ end }}

inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'provider', 'model']
```

### 5.7 Historical Analysis

#### 5.7.1 Long-Term Storage

```yaml
# prometheus-storage.yaml
storage:
  tsdb:
    path: /prometheus/data
    retention.time: 90d
    retention.size: 500GB

  # Remote write for long-term storage
  remote_write:
    - url: "https://prometheus-long-term.company.com/api/v1/write"
      queue_config:
        capacity: 10000
        max_shards: 50
        max_samples_per_send: 5000

  # Remote read for historical queries
  remote_read:
    - url: "https://prometheus-long-term.company.com/api/v1/read"
      read_recent: true
```

#### 5.7.2 Trend Analysis Queries

```promql
# Average latency trend (7 days)
avg_over_time(
  histogram_quantile(0.95,
    rate(llm_request_duration_seconds_bucket[1h])
  )[7d:1h]
)

# Week-over-week comparison
(
  avg_over_time(
    histogram_quantile(0.95,
      rate(llm_request_duration_seconds_bucket[1h])
    )[7d:1h]
  )
  -
  avg_over_time(
    histogram_quantile(0.95,
      rate(llm_request_duration_seconds_bucket[1h] offset 7d)
    )[7d:1h]
  )
) /
avg_over_time(
  histogram_quantile(0.95,
    rate(llm_request_duration_seconds_bucket[1h] offset 7d)
  )[7d:1h]
) * 100

# Model performance comparison over time
max by (model) (
  histogram_quantile(0.95,
    rate(llm_request_duration_seconds_bucket[30d])
  )
) -
min by (model) (
  histogram_quantile(0.95,
    rate(llm_request_duration_seconds_bucket[30d])
  )
)

# Cost trend analysis
deriv(
  sum_over_time(llm_cost_usd_total[7d])
[1d:1h])
```

### 5.8 Best Practices

#### 5.8.1 Monitoring Strategy

1. **Golden Signals**: Focus on latency, traffic, errors, saturation
2. **Appropriate Granularity**: 15s scrape interval, 1m aggregation
3. **Label Cardinality**: Limit high-cardinality labels
4. **Alert Fatigue**: Set appropriate thresholds and grouping
5. **Documentation**: Maintain runbooks for all alerts

#### 5.8.2 Dashboard Design

1. **Overview First**: Start with high-level metrics
2. **Drill-Down Capability**: Enable filtering and zooming
3. **Context**: Show related metrics together
4. **Actionable**: Include links to runbooks
5. **Performance**: Optimize query performance

---

## 6. Cross-Cutting Concerns

### 6.1 Security

#### 6.1.1 API Key Management

```yaml
# Use secret management
secrets:
  provider: "vault"  # vault, aws-secrets, azure-keyvault, gcp-secrets

  vault:
    address: "https://vault.company.com:8200"
    auth_method: "kubernetes"
    role: "llm-profiler"
    mount: "secret"

  keys:
    - name: "openai_api_key"
      path: "llm/openai"
      field: "api_key"

    - name: "anthropic_api_key"
      path: "llm/anthropic"
      field: "api_key"
```

#### 6.1.2 Network Security

```yaml
# network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: llm-lens-policy
  namespace: benchmarking
spec:
  podSelector:
    matchLabels:
      app: llm-lens
  policyTypes:
    - Ingress
    - Egress
  egress:
    # Allow LLM API access
    - to:
        - namespaceSelector: {}
      ports:
        - protocol: TCP
          port: 443
    # Allow metrics export
    - to:
        - namespaceSelector:
            matchLabels:
              name: monitoring
      ports:
        - protocol: TCP
          port: 9090
```

### 6.2 Cost Management

```yaml
# cost-management.yaml
cost_controls:
  # Budget limits
  budgets:
    daily_usd: 100.0
    monthly_usd: 2000.0

  # Rate limiting
  rate_limits:
    requests_per_minute: 100
    tokens_per_minute: 100000

  # Cost optimization
  optimization:
    # Prefer cheaper models when possible
    model_selection: "cost_optimized"

    # Cache repeated requests
    caching:
      enabled: true
      ttl: 3600s

    # Batch similar requests
    batching:
      enabled: true
      max_batch_size: 100
```

### 6.3 Data Privacy

```yaml
# privacy-config.yaml
privacy:
  # PII detection and redaction
  pii_detection:
    enabled: true
    patterns:
      - email
      - phone
      - ssn
      - credit_card
    action: "redact"  # redact, hash, block

  # Data retention
  retention:
    results: 90d
    logs: 30d
    traces: 7d

  # Compliance
  compliance:
    gdpr: true
    ccpa: true
    hipaa: false
```

---

## 7. Migration Paths

### 7.1 From Standalone to Distributed

```bash
# Step 1: Current standalone setup
llm-lens run --config standalone.yaml

# Step 2: Deploy distributed infrastructure
kubectl apply -f k8s/distributed/

# Step 3: Migrate configuration
llm-lens migrate config \
  --from standalone.yaml \
  --to distributed.yaml \
  --mode distributed

# Step 4: Test distributed setup
llm-lens distributed submit \
  --config distributed.yaml \
  --dry-run

# Step 5: Run distributed benchmark
llm-lens distributed submit \
  --config distributed.yaml
```

### 7.2 From Library to Observatory

```rust
// Step 1: Add observatory dependencies
[dependencies]
llm-latency-lens = { version = "0.1.0", features = ["observatory"] }

// Step 2: Enable metrics export
let profiler = ProfilerBuilder::new()
    .with_prometheus_export(9090)
    .with_opentelemetry_export("http://jaeger:4317")
    .build()?;

// Step 3: Deploy monitoring stack
// kubectl apply -f k8s/monitoring/

// Step 4: Configure Grafana dashboards
// Import dashboards from grafana/

// Step 5: Set up alerts
// kubectl apply -f k8s/monitoring/alerts.yaml
```

---

## 8. Reference Architectures

### 8.1 Small Team Setup

```
Single developer or small team

Components:
- Standalone CLI for local development
- GitHub Actions for CI/CD
- SQLite for result storage
- Basic Grafana dashboard

Cost: ~$50-100/month
```

### 8.2 Medium Organization

```
Multiple teams, moderate scale

Components:
- Embedded library in services
- GitLab CI for benchmarking
- PostgreSQL for results
- Full Prometheus + Grafana stack
- Alert Manager

Cost: ~$500-1000/month
```

### 8.3 Enterprise Scale

```
Large organization, high scale

Components:
- Distributed execution across regions
- Full observatory integration
- Multi-region deployment
- Advanced analytics
- Custom SLO tracking

Cost: ~$5000-10000/month
```

---

## Conclusion

This deployment strategy guide provides comprehensive coverage of all deployment modes for LLM-Latency-Lens:

1. **Standalone CLI**: Perfect for local development and quick benchmarks
2. **CI/CD Integration**: Automated performance testing in pipelines
3. **Embedded Library**: In-process profiling for production services
4. **Distributed Execution**: High-scale load testing across regions
5. **Observatory Integration**: Real-time monitoring and alerting

Each mode is production-ready with detailed configurations, best practices, and migration paths. Organizations can start with standalone CLI and progressively adopt more advanced deployment modes as needs evolve.

### Next Steps

1. Choose appropriate deployment mode(s) for your use case
2. Review configuration examples and adapt to your environment
3. Set up monitoring and alerting infrastructure
4. Implement cost controls and budgets
5. Train team on operational procedures
6. Establish baseline metrics and SLOs
7. Iterate and optimize based on usage patterns
