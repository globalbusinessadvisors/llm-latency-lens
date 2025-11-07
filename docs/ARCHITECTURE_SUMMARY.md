# LLM-Latency-Lens: Architecture Summary

## Document Overview

This is the executive summary of the complete architecture design for LLM-Latency-Lens, a high-performance Rust-based latency profiler for Large Language Model APIs.

## Core Documentation Files

1. **ARCHITECTURE.md** - Complete system architecture (27,000+ words)
   - High-level system design
   - Component architecture
   - Data models and schemas
   - Rust crate recommendations
   - Performance considerations
   - Testing strategy
   - Implementation roadmap

2. **CRATE_STRUCTURE.md** - Project structure and organization
   - Detailed file structure
   - Module dependency graph
   - Implementation priority order
   - Build configuration
   - Development workflow

3. **DATA_FLOW.md** - Data flow and timing pipeline
   - Request execution pipeline with nanosecond-level detail
   - Concurrent execution architecture
   - Metrics collection flow
   - Provider-specific streaming protocols
   - Storage and export pipeline

4. **IMPLEMENTATION_GUIDE.md** - Code patterns and examples
   - Production-ready code samples
   - Core implementation patterns
   - Provider implementations
   - Testing examples

## Quick Start: Key Architectural Decisions

### 1. Technology Stack

| Component | Choice | Justification |
|-----------|--------|---------------|
| **Async Runtime** | tokio 1.37 | Industry standard, mature, excellent I/O performance |
| **HTTP Client** | reqwest 0.12 | High-level API, streaming support, built on hyper |
| **Timing** | quanta 0.12 | Sub-millisecond precision, low overhead (<10ns) |
| **Statistics** | hdrhistogram 7.5 | Accurate percentiles, memory-efficient |
| **Serialization** | serde + bincode/msgpack | Fast, compact, ecosystem support |
| **CLI** | clap 4.5 | Ergonomic derive macros, powerful features |
| **Concurrency** | tokio::sync + dashmap | Lock-free collections, async primitives |
| **Rate Limiting** | governor 0.6 | Token bucket algorithm, tokio integration |

### 2. Core Metrics (Priority Order)

1. **Time to First Token (TTFT)** - Primary user-perceived latency metric
2. **Total Request Duration** - Overall completion time
3. **Tokens per Second** - Throughput measurement
4. **Inter-token Latency Distribution** - Streaming quality indicator
5. **Cost per Request** - Economic efficiency

### 3. Performance Targets

| Metric | Target | Justification |
|--------|--------|---------------|
| Timing Overhead | <5 μs/request | Sub-1% impact on measurements |
| Memory Usage | <50 MB @ 1000 concurrent | Efficient resource utilization |
| Throughput | >500 req/sec/core | High benchmark velocity |
| Timing Precision | Nanosecond resolution | Accurate TTFT measurement |
| Histogram Memory | ~32 KB each | HDR histogram efficiency |

### 4. Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│                    CLI Layer                             │
│              (clap, console, indicatif)                  │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────┐
│              Configuration Layer                         │
│         (YAML/JSON parsing, validation)                  │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────┐
│            Orchestration Engine                          │
│  (Workload scheduler, concurrency control, dispatcher)   │
└───────┬──────────────────────────────────┬──────────────┘
        │                                  │
┌───────┴──────────┐            ┌─────────┴────────────┐
│ Request Executor │◄──────────►│ Metrics Collector    │
│ (HTTP, retry,    │            │ (timing, stats,      │
│  rate limiting)  │            │  aggregation)        │
└───────┬──────────┘            └─────────┬────────────┘
        │                                  │
┌───────┴──────────┐            ┌─────────┴────────────┐
│ Provider Layer   │            │ Storage Layer        │
│ (OpenAI, Claude, │            │ (JSON, CSV, binary,  │
│  Gemini, etc.)   │            │  time-series DB)     │
└──────────────────┘            └──────────────────────┘
```

## Critical Implementation Patterns

### 1. Request Timing Pipeline

```
T0: Request Start
  ├─> DNS Lookup
  ├─> TCP Connection
  ├─> TLS Handshake
  ├─> Request Send
  ├─> TTFB (Time to First Byte)
  ├─> TTFT (Time to First Token) ⭐ CRITICAL
  ├─> Token Streaming (record each token timestamp)
  └─> Total Duration ⭐ CRITICAL
```

### 2. Concurrency Model

```rust
// Semaphore-based concurrency control
let semaphore = Arc::new(Semaphore::new(max_concurrency));

