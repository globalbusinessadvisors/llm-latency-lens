# LLM-Latency-Lens User Guide

**Complete guide to using LLM-Latency-Lens for LLM performance profiling**

Version: 0.1.0
Last Updated: 2025-11-07

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [CLI Commands](#cli-commands)
5. [Configuration](#configuration)
6. [Provider Configuration](#provider-configuration)
7. [Advanced Usage](#advanced-usage)
8. [Integration Guides](#integration-guides)
9. [Performance Tuning](#performance-tuning)
10. [Troubleshooting](#troubleshooting)
11. [FAQ](#faq)

---

## Getting Started

### What is LLM-Latency-Lens?

LLM-Latency-Lens is an enterprise-grade profiling tool designed to measure and analyze the performance of Large Language Model APIs. It provides:

- **Precision Timing**: Nanosecond-accurate measurements of Time-to-First-Token (TTFT) and token streaming
- **Multi-Provider Support**: Profile OpenAI, Anthropic, Google, and more from a single tool
- **Comprehensive Metrics**: Latency distributions, throughput, cost analysis, and success rates
- **Production Ready**: Battle-tested concurrency, retry logic, and error handling

### Who Should Use This Tool?

- **ML Engineers**: Profile LLM APIs during development and testing
- **DevOps Teams**: Monitor production latency and cost
- **Product Teams**: Make data-driven decisions on model selection
- **Researchers**: Benchmark LLM performance characteristics
- **CTOs/Engineering Leaders**: Track ROI and optimize spending

### System Requirements

- **Operating System**: Linux, macOS, or Windows
- **Rust**: Version 1.75 or higher (if building from source)
- **Memory**: Minimum 1GB RAM, 4GB recommended for high concurrency
- **Network**: Stable internet connection for API calls
- **API Keys**: Valid credentials for providers you want to test

---

## Installation

### Option 1: Install via Cargo (Recommended)

```bash
# Install from crates.io
cargo install llm-latency-lens

# Verify installation
llm-latency-lens --version
```

### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/llm-devops/llm-latency-lens.git
cd llm-latency-lens

# Build release binary
cargo build --release

# Install to system
cargo install --path .

# Or run directly
./target/release/llm-latency-lens --version
```

### Option 3: Download Pre-built Binaries

Visit our [releases page](https://github.com/llm-devops/llm-latency-lens/releases) and download the binary for your platform:

**Linux (x86_64)**
```bash
curl -LO https://github.com/llm-devops/llm-latency-lens/releases/download/v0.1.0/llm-latency-lens-linux-x86_64.tar.gz
tar xzf llm-latency-lens-linux-x86_64.tar.gz
sudo mv llm-latency-lens /usr/local/bin/
```

**macOS (Apple Silicon)**
```bash
curl -LO https://github.com/llm-devops/llm-latency-lens/releases/download/v0.1.0/llm-latency-lens-darwin-arm64.tar.gz
tar xzf llm-latency-lens-darwin-arm64.tar.gz
sudo mv llm-latency-lens /usr/local/bin/
```

**Windows (x86_64)**
```powershell
# Download from releases page and add to PATH
# Or use winget (coming soon)
winget install llm-devops.llm-latency-lens
```

### Option 4: Docker

```bash
# Pull latest image
docker pull llm-devops/llm-latency-lens:latest

# Run with environment variables
docker run -e OPENAI_API_KEY=$OPENAI_API_KEY \
  llm-devops/llm-latency-lens:latest \
  profile --provider openai --model gpt-4 --prompt "Hello, world!"

# Create alias for convenience
alias llm-lens='docker run -e OPENAI_API_KEY=$OPENAI_API_KEY llm-devops/llm-latency-lens:latest'
```

### Verify Installation

```bash
# Check version
llm-latency-lens --version

# View help
llm-latency-lens --help

# List supported providers
llm-latency-lens providers
```

---

## Quick Start

### 1. Set Up API Keys

Export your API keys as environment variables:

```bash
# OpenAI
export OPENAI_API_KEY=sk-...

# Anthropic
export ANTHROPIC_API_KEY=sk-ant-...

# Google (when supported)
export GOOGLE_API_KEY=AIza...

# Or create a .env file
cat > .env <<EOF
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
EOF
```

### 2. Run Your First Benchmark

```bash
# Simple benchmark with OpenAI
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "What is the capital of France?" \
  --iterations 10

# With streaming enabled
llm-latency-lens profile \
  --provider anthropic \
  --model claude-3-opus-20240229 \
  --prompt "Explain quantum computing" \
  --stream \
  --iterations 20 \
  --concurrency 5
```

### 3. Save Results

```bash
# Export to JSON
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Hello, world!" \
  --iterations 50 \
  --output results.json \
  --format json

# Export to CSV
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Hello, world!" \
  --iterations 50 \
  --output results.csv \
  --format csv
```

### 4. Compare Providers

```bash
# Create config file
cat > compare.yaml <<EOF
providers:
  - name: openai
    models: [gpt-4-turbo-preview]
  - name: anthropic
    models: [claude-3-opus-20240229]

workload:
  scenarios:
    - name: simple_test
      prompt: "What is 2+2?"
      requests: 50
      concurrency: 10
EOF

# Run comparison
llm-latency-lens compare --config compare.yaml
```

---

## CLI Commands

### Global Options

```bash
llm-latency-lens [OPTIONS] <COMMAND>

OPTIONS:
  -v, --verbose        Enable verbose logging
  -q, --quiet          Suppress all output except errors
  --log-level <LEVEL>  Set log level [debug, info, warn, error]
  --no-color           Disable colored output
  -h, --help           Print help information
  -V, --version        Print version information
```

### `profile` - Profile a single provider/model

Profile a specific LLM model with custom parameters.

```bash
llm-latency-lens profile [OPTIONS]

REQUIRED OPTIONS:
  --provider <NAME>      Provider name (openai, anthropic, google)
  --model <NAME>         Model identifier (e.g., gpt-4, claude-3-opus)
  --prompt <TEXT>        Prompt text to send

OPTIONAL:
  --iterations <NUM>     Number of requests to execute [default: 10]
  --concurrency <NUM>    Concurrent requests [default: 1]
  --stream               Enable streaming responses
  --max-tokens <NUM>     Maximum tokens to generate [default: 500]
  --temperature <FLOAT>  Sampling temperature [default: 0.7]
  --top-p <FLOAT>        Nucleus sampling threshold [default: 1.0]
  --timeout <SECS>       Request timeout in seconds [default: 30]
  --output <FILE>        Output file path
  --format <FORMAT>      Output format (json, csv, table) [default: table]
  --warmup <NUM>         Warmup requests before measurement [default: 0]
```

**Examples:**

```bash
# Basic profile
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Explain machine learning"

# High concurrency test
llm-latency-lens profile \
  --provider openai \
  --model gpt-4-turbo-preview \
  --prompt "Hello" \
  --iterations 100 \
  --concurrency 20 \
  --stream

# Custom parameters
llm-latency-lens profile \
  --provider anthropic \
  --model claude-3-opus-20240229 \
  --prompt "Write a haiku about code" \
  --max-tokens 100 \
  --temperature 0.9 \
  --iterations 50
```

### `compare` - Compare multiple providers/models

Run comprehensive benchmarks across multiple providers and models.

```bash
llm-latency-lens compare [OPTIONS]

OPTIONS:
  --config <FILE>        Configuration file (YAML/JSON)
  --output <FILE>        Output file for results
  --format <FORMAT>      Output format [default: table]
  --parallel             Run providers in parallel
```

**Example config file:**

```yaml
# compare.yaml
providers:
  - name: openai
    models:
      - gpt-4-turbo-preview
      - gpt-3.5-turbo

  - name: anthropic
    models:
      - claude-3-opus-20240229
      - claude-3-sonnet-20240229

workload:
  scenarios:
    - name: short_response
      prompt: "What is 2+2?"
      requests: 100
      concurrency: 10

    - name: long_response
      prompt: "Explain quantum computing in detail"
      requests: 50
      concurrency: 5
      max_tokens: 1000

execution:
  max_concurrency: 50
  warmup_requests: 5
  timeout: 60

output:
  console: true
  export:
    - format: json
      path: ./results/benchmark_{timestamp}.json
```

```bash
llm-latency-lens compare --config compare.yaml
```

### `validate` - Validate provider credentials

Test API credentials and connectivity.

```bash
llm-latency-lens validate [OPTIONS]

OPTIONS:
  --provider <NAME>      Provider to validate (or 'all')
  --api-key <KEY>        API key (overrides environment variable)
```

**Examples:**

```bash
# Validate OpenAI credentials
llm-latency-lens validate --provider openai

# Validate all configured providers
llm-latency-lens validate --provider all

# Validate with explicit key
llm-latency-lens validate \
  --provider anthropic \
  --api-key sk-ant-...
```

### `providers` - List supported providers

Display all supported providers and their available models.

```bash
llm-latency-lens providers [OPTIONS]

OPTIONS:
  --provider <NAME>      Show details for specific provider
  --show-pricing         Include pricing information
```

**Examples:**

```bash
# List all providers
llm-latency-lens providers

# Show OpenAI details
llm-latency-lens providers --provider openai

# Show pricing
llm-latency-lens providers --show-pricing
```

### `analyze` - Analyze existing results

Post-process and analyze previously saved benchmark results.

```bash
llm-latency-lens analyze [OPTIONS] <FILE>

OPTIONS:
  --metric <NAME>        Metric to focus on (ttft, throughput, cost)
  --percentile <NUM>     Show specific percentile [default: 95]
  --compare <FILE>       Compare with another result file
  --format <FORMAT>      Output format [default: table]
```

**Examples:**

```bash
# Analyze results file
llm-latency-lens analyze results.json

# Focus on TTFT
llm-latency-lens analyze results.json --metric ttft

# Compare two runs
llm-latency-lens analyze run1.json --compare run2.json
```

---

## Configuration

### Configuration File Format

LLM-Latency-Lens supports YAML, JSON, and TOML configuration files.

**Complete YAML Example:**

```yaml
# config.yaml
version: "1.0"

# Provider configurations
providers:
  - name: openai
    endpoint: https://api.openai.com/v1
    auth:
      type: bearer
      token: ${OPENAI_API_KEY}  # Environment variable reference
    models:
      - gpt-4-turbo-preview
      - gpt-3.5-turbo
    settings:
      organization_id: null  # Optional organization ID

  - name: anthropic
    endpoint: https://api.anthropic.com/v1
    auth:
      type: api_key
      header: x-api-key
      token: ${ANTHROPIC_API_KEY}
    models:
      - claude-3-opus-20240229
      - claude-3-sonnet-20240229
    settings:
      api_version: "2023-06-01"

# Workload definition
workload:
  # Test scenarios
  scenarios:
    - name: quick_response
      prompt:
        template: simple_question
        variables:
          question: "What is the capital of France?"
      requests: 100
      concurrency: 20
      rate_limit: null  # No rate limit

    - name: detailed_response
      prompt:
        template: code_generation
        variables:
          task: "Write a Python function to calculate Fibonacci"
      requests: 50
      concurrency: 5
      rate_limit: 2.0  # 2 requests per second

  # Prompt templates
  prompts:
    - name: simple_question
      template: "{{question}}"

    - name: code_generation
      template: |
        You are an expert programmer. {{task}}.
        Provide clean, well-commented code with examples.

  # Default request parameters
  request_params:
    max_tokens: 500
    temperature: 0.7
    top_p: 0.9
    stream: true
    timeout: 30

# Execution configuration
execution:
  max_concurrency: 50
  warmup_requests: 5

  retry:
    max_attempts: 3
    initial_backoff_ms: 1000
    max_backoff_ms: 30000
    multiplier: 2.0

  http:
    pool_size: 100
    connect_timeout_ms: 5000
    keep_alive_secs: 90
    http2: true

# Output configuration
output:
  console:
    enabled: true
    format: table  # table, json, minimal
    show_progress: true

  export:
    - format: json
      path: ./results/benchmark_{timestamp}.json
      pretty: true

    - format: csv
      path: ./results/benchmark_{timestamp}.csv

    - format: binary
      path: ./results/benchmark_{timestamp}.bin
      codec: messagepack

  database:
    enabled: false
    type: influxdb
    connection:
      url: http://localhost:8086
      database: llm_benchmarks
```

### Configuration Precedence

Configuration values are resolved in this order (highest to lowest priority):

1. **CLI Arguments**: `--provider openai --model gpt-4`
2. **Environment Variables**: `OPENAI_API_KEY=sk-...`
3. **Config File**: `--config config.yaml`
4. **Default Values**: Built-in defaults

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `OPENAI_API_KEY` | OpenAI API key | `sk-...` |
| `ANTHROPIC_API_KEY` | Anthropic API key | `sk-ant-...` |
| `GOOGLE_API_KEY` | Google API key | `AIza...` |
| `LLM_LENS_LOG_LEVEL` | Logging level | `debug`, `info`, `warn`, `error` |
| `LLM_LENS_CONFIG` | Default config file | `./config.yaml` |
| `LLM_LENS_OUTPUT_DIR` | Default output directory | `./results` |

---

## Provider Configuration

### OpenAI

**Supported Models:**
- GPT-4: `gpt-4`, `gpt-4-turbo-preview`, `gpt-4-32k`
- GPT-4o: `gpt-4o`, `gpt-4o-mini`
- GPT-3.5: `gpt-3.5-turbo`, `gpt-3.5-turbo-16k`
- o1/o3: `o1-preview`, `o1-mini`, `o3-mini`

**Configuration:**

```yaml
providers:
  - name: openai
    endpoint: https://api.openai.com/v1
    auth:
      type: bearer
      token: ${OPENAI_API_KEY}
    settings:
      organization_id: org-...  # Optional
      project_id: proj_...       # Optional
```

**CLI Usage:**

```bash
# Basic usage
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Hello"

# With organization
export OPENAI_ORGANIZATION=org-...
llm-latency-lens profile --provider openai --model gpt-4 --prompt "Hello"
```

**Pricing (as of 2024):**
- GPT-4 Turbo: $10/M input tokens, $30/M output tokens
- GPT-4o: $5/M input tokens, $15/M output tokens
- GPT-3.5 Turbo: $0.50/M input tokens, $1.50/M output tokens

### Anthropic

**Supported Models:**
- Claude 3 Opus: `claude-3-opus-20240229`
- Claude 3 Sonnet: `claude-3-sonnet-20240229`
- Claude 3 Haiku: `claude-3-haiku-20240307`
- Claude 3.5 Sonnet: `claude-3-5-sonnet-20241022`

**Configuration:**

```yaml
providers:
  - name: anthropic
    endpoint: https://api.anthropic.com/v1
    auth:
      type: api_key
      header: x-api-key
      token: ${ANTHROPIC_API_KEY}
    settings:
      api_version: "2023-06-01"
```

**CLI Usage:**

```bash
# Basic usage
llm-latency-lens profile \
  --provider anthropic \
  --model claude-3-opus-20240229 \
  --prompt "Hello"

# With extended thinking
llm-latency-lens profile \
  --provider anthropic \
  --model claude-3-opus-20240229 \
  --prompt "Solve this complex problem..." \
  --extended-thinking
```

**Pricing (as of 2024):**
- Claude 3 Opus: $15/M input tokens, $75/M output tokens
- Claude 3 Sonnet: $3/M input tokens, $15/M output tokens
- Claude 3 Haiku: $0.25/M input tokens, $1.25/M output tokens

### Google (Coming Soon)

**Supported Models:**
- Gemini Pro: `gemini-pro`
- Gemini Ultra: `gemini-ultra`

**Configuration:**

```yaml
providers:
  - name: google
    endpoint: https://generativelanguage.googleapis.com/v1
    auth:
      type: api_key
      token: ${GOOGLE_API_KEY}
```

---

## Advanced Usage

### Custom Prompts and Templates

**Using Prompt Templates:**

```yaml
workload:
  prompts:
    - name: code_review
      template: |
        Review the following {{language}} code for:
        - Best practices
        - Performance issues
        - Security vulnerabilities

        Code:
        {{code}}

  scenarios:
    - name: review_python
      prompt:
        template: code_review
        variables:
          language: Python
          code: |
            def factorial(n):
                if n == 0: return 1
                return n * factorial(n-1)
      requests: 10
```

**Loading Prompts from Files:**

```bash
# Use a prompt from file
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt-file ./prompts/complex_task.txt \
  --iterations 50
```

### Rate Limiting

**Per-Provider Rate Limits:**

```yaml
providers:
  - name: openai
    rate_limit:
      requests_per_second: 10
      burst: 20  # Allow burst of 20 requests

  - name: anthropic
    rate_limit:
      requests_per_second: 5
      burst: 10
```

**CLI Rate Limiting:**

```bash
# Limit to 5 requests per second
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Hello" \
  --iterations 100 \
  --rate-limit 5
```

### Retry Configuration

**Configure Retry Behavior:**

```yaml
execution:
  retry:
    max_attempts: 5          # Maximum retry attempts
    initial_backoff_ms: 500  # Initial backoff delay
    max_backoff_ms: 60000    # Maximum backoff delay
    multiplier: 2.5          # Exponential multiplier
    jitter: true             # Add random jitter
```

**Retryable Errors:**
- Rate limit exceeded (429)
- Server errors (5xx)
- Timeout errors
- Connection errors

**Non-retryable Errors:**
- Authentication errors (401, 403)
- Invalid requests (400)
- Not found (404)

### Warmup Requests

Execute warmup requests before measurement to ensure fair benchmarking:

```bash
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Hello" \
  --iterations 100 \
  --warmup 10  # 10 warmup requests (not measured)
```

### Streaming vs Non-Streaming

**Streaming Mode:**
- Measures TTFT (Time to First Token)
- Tracks inter-token latency
- Shows token throughput
- Requires `--stream` flag

```bash
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Write a story" \
  --stream \
  --iterations 50
```

**Non-Streaming Mode:**
- Only measures total request time
- Lower memory usage
- Simpler output
- Default mode

```bash
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Write a story" \
  --iterations 50
```

---

## Integration Guides

### CI/CD Integration

#### GitHub Actions

```yaml
# .github/workflows/benchmark.yml
name: LLM Performance Benchmark

on:
  push:
    branches: [main]
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install LLM-Latency-Lens
        run: |
          cargo install llm-latency-lens

      - name: Run Benchmark
        env:
          OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
        run: |
          llm-latency-lens compare \
            --config .github/benchmark-config.yaml \
            --output benchmark-results.json

      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: benchmark-results.json

      - name: Check Performance Regression
        run: |
          # Add custom regression detection logic
          python scripts/check_regression.py benchmark-results.json
```

#### GitLab CI

```yaml
# .gitlab-ci.yml
benchmark:
  image: rust:latest
  stage: test
  script:
    - cargo install llm-latency-lens
    - llm-latency-lens compare --config benchmark.yaml --output results.json
  artifacts:
    paths:
      - results.json
    expire_in: 1 week
  only:
    - main
```

### Prometheus Integration

Export metrics to Prometheus for monitoring:

```yaml
# config.yaml
output:
  prometheus:
    enabled: true
    port: 9090
    path: /metrics
    labels:
      environment: production
      service: llm-api
```

**Scrape Configuration:**

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'llm-latency-lens'
    static_configs:
      - targets: ['localhost:9090']
```

**Available Metrics:**
- `llm_requests_total{provider, model, status}`
- `llm_request_duration_seconds{provider, model, quantile}`
- `llm_ttft_seconds{provider, model, quantile}`
- `llm_tokens_per_second{provider, model, quantile}`
- `llm_cost_dollars_total{provider, model}`

### Grafana Dashboards

Import pre-built dashboards from our [dashboards repository](https://github.com/llm-devops/llm-latency-lens-dashboards).

```bash
# Download dashboard
curl -o llm-dashboard.json \
  https://raw.githubusercontent.com/llm-devops/llm-latency-lens-dashboards/main/grafana/overview.json

# Import in Grafana UI
# Dashboards > Import > Upload JSON
```

---

## Performance Tuning

### Optimizing Concurrency

**Finding Optimal Concurrency:**

```bash
# Test different concurrency levels
for c in 1 5 10 20 50 100; do
  echo "Testing concurrency: $c"
  llm-latency-lens profile \
    --provider openai \
    --model gpt-4 \
    --prompt "Hello" \
    --iterations 100 \
    --concurrency $c \
    --output results-c${c}.json
done
```

**Guidelines:**
- Start with low concurrency (5-10) and increase gradually
- Monitor for rate limiting (429 errors)
- Watch for increased latency at high concurrency
- Consider provider-specific limits

### HTTP Configuration

**Optimize HTTP Client:**

```yaml
execution:
  http:
    pool_size: 200          # Increase for high concurrency
    connect_timeout_ms: 3000  # Reduce for faster failure
    keep_alive_secs: 120    # Keep connections alive longer
    http2: true             # Enable HTTP/2 multiplexing
    tcp_nodelay: true       # Disable Nagle's algorithm
```

### Memory Management

**For Large Benchmarks:**

```bash
# Limit iterations to manage memory
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Hello" \
  --iterations 10000 \
  --concurrency 50 \
  --output results.json \
  --stream-results  # Stream to disk instead of buffering

# Use binary format for smaller files
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Hello" \
  --iterations 10000 \
  --output results.bin \
  --format binary
```

---

## Troubleshooting

### Common Issues

#### Authentication Errors

**Problem:** `Error: Authentication failed`

**Solutions:**
```bash
# Verify API key is set
echo $OPENAI_API_KEY

# Test with explicit key
llm-latency-lens validate \
  --provider openai \
  --api-key sk-...

# Check for whitespace/newlines
export OPENAI_API_KEY=$(echo $OPENAI_API_KEY | tr -d '[:space:]')
```

#### Rate Limiting

**Problem:** `Error: Rate limit exceeded (429)`

**Solutions:**
```bash
# Reduce concurrency
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Hello" \
  --concurrency 5  # Lower concurrency

# Add rate limiting
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Hello" \
  --rate-limit 2  # 2 requests per second

# Increase retry backoff
# Edit config.yaml:
execution:
  retry:
    max_attempts: 5
    initial_backoff_ms: 2000
```

#### Timeout Errors

**Problem:** `Error: Request timeout`

**Solutions:**
```bash
# Increase timeout
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Long complex prompt..." \
  --timeout 120  # 120 seconds

# Reduce max tokens
llm-latency-lens profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Hello" \
  --max-tokens 100  # Limit response length
```

#### Connection Issues

**Problem:** `Error: Connection failed`

**Solutions:**
```bash
# Check network connectivity
curl https://api.openai.com/v1/models \
  -H "Authorization: Bearer $OPENAI_API_KEY"

# Use HTTP/1.1 instead of HTTP/2
# Edit config.yaml:
execution:
  http:
    http2: false

# Increase connection timeout
execution:
  http:
    connect_timeout_ms: 10000
```

### Debug Logging

Enable verbose logging to diagnose issues:

```bash
# Debug level logging
llm-latency-lens --log-level debug profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Hello"

# Save logs to file
llm-latency-lens --log-level debug profile \
  --provider openai \
  --model gpt-4 \
  --prompt "Hello" 2>&1 | tee debug.log
```

### Getting Help

If you encounter issues:

1. Check the [FAQ](#faq) below
2. Search [GitHub Issues](https://github.com/llm-devops/llm-latency-lens/issues)
3. Ask in [GitHub Discussions](https://github.com/llm-devops/llm-latency-lens/discussions)
4. Join our [Discord community](https://discord.gg/llm-latency-lens)

---

## FAQ

### General

**Q: Which providers are supported?**
A: Currently OpenAI and Anthropic are fully supported. Google Gemini support is coming soon. See the [provider configuration](#provider-configuration) section for details.

**Q: How accurate are the timing measurements?**
A: LLM-Latency-Lens uses hardware-based timing with nanosecond precision. Timing overhead is less than 100 nanoseconds per measurement.

**Q: Can I use this in production?**
A: Yes! The tool is designed for both development and production use. It has minimal overhead and won't affect your API quotas differently than normal usage.

**Q: Is there a GUI?**
A: Not currently. LLM-Latency-Lens is a CLI tool, but you can visualize results using Grafana dashboards or export to CSV for Excel/Google Sheets.

### Pricing & Costs

**Q: How are costs calculated?**
A: Costs are calculated based on current provider pricing (as of 2024) and actual token usage. Pricing is built into the tool and updated regularly.

**Q: Will benchmarking be expensive?**
A: Costs depend on your usage. A typical benchmark with 100 requests might cost $0.50-$5 depending on the model. Use smaller iterations for testing.

**Q: Can I set a cost limit?**
A: Not currently, but you can estimate costs before running:
```bash
# For GPT-4 Turbo: ~$0.012 per request (500 tokens)
# 100 requests = ~$1.20
```

### Performance

**Q: How many concurrent requests can I run?**
A: The tool supports 1000+ concurrent requests. However, you'll be limited by provider rate limits (typically 10-100 requests per second).

**Q: Does the tool affect the latency being measured?**
A: Overhead is minimal (<100ns per measurement). Network latency and API processing time dominate measurements.

**Q: Can I benchmark from multiple regions?**
A: Not currently in a single run. You can run the tool from multiple machines/regions and compare results.

### Features

**Q: Can I test custom/internal LLM APIs?**
A: Yes! Use the generic HTTP adapter (coming soon) to test any OpenAI-compatible API.

**Q: Does it support function calling?**
A: Not currently. This is on the roadmap for v0.2.0.

**Q: Can I export results to a database?**
A: Yes! InfluxDB and Prometheus export are supported. See the [integration guides](#integration-guides).

### Troubleshooting

**Q: Why am I getting rate limit errors?**
A: Reduce concurrency or add rate limiting. See [Rate Limiting](#rate-limiting) section.

**Q: Results seem inconsistent between runs. Why?**
A: LLM APIs have variable latency. Run more iterations and use warmup requests for more stable results.

**Q: Can I resume a failed benchmark?**
A: Not currently. This feature is planned for a future release.

---

## Next Steps

- **Read the [API Documentation](API.md)** to use LLM-Latency-Lens as a library
- **Explore [Architecture](ARCHITECTURE.md)** to understand the internals
- **Check the [Contributing Guide](../CONTRIBUTING.md)** to contribute
- **Join the community** on [Discord](https://discord.gg/llm-latency-lens)

---

**Questions or feedback?** Open an issue on [GitHub](https://github.com/llm-devops/llm-latency-lens/issues) or reach out on [Discord](https://discord.gg/llm-latency-lens).
