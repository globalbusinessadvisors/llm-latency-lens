# LLM-Latency-Lens: Architecture Design Document

## Executive Summary

LLM-Latency-Lens is a high-performance, production-grade latency profiler for Large Language Model APIs. Built in Rust, it provides comprehensive performance metrics, concurrent benchmarking capabilities, and multi-provider support with sub-millisecond timing precision.

---

## 1. System Architecture

### 1.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI Interface                            │
│                  (clap, console output)                          │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────┴────────────────────────────────────┐
│                    Configuration Layer                           │
│         (YAML/JSON config, CLI args, environment vars)           │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────┴────────────────────────────────────┐
│                  Orchestration Engine                            │
│    - Workload Scheduler                                          │
│    - Concurrency Controller                                      │
│    - Provider Dispatcher                                         │
└──────────┬──────────────────────────────────────┬───────────────┘
           │                                      │
┌──────────┴──────────┐              ┌───────────┴──────────────┐
│  Request Executor   │              │   Metrics Collector      │
│  - HTTP Client Pool │              │   - Timing Pipeline      │
│  - Retry Logic      │◄────────────►│   - Statistics Engine    │
│  - Rate Limiting    │              │   - Aggregation          │
└──────────┬──────────┘              └───────────┬──────────────┘
           │                                     │
┌──────────┴──────────┐              ┌───────────┴──────────────┐
│  Provider Adapters  │              │   Storage Layer          │
│  - OpenAI           │              │   - Time-series DB       │
│  - Anthropic        │              │   - JSON Export          │
│  - Google           │              │   - Binary Format        │
│  - Azure            │              │   - Streaming Output     │
│  - Cohere           │              └──────────────────────────┘
│  - Custom/Generic   │
└─────────────────────┘
```

### 1.2 Core Components

#### 1.2.1 Configuration Layer
- **Purpose**: Unified configuration management
- **Features**:
  - YAML/JSON file parsing
  - CLI argument override
  - Environment variable support
  - Schema validation
  - Default presets

#### 1.2.2 Orchestration Engine
- **Purpose**: Coordinate concurrent benchmark execution
- **Components**:
  - **Workload Scheduler**: Manages test scenarios and execution order
  - **Concurrency Controller**: Controls parallel request limits
  - **Provider Dispatcher**: Routes requests to appropriate adapters

#### 1.2.3 Request Executor
- **Purpose**: Execute HTTP requests with precise timing
- **Features**:
  - Connection pooling
  - Automatic retry with exponential backoff
  - Rate limiting per provider
  - Request/response streaming
  - Timeout management

#### 1.2.4 Metrics Collector
- **Purpose**: Capture and aggregate performance data
- **Components**:
  - Timing pipeline with nanosecond precision
  - Statistical aggregation
  - Real-time metric computation
  - Histogram generation

#### 1.2.5 Provider Adapters
- **Purpose**: Abstract provider-specific API differences
- **Features**:
  - Unified request/response interface
  - Provider-specific authentication
  - Model catalog management
  - Streaming vs non-streaming handling

---

## 2. Data Flow Architecture

### 2.1 Request Lifecycle

```
┌─────────────┐
│   Config    │
│   Loaded    │
└──────┬──────┘
       │
       ▼
┌─────────────────────┐
│  Generate Workload  │
│  - Test scenarios   │
│  - Request matrix   │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────────────────┐
│  Concurrent Execution           │
│  ┌──────────────────────┐       │
│  │ For each request:    │       │
│  │                      │       │
│  │ 1. Start timing      │       │
│  │ 2. Send HTTP request │       │
│  │ 3. Record TTFT       │       │
│  │ 4. Stream tokens     │       │
│  │ 5. Record completion │       │
│  │ 6. Calculate metrics │       │
│  └──────────────────────┘       │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────┐
│  Aggregate Metrics  │
│  - Group by model   │
│  - Compute stats    │
│  - Generate reports │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│  Output Results     │
│  - Console display  │
│  - JSON export      │
│  - Database write   │
└─────────────────────┘
```

### 2.2 Timing Pipeline

```
Request Start
    │
    ├─► DNS Lookup Time
    │
    ├─► TCP Connection Time
    │
    ├─► TLS Handshake Time
    │
    ├─► Request Send Time
    │
    ├─► Time to First Byte (TTFB)
    │
    ├─► Time to First Token (TTFT) ◄── Critical Metric
    │
    ├─► Token Streaming Phase
    │   ├─► Inter-token latency
    │   └─► Token throughput
    │
    └─► Total Request Time ◄── Critical Metric
```

---

## 3. Data Models & Schemas

### 3.1 Core Data Structures

```rust
/// Configuration for a benchmark run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Provider configurations
    pub providers: Vec<ProviderConfig>,

    /// Workload specification
    pub workload: WorkloadConfig,

    /// Execution parameters
    pub execution: ExecutionConfig,

    /// Output configuration
    pub output: OutputConfig,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider name (e.g., "openai", "anthropic")
    pub name: String,

    /// API endpoint URL
    pub endpoint: String,

    /// Authentication details
    pub auth: AuthConfig,

    /// Models to test
    pub models: Vec<String>,

    /// Provider-specific settings
    pub settings: HashMap<String, Value>,
}

