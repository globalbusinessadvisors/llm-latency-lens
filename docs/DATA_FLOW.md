# Data Flow Architecture - Detailed Analysis

## 1. Request Execution Pipeline

### 1.1 Single Request Flow (Detailed)

```
┌─────────────────────────────────────────────────────────────────────┐
│ Step 1: Request Initialization                                      │
├─────────────────────────────────────────────────────────────────────┤
│ Input: CompletionRequest                                            │
│   - model: String                                                   │
│   - prompt: String                                                  │
│   - parameters: RequestParams                                       │
│                                                                     │
│ Actions:                                                            │
│   1. Generate request_id (UUID v4)                                 │
│   2. Capture timestamp (Utc::now())                                │
│   3. Initialize metrics container                                  │
│   4. Select provider adapter                                       │
│   5. Acquire concurrency permit                                    │
│                                                                     │
│ Output: Initialized RequestContext                                 │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 2: Pre-Request Phase                                          │
├─────────────────────────────────────────────────────────────────────┤
│ Actions:                                                            │
│   1. Check rate limiter                                            │
│      └─> If limited: wait or queue                                 │
│   2. Get HTTP client from pool                                     │
│   3. Build provider-specific request payload                       │
│   4. Add authentication headers                                    │
│   5. Start precision timer (T0)                                    │
│                                                                     │
│ Timing Markers:                                                     │
│   T0: Request start time (nanosecond precision)                    │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 3: Network Phase                                              │
├─────────────────────────────────────────────────────────────────────┤
│ Sub-step 3a: DNS Lookup                                            │
│   Timer: T_dns_start                                               │
│   Action: Resolve hostname to IP                                   │
│   Timer: T_dns_end                                                 │
│   Metric: dns_lookup_ns = T_dns_end - T_dns_start                 │
│                                                                     │
│ Sub-step 3b: TCP Connection                                        │
│   Timer: T_tcp_start                                               │
│   Action: Establish TCP connection                                 │
│   Timer: T_tcp_end                                                 │
│   Metric: tcp_connect_ns = T_tcp_end - T_tcp_start                │
│                                                                     │
│ Sub-step 3c: TLS Handshake                                         │
│   Timer: T_tls_start                                               │
│   Action: Complete TLS handshake                                   │
│   Timer: T_tls_end                                                 │
│   Metric: tls_handshake_ns = T_tls_end - T_tls_start              │
│                                                                     │
│ Sub-step 3d: Request Send                                          │
│   Timer: T_send_start                                              │
│   Action: Send HTTP request body                                   │
│   Timer: T_send_end                                                │
│   Metric: request_send_ns = T_send_end - T_send_start             │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 4: Response Waiting Phase                                     │
├─────────────────────────────────────────────────────────────────────┤
│ Sub-step 4a: Wait for First Byte                                   │
│   Timer: T_ttfb (Time to First Byte)                              │
│   Action: Wait for first response byte                             │
│   Metric: ttfb_ns = T_ttfb - T0                                    │
│                                                                     │
│ Sub-step 4b: Wait for First Token                                  │
│   Timer: T_ttft (Time to First Token) ⭐ CRITICAL METRIC           │
│   Action: Parse first token from response stream                   │
│   Metric: ttft_ns = T_ttft - T0                                    │
│           This is the most important latency metric!               │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 5: Token Streaming Phase                                      │
├─────────────────────────────────────────────────────────────────────┤
│ For each token in stream:                                          │
│   1. Parse SSE chunk                                               │
│      └─> Format: "data: {json}\n\n"                                │
│   2. Extract token/delta                                           │
│   3. Capture timestamp: T_token_i                                  │
│   4. Calculate inter-token latency:                                │
│      └─> inter_token_latency_ns = T_token_i - T_token_(i-1)       │
│   5. Append to token_latencies_ns vector                           │
│   6. Increment token counter                                       │
│                                                                     │
│ Collected Metrics:                                                  │
│   - token_latencies_ns: Vec<u64>                                   │
│   - completion_tokens: u32                                         │
│                                                                     │
│ Statistics Computed:                                                │
│   - mean_inter_token_latency                                       │
│   - p50, p95, p99 inter-token latency                              │
│   - tokens_per_second = tokens / (T_end - T_ttft)                 │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 6: Response Completion                                        │
├─────────────────────────────────────────────────────────────────────┤
│ Actions:                                                            │
│   1. Detect stream end (final SSE chunk or connection close)       │
│   2. Capture completion time: T_end                                │
│   3. Calculate total duration:                                     │
│      └─> total_duration_ns = T_end - T0 ⭐ CRITICAL METRIC         │
│   4. Parse final metadata (token counts, finish reason)            │
│   5. Extract usage information from response                       │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 7: Post-Processing                                            │
├─────────────────────────────────────────────────────────────────────┤
│ Actions:                                                            │
│   1. Calculate derived metrics:                                    │
│      - tokens_per_second = completion_tokens / duration_seconds    │
│      - cost metrics (using pricing database)                       │
│      - latency statistics                                          │
│   2. Build RequestMetrics object                                   │
│   3. Send to metrics collector                                     │
│   4. Release concurrency permit                                    │
│   5. Return HTTP connection to pool                                │
│                                                                     │
│ Output: RequestMetrics                                             │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.2 Timing Data Structure

```rust
pub struct TimingBreakdown {
    // Absolute timestamps (nanoseconds since epoch)
    t0_request_start: u64,
    t_dns_end: Option<u64>,
    t_tcp_end: Option<u64>,
    t_tls_end: Option<u64>,
    t_send_end: u64,
    t_ttfb: u64,
    t_ttft: u64,              // ⭐ Critical: First token received
    t_tokens: Vec<u64>,       // Timestamp for each token
    t_end: u64,               // ⭐ Critical: Request complete