// Per-worker execution
loop {
    let _permit = semaphore.acquire().await; // Blocks if at limit
    let result = execute_request().await;
    metrics_channel.send(result).await;
    // Permit released automatically on drop
}
```

### 3. Metrics Collection

```rust
// Lock-free collection with DashMap
let accumulators: DashMap<(Provider, Model), MetricsAccumulator>;

// Per-request recording
accumulator.ttft_histogram.record(metrics.ttft_ns);
accumulator.duration_histogram.record(metrics.total_ns);

// Final aggregation
let percentiles = histogram.value_at_quantiles(&[0.50, 0.95, 0.99]);
```

## Provider Abstraction

### Unified Provider Trait

```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    fn name(&self) -> &str;

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse>;

    async fn complete_streaming(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>>>>>;

    fn get_pricing(&self, model: &str) -> Option<PricingInfo>;
}
```

### Supported Providers (Planned)

1. **OpenAI** (GPT-4, GPT-3.5)
2. **Anthropic** (Claude 3 family)
3. **Google** (Gemini via Vertex AI)
4. **Azure OpenAI** (Enterprise deployments)
5. **Cohere** (Command models)
6. **Generic HTTP** (Custom/self-hosted)

## Data Models

### Core Structures

```rust
// Input Configuration
BenchmarkConfig {
    providers: Vec<ProviderConfig>,
    workload: WorkloadConfig,
    execution: ExecutionConfig,
    output: OutputConfig,
}

// Per-Request Metrics
RequestMetrics {
    timing: TimingMetrics {
        ttft_ns: u64,           // ⭐ Most critical
        total_duration_ns: u64, // ⭐ Second most critical
        token_latencies_ns: Vec<u64>,
        ...
    },
    tokens: TokenMetrics,
    cost: CostMetrics,
    status: RequestStatus,
}

// Aggregated Results
AggregatedMetrics {
    latency: LatencyStats {
        ttft: DistributionStats {
            min, max, mean, median, p95, p99, p999, ...
        },
        total_duration: DistributionStats,
        throughput: DistributionStats,
    },
    token_stats: AggregatedTokenStats,
    cost_stats: AggregatedCostStats,
}
```

## Configuration Example

```yaml
providers:
  - name: openai
    endpoint: https://api.openai.com/v1
    auth:
      type: bearer
      token: ${OPENAI_API_KEY}
    models:
      - gpt-4-turbo-preview
      - gpt-3.5-turbo

workload:
  scenarios:
    - name: high_concurrency
      requests: 100
      concurrency: 20
      prompt:
        template: "What is {{question}}?"
        variables:
          question: "the capital of France"

  request_params:
    max_tokens: 500
    temperature: 0.7
    stream: true
    timeout: 30

execution:
  max_concurrency: 50
  retry:
    max_attempts: 3
    initial_backoff_ms: 1000

output:
  export:
    - format: json
      path: ./results/bench_{timestamp}.json
    - format: csv
      path: ./results/bench_{timestamp}.csv
```

## Output Formats

### 1. Console Output (Rich Tables)

```
┌──────────────────────┬──────────┬──────────┬──────────┬──────────┐
│ Metric               │ Min      │ Mean     │ p95      │ p99      │
├──────────────────────┼──────────┼──────────┼──────────┼──────────┤
│ TTFT (ms)            │ 234.2    │ 456.8    │ 678.9    │ 789.3    │
│ Total Duration (ms)  │ 1234.5   │ 2456.7   │ 3456.8   │ 3789.2   │
│ Tokens/sec           │ 12.3     │ 45.6     │ 67.8     │ 72.1     │
└──────────────────────┴──────────┴──────────┴──────────┴──────────┘
```

### 2. JSON Export

```json
{
  "metadata": {
    "timestamp": "2024-11-07T18:30:00Z",
    "duration_secs": 15.3
  },
  "scenarios": [
    {
      "name": "high_concurrency",
      "results": [
        {
          "provider": "openai",
          "model": "gpt-4-turbo",
          "metrics": {
            "latency": {
              "ttft": { "mean": 456.8, "p95": 678.9, "p99": 789.3 },
              "total_duration": { "mean": 2456.7, "p95": 3456.8 }
            }
          }
        }
      ]
    }
  ]
}
```

### 3. Binary Format (Compact)

- **bincode**: ~80 KB per 1000 requests
- **messagepack**: ~70 KB per 1000 requests
- **compressed**: ~20 KB per 1000 requests

### 4. Time-series Database (Real-time)

- InfluxDB integration for live dashboards
- Prometheus metrics export
- Grafana visualization support

## Error Handling Strategy

### Retry Policy

```
Error occurs
  ├─> Rate Limit (429) → Exponential backoff + retry
  ├─> Timeout → Retry with backoff
  ├─> 5xx Server Error → Retry with backoff
  ├─> 4xx Client Error → No retry (fail fast)
  └─> Auth Error → No retry (fail fast)