/// Workload configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadConfig {
    /// Test scenarios to run
    pub scenarios: Vec<Scenario>,

    /// Prompt templates
    pub prompts: Vec<PromptTemplate>,

    /// Request parameters
    pub request_params: RequestParams,
}

/// Individual test scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Scenario name
    pub name: String,

    /// Prompt configuration
    pub prompt: PromptConfig,

    /// Number of requests
    pub requests: usize,

    /// Concurrency level
    pub concurrency: usize,

    /// Optional rate limit (requests per second)
    pub rate_limit: Option<f64>,
}

/// Request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestParams {
    /// Maximum tokens to generate
    pub max_tokens: u32,

    /// Temperature setting
    pub temperature: f32,

    /// Top-p sampling
    pub top_p: Option<f32>,

    /// Enable streaming
    pub stream: bool,

    /// Request timeout (seconds)
    pub timeout: u64,
}

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Maximum concurrent requests across all providers
    pub max_concurrency: usize,

    /// Warmup requests before measurement
    pub warmup_requests: usize,

    /// Retry configuration
    pub retry: RetryConfig,

    /// HTTP client settings
    pub http: HttpConfig,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,

    /// Initial backoff duration (ms)
    pub initial_backoff_ms: u64,

    /// Maximum backoff duration (ms)
    pub max_backoff_ms: u64,

    /// Backoff multiplier
    pub multiplier: f64,
}

/// HTTP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Connection pool size per provider
    pub pool_size: usize,

    /// Connection timeout (ms)
    pub connect_timeout_ms: u64,

    /// Keep-alive duration (seconds)
    pub keep_alive_secs: u64,

    /// Enable HTTP/2
    pub http2: bool,
}
```

### 3.2 Metrics Data Structures

```rust
/// Single request measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Unique request ID
    pub request_id: Uuid,

    /// Timestamp when request started
    pub timestamp: DateTime<Utc>,

    /// Provider name
    pub provider: String,

    /// Model name
    pub model: String,

    /// Timing breakdown
    pub timing: TimingMetrics,

    /// Token statistics
    pub tokens: TokenMetrics,

    /// Cost information
    pub cost: CostMetrics,

    /// Request status
    pub status: RequestStatus,

    /// Error information (if failed)
    pub error: Option<ErrorInfo>,
}

/// Detailed timing measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingMetrics {
    /// DNS lookup duration (ns)
    pub dns_lookup_ns: Option<u64>,

    /// TCP connection duration (ns)
    pub tcp_connect_ns: Option<u64>,

    /// TLS handshake duration (ns)
    pub tls_handshake_ns: Option<u64>,

    /// Request send duration (ns)
    pub request_send_ns: u64,

    /// Time to first byte (ns)
    pub ttfb_ns: u64,

    /// Time to first token (ns) - Critical metric
    pub ttft_ns: u64,

    /// Total request duration (ns)
    pub total_duration_ns: u64,

    /// Token streaming durations (ns between tokens)
    pub token_latencies_ns: Vec<u64>,
}

/// Token statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetrics {
    /// Prompt tokens
    pub prompt_tokens: u32,

    /// Completion tokens
    pub completion_tokens: u32,

    /// Total tokens
    pub total_tokens: u32,

    /// Tokens per second (throughput)
    pub tokens_per_second: f64,

    /// Mean inter-token latency (ms)
    pub mean_inter_token_latency_ms: f64,

    /// Token latency percentiles
    pub token_latency_p50_ms: f64,
    pub token_latency_p95_ms: f64,
    pub token_latency_p99_ms: f64,
}

/// Cost calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMetrics {
    /// Prompt cost (USD)
    pub prompt_cost: f64,

    /// Completion cost (USD)
    pub completion_cost: f64,

    /// Total cost (USD)
    pub total_cost: f64,

    /// Cost per token (USD)
    pub cost_per_token: f64,
}

/// Request status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestStatus {
    Success,
    Failed,
    Timeout,
    RateLimited,
    Retried { attempts: u32 },
}

/// Error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error code
    pub code: String,

    /// Error message
    pub message: String,

    /// HTTP status code
    pub status_code: Option<u16>,

    /// Retry attempt when error occurred
    pub retry_attempt: u32,
}
```

### 3.3 Aggregated Statistics

```rust
/// Aggregated metrics for a scenario/model combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Provider name
    pub provider: String,

    /// Model name
    pub model: String,

    /// Scenario name
    pub scenario: String,

    /// Total requests executed
    pub total_requests: usize,

    /// Successful requests
    pub successful_requests: usize,

    /// Failed requests
    pub failed_requests: usize,

    /// Latency statistics
    pub latency: LatencyStats,

    /// Token statistics
    pub token_stats: AggregatedTokenStats,

    /// Cost statistics
    pub cost_stats: AggregatedCostStats,

    /// Error breakdown
    pub errors: HashMap<String, usize>,

    /// Execution duration
    pub execution_duration_secs: f64,

    /// Actual requests per second
    pub actual_rps: f64,
}