    // Calculated durations (nanoseconds)
    dns_lookup_ns: Option<u64>,
    tcp_connect_ns: Option<u64>,
    tls_handshake_ns: Option<u64>,
    request_send_ns: u64,
    ttfb_ns: u64,
    ttft_ns: u64,             // ⭐ Most important latency metric
    token_latencies_ns: Vec<u64>,
    total_duration_ns: u64,   // ⭐ Second most important metric
}
```

---

## 2. Concurrent Execution Flow

### 2.1 Workload Scheduler Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│ Main Thread: Orchestrator                                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│ 1. Load Configuration                                              │
│    └─> Parse scenarios, providers, workload params                 │
│                                                                     │
│ 2. Initialize Components                                           │
│    ├─> Create HTTP client pool (size: config.pool_size)           │
│    ├─> Initialize provider adapters                                │
│    ├─> Create metrics collector                                    │
│    ├─> Setup concurrency controller (max: config.max_concurrency) │
│    └─> Initialize rate limiters (per provider)                     │
│                                                                     │
│ 3. Generate Request Matrix                                         │
│    For each scenario:                                              │
│      For each provider:                                            │
│        For each model:                                             │
│          Generate N requests                                       │
│    Result: Vec<PendingRequest>                                     │
│                                                                     │
│ 4. Spawn Worker Pool                                               │
│    └─> Create M worker tasks (M = max_concurrency)                 │
│                                                                     │
│ 5. Distribute Requests                                             │
│    └─> Send requests to worker queue (bounded channel)             │
│                                                                     │
│ 6. Monitor Progress                                                │
│    └─> Collect results, update progress bars                       │
│                                                                     │
│ 7. Aggregate Results                                               │
│    └─> Compute statistics, generate reports                        │
└─────────────────────────────────────────────────────────────────────┘
          │
          │ Bounded Channel (capacity: 2 * max_concurrency)
          │
          ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Worker Pool (M concurrent workers)                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│ ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐    │
│ │   Worker #1     │  │   Worker #2     │  │   Worker #N     │    │
│ │                 │  │                 │  │                 │    │
│ │ Loop:           │  │ Loop:           │  │ Loop:           │    │
│ │ 1. Recv request │  │ 1. Recv request │  │ 1. Recv request │    │
│ │ 2. Acquire sem  │  │ 2. Acquire sem  │  │ 2. Acquire sem  │    │
│ │ 3. Rate limit   │  │ 3. Rate limit   │  │ 3. Rate limit   │    │
│ │ 4. Execute      │  │ 4. Execute      │  │ 4. Execute      │    │
│ │ 5. Collect mtrc │  │ 5. Collect mtrc │  │ 5. Collect mtrc │    │
│ │ 6. Release sem  │  │ 6. Release sem  │  │ 6. Release sem  │    │
│ │ 7. Send result  │  │ 7. Send result  │  │ 7. Send result  │    │
│ └─────────────────┘  └─────────────────┘  └─────────────────┘    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
          │
          │ Result Channel
          │
          ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Metrics Collector Task                                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│ Loop:                                                               │
│   1. Receive RequestMetrics from result channel                    │
│   2. Update real-time statistics                                   │
│      ├─> Increment counters                                        │
│      ├─> Update histograms                                         │
│      └─> Track errors                                              │
│   3. Optionally stream to time-series database                     │
│   4. Update progress indicators                                    │
│                                                                     │
│ Data Structures:                                                    │
│   - DashMap<(Provider, Model), MetricsAccumulator>                 │
│   - HDR Histogram per metric type                                  │
│   - Error counter: HashMap<ErrorType, usize>                       │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Concurrency Control Flow

```rust
// Pseudo-code for concurrent execution