```

### Rate Limiting

```
Token Bucket Algorithm:
- Capacity: requests_per_second
- Refill rate: 1 token per 1/rps seconds
- Per-provider enforcement
- Automatic backoff on 429 responses
```

## Testing Strategy

### 1. Unit Tests
- Timing precision validation
- Statistical correctness (histograms)
- Configuration validation
- Error handling logic

### 2. Integration Tests
- End-to-end benchmark execution
- Provider API mocking
- Concurrent request handling
- Metrics aggregation accuracy

### 3. Benchmark Tests
- Metrics collection overhead
- Histogram performance
- Concurrent throughput
- Memory usage profiling

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
- Project structure setup
- Core data models
- Configuration loading
- HTTP client infrastructure
- Precision timing

### Phase 2: Provider Integration (Weeks 3-4)
- Provider trait system
- OpenAI adapter
- Anthropic adapter
- Streaming response handling
- Retry logic

### Phase 3: Execution Engine (Weeks 5-6)
- Concurrency controller
- Workload scheduler
- Rate limiting
- Request executor
- Metrics aggregation

### Phase 4: Output & Analysis (Week 7)
- Statistical computation
- Console output
- Export formats (JSON, CSV, binary)
- Cost calculation

### Phase 5: Testing & Polish (Week 8)
- Comprehensive test suite
- Performance benchmarks
- Documentation
- CLI refinement

### Phase 6: Advanced Features (Weeks 9-10)
- Additional providers
- Time-series database integration
- Advanced workload patterns
- Prometheus export

## Success Criteria

### Technical KPIs
- ✅ Sub-millisecond TTFT accuracy
- ✅ Handle 1000+ concurrent requests
- ✅ <100MB baseline memory
- ✅ <5% CPU overhead per request
- ✅ 0.1% percentile accuracy

### Quality Metrics
- ✅ >80% test coverage
- ✅ 100% public API documentation
- ✅ <0.1% error rate on valid requests
- ✅ <60 second full build time

## Resource Estimates

### Memory Budget (1000 concurrent requests)
- Active requests: 21 MB
- Metrics storage: 500 KB
- HTTP client pool: 10 MB
- Histograms: 1.5 MB
- **Total: ~32 MB**

### CPU Budget (per request)
- Timing overhead: 5 μs
- Metric recording: 1 μs
- HTTP processing: varies (network-bound)
- **Total overhead: <10 μs**

### Disk Space (per 1000 requests)
- JSON (pretty): 500 KB
- JSON (minified): 250 KB
- CSV: 150 KB
- Binary: 80 KB
- Compressed: 20 KB

## Key Files to Read

1. **Start Here**: ARCHITECTURE.md - Complete system design
2. **Project Structure**: CRATE_STRUCTURE.md - File organization
3. **Data Flow**: DATA_FLOW.md - Request lifecycle and timing
4. **Code Examples**: IMPLEMENTATION_GUIDE.md - Production patterns

## Next Steps

1. ✅ Review and approve architecture documents
2. ⬜ Initialize Cargo project structure
3. ⬜ Implement core data models (Phase 1)
4. ⬜ Build HTTP client infrastructure
5. ⬜ Implement first provider (OpenAI)
6. ⬜ Create metrics collection system
7. ⬜ Build execution engine
8. ⬜ Implement CLI and output formatting
9. ⬜ Write comprehensive tests
10. ⬜ Performance optimization and profiling

## Conclusion

This architecture provides a robust, scalable, and maintainable foundation for LLM-Latency-Lens. The design prioritizes:

1. **Performance**: Async I/O, efficient concurrency, minimal overhead
2. **Accuracy**: Nanosecond-precision timing, HDR histograms
3. **Extensibility**: Plugin-based providers, flexible configuration
4. **Reliability**: Comprehensive error handling, retry logic
5. **Usability**: Clear outputs, multiple formats, intuitive CLI

The Rust ecosystem provides excellent performance characteristics while maintaining type safety and developer ergonomics. The modular architecture enables incremental development and thorough testing.

---

**Document Version**: 1.0
**Date**: 2025-11-07
**Total Documentation**: ~50,000 words across 4 documents
**Estimated Project Size**: 12,000-15,000 lines of Rust code
**Implementation Timeline**: 10 weeks for full feature set
**Status**: ✅ Architecture Complete - Ready for Implementation