/// Latency distribution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    /// Time to First Token (TTFT) statistics
    pub ttft: DistributionStats,

    /// Total request duration statistics
    pub total_duration: DistributionStats,

    /// Token throughput (tokens/sec) statistics
    pub throughput: DistributionStats,

    /// Inter-token latency statistics
    pub inter_token_latency: DistributionStats,
}

/// Statistical distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStats {
    /// Minimum value
    pub min: f64,

    /// Maximum value
    pub max: f64,

    /// Mean value
    pub mean: f64,

    /// Median (p50)
    pub median: f64,

    /// Standard deviation
    pub std_dev: f64,

    /// 95th percentile
    pub p95: f64,

    /// 99th percentile
    pub p99: f64,

    /// 99.9th percentile
    pub p999: f64,
}

/// Aggregated token statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedTokenStats {
    /// Total prompt tokens
    pub total_prompt_tokens: u64,

    /// Total completion tokens
    pub total_completion_tokens: u64,

    /// Total tokens
    pub total_tokens: u64,

    /// Mean tokens per second
    pub mean_tokens_per_second: f64,

    /// Tokens per second distribution
    pub tokens_per_second_stats: DistributionStats,
}

/// Aggregated cost statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedCostStats {
    /// Total cost (USD)
    pub total_cost: f64,

    /// Mean cost per request (USD)
    pub mean_cost_per_request: f64,

    /// Cost per 1K tokens (USD)
    pub cost_per_1k_tokens: f64,
}
```

---

## 4. Module Structure

### 4.1 Recommended Crate Organization

```
llm-latency-lens/
├── Cargo.toml
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Library exports
│   │
│   ├── config/
│   │   ├── mod.rs              # Configuration module
│   │   ├── loader.rs           # Config file parsing
│   │   ├── validation.rs       # Schema validation
│   │   └── defaults.rs         # Default configurations
│   │
│   ├── providers/
│   │   ├── mod.rs              # Provider abstraction
│   │   ├── traits.rs           # Provider trait definitions
│   │   ├── openai.rs           # OpenAI adapter
│   │   ├── anthropic.rs        # Anthropic adapter
│   │   ├── google.rs           # Google Vertex AI adapter
│   │   ├── azure.rs            # Azure OpenAI adapter
│   │   ├── cohere.rs           # Cohere adapter
│   │   └── generic.rs          # Generic HTTP adapter
│   │
│   ├── executor/
│   │   ├── mod.rs              # Execution orchestration
│   │   ├── scheduler.rs        # Workload scheduler
│   │   ├── worker.rs           # Request worker pool
│   │   ├── rate_limiter.rs     # Rate limiting logic
│   │   └── retry.rs            # Retry logic
│   │
│   ├── http/
│   │   ├── mod.rs              # HTTP client management
│   │   ├── client.rs           # Pooled HTTP client
│   │   ├── streaming.rs        # SSE/streaming handler
│   │   └── middleware.rs       # Request/response middleware
│   │
│   ├── metrics/
│   │   ├── mod.rs              # Metrics collection
│   │   ├── collector.rs        # Metric collector
│   │   ├── timer.rs            # High-precision timing
│   │   ├── aggregator.rs       # Statistical aggregation
│   │   ├── histogram.rs        # Histogram implementation
│   │   └── cost.rs             # Cost calculation
│   │
│   ├── storage/
│   │   ├── mod.rs              # Storage backends
│   │   ├── json.rs             # JSON export
│   │   ├── binary.rs           # Binary format (MessagePack/Bincode)
│   │   ├── csv.rs              # CSV export
│   │   └── timeseries.rs       # Time-series DB integration
│   │
│   ├── cli/
│   │   ├── mod.rs              # CLI interface
│   │   ├── args.rs             # Argument parsing
│   │   ├── output.rs           # Console output formatting
│   │   └── progress.rs         # Progress indicators
│   │
│   └── models/
│       ├── mod.rs              # Data models
│       ├── config.rs           # Configuration structs
│       ├── metrics.rs          # Metrics structs
│       └── error.rs            # Error types
│
└── tests/
    ├── integration/
    │   ├── openai_test.rs
    │   ├── anthropic_test.rs
    │   └── concurrent_test.rs
    └── fixtures/
        ├── configs/
        └── responses/
```

---

## 5. Rust Crate Selection & Justification

### 5.1 Core Dependencies

```toml
[dependencies]
# Async Runtime
tokio = { version = "1.37", features = ["full"] }
# Justification: Industry standard, mature, excellent performance for concurrent I/O

# HTTP Client
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"] }
# Justification: High-level, ergonomic API, built on hyper, excellent async support
# Features: JSON serialization, streaming, TLS with rustls

# Alternative for fine-grained control:
# hyper = { version = "1.0", features = ["full"] }
# hyper-util = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
# Justification: De facto standard for Rust serialization

# Binary serialization
bincode = "1.3"
# Justification: Fast binary encoding for storage
rmp-serde = "1.1"  # MessagePack alternative
# Justification: More compact than bincode, wider ecosystem support