async fn execute_workload(config: BenchmarkConfig) -> Result<BenchmarkResults> {
    // 1. Setup phase
    let semaphore = Arc::new(Semaphore::new(config.max_concurrency));
    let metrics = Arc::new(MetricsCollector::new());
    let (tx_result, rx_result) = mpsc::channel(1000);

    // 2. Spawn metrics collector
    let collector_handle = tokio::spawn(collect_metrics(rx_result, metrics.clone()));

    // 3. Generate all requests
    let requests = generate_request_matrix(&config);

    // 4. Execute requests concurrently
    let request_handles = requests.into_iter().map(|req| {
        let sem = semaphore.clone();
        let tx = tx_result.clone();

        tokio::spawn(async move {
            // Acquire permit (blocks if at limit)
            let _permit = sem.acquire().await.unwrap();

            // Execute with rate limiting and retry
            let result = execute_with_retry(req).await;

            // Send metrics
            tx.send(result).await.unwrap();
        })
    });

    // 5. Wait for all requests
    futures::future::join_all(request_handles).await;

    // 6. Signal completion and aggregate
    drop(tx_result); // Close channel
    collector_handle.await?;

    // 7. Compute final statistics
    Ok(metrics.finalize())
}
```

---

## 3. Metrics Collection Pipeline

### 3.1 Real-time Aggregation Flow

```
Request Completion
       │
       │ RequestMetrics
       │
       ▼
┌──────────────────────────────────────────────────────────────┐
│ Metrics Collector (Single-threaded aggregation)             │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│ Step 1: Categorize                                          │
│   key = (provider, model, scenario)                         │
│   Get or create MetricsAccumulator for key                  │
│                                                              │
│ Step 2: Update Counters                                     │
│   accumulator.total_requests += 1                           │
│   if success:                                               │
│       accumulator.successful_requests += 1                  │
│   else:                                                     │
│       accumulator.failed_requests += 1                      │
│       accumulator.errors[error_type] += 1                   │
│                                                              │
│ Step 3: Update Histograms                                   │
│   accumulator.ttft_histogram.record(metrics.ttft_ns)        │
│   accumulator.duration_histogram.record(metrics.total_ns)   │
│   accumulator.throughput_histogram.record(metrics.tps)      │
│   for latency in metrics.token_latencies:                   │
│       accumulator.token_latency_histogram.record(latency)   │
│                                                              │
│ Step 4: Update Token Counters                               │
│   accumulator.total_prompt_tokens += metrics.prompt_tokens  │
│   accumulator.total_completion_tokens += metrics.completion │
│                                                              │
│ Step 5: Update Cost                                         │
│   accumulator.total_cost += metrics.cost                    │
│                                                              │
│ Step 6: Streaming (Optional)                                │
│   if config.database.enabled:                               │
│       send_to_timeseries_db(metrics)                        │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### 3.2 Metrics Accumulator Structure

