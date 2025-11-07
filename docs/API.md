# LLM-Latency-Lens API Documentation

**Complete API reference for using LLM-Latency-Lens as a Rust library**

Version: 0.1.0
Last Updated: 2025-11-07

---

## Table of Contents

1. [Overview](#overview)
2. [Getting Started](#getting-started)
3. [Core API](#core-api)
4. [Provider API](#provider-api)
5. [Metrics API](#metrics-api)
6. [Export API](#export-api)
7. [Integration Patterns](#integration-patterns)
8. [Best Practices](#best-practices)
9. [Examples](#examples)

---

## Overview

LLM-Latency-Lens can be used as a Rust library in your applications for:

- **Embedded Profiling**: Profile LLM calls within your application
- **Custom Tooling**: Build custom benchmarking tools
- **CI/CD Integration**: Automated performance testing
- **Production Monitoring**: Real-time latency tracking

### Crate Organization

```
llm-latency-lens
├── llm-latency-lens-core      # Core types and timing engine
├── llm-latency-lens-providers # Provider adapters (OpenAI, Anthropic, etc.)
├── llm-latency-lens-metrics   # Metrics collection and aggregation
└── llm-latency-lens-exporters # Export to various formats
```

---

## Getting Started

### Add Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
# Core crate
llm-latency-lens-core = "0.1"

# Provider support
llm-latency-lens-providers = "0.1"

# Metrics collection
llm-latency-lens-metrics = "0.1"

# Export functionality (optional)
llm-latency-lens-exporters = "0.1"

# Async runtime
tokio = { version = "1.41", features = ["full"] }
```

### Basic Example

```rust
use llm_latency_lens_core::TimingEngine;
use llm_latency_lens_providers::{
    OpenAIProvider, Provider, StreamingRequest, MessageRole
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create timing engine
    let timing = TimingEngine::new();

    // Create provider
    let provider = OpenAIProvider::new("sk-...");

    // Build request
    let request = StreamingRequest::builder()
        .model("gpt-4o")
        .message(MessageRole::User, "Explain quantum computing")
        .max_tokens(500)
        .temperature(0.7)
        .build();

    // Execute with streaming
    let response = provider.stream(request, &timing).await?;

    // Access metrics
    println!("TTFT: {:?}", response.ttft);
    println!("Total time: {:?}", response.total_time);
    println!("Tokens: {}", response.metadata.completion_tokens);
    println!("Cost: ${:.6}", response.metadata.cost_usd);

    Ok(())
}
```

---

## Core API

### TimingEngine

High-precision timing engine using hardware counters.

```rust
use llm_latency_lens_core::{TimingEngine, Timestamp};

// Create timing engine
let timing = TimingEngine::new();

// Record timestamp
let start = timing.now();

// ... perform operation ...

// Calculate elapsed time
let elapsed_ns = timing.elapsed_nanos(start);
let elapsed_ms = timing.elapsed_millis(start);
```

**Methods:**

```rust
impl TimingEngine {
    /// Create a new timing engine
    pub fn new() -> Self;

    /// Get current timestamp
    pub fn now(&self) -> Timestamp;

    /// Calculate elapsed time in nanoseconds
    pub fn elapsed_nanos(&self, start: Timestamp) -> u64;

    /// Calculate elapsed time in milliseconds
    pub fn elapsed_millis(&self, start: Timestamp) -> f64;

    /// Calculate elapsed time in seconds
    pub fn elapsed_secs(&self, start: Timestamp) -> f64;
}
```

### Error Types

Comprehensive error handling across all crates.

```rust
use llm_latency_lens_core::{Error, Result};

// Core error type
pub enum Error {
    /// Configuration error
    Config(String),

    /// I/O error
    Io(std::io::Error),

    /// Serialization error
    Serialization(String),

    /// Other errors
    Other(String),
}

// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
```

---

## Provider API

### Provider Trait

Common interface for all LLM providers.

```rust
use llm_latency_lens_providers::{
    Provider, StreamingRequest, StreamingResponse,
    CompletionResult, ProviderError
};
use llm_latency_lens_core::TimingEngine;
use async_trait::async_trait;

#[async_trait]
pub trait Provider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// List available models
    async fn list_models(&self) -> Result<Vec<String>, ProviderError>;

    /// Execute streaming request
    async fn stream(
        &self,
        request: StreamingRequest,
        timing: &TimingEngine,
    ) -> Result<StreamingResponse, ProviderError>;

    /// Calculate cost for token usage
    fn calculate_cost(
        &self,
        model: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
    ) -> Option<f64>;
}
```

### StreamingRequest

Request builder for LLM calls.

```rust
use llm_latency_lens_providers::{StreamingRequest, MessageRole};

// Using builder pattern
let request = StreamingRequest::builder()
    .model("gpt-4o")
    .message(MessageRole::System, "You are a helpful assistant")
    .message(MessageRole::User, "What is Rust?")
    .max_tokens(500)
    .temperature(0.7)
    .top_p(0.9)
    .stream(true)
    .timeout(std::time::Duration::from_secs(30))
    .build();

// Manual construction
let request = StreamingRequest {
    model: "gpt-4o".to_string(),
    messages: vec![
        Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
        }
    ],
    max_tokens: Some(500),
    temperature: Some(0.7),
    top_p: Some(0.9),
    stream: true,
    timeout: std::time::Duration::from_secs(30),
};
```

**Builder Methods:**

```rust
impl StreamingRequestBuilder {
    pub fn model(self, model: impl Into<String>) -> Self;
    pub fn message(self, role: MessageRole, content: impl Into<String>) -> Self;
    pub fn max_tokens(self, max_tokens: u32) -> Self;
    pub fn temperature(self, temperature: f32) -> Self;
    pub fn top_p(self, top_p: f32) -> Self;
    pub fn stream(self, stream: bool) -> Self;
    pub fn timeout(self, timeout: Duration) -> Self;
    pub fn build(self) -> StreamingRequest;
}
```

### StreamingResponse

Response with timing and token metrics.

```rust
use llm_latency_lens_providers::{StreamingResponse, TokenEvent};
use futures::StreamExt;

let response: StreamingResponse = provider.stream(request, &timing).await?;

// Access timing metrics
println!("TTFT: {:?}", response.ttft);
println!("Total time: {:?}", response.total_time);

// Access metadata
println!("Model: {}", response.metadata.model);
println!("Prompt tokens: {}", response.metadata.prompt_tokens);
println!("Completion tokens: {}", response.metadata.completion_tokens);
println!("Cost: ${:.6}", response.metadata.cost_usd);

// Process token stream
let mut token_stream = response.token_stream;
while let Some(event) = token_stream.next().await {
    let event = event?;
    println!("Token {}: {:?} (latency: {:?})",
        event.sequence,
        event.content,
        event.inter_token_latency
    );
}
```

**Response Structure:**

```rust
pub struct StreamingResponse {
    /// Time to first token
    pub ttft: Duration,

    /// Total request time
    pub total_time: Duration,

    /// Response metadata
    pub metadata: ResponseMetadata,

    /// Token stream
    pub token_stream: Pin<Box<dyn Stream<Item = Result<TokenEvent>> + Send>>,
}

pub struct ResponseMetadata {
    /// Provider name
    pub provider: String,

    /// Model name
    pub model: String,

    /// Request ID
    pub request_id: String,

    /// Prompt tokens
    pub prompt_tokens: u32,

    /// Completion tokens
    pub completion_tokens: u32,

    /// Total tokens
    pub total_tokens: u32,

    /// Cost in USD
    pub cost_usd: f64,
}

pub struct TokenEvent {
    /// Token sequence number
    pub sequence: u32,

    /// Token content
    pub content: Option<String>,

    /// Timestamp
    pub timestamp: Timestamp,

    /// Inter-token latency
    pub inter_token_latency: Option<Duration>,
}
```

### OpenAI Provider

```rust
use llm_latency_lens_providers::OpenAIProvider;

// Simple constructor
let provider = OpenAIProvider::new("sk-...");

// With builder
let provider = OpenAIProvider::builder()
    .api_key("sk-...")
    .organization("org-...")  // Optional
    .max_retries(3)
    .timeout(Duration::from_secs(30))
    .build();

// List models
let models = provider.list_models().await?;
for model in models {
    println!("Model: {}", model);
}

// Calculate cost
if let Some(cost) = provider.calculate_cost("gpt-4o", 1000, 2000) {
    println!("Estimated cost: ${:.6}", cost);
}
```

### Anthropic Provider

```rust
use llm_latency_lens_providers::AnthropicProvider;

// Simple constructor
let provider = AnthropicProvider::new("sk-ant-...");

// With builder
let provider = AnthropicProvider::builder()
    .api_key("sk-ant-...")
    .api_version("2023-06-01")
    .max_retries(3)
    .timeout(Duration::from_secs(60))
    .build();

// Stream with extended thinking
let request = StreamingRequest::builder()
    .model("claude-3-opus-20240229")
    .message(MessageRole::User, "Solve this complex problem...")
    .max_tokens(4000)
    .build();

let response = provider.stream(request, &timing).await?;
```

---

## Metrics API

### MetricsCollector

Collect and aggregate timing metrics.

```rust
use llm_latency_lens_metrics::{MetricsCollector, RequestMetrics};

// Create collector
let collector = MetricsCollector::new();

// Record individual request
let metrics = RequestMetrics {
    request_id: uuid::Uuid::new_v4(),
    timestamp: chrono::Utc::now(),
    provider: "openai".to_string(),
    model: "gpt-4o".to_string(),
    ttft: Duration::from_millis(432),
    total_time: Duration::from_millis(2389),
    prompt_tokens: 152,
    completion_tokens: 456,
    cost_usd: 0.012,
    success: true,
};

collector.record(metrics).await;

// Get aggregated statistics
let stats = collector.aggregate().await;
println!("Mean TTFT: {:.2}ms", stats.ttft.mean);
println!("P95 TTFT: {:.2}ms", stats.ttft.p95);
println!("Total requests: {}", stats.total_requests);
println!("Success rate: {:.1}%", stats.success_rate * 100.0);
```

**Metrics Types:**

```rust
pub struct RequestMetrics {
    pub request_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub provider: String,
    pub model: String,
    pub ttft: Duration,
    pub total_time: Duration,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub cost_usd: f64,
    pub success: bool,
    pub error: Option<String>,
}

pub struct AggregatedMetrics {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub success_rate: f64,
    pub ttft: DistributionStats,
    pub total_time: DistributionStats,
    pub throughput: DistributionStats,
    pub total_cost_usd: f64,
}

pub struct DistributionStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub p999: f64,
}
```

---

## Export API

### JSON Export

```rust
use llm_latency_lens_exporters::JsonExporter;

let exporter = JsonExporter::new()
    .pretty(true)  // Pretty-print JSON
    .indent(2);    // 2-space indentation

// Export metrics
exporter.export(&metrics, "results.json").await?;

// Export to writer
let mut buffer = Vec::new();
exporter.export_to_writer(&metrics, &mut buffer).await?;
```

### CSV Export

```rust
use llm_latency_lens_exporters::CsvExporter;

let exporter = CsvExporter::new()
    .delimiter(',')
    .with_headers(true);

exporter.export(&metrics, "results.csv").await?;
```

### Console Export

```rust
use llm_latency_lens_exporters::ConsoleExporter;

let exporter = ConsoleExporter::new()
    .with_colors(true)
    .table_format(TableFormat::Grid);

// Print to stdout
exporter.print(&metrics).await?;

// Get formatted string
let output = exporter.format(&metrics).await?;
println!("{}", output);
```

### Prometheus Export

```rust
use llm_latency_lens_exporters::PrometheusExporter;

let exporter = PrometheusExporter::new()
    .port(9090)
    .path("/metrics")
    .label("environment", "production")
    .label("service", "llm-api");

// Start metrics server
exporter.start().await?;

// Record metrics (automatically exported)
exporter.record(&metrics).await?;
```

---

## Integration Patterns

### Pattern 1: Simple Profiling

Profile individual LLM calls in your application.

```rust
use llm_latency_lens_core::TimingEngine;
use llm_latency_lens_providers::{OpenAIProvider, Provider, StreamingRequest, MessageRole};

pub struct LLMClient {
    provider: OpenAIProvider,
    timing: TimingEngine,
}

impl LLMClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            provider: OpenAIProvider::new(api_key),
            timing: TimingEngine::new(),
        }
    }

    pub async fn chat(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let request = StreamingRequest::builder()
            .model("gpt-4o")
            .message(MessageRole::User, prompt)
            .build();

        let response = self.provider.stream(request, &self.timing).await?;

        // Log metrics
        tracing::info!(
            ttft = ?response.ttft,
            total_time = ?response.total_time,
            tokens = response.metadata.completion_tokens,
            cost = response.metadata.cost_usd,
            "LLM request completed"
        );

        // Collect tokens
        let mut result = String::new();
        let mut stream = response.token_stream;
        while let Some(event) = stream.next().await {
            if let Some(content) = event?.content {
                result.push_str(&content);
            }
        }

        Ok(result)
    }
}
```

### Pattern 2: Comparative Benchmarking

Compare multiple providers and models.

```rust
use llm_latency_lens_providers::{Provider, OpenAIProvider, AnthropicProvider};
use llm_latency_lens_metrics::{MetricsCollector, RequestMetrics};

pub struct Benchmarker {
    providers: Vec<Box<dyn Provider>>,
    collector: MetricsCollector,
    timing: TimingEngine,
}

impl Benchmarker {
    pub fn new() -> Self {
        let mut providers: Vec<Box<dyn Provider>> = vec![
            Box::new(OpenAIProvider::new("sk-...")),
            Box::new(AnthropicProvider::new("sk-ant-...")),
        ];

        Self {
            providers,
            collector: MetricsCollector::new(),
            timing: TimingEngine::new(),
        }
    }

    pub async fn benchmark(&self, prompt: &str, iterations: usize) -> AggregatedMetrics {
        for _ in 0..iterations {
            for provider in &self.providers {
                let request = StreamingRequest::builder()
                    .model(self.get_default_model(provider.name()))
                    .message(MessageRole::User, prompt)
                    .build();

                match provider.stream(request, &self.timing).await {
                    Ok(response) => {
                        let metrics = RequestMetrics::from_response(response);
                        self.collector.record(metrics).await;
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }

        self.collector.aggregate().await
    }

    fn get_default_model(&self, provider: &str) -> &str {
        match provider {
            "openai" => "gpt-4o",
            "anthropic" => "claude-3-opus-20240229",
            _ => "unknown",
        }
    }
}
```

### Pattern 3: CI/CD Integration

Automated performance regression detection.

```rust
use llm_latency_lens_metrics::{MetricsCollector, AggregatedMetrics};

pub struct PerformanceTest {
    collector: MetricsCollector,
    baseline: Option<AggregatedMetrics>,
}

impl PerformanceTest {
    pub fn new() -> Self {
        Self {
            collector: MetricsCollector::new(),
            baseline: None,
        }
    }

    pub fn load_baseline(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        self.baseline = Some(serde_json::from_reader(file)?);
        Ok(())
    }

    pub async fn run_test(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Run benchmark
        let current = self.run_benchmark().await?;

        // Compare with baseline
        if let Some(baseline) = &self.baseline {
            self.check_regression(baseline, &current)?;
        }

        Ok(())
    }

    fn check_regression(
        &self,
        baseline: &AggregatedMetrics,
        current: &AggregatedMetrics,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ttft_regression = (current.ttft.p95 - baseline.ttft.p95) / baseline.ttft.p95;

        if ttft_regression > 0.1 {  // 10% regression threshold
            return Err(format!(
                "Performance regression detected! TTFT p95 increased by {:.1}%",
                ttft_regression * 100.0
            ).into());
        }

        println!("✓ No performance regression detected");
        Ok(())
    }

    async fn run_benchmark(&self) -> Result<AggregatedMetrics, Box<dyn std::error::Error>> {
        // Implementation
        todo!()
    }
}
```

### Pattern 4: Production Monitoring

Real-time latency monitoring with alerting.

```rust
use llm_latency_lens_metrics::MetricsCollector;
use llm_latency_lens_exporters::PrometheusExporter;

pub struct LatencyMonitor {
    collector: MetricsCollector,
    exporter: PrometheusExporter,
    alert_threshold_ms: f64,
}

impl LatencyMonitor {
    pub fn new(alert_threshold_ms: f64) -> Self {
        Self {
            collector: MetricsCollector::new(),
            exporter: PrometheusExporter::new().port(9090),
            alert_threshold_ms,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.exporter.start().await?;
        Ok(())
    }

    pub async fn record_request(&self, metrics: RequestMetrics) {
        // Check if alert threshold exceeded
        let ttft_ms = metrics.ttft.as_millis() as f64;
        if ttft_ms > self.alert_threshold_ms {
            self.send_alert(ttft_ms).await;
        }

        // Record metrics
        self.collector.record(metrics.clone()).await;
        self.exporter.record(&metrics).await.ok();
    }

    async fn send_alert(&self, ttft_ms: f64) {
        // Send alert to monitoring system
        eprintln!(
            "🚨 ALERT: High latency detected! TTFT: {:.2}ms (threshold: {:.2}ms)",
            ttft_ms, self.alert_threshold_ms
        );
    }
}
```

---

## Best Practices

### 1. Error Handling

Always handle errors gracefully:

```rust
use llm_latency_lens_providers::{Provider, ProviderError};

async fn safe_llm_call(
    provider: &impl Provider,
    request: StreamingRequest,
    timing: &TimingEngine,
) -> Result<StreamingResponse, ProviderError> {
    let max_retries = 3;
    let mut attempt = 0;

    loop {
        attempt += 1;

        match provider.stream(request.clone(), timing).await {
            Ok(response) => return Ok(response),
            Err(e) if e.is_retryable() && attempt < max_retries => {
                eprintln!("Retryable error (attempt {}): {}", attempt, e);
                if let Some(delay) = e.retry_delay() {
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                }
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 2. Resource Management

Use connection pooling and proper cleanup:

```rust
use llm_latency_lens_providers::OpenAIProvider;

// Reuse provider instances
pub struct LLMService {
    provider: OpenAIProvider,  // Reuse connection pool
}

impl LLMService {
    pub fn new(api_key: String) -> Self {
        Self {
            provider: OpenAIProvider::builder()
                .api_key(api_key)
                .max_connections(100)
                .build(),
        }
    }
}
```

### 3. Metrics Collection

Collect metrics asynchronously:

```rust
use tokio::sync::mpsc;

pub struct AsyncMetricsCollector {
    sender: mpsc::UnboundedSender<RequestMetrics>,
}

impl AsyncMetricsCollector {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<RequestMetrics>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { sender: tx }, rx)
    }

    pub fn record(&self, metrics: RequestMetrics) {
        // Non-blocking send
        let _ = self.sender.send(metrics);
    }
}

// Process metrics in background
tokio::spawn(async move {
    while let Some(metrics) = rx.recv().await {
        // Process metrics
    }
});
```

### 4. Testing

Write comprehensive tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_llm_call() {
        let mut server = Server::new();

        let _mock = server.mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"choices": [{"message": {"content": "test"}}]}"#)
            .create();

        let provider = OpenAIProvider::builder()
            .api_key("test")
            .base_url(server.url())
            .build();

        // Test implementation
    }
}
```

---

## Examples

Complete examples are available in the `examples/` directory:

- `examples/simple_profile.rs` - Basic profiling
- `examples/compare_providers.rs` - Multi-provider comparison
- `examples/streaming_tokens.rs` - Token-by-token processing
- `examples/metrics_collection.rs` - Collecting and analyzing metrics
- `examples/ci_integration.rs` - CI/CD integration
- `examples/production_monitoring.rs` - Production monitoring setup

Run examples:

```bash
cargo run --example simple_profile
cargo run --example compare_providers
```

---

## API Reference

Full API documentation is available at:
- **Docs.rs**: https://docs.rs/llm-latency-lens
- **GitHub Pages**: https://llm-devops.github.io/llm-latency-lens

Generate locally:

```bash
cargo doc --open --no-deps
```

---

## Support

- **Issues**: [GitHub Issues](https://github.com/llm-devops/llm-latency-lens/issues)
- **Discussions**: [GitHub Discussions](https://github.com/llm-devops/llm-latency-lens/discussions)
- **Discord**: [Community Chat](https://discord.gg/llm-latency-lens)

---

**Next**: Check out [Integration Patterns](ECOSYSTEM_INTEGRATION.md) for advanced usage.