# Configuration
config = "0.14"
# Justification: Flexible config file parsing with layered configs
serde_yaml = "0.9"
# Justification: YAML config file support
toml = "0.8"
# Justification: TOML support for Rust-native configs

# CLI
clap = { version = "4.5", features = ["derive", "cargo"] }
# Justification: Powerful, ergonomic CLI with derive macros
console = "0.15"
# Justification: Terminal styling and user interaction
indicatif = "0.17"
# Justification: Progress bars and spinners

# Timing & Performance
quanta = "0.12"
# Justification: High-resolution, low-overhead timing
# Alternative: instant = "0.1" (simpler but less precise)

# Statistics
hdrhistogram = "7.5"
# Justification: High Dynamic Range histogram for latency percentiles
# Memory-efficient, accurate percentile calculation
statrs = "0.16"
# Justification: Statistical functions (mean, std dev, etc.)

# Error Handling
thiserror = "1.0"
# Justification: Ergonomic error type derivation
anyhow = "1.0"
# Justification: Flexible error handling for application code

# Async Utilities
futures = "0.3"
# Justification: Async utilities, stream combinators
tokio-stream = "0.1"
# Justification: Async stream utilities for token streaming
async-stream = "0.3"
# Justification: Ergonomic async stream creation

# Rate Limiting
governor = "0.6"
# Justification: Token-bucket rate limiter with Tokio support

# Retry Logic
backoff = { version = "0.4", features = ["tokio"] }
# Justification: Exponential backoff with jitter

# UUID Generation
uuid = { version = "1.8", features = ["v4", "serde"] }
# Justification: Request ID generation

# Date/Time
chrono = { version = "0.4", features = ["serde"] }
# Justification: Date/time handling and formatting

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
# Justification: Structured logging with async support

# Environment Variables
dotenvy = "0.15"
# Justification: .env file support for API keys

# HTTP/SSE Streaming
eventsource-stream = "0.2"
# Justification: Server-Sent Events parsing for streaming responses

# JSON Streaming
serde_json_path = "0.6"
# Justification: JSONPath queries for streaming token extraction

# Concurrency
dashmap = "5.5"
# Justification: Concurrent HashMap for metrics collection
parking_lot = "0.12"
# Justification: Faster RwLock/Mutex implementations

# CSV Export
csv = "1.3"
# Justification: Fast CSV writing

# Pretty Tables
tabled = "0.15"
# Justification: Rich table formatting for console output
```

### 5.2 Development Dependencies

```toml
[dev-dependencies]
# Testing
tokio-test = "0.4"
mockito = "1.4"
# Justification: HTTP mocking for tests
criterion = "0.5"
# Justification: Benchmarking framework
proptest = "1.4"
# Justification: Property-based testing

# Test Utilities
serial_test = "3.0"
# Justification: Serialize test execution for integration tests
temp-env = "0.3"
# Justification: Temporary environment variables in tests
```

### 5.3 Optional Dependencies

```toml
[dependencies]
# Time-series Database Integration
influxdb = { version = "0.7", optional = true }
# Justification: InfluxDB client for time-series storage
prometheus = { version = "0.13", optional = true }
# Justification: Prometheus metrics export

# Advanced Storage
redb = { version = "2.0", optional = true }
# Justification: Embedded database for local storage
rocksdb = { version = "0.22", optional = true }
# Justification: High-performance key-value store

# Profiling
pprof = { version = "0.13", features = ["flamegraph"], optional = true }
# Justification: CPU profiling and flamegraph generation

[features]
default = []
influxdb = ["dep:influxdb"]
prometheus = ["dep:prometheus"]
storage = ["dep:redb"]
profiling = ["dep:pprof"]
```

---

## 6. Performance Considerations

### 6.1 Concurrency Strategy

**Design Pattern: Work-Stealing Thread Pool with Bounded Channels**

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct ConcurrencyController {
    semaphore: Arc<Semaphore>,
    max_concurrency: usize,
}

impl ConcurrencyController {
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
            max_concurrency,
        }
    }

    pub async fn acquire(&self) -> SemaphorePermit {
        self.semaphore.acquire().await.unwrap()
    }
}

// Usage:
// let controller = ConcurrencyController::new(100);
// let _permit = controller.acquire().await;
// execute_request().await;
// drop(_permit); // Automatically releases
```

**Key Decisions:**
1. Use Tokio semaphores for concurrency limiting
2. Implement per-provider rate limiters
3. Use bounded channels to prevent memory exhaustion
4. Connection pooling with keep-alive for reduced latency

### 6.2 Memory Management

**Strategies:**
1. **Streaming Response Handling**: Don't buffer entire responses
2. **Ring Buffer for Metrics**: Fixed-size buffers for real-time metrics
3. **Lazy Histogram Allocation**: Only allocate histograms when needed
4. **Arc/Rc for Shared Config**: Avoid cloning large config objects

```rust
// Example: Streaming token handler
async fn handle_streaming_response(
    response: reqwest::Response,
    metrics: Arc<MetricsCollector>,
) -> Result<Vec<String>> {
    let mut stream = response.bytes_stream();
    let mut tokens = Vec::new();
    let mut first_token = true;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if first_token {
            metrics.record_ttft();
            first_token = false;
        }

        // Parse SSE and extract tokens
        let token = parse_sse_token(&chunk)?;
        tokens.push(token);
        metrics.record_token_received();
    }

    Ok(tokens)
}
```