```rust
pub struct MetricsAccumulator {
    // Request counters
    total_requests: AtomicUsize,
    successful_requests: AtomicUsize,
    failed_requests: AtomicUsize,

    // HDR Histograms (thread-safe via Arc<Mutex<>>)
    ttft_histogram: Arc<Mutex<Histogram<u64>>>,
    duration_histogram: Arc<Mutex<Histogram<u64>>>,
    throughput_histogram: Arc<Mutex<Histogram<u64>>>,
    token_latency_histogram: Arc<Mutex<Histogram<u64>>>,

    // Token statistics
    total_prompt_tokens: AtomicU64,
    total_completion_tokens: AtomicU64,

    // Cost tracking
    total_cost: Arc<Mutex<f64>>,

    // Error tracking
    errors: Arc<Mutex<HashMap<String, usize>>>,

    // Timing
    start_time: Instant,
    end_time: Arc<Mutex<Option<Instant>>>,
}

impl MetricsAccumulator {
    pub fn record(&self, metrics: RequestMetrics) {
        // Atomic updates for counters
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        match metrics.status {
            RequestStatus::Success => {
                self.successful_requests.fetch_add(1, Ordering::Relaxed);

                // Update histograms (requires lock)
                self.ttft_histogram.lock().unwrap()
                    .record(metrics.timing.ttft_ns).unwrap();

                self.duration_histogram.lock().unwrap()
                    .record(metrics.timing.total_duration_ns).unwrap();

                // Token statistics
                self.total_prompt_tokens.fetch_add(
                    metrics.tokens.prompt_tokens as u64,
                    Ordering::Relaxed
                );

                // Cost tracking
                *self.total_cost.lock().unwrap() += metrics.cost.total_cost;
            }
            _ => {
                self.failed_requests.fetch_add(1, Ordering::Relaxed);

                if let Some(error) = metrics.error {
                    let mut errors = self.errors.lock().unwrap();
                    *errors.entry(error.code).or_insert(0) += 1;
                }
            }
        }
    }

    pub fn finalize(self) -> AggregatedMetrics {
        // Compute percentiles from histograms
        let ttft_hist = self.ttft_histogram.lock().unwrap();
        let ttft_stats = DistributionStats {
            min: ttft_hist.min() as f64 / 1_000_000.0, // Convert to ms
            max: ttft_hist.max() as f64 / 1_000_000.0,
            mean: ttft_hist.mean() / 1_000_000.0,
            median: ttft_hist.value_at_quantile(0.5) as f64 / 1_000_000.0,
            p95: ttft_hist.value_at_quantile(0.95) as f64 / 1_000_000.0,
            p99: ttft_hist.value_at_quantile(0.99) as f64 / 1_000_000.0,
            p999: ttft_hist.value_at_quantile(0.999) as f64 / 1_000_000.0,
            std_dev: ttft_hist.stdev() / 1_000_000.0,
        };

        // Similar calculations for other metrics...

        AggregatedMetrics {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            latency: LatencyStats {
                ttft: ttft_stats,
                // ... other stats
            },
            // ... rest of fields
        }
    }
}
```

---

## 4. Provider-Specific Data Flows

### 4.1 OpenAI Streaming Response Flow

```
HTTP POST /v1/chat/completions
Headers:
  Authorization: Bearer sk-...
  Content-Type: application/json
Body:
{
  "model": "gpt-4-turbo-preview",
  "messages": [...],
  "stream": true
}

─────────────────────────────────────────────────

Response Stream (Server-Sent Events):

Chunk 1:
data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1234567890,"model":"gpt-4-turbo-preview","choices":[{"index":0,"delta":{"role":"assistant","content":""},"finish_reason":null}]}

Action: Start TTFT timer, this is first chunk

Chunk 2:
data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1234567890,"model":"gpt-4-turbo-preview","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

Action:
  - Mark TTFT (first actual token received) ⭐
  - Record token timestamp
  - Extract "Hello"

Chunk 3:
data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1234567890,"model":"gpt-4-turbo-preview","choices":[{"index":0,"delta":{"content":" world"},"finish_reason":null}]}

Action:
  - Record token timestamp
  - Calculate inter-token latency
  - Extract " world"

... (more chunks)

Final Chunk:
data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1234567890,"model":"gpt-4-turbo-preview","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]

Action:
  - Mark completion time
  - Parse finish_reason
  - Finalize metrics
```

