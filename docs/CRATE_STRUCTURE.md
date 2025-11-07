# Rust Crate Structure - Quick Reference

## Recommended Cargo.toml

```toml
[package]
name = "llm-latency-lens"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
authors = ["Your Team"]
license = "MIT OR Apache-2.0"
description = "High-performance latency profiler for Large Language Model APIs"
repository = "https://github.com/yourusername/llm-latency-lens"
keywords = ["llm", "benchmarking", "latency", "performance", "profiling"]
categories = ["command-line-utilities", "development-tools::profiling"]

[dependencies]
# Async Runtime
tokio = { version = "1.37", features = ["full"] }
futures = "0.3"
tokio-stream = "0.1"
async-stream = "0.3"

# HTTP Client
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"] }
eventsource-stream = "0.2"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"
rmp-serde = "1.1"

# Configuration
config = "0.14"
serde_yaml = "0.9"
toml = "0.8"

# CLI
clap = { version = "4.5", features = ["derive", "cargo", "env"] }
console = "0.15"
indicatif = "0.17"
tabled = "0.15"

# Timing & Statistics
quanta = "0.12"
hdrhistogram = "7.5"
statrs = "0.16"

# Error Handling
thiserror = "1.0"
anyhow = "1.0"

# Utilities
uuid = { version = "1.8", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
dotenvy = "0.15"

# Concurrency
dashmap = "5.5"
parking_lot = "0.12"
governor = "0.6"

# Retry Logic
backoff = { version = "0.4", features = ["tokio"] }

# Export Formats
csv = "1.3"

# Optional Features
influxdb = { version = "0.7", optional = true }
prometheus = { version = "0.13", optional = true }
redb = { version = "2.0", optional = true }

[dev-dependencies]
tokio-test = "0.4"
mockito = "1.4"
criterion = "0.5"
proptest = "1.4"
serial_test = "3.0"
temp-env = "0.3"

[features]
default = []
influxdb = ["dep:influxdb"]
prometheus = ["dep:prometheus"]
storage = ["dep:redb"]

[[bin]]
name = "llm-bench"
path = "src/main.rs"

[lib]
name = "llm_latency_lens"
path = "src/lib.rs"

[[bench]]
name = "metrics_bench"
harness = false

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[profile.dev]
opt-level = 0

[profile.test]
opt-level = 1
```

## File Structure with Estimated Lines of Code