### 6.3 Timing Precision

**Requirements:**
- Sub-millisecond precision for TTFT
- Nanosecond-resolution timestamps
- Minimal timing overhead

**Implementation:**

```rust
use quanta::Clock;

pub struct PrecisionTimer {
    clock: Clock,
}

impl PrecisionTimer {
    pub fn new() -> Self {
        Self {
            clock: Clock::new(),
        }
    }

    pub fn now(&self) -> u64 {
        self.clock.raw()
    }

    pub fn elapsed_nanos(&self, start: u64) -> u64 {
        let end = self.clock.raw();
        self.clock.delta(start, end)
    }
}

// Usage:
// let timer = PrecisionTimer::new();
// let start = timer.now();
// perform_operation().await;
// let duration_ns = timer.elapsed_nanos(start);
```

### 6.4 Statistical Accuracy

**HDR Histogram Configuration:**

```rust
use hdrhistogram::Histogram;

pub fn create_latency_histogram() -> Histogram<u64> {
    // Configure for 1 nanosecond to 60 seconds range
    // 3 significant figures of precision
    Histogram::<u64>::new_with_bounds(1, 60_000_000_000, 3)
        .expect("Failed to create histogram")
}

pub fn compute_percentiles(histogram: &Histogram<u64>) -> LatencyPercentiles {
    LatencyPercentiles {
        p50: histogram.value_at_quantile(0.50) as f64 / 1_000_000.0, // Convert to ms
        p95: histogram.value_at_quantile(0.95) as f64 / 1_000_000.0,
        p99: histogram.value_at_quantile(0.99) as f64 / 1_000_000.0,
        p999: histogram.value_at_quantile(0.999) as f64 / 1_000_000.0,
    }
}
```

---

## 7. Provider Abstraction Layer

### 7.1 Provider Trait Design

```rust
use async_trait::async_trait;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Provider identification
    fn name(&self) -> &str;

    /// Available models
    async fn list_models(&self) -> Result<Vec<String>>;

    /// Execute a completion request
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse>;

    /// Execute a streaming completion request
    async fn complete_streaming(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>>;

    /// Validate credentials
    async fn validate_credentials(&self) -> Result<bool>;

    /// Get pricing information
    fn get_pricing(&self, model: &str) -> Option<PricingInfo>;
}

/// Unified completion request
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub model: String,
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: Option<f32>,
    pub stream: bool,
    pub timeout: Duration,
}

/// Unified completion response
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub id: String,
    pub model: String,
    pub content: String,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
}

/// Token usage information
#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Stream chunk for streaming responses
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub id: String,
    pub model: String,
    pub delta: String,
    pub finish_reason: Option<FinishReason>,
}
```

### 7.2 Provider Implementation Example

```rust
pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    pricing: HashMap<String, PricingInfo>,
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse> {
        let timer = PrecisionTimer::new();
        let start = timer.now();

        let response = self.client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&OpenAIChatRequest::from(request))
            .timeout(request.timeout)
            .send()
            .await?;

        let elapsed = timer.elapsed_nanos(start);

        let openai_response: OpenAIChatResponse = response.json().await?;

        Ok(CompletionResponse::from(openai_response))
    }

    async fn complete_streaming(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>> {
        let response = self.client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&OpenAIChatRequest::from(request))
            .timeout(request.timeout)
            .send()
            .await?;

        let stream = response.bytes_stream();
        let parsed_stream = stream.map(|chunk| {
            // Parse SSE format: "data: {...}\n\n"
            parse_openai_sse_chunk(chunk?)
        });

        Ok(Box::pin(parsed_stream))
    }

    fn get_pricing(&self, model: &str) -> Option<PricingInfo> {
        self.pricing.get(model).cloned()
    }
}
```

---

## 8. Configuration Schema

### 8.1 Example YAML Configuration

```yaml
# LLM Latency Lens Configuration
version: "1.0"

# Provider configurations
providers:
  - name: openai
    endpoint: https://api.openai.com/v1
    auth:
      type: bearer
      token: ${OPENAI_API_KEY}  # Environment variable
    models:
      - gpt-4-turbo-preview
      - gpt-3.5-turbo
    settings:
      organization_id: null

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
      version: "2023-06-01"

# Workload definition
workload:
  scenarios:
    - name: "short_prompt_high_concurrency"
      prompt:
        template: "simple_question"
        variables:
          question: "What is the capital of France?"
      requests: 100
      concurrency: 20
      rate_limit: null  # No rate limit

    - name: "long_prompt_streaming"
      prompt:
        template: "code_generation"
        variables:
          task: "Write a Python function to calculate Fibonacci"
      requests: 50
      concurrency: 5
      rate_limit: 2.0  # 2 requests per second

  prompts:
    - name: simple_question
      template: "{{question}}"

    - name: code_generation
      template: |
        You are an expert programmer. {{task}}.
        Provide clean, well-commented code with examples.

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
      codec: messagepack  # messagepack or bincode

  database:
    enabled: false
    type: influxdb  # influxdb, prometheus, sqlite
    connection:
      url: http://localhost:8086
      database: llm_benchmarks
      retention_policy: autogen
```