**Parsing Implementation:**

```rust
async fn parse_openai_stream(
    response: reqwest::Response,
) -> Result<Vec<StreamChunk>> {
    let mut stream = response.bytes_stream();
    let mut chunks = Vec::new();
    let mut buffer = String::new();

    while let Some(item) = stream.next().await {
        let bytes = item?;
        buffer.push_str(&String::from_utf8_lossy(&bytes));

        // SSE format: "data: {json}\n\n"
        while let Some(pos) = buffer.find("\n\n") {
            let line = buffer[..pos].trim();
            buffer = buffer[pos + 2..].to_string();

            if line.starts_with("data: ") {
                let json_str = &line[6..]; // Remove "data: " prefix

                if json_str == "[DONE]" {
                    break;
                }

                let chunk: OpenAIStreamChunk = serde_json::from_str(json_str)?;

                // Extract delta content
                if let Some(content) = chunk.choices[0].delta.content {
                    chunks.push(StreamChunk {
                        id: chunk.id,
                        model: chunk.model,
                        delta: content,
                        finish_reason: chunk.choices[0].finish_reason,
                    });
                }
            }
        }
    }

    Ok(chunks)
}
```

### 4.2 Anthropic Streaming Response Flow

```
HTTP POST /v1/messages
Headers:
  x-api-key: sk-ant-...
  anthropic-version: 2023-06-01
  Content-Type: application/json
Body:
{
  "model": "claude-3-opus-20240229",
  "messages": [...],
  "stream": true
}

─────────────────────────────────────────────────

Response Stream:

event: message_start
data: {"type":"message_start","message":{"id":"msg_123","type":"message","role":"assistant","content":[],"model":"claude-3-opus-20240229"}}

event: content_block_start
data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}
⭐ TTFT: First content delta

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":" world"}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"end_turn","stop_sequence":null},"usage":{"output_tokens":50}}

event: message_stop
data: {"type":"message_stop"}
```

**Key Differences from OpenAI:**
1. Uses named events (event: type)
2. Separate content_block_delta events for tokens
3. Token usage reported in message_delta event
4. More structured event types

---

## 5. Storage & Export Flow

### 5.1 Multi-format Export Pipeline

```
AggregatedMetrics
       │
       ├────────────────────────┬────────────────────────┐
       │                        │                        │
       ▼                        ▼                        ▼
┌─────────────┐        ┌─────────────┐        ┌─────────────┐
│ JSON Export │        │ CSV Export  │        │Binary Export│
├─────────────┤        ├─────────────┤        ├─────────────┤
│             │        │             │        │             │
│ 1. Serialize│        │ 1. Flatten  │        │ 1. Serialize│
│    to JSON  │        │    nested   │        │    with     │
│    (pretty) │        │    structs  │        │    bincode  │
│             │        │             │        │    or       │
│ 2. Write to │        │ 2. Generate │        │    msgpack  │
│    file     │        │    CSV rows │        │             │
│             │        │             │        │ 2. Compress │
│ 3. Optional │        │ 3. Write    │        │    (gzip)   │
│    compress │        │    headers  │        │             │
│             │        │    and data │        │ 3. Write to │
│             │        │             │        │    file     │
└─────────────┘        └─────────────┘        └─────────────┘
       │                        │                        │
       ▼                        ▼                        ▼
  results/                  results/                results/
  bench.json               bench.csv               bench.bin

File sizes (typical for 1000 requests):
- JSON (pretty): ~500 KB
- JSON (minified): ~250 KB
- CSV: ~150 KB
- Binary (bincode): ~80 KB
- Binary (msgpack): ~70 KB
- Binary (compressed): ~20 KB
```