```
llm-latency-lens/
├── Cargo.toml                          (~100 lines)
├── Cargo.lock                          (generated)
├── README.md                           (~200 lines)
├── ARCHITECTURE.md                     (this document)
├── LICENSE                             (standard)
│
├── src/
│   ├── main.rs                         (~150 lines) - CLI entry point
│   ├── lib.rs                          (~50 lines)  - Library exports
│   │
│   ├── config/
│   │   ├── mod.rs                      (~100 lines) - Module exports
│   │   ├── loader.rs                   (~200 lines) - Config file loading
│   │   ├── validation.rs               (~150 lines) - Schema validation
│   │   └── defaults.rs                 (~100 lines) - Default configs
│   │
│   ├── providers/
│   │   ├── mod.rs                      (~150 lines) - Provider registry
│   │   ├── traits.rs                   (~200 lines) - Core traits
│   │   ├── openai.rs                   (~400 lines) - OpenAI implementation
│   │   ├── anthropic.rs                (~400 lines) - Anthropic implementation
│   │   ├── google.rs                   (~400 lines) - Google Vertex AI
│   │   ├── azure.rs                    (~350 lines) - Azure OpenAI
│   │   ├── cohere.rs                   (~350 lines) - Cohere
│   │   ├── generic.rs                  (~300 lines) - Generic HTTP adapter
│   │   └── pricing.rs                  (~150 lines) - Pricing database
│   │
│   ├── executor/
│   │   ├── mod.rs                      (~100 lines) - Orchestration
│   │   ├── scheduler.rs                (~300 lines) - Workload scheduling
│   │   ├── worker.rs                   (~250 lines) - Worker pool
│   │   ├── rate_limiter.rs             (~150 lines) - Rate limiting
│   │   └── retry.rs                    (~200 lines) - Retry logic
│   │
│   ├── http/
│   │   ├── mod.rs                      (~100 lines) - HTTP module
│   │   ├── client.rs                   (~300 lines) - Pooled client
│   │   ├── streaming.rs                (~250 lines) - SSE handling
│   │   └── middleware.rs               (~200 lines) - Request/response middleware
│   │
│   ├── metrics/
│   │   ├── mod.rs                      (~100 lines) - Metrics exports
│   │   ├── collector.rs                (~300 lines) - Collection logic
│   │   ├── timer.rs                    (~150 lines) - Precision timing
│   │   ├── aggregator.rs               (~350 lines) - Statistical aggregation
│   │   ├── histogram.rs                (~200 lines) - HDR histogram wrapper
│   │   └── cost.rs                     (~150 lines) - Cost calculations
│   │
│   ├── storage/
│   │   ├── mod.rs                      (~100 lines) - Storage backends
│   │   ├── json.rs                     (~150 lines) - JSON export
│   │   ├── binary.rs                   (~150 lines) - Binary serialization
│   │   ├── csv.rs                      (~200 lines) - CSV export
│   │   └── timeseries.rs               (~250 lines) - Time-series DB
│   │
│   ├── cli/
│   │   ├── mod.rs                      (~50 lines)  - CLI module
│   │   ├── args.rs                     (~200 lines) - Argument parsing
│   │   ├── output.rs                   (~300 lines) - Console formatting
│   │   └── progress.rs                 (~150 lines) - Progress bars
│   │
│   └── models/
│       ├── mod.rs                      (~50 lines)  - Model exports
│       ├── config.rs                   (~300 lines) - Config structs
│       ├── metrics.rs                  (~400 lines) - Metrics structs
│       ├── request.rs                  (~200 lines) - Request/response types
│       └── error.rs                    (~250 lines) - Error types
│
├── tests/
│   ├── integration/
│   │   ├── openai_test.rs              (~200 lines)
│   │   ├── anthropic_test.rs           (~200 lines)
│   │   ├── concurrent_test.rs          (~150 lines)
│   │   ├── config_test.rs              (~150 lines)
│   │   └── metrics_test.rs             (~200 lines)
│   │
│   └── fixtures/
│       ├── configs/
│       │   ├── basic.yaml
│       │   ├── advanced.yaml
│       │   └── invalid.yaml
│       └── responses/
│           ├── openai_response.json
│           └── anthropic_response.json
│
├── benches/
│   └── metrics_bench.rs                (~150 lines)
│
├── examples/
│   ├── simple_benchmark.rs             (~100 lines)
│   ├── streaming_benchmark.rs          (~150 lines)
│   └── multi_provider.rs               (~200 lines)
│
└── docs/
    ├── USER_GUIDE.md
    ├── API_REFERENCE.md
    └── CONTRIBUTING.md

Total Estimated Lines: ~12,000-15,000 lines
```

## Module Dependency Graph

```
main.rs
  └─> cli::args
      └─> config::loader
          └─> models::config
              └─> providers::traits
                  ├─> providers::openai
                  ├─> providers::anthropic
                  └─> providers::generic
                      └─> http::client
                          └─> http::streaming
                              └─> metrics::collector
                                  ├─> metrics::timer
                                  ├─> metrics::histogram
                                  └─> metrics::aggregator
                                      └─> storage::json
                                          └─> cli::output
```

## Key Implementation Order

### Phase 1: Foundation
1. `models/error.rs` - Error types first
2. `models/config.rs` - Configuration structures
3. `models/metrics.rs` - Metrics data structures
4. `config/loader.rs` - Configuration loading
5. `metrics/timer.rs` - Precision timing utilities

### Phase 2: Core Infrastructure
6. `http/client.rs` - HTTP client setup
7. `http/streaming.rs` - SSE parsing
8. `metrics/collector.rs` - Metrics collection
9. `metrics/histogram.rs` - HDR histogram wrapper

### Phase 3: Provider Layer
10. `providers/traits.rs` - Provider abstraction
11. `providers/openai.rs` - First provider implementation
12. `providers/anthropic.rs` - Second provider
13. `providers/generic.rs` - Generic adapter