---

## 9. Output Formats

### 9.1 Console Output (Table Format)

```
LLM Latency Lens - Benchmark Results
=====================================

Scenario: short_prompt_high_concurrency
Duration: 15.3s | Requests: 100 | Concurrency: 20

Provider: openai | Model: gpt-4-turbo-preview
┌──────────────────────┬──────────┬──────────┬──────────┬──────────┬──────────┐
│ Metric               │ Min      │ Mean     │ Median   │ p95      │ p99      │
├──────────────────────┼──────────┼──────────┼──────────┼──────────┼──────────┤
│ TTFT (ms)            │ 234.2    │ 456.8    │ 432.1    │ 678.9    │ 789.3    │
│ Total Duration (ms)  │ 1234.5   │ 2456.7   │ 2389.4   │ 3456.8   │ 3789.2   │
│ Tokens/sec           │ 12.3     │ 45.6     │ 44.2     │ 67.8     │ 72.1     │
│ Inter-token (ms)     │ 8.2      │ 22.4     │ 21.8     │ 34.6     │ 42.3     │
└──────────────────────┴──────────┴──────────┴──────────┴──────────┴──────────┘

Token Usage:
  Prompt: 15,234 tokens | Completion: 45,678 tokens | Total: 60,912 tokens

Cost Analysis:
  Total Cost: $1.23 | Cost/Request: $0.012 | Cost/1K tokens: $0.020

Success Rate: 98.0% (98/100) | Errors: 2 (2.0%)
Error Breakdown:
  - rate_limit_exceeded: 1
  - timeout: 1

Provider: anthropic | Model: claude-3-opus-20240229
... (similar output)

=====================================
Comparative Summary
┌──────────────────────────────────┬──────────┬──────────┬──────────┬──────────┐
│ Provider - Model                  │ TTFT p50 │ Throughput│ Cost/Req │ Success  │
├──────────────────────────────────┼──────────┼──────────┼──────────┼──────────┤
│ openai - gpt-4-turbo-preview     │ 432.1ms  │ 44.2 t/s │ $0.012   │ 98.0%    │
│ anthropic - claude-3-opus        │ 389.7ms  │ 48.9 t/s │ $0.015   │ 99.0%    │
└──────────────────────────────────┴──────────┴──────────┴──────────┴──────────┘
```

### 9.2 JSON Output Schema

```json
{
  "metadata": {
    "version": "1.0",
    "timestamp": "2024-11-07T18:30:00Z",
    "duration_secs": 15.3,
    "config_hash": "abc123...",
    "system_info": {
      "os": "Linux",
      "arch": "x86_64",
      "cpu_count": 16,
      "rust_version": "1.75.0"
    }
  },
  "scenarios": [
    {
      "name": "short_prompt_high_concurrency",
      "config": {
        "requests": 100,
        "concurrency": 20,
        "rate_limit": null
      },
      "results": [
        {
          "provider": "openai",
          "model": "gpt-4-turbo-preview",
          "metrics": {
            "total_requests": 100,
            "successful_requests": 98,
            "failed_requests": 2,
            "latency": {
              "ttft": {
                "min": 234.2,
                "max": 789.3,
                "mean": 456.8,
                "median": 432.1,
                "std_dev": 123.4,
                "p95": 678.9,
                "p99": 789.3,
                "p999": 800.1
              },
              "total_duration": {
                "min": 1234.5,
                "max": 3789.2,
                "mean": 2456.7,
                "median": 2389.4,
                "std_dev": 567.8,
                "p95": 3456.8,
                "p99": 3789.2,
                "p999": 3890.5
              },
              "throughput": {
                "min": 12.3,
                "max": 72.1,
                "mean": 45.6,
                "median": 44.2,
                "std_dev": 15.3,
                "p95": 67.8,
                "p99": 72.1,
                "p999": 75.2
              }
            },
            "token_stats": {
              "total_prompt_tokens": 15234,
              "total_completion_tokens": 45678,
              "total_tokens": 60912,
              "mean_tokens_per_second": 45.6
            },
            "cost_stats": {
              "total_cost": 1.23,
              "mean_cost_per_request": 0.012,
              "cost_per_1k_tokens": 0.020
            },
            "errors": {
              "rate_limit_exceeded": 1,
              "timeout": 1
            },
            "execution_duration_secs": 7.8,
            "actual_rps": 12.5
          }
        }
      ]
    }
  ],
  "raw_requests": [
    {
      "request_id": "550e8400-e29b-41d4-a716-446655440000",
      "timestamp": "2024-11-07T18:30:01.234Z",
      "provider": "openai",
      "model": "gpt-4-turbo-preview",
      "timing": {
        "ttft_ns": 432100000,
        "total_duration_ns": 2389400000,
        "token_latencies_ns": [21800000, 22100000, 21900000]
      },
      "tokens": {
        "prompt_tokens": 152,
        "completion_tokens": 456,
        "total_tokens": 608,
        "tokens_per_second": 44.2
      },
      "cost": {
        "total_cost": 0.012
      },
      "status": "Success"
    }
  ]
}
```