### 5.2 Time-series Database Flow

```
RequestMetrics (real-time)
       │
       │ As requests complete
       │
       ▼
┌──────────────────────────────────────────┐
│ Time-series Writer Task                  │
├──────────────────────────────────────────┤
│                                          │
│ 1. Receive metrics from channel          │
│                                          │
│ 2. Transform to time-series format       │
│    Measurement: llm_request              │
│    Tags:                                 │
│      - provider: "openai"                │
│      - model: "gpt-4-turbo"              │
│      - scenario: "high_concurrency"      │
│    Fields:                               │
│      - ttft_ms: 432.1                    │
│      - duration_ms: 2389.4               │
│      - tokens_per_sec: 44.2              │
│      - cost: 0.012                       │
│    Timestamp: 2024-11-07T18:30:01.234Z   │
│                                          │
│ 3. Batch writes (every 100 points or 1s)│
│                                          │
│ 4. Write to InfluxDB/Prometheus          │
│                                          │
└──────────────────────────────────────────┘
       │
       │ HTTP POST
       │
       ▼
┌──────────────────────────────────────────┐
│ InfluxDB / Time-series Database          │
├──────────────────────────────────────────┤
│                                          │
│ Enables:                                 │
│ - Real-time dashboards (Grafana)         │
│ - Historical trend analysis              │
│ - Alerting on thresholds                 │
│ - Performance regression detection       │
│                                          │
└──────────────────────────────────────────┘
```

**InfluxDB Line Protocol Example:**

```
llm_request,provider=openai,model=gpt-4-turbo,scenario=high_concurrency ttft_ms=432.1,duration_ms=2389.4,tokens_per_sec=44.2,cost=0.012 1699380601234000000
llm_request,provider=anthropic,model=claude-3-opus,scenario=high_concurrency ttft_ms=389.7,duration_ms=2156.3,tokens_per_sec=48.9,cost=0.015 1699380602345000000
```

---

## 6. Error Handling Flow

### 6.1 Retry with Backoff

```
Request Attempt #1
       │
       ▼
   Execute
       │
       ├─── Success ────────────────────► Return Result
       │
       └─── Error
               │
               ▼
          Check Retry Policy
               │
               ├─── Should NOT Retry ────► Return Error
               │    (Auth failure,
               │     invalid request,
               │     max attempts)
               │
               └─── Should Retry
                       │
                       ▼
                  Calculate Backoff
                  (exponential with jitter)
                       │
                       │ Wait: initial_backoff * (multiplier ^ attempt)
                       │       + random_jitter(0..backoff*0.1)
                       │
                       ▼
              Request Attempt #2
                       │
                       ▼
                   Execute
                       │
                       ├─── Success ──────► Record retry count
                       │                    Return Result
                       │
                       └─── Error ─────────► Repeat loop
                                            (up to max_attempts)
```

### 6.2 Rate Limit Handling

```
Request Ready
       │
       ▼
┌──────────────────────────┐
│ Rate Limiter Check       │
├──────────────────────────┤
│ Algorithm: Token Bucket  │
│                          │
│ if tokens_available > 0: │
│   tokens -= 1            │
│   proceed                │
│ else:                    │
│   wait_time = next_refill│
│   sleep(wait_time)       │
│   retry check            │
└──────────────────────────┘
       │
       ▼
   Execute Request
       │
       ├─── 429 Rate Limit Error
       │       │
       │       ├─ Has Retry-After header?
       │       │    └─ Yes: wait(retry_after)
       │       │    └─ No: exponential backoff
       │       │
       │       └─► Retry
       │
       └─── Success
```

---

## 7. Memory Management

### 7.1 Request Lifecycle Memory