### Phase 4: Execution Engine
14. `executor/retry.rs` - Retry logic
15. `executor/rate_limiter.rs` - Rate limiting
16. `executor/worker.rs` - Worker pool
17. `executor/scheduler.rs` - Orchestration

### Phase 5: Analysis & Output
18. `metrics/aggregator.rs` - Statistical computation
19. `metrics/cost.rs` - Cost calculations
20. `storage/json.rs` - JSON export
21. `cli/output.rs` - Console formatting
22. `cli/args.rs` - CLI argument parsing
23. `main.rs` - Main entry point

## Critical Files (Priority Implementation)

### High Priority (Week 1)
1. `models/error.rs`
2. `models/config.rs`
3. `models/metrics.rs`
4. `metrics/timer.rs`
5. `config/loader.rs`

### Medium Priority (Week 2)
6. `http/client.rs`
7. `providers/traits.rs`
8. `providers/openai.rs`
9. `metrics/collector.rs`

### Standard Priority (Weeks 3-4)
10. All remaining provider implementations
11. Executor components
12. Storage backends
13. CLI interface

## Testing Strategy by Module

### Unit Tests (High Coverage Required)
- `metrics/timer.rs` - Timing accuracy tests
- `metrics/aggregator.rs` - Statistical correctness
- `metrics/histogram.rs` - Percentile calculations
- `executor/retry.rs` - Retry logic verification
- `config/validation.rs` - Schema validation

### Integration Tests (Critical Paths)
- Provider implementations (mocked HTTP)
- End-to-end benchmark execution
- Concurrent request handling
- Metrics aggregation pipeline

### Performance Benchmarks
- Metrics collection overhead
- Histogram operations
- Concurrent request throughput
- Memory usage under load

## Build and CI Configuration

### .cargo/config.toml
```toml
[build]
incremental = true

[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "target-cpu=native"]

[alias]
bench-all = "bench --all-features"
test-all = "test --all-features"
```

### GitHub Actions CI
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Run tests
        run: cargo test --all-features
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Check formatting
        run: cargo fmt -- --check

  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Run benchmarks
        run: cargo bench --all-features
```

## Development Workflow

### Local Development
```bash
# Run tests
cargo test

# Run specific test
cargo test test_metrics_collection

# Run with logging
RUST_LOG=debug cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Build release
cargo build --release

# Run benchmarks
cargo bench

# Generate documentation
cargo doc --open
```

### Pre-commit Checklist
- [ ] All tests pass: `cargo test --all-features`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code formatted: `cargo fmt`
- [ ] Documentation updated
- [ ] Changelog entry added (if applicable)

## Memory and Performance Budgets

### Memory Budget (per 1000 requests)
- Request metadata: ~500 KB
- Timing data: ~200 KB
- Histograms: ~100 KB
- Total: <1 MB

### Timing Overhead Budget
- Timer creation: <10 ns
- Timestamp capture: <50 ns
- Metric recording: <1 μs
- Total per request: <5 μs

### Throughput Target
- Minimum: 100 requests/sec/core
- Target: 500 requests/sec/core
- Stretch: 1000 requests/sec/core

## Common Patterns

### Provider Implementation Template
```rust
pub struct MyProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl MyProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            api_key: config.api_key,
            base_url: config.endpoint,
        })
    }
}

#[async_trait]
impl LLMProvider for MyProvider {
    // Implement trait methods
}
```

### Metrics Collection Pattern
```rust
let timer = PrecisionTimer::new();
let start = timer.now();

// Perform operation
let result = operation().await?;

let duration_ns = timer.elapsed_nanos(start);

collector.record(RequestMetrics {
    timing: TimingMetrics {
        total_duration_ns: duration_ns,
        ..Default::default()
    },
    ..Default::default()
}).await;
```

### Error Handling Pattern
```rust
async fn execute_with_context() -> Result<Response> {
    api_call().await
        .map_err(|e| ProviderError::ApiError {
            status_code: e.status_code(),
            message: e.to_string(),
        })
        .context("Failed to execute API call")?;

    Ok(response)
}
```