---

## 10. Error Handling Strategy

### 10.1 Error Type Hierarchy

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BenchmarkError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("HTTP error: {0}")]
    Http(#[from] HttpError),

    #[error("Metrics error: {0}")]
    Metrics(#[from] MetricsError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },

    #[error("Rate limit exceeded: retry after {retry_after_secs}s")]
    RateLimitExceeded { retry_after_secs: u64 },

    #[error("Invalid model: {model}")]
    InvalidModel { model: String },

    #[error("API error: {status_code} - {message}")]
    ApiError { status_code: u16, message: String },

    #[error("Request timeout after {timeout_secs}s")]
    Timeout { timeout_secs: u64 },
}

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("TLS error: {0}")]
    TlsError(String),

    #[error("DNS lookup failed: {0}")]
    DnsLookupFailed(String),
}
```

### 10.2 Retry Strategy

```rust
use backoff::{ExponentialBackoff, backoff::Backoff};

pub struct RetryPolicy {
    config: RetryConfig,
}

impl RetryPolicy {
    pub fn should_retry(&self, error: &ProviderError, attempt: u32) -> bool {
        if attempt >= self.config.max_attempts {
            return false;
        }

        match error {
            ProviderError::RateLimitExceeded { .. } => true,
            ProviderError::Timeout { .. } => true,
            ProviderError::ApiError { status_code, .. } => {
                // Retry on 5xx errors
                *status_code >= 500 && *status_code < 600
            }
            _ => false,
        }
    }

    pub async fn execute_with_retry<F, Fut, T>(
        &self,
        f: F,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut backoff = ExponentialBackoff {
            initial_interval: Duration::from_millis(self.config.initial_backoff_ms),
            max_interval: Duration::from_millis(self.config.max_backoff_ms),
            multiplier: self.config.multiplier,
            ..Default::default()
        };

        let mut attempt = 0;
        loop {
            attempt += 1;

            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if !self.should_retry(&e, attempt) {
                        return Err(e);
                    }

                    if let Some(duration) = backoff.next_backoff() {
                        tracing::warn!(
                            "Request failed (attempt {}), retrying in {:?}: {}",
                            attempt,
                            duration,
                            e
                        );
                        tokio::time::sleep(duration).await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }
}
```

---

## 11. Testing Strategy

### 11.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();

        let request = RequestMetrics {
            request_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            provider: "test".to_string(),
            model: "test-model".to_string(),
            timing: TimingMetrics {
                ttft_ns: 1_000_000_000, // 1 second
                total_duration_ns: 2_000_000_000, // 2 seconds
                ..Default::default()
            },
            tokens: TokenMetrics {
                completion_tokens: 100,
                tokens_per_second: 50.0,
                ..Default::default()
            },
            ..Default::default()
        };

        collector.record(request).await;

        let stats = collector.aggregate().await;
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.latency.ttft.mean, 1000.0); // ms
    }

    #[test]
    fn test_config_validation() {
        let config = BenchmarkConfig {
            providers: vec![],
            workload: WorkloadConfig::default(),
            execution: ExecutionConfig::default(),
            output: OutputConfig::default(),
        };

        assert!(config.validate().is_err());
    }
}
```

### 11.2 Integration Tests

```rust
#[tokio::test]
#[serial] // Serialize to avoid rate limits
async fn test_openai_integration() {
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY not set");

    let provider = OpenAIProvider::new(api_key);

    let request = CompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        prompt: "Hello, world!".to_string(),
        max_tokens: 50,
        temperature: 0.7,
        top_p: None,
        stream: false,
        timeout: Duration::from_secs(30),
    };

    let response = provider.complete(request).await
        .expect("Request failed");

    assert!(!response.content.is_empty());
    assert!(response.usage.total_tokens > 0);
}
```

### 11.3 Benchmark Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_metrics_collection(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("collect_1000_metrics", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let collector = MetricsCollector::new();

                for _ in 0..1000 {
                    let metrics = generate_test_metrics();
                    collector.record(black_box(metrics)).await;
                }
            })
        })
    });
}

criterion_group!(benches, benchmark_metrics_collection);
criterion_main!(benches);
```

---

## 12. Deployment & Operations

### 12.1 Observability

**Tracing Integration:**

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(client), fields(provider = %provider.name(), model = %request.model))]
async fn execute_request(
    client: &impl LLMProvider,
    request: CompletionRequest,
) -> Result<CompletionResponse> {
    info!("Starting request");

    let start = Instant::now();

    match client.complete(request).await {
        Ok(response) => {
            let duration = start.elapsed();
            info!(
                duration_ms = duration.as_millis(),
                tokens = response.usage.total_tokens,
                "Request completed successfully"
            );
            Ok(response)
        }
        Err(e) => {
            error!(error = %e, "Request failed");
            Err(e)
        }
    }
}
```

### 12.2 Metrics Export

**Prometheus Integration:**