```
┌──────────────────────────────────────┐
│ Request Start                        │
│ Allocations:                         │
│ - RequestMetrics: ~300 bytes         │
│ - TimingMetrics: ~200 bytes          │
│ - Vec<u64> (token times): ~8n bytes  │
│ Total: ~500 bytes + 8*tokens         │
└──────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────┐
│ During Execution                     │
│ Additional:                          │
│ - HTTP buffers: ~16 KB (pooled)      │
│ - SSE parsing buffer: ~4 KB          │
│ Peak: ~21 KB per active request      │
└──────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────┐
│ After Completion                     │
│ - HTTP buffers returned to pool      │
│ - RequestMetrics sent to collector   │
│ - Parsing buffers dropped            │
│ Retained: ~500 bytes (metrics only)  │
└──────────────────────────────────────┘

Memory Budget for 1000 concurrent requests:
- Active requests: 1000 * 21 KB = 21 MB
- Metrics storage: 1000 * 500 bytes = 500 KB
- HTTP client pool: ~10 MB
- Histograms: ~500 KB
- Total: ~32 MB
```

### 7.2 Histogram Memory

```
HDR Histogram Configuration:
- Range: 1 ns to 60 seconds
- Precision: 3 significant figures
- Memory per histogram: ~32 KB

Number of histograms:
- Per (provider, model, scenario) combination
- 4 histograms per combination:
  - TTFT
  - Total duration
  - Throughput
  - Inter-token latency

Example: 3 providers * 2 models * 2 scenarios * 4 histograms
        = 48 histograms * 32 KB = 1.5 MB
```

---

## 8. Performance Optimization Strategies

### 8.1 Connection Pooling

```
┌────────────────────────────────────────────────┐
│ HTTP Client Pool (per provider)                │
├────────────────────────────────────────────────┤
│                                                │
│ ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│ │  Conn 1  │  │  Conn 2  │  │  Conn N  │      │
│ │ (idle)   │  │ (active) │  │ (idle)   │      │
│ └──────────┘  └──────────┘  └──────────┘      │
│                                                │
│ Features:                                      │
│ - Keep-alive: 90 seconds                       │
│ - HTTP/2 multiplexing                          │
│ - Automatic reconnect                          │
│ - Per-host connection limits                   │
│                                                │
│ Benefits:                                      │
│ - Eliminate TCP handshake (save ~50-100ms)     │
│ - Eliminate TLS handshake (save ~100-200ms)    │
│ - Reduce DNS lookups                           │
│ - Share connections across concurrent requests │
└────────────────────────────────────────────────┘

Connection reuse ratio: 95%+
Latency reduction: 150-300ms per request
```

### 8.2 Zero-copy Streaming

```
HTTP Response Stream
       │
       │ (no buffering of full response)
       │
       ▼
┌─────────────────────────────────┐
│ Stream Processing               │
├─────────────────────────────────┤
│                                 │
│ For each chunk:                 │
│   1. Receive bytes              │
│   2. Parse SSE (in-place)       │
│   3. Extract token (string ref) │
│   4. Record timestamp           │
│   5. Drop chunk bytes           │
│                                 │
│ Memory: O(1) per chunk          │
│ No accumulation of full response│
└─────────────────────────────────┘

Memory savings:
- Typical response: 5-50 KB
- Streaming: ~4 KB buffer
- Savings: 1-46 KB per request
- At 1000 concurrent: 1-46 MB saved
```

---

## Summary: Critical Data Paths

1. **Most Important Metrics** (Priority 1):
   - TTFT (Time to First Token): Primary user-perceived latency
   - Total Duration: Overall request completion time
   - Tokens per Second: Throughput metric

2. **Secondary Metrics** (Priority 2):
   - Inter-token latency distribution
   - Network timing breakdown (DNS, TCP, TLS)
   - Cost per request

3. **Operational Metrics** (Priority 3):
   - Success/failure rates
   - Error type distribution
   - Retry counts
   - Rate limit encounters

4. **Data Flow Bottlenecks to Avoid**:
   - Synchronous metric collection (use channels)
   - Unbounded buffers (use bounded channels)
   - Excessive cloning (use Arc/Rc)
   - JSON parsing on hot path (use simd-json for perf)

5. **Performance Targets**:
   - Timing overhead: <5 μs per request
   - Memory: <50 MB for 1000 concurrent requests
   - Throughput: >500 requests/sec per core
   - Metric aggregation: <1ms per request