```rust
use prometheus::{Registry, Counter, Histogram};

pub struct PrometheusExporter {
    registry: Registry,
    request_total: Counter,
    request_duration: Histogram,
    ttft: Histogram,
}

impl PrometheusExporter {
    pub fn new() -> Self {
        let registry = Registry::new();

        let request_total = Counter::new(
            "llm_requests_total",
            "Total number of LLM requests"
        ).unwrap();

        let request_duration = Histogram::with_opts(
            HistogramOpts::new(
                "llm_request_duration_seconds",
                "Request duration distribution"
            )
        ).unwrap();

        let ttft = Histogram::with_opts(
            HistogramOpts::new(
                "llm_ttft_seconds",
                "Time to first token distribution"
            )
        ).unwrap();

        registry.register(Box::new(request_total.clone())).unwrap();
        registry.register(Box::new(request_duration.clone())).unwrap();
        registry.register(Box::new(ttft.clone())).unwrap();

        Self {
            registry,
            request_total,
            request_duration,
            ttft,
        }
    }

    pub fn record_request(&self, metrics: &RequestMetrics) {
        self.request_total.inc();
        self.request_duration.observe(
            metrics.timing.total_duration_ns as f64 / 1_000_000_000.0
        );
        self.ttft.observe(
            metrics.timing.ttft_ns as f64 / 1_000_000_000.0
        );
    }
}
```

---

## 13. Future Enhancements

### 13.1 Phase 2 Features

1. **Real-time Dashboard**
   - WebSocket-based live metrics
   - Web UI with charts and graphs
   - Alert thresholds

2. **Advanced Workloads**
   - Variable prompt lengths
   - Realistic user behavior simulation
   - Load testing scenarios

3. **Cost Optimization**
   - Automatic model selection based on requirements
   - Cost vs. performance tradeoffs
   - Budget alerting

4. **Multi-region Testing**
   - Geographic latency comparison
   - Edge location benchmarking

5. **Historical Analysis**
   - Trend detection
   - Regression identification
   - Performance baselines

### 13.2 Technical Debt Prevention

1. **Code Quality**
   - Maintain >80% test coverage
   - Regular dependency updates
   - Clippy lints enforced
   - Rustfmt for consistent style

2. **Performance**
   - Regular benchmarking
   - Memory leak detection
   - Profile CPU hotspots

3. **Documentation**
   - API documentation with examples
   - Architecture decision records
   - User guides and tutorials

---

## 14. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
- [ ] Project setup with Cargo workspace
- [ ] Core data models implementation
- [ ] Configuration loading system
- [ ] HTTP client infrastructure
- [ ] Basic timing and metrics collection

### Phase 2: Provider Integration (Weeks 3-4)
- [ ] Provider trait and abstraction layer
- [ ] OpenAI adapter implementation
- [ ] Anthropic adapter implementation
- [ ] Streaming response handling
- [ ] Error handling and retry logic

### Phase 3: Execution Engine (Weeks 5-6)
- [ ] Concurrency controller
- [ ] Workload scheduler
- [ ] Rate limiting
- [ ] Request executor with timing
- [ ] Metrics aggregation

### Phase 4: Output & Analysis (Week 7)
- [ ] Statistical computation (HDR histogram)
- [ ] Console output formatting
- [ ] JSON/CSV export
- [ ] Cost calculation

### Phase 5: Testing & Polish (Week 8)
- [ ] Unit test suite
- [ ] Integration tests
- [ ] Performance benchmarks
- [ ] Documentation
- [ ] CLI refinement

### Phase 6: Advanced Features (Weeks 9-10)
- [ ] Additional provider adapters
- [ ] Time-series database integration
- [ ] Advanced workload patterns
- [ ] Prometheus metrics export

---

## 15. Success Metrics

### Technical KPIs
- **Timing Precision**: Sub-millisecond accuracy for TTFT
- **Throughput**: Handle 1000+ concurrent requests
- **Memory Efficiency**: <100MB baseline memory usage
- **CPU Efficiency**: <5% overhead per request
- **Accuracy**: Percentile calculations within 0.1% of actual

### Quality Metrics
- **Test Coverage**: >80%
- **Documentation Coverage**: 100% of public APIs
- **Error Rate**: <0.1% failures on valid requests
- **Build Time**: <60 seconds for full build

---

## Conclusion

This architecture provides a robust, scalable foundation for LLM-Latency-Lens. The design emphasizes:

1. **Performance**: Async I/O, efficient concurrency, minimal overhead
2. **Accuracy**: High-precision timing, HDR histograms, comprehensive metrics
3. **Extensibility**: Plugin-based provider system, flexible configuration
4. **Reliability**: Comprehensive error handling, retry logic, observability
5. **Usability**: Clear outputs, multiple export formats, intuitive CLI

The chosen Rust ecosystem provides excellent performance characteristics while maintaining type safety and ergonomics. The modular architecture allows for incremental development and easy testing.

**Next Steps:**
1. Review and approve architecture
2. Set up initial Cargo project structure
3. Begin Phase 1 implementation
4. Establish CI/CD pipeline
5. Create initial provider implementations

---

**Document Version**: 1.0
**Date**: 2025-11-07
**Author**: Architecture Design Agent
**Status**: Approved for Implementation
