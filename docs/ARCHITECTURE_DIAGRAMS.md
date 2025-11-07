# LLM-Latency-Lens Architecture Diagrams

This document provides detailed architectural diagrams for LLM-Latency-Lens ecosystem integration.

---

## 1. System-Level Architecture

### 1.1 High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Control Plane Layer                              │
│                    (Kubernetes / Orchestrator)                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│  │ Config Store │  │  Scheduler   │  │  API Gateway │                  │
│  └──────────────┘  └──────────────┘  └──────────────┘                  │
└──────────────────────────────┬──────────────────────────────────────────┘
                               │
        ┌──────────────────────┼──────────────────────┐
        │                      │                      │
        ▼                      ▼                      ▼
┌───────────────┐   ┌──────────────────┐   ┌─────────────────┐
│ LLM-Test-     │   │ LLM-Latency-     │   │ LLM-Auto-       │
│ Bench         │   │ Lens             │   │ Optimizer       │
│               │   │                  │   │                 │
│ ┌───────────┐ │   │ ┌──────────────┐│   │ ┌─────────────┐ │
│ │Workload   │ │   │ │Collector     ││   │ │Trigger      │ │
│ │Generator  │ │   │ │Engine        ││   │ │Engine       │ │
│ └─────┬─────┘ │   │ └──────┬───────┘│   │ └──────┬──────┘ │
│       │       │   │        │        │   │        │        │
│ ┌─────▼─────┐ │   │ ┌──────▼───────┐│   │ ┌──────▼──────┐ │
│ │Results    │ │   │ │Statistics    ││   │ │Decision     │ │
│ │Store      │ │   │ │Processor     ││   │ │Engine       │ │
│ └───────────┘ │   │ └──────┬───────┘│   │ └──────┬──────┘ │
└───────┬───────┘   │        │        │   │        │        │
        │           │ ┌──────▼───────┐│   │ ┌──────▼──────┐ │
        │           │ │Multi-Format  ││   │ │Config       │ │
        │           │ │Exporter      ││   │ │Adjuster     │ │
        │           │ └──────┬───────┘│   │ └─────────────┘ │
        │           └────────┼────────┘   └─────────────────┘
        │                    │
        └────────────────────┼────────────────────┐
                             │                    │
                             ▼                    ▼
        ┌────────────────────────────────────────────────────┐
        │          LLM-Observatory (Observability)           │
        │  ┌──────────────┐  ┌──────────────┐               │
        │  │ Prometheus   │  │ Jaeger       │               │
        │  │ (Metrics)    │  │ (Traces)     │               │
        │  └──────┬───────┘  └──────┬───────┘               │
        │         │                  │                       │
        │         └────────┬─────────┘                       │
        │                  │                                 │
        │         ┌────────▼─────────┐                       │
        │         │    Grafana       │                       │
        │         │  (Visualization) │                       │
        │         └──────────────────┘                       │
        └────────────────────────────────────────────────────┘
                             │
        ┌────────────────────┴────────────────────┐
        │                    │                    │
        ▼                    ▼                    ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│ LLM-Shield   │   │ LLM-Edge-    │   │ Inference    │
│ (Security)   │   │ Agent        │   │ Engines      │
│              │   │              │   │              │
│ Guardrails   │   │ Distributed  │   │ vLLM / TGI / │
│ PII Filter   │   │ Monitoring   │   │ Ollama       │
│ Threat Det.  │   │ Edge Sync    │   │              │
└──────────────┘   └──────────────┘   └──────────────┘
```

---

## 2. Data Flow Architecture

### 2.1 Metrics Collection Flow

```
┌──────────────────────────────────────────────────────────────┐
│                    Inference Request                          │
│                          ↓                                    │
│  ┌────────────────────────────────────────────────────┐      │
│  │          LLM Inference Engine (vLLM/TGI)           │      │
│  │                                                     │      │
│  │  Prefill Phase  →  Decode Phase  →  Complete      │      │
│  └──────┬────────────────────┬────────────────┬───────┘      │
│         │                    │                │              │
│         ▼                    ▼                ▼              │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐       │
│  │ TTFT Event  │   │ TPOT Events │   │ E2E Latency │       │
│  │ t=50ms      │   │ [5ms/token] │   │ t=500ms     │       │
│  └─────┬───────┘   └──────┬──────┘   └──────┬──────┘       │
└────────┼──────────────────┼─────────────────┼──────────────┘
         │                  │                 │
         └──────────────────┼─────────────────┘
                            │
         ┌──────────────────▼──────────────────┐
         │   LLM-Latency-Lens Collector        │
         │   (Async Tokio Stream Processing)   │
         │                                      │
         │   ┌────────────────────────┐         │
         │   │  Event Aggregator      │         │
         │   │  - Buffer events       │         │
         │   │  - Calculate metrics   │         │
         │   │  - Enrich with context │         │
         │   └───────────┬────────────┘         │
         │               │                      │
         │   ┌───────────▼────────────┐         │
         │   │  Measurement Builder   │         │
         │   │  - Create typed record │         │
         │   │  - Add metadata        │         │
         │   │  - Validate schema     │         │
         │   └───────────┬────────────┘         │
         └───────────────┼──────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
         ▼               ▼               ▼
┌─────────────┐  ┌──────────────┐  ┌──────────────┐
│ Statistics  │  │ Real-time    │  │ Exporters    │
│ Processor   │  │ Stream       │  │ (Multi-sink) │
│             │  │              │  │              │
│ - P50/P95/  │  │ - WebSocket  │  │ - JSON       │
│   P99       │  │ - SSE        │  │ - Protobuf   │
│ - Mean/     │  │ - Callbacks  │  │ - Arrow IPC  │
│   StdDev    │  │              │  │ - OTLP       │
│ - Histogram │  │              │  │ - Prometheus │
└─────────────┘  └──────────────┘  └──────┬───────┘
                                           │
                    ┌──────────────────────┼────────────────────┐
                    │                      │                    │
                    ▼                      ▼                    ▼
        ┌──────────────────┐   ┌──────────────────┐   ┌──────────────────┐
        │ Benchmark Store  │   │ Time-Series DB   │   │ Feedback Loop    │
        │ (JSON/Parquet)   │   │ (Prometheus)     │   │ (Optimizer)      │
        └──────────────────┘   └──────────────────┘   └──────────────────┘
```

### 2.2 Telemetry Export Flow (OpenTelemetry)

```
┌────────────────────────────────────────────────────────────┐
│              LLM-Latency-Lens Application                   │
│                                                             │
│  ┌───────────────────────────────────────────────────┐     │
│  │           Instrumentation Layer                   │     │
│  │                                                    │     │
│  │  ┌──────────────┐  ┌──────────────┐              │     │
│  │  │ Span Builder │  │ Metric Builder│              │     │
│  │  │              │  │               │              │     │
│  │  │ - Operation  │  │ - Counters   │              │     │
│  │  │   context    │  │ - Histograms │              │     │
│  │  │ - Attributes │  │ - Gauges     │              │     │
│  │  └──────┬───────┘  └──────┬───────┘              │     │
│  │         │                  │                      │     │
│  │         └────────┬─────────┘                      │     │
│  │                  │                                │     │
│  └──────────────────┼────────────────────────────────┘     │
│                     │                                      │
│  ┌──────────────────▼────────────────────────────────┐     │
│  │         OpenTelemetry SDK                         │     │
│  │                                                    │     │
│  │  ┌──────────────────┐  ┌──────────────────┐      │     │
│  │  │ Tracer Provider  │  │ Meter Provider   │      │     │
│  │  │                  │  │                  │      │     │
│  │  │ - Span Processor │  │ - Metric Reader  │      │     │
│  │  │ - Sampling       │  │ - Aggregation    │      │     │
│  │  │ - Context Prop.  │  │ - Temporality    │      │     │
│  │  └─────────┬────────┘  └─────────┬────────┘      │     │
│  │            │                      │               │     │
│  │            └──────────┬───────────┘               │     │
│  │                       │                           │     │
│  │            ┌──────────▼──────────┐                │     │
│  │            │   Resource          │                │     │
│  │            │   (service.name,    │                │     │
│  │            │    version, etc.)   │                │     │
│  │            └──────────┬──────────┘                │     │
│  │                       │                           │     │
│  └───────────────────────┼───────────────────────────┘     │
│                          │                                 │
│  ┌───────────────────────▼───────────────────────────┐     │
│  │         OTLP Exporter                             │     │
│  │                                                    │     │
│  │  ┌──────────────────────────────────────────┐     │     │
│  │  │  Protocol Buffer Serialization           │     │     │
│  │  │  - Encode spans as TraceData             │     │     │
│  │  │  - Encode metrics as MetricsData         │     │     │
│  │  │  - Batch and compress                    │     │     │
│  │  └──────────────┬───────────────────────────┘     │     │
│  │                 │                                 │     │
│  │  ┌──────────────▼───────────────────────────┐     │     │
│  │  │  gRPC Client (Tonic)                     │     │     │
│  │  │  - TLS/mTLS support                      │     │     │
│  │  │  - Authentication headers                │     │     │
│  │  │  - Retry logic                           │     │     │
│  │  └──────────────┬───────────────────────────┘     │     │
│  └─────────────────┼─────────────────────────────────┘     │
└────────────────────┼───────────────────────────────────────┘
                     │
                     │ OTLP/gRPC over HTTP/2
                     │
         ┌───────────▼────────────┐
         │   OpenTelemetry        │
         │   Collector            │
         │                        │
         │  ┌──────────────────┐  │
         │  │   Receivers      │  │
         │  │   - OTLP         │  │
         │  └────────┬─────────┘  │
         │           │            │
         │  ┌────────▼─────────┐  │
         │  │   Processors     │  │
         │  │   - Batch        │  │
         │  │   - Filter       │  │
         │  │   - Enrich       │  │
         │  └────────┬─────────┘  │
         │           │            │
         │  ┌────────▼─────────┐  │
         │  │   Exporters      │  │
         │  │   - Prometheus   │  │
         │  │   - Jaeger       │  │
         │  │   - Backend APIs │  │
         │  └──────────────────┘  │
         └────────────────────────┘
                     │
         ┌───────────┼────────────┐
         │           │            │
         ▼           ▼            ▼
    ┌─────────┐ ┌─────────┐ ┌─────────┐
    │Prometheus│ │ Jaeger │ │ Custom  │
    │         │ │        │ │ Backend │
    └─────────┘ └────────┘ └─────────┘
```

---

## 3. Benchmark Integration Architecture

### 3.1 LLMPerf-Style Benchmark Flow

```
┌──────────────────────────────────────────────────────────────────┐
│                   Benchmark Orchestrator                          │
│                                                                   │
│  ┌────────────────────────────────────────────────────────┐      │
│  │  Configuration Loader                                  │      │
│  │  - Test parameters (concurrency, token counts, etc.)   │      │
│  │  - Model endpoints                                     │      │
│  │  - SLA thresholds                                      │      │
│  └────────────────────┬───────────────────────────────────┘      │
│                       │                                          │
│  ┌────────────────────▼───────────────────────────────────┐      │
│  │  Workload Generator                                    │      │
│  │  - Token distribution sampling (Normal/Uniform)        │      │
│  │  - Prompt generation from dataset                      │      │
│  │  - Request scheduling (fixed/poisson arrival)          │      │
│  └────────────────────┬───────────────────────────────────┘      │
│                       │                                          │
│  ┌────────────────────▼───────────────────────────────────┐      │
│  │  Concurrent Executor (Tokio Runtime)                   │      │
│  │                                                         │      │
│  │    Thread 1         Thread 2        Thread N           │      │
│  │  ┌──────────┐    ┌──────────┐    ┌──────────┐         │      │
│  │  │ Request  │    │ Request  │    │ Request  │         │      │
│  │  │ Worker   │    │ Worker   │    │ Worker   │         │      │
│  │  └────┬─────┘    └────┬─────┘    └────┬─────┘         │      │
│  │       │               │               │                │      │
│  └───────┼───────────────┼───────────────┼────────────────┘      │
└──────────┼───────────────┼───────────────┼───────────────────────┘
           │               │               │
           │  HTTP/SSE Stream Requests     │
           │               │               │
           ▼               ▼               ▼
    ┌──────────────────────────────────────────────┐
    │      LLM Inference Endpoint (vLLM)           │
    │                                              │
    │  /v1/completions (streaming)                 │
    └───────────────┬──────────────────────────────┘
                    │
                    │ SSE: data: {...chunk...}
                    │
    ┌───────────────▼──────────────────────────────┐
    │   LLM-Latency-Lens Measurement Collector     │
    │                                              │
    │  Per-request tracking:                       │
    │  ┌──────────────────────────────────────┐    │
    │  │  Request ID: req-123                 │    │
    │  │  Start time: T0                      │    │
    │  │  First token: T1 (TTFT = T1 - T0)   │    │
    │  │  Token 2: T2 (TPOT = T2 - T1)       │    │
    │  │  Token 3: T3 (TPOT = T3 - T2)       │    │
    │  │  ...                                 │    │
    │  │  Complete: Tn (E2E = Tn - T0)       │    │
    │  └──────────────────────────────────────┘    │
    │                                              │
    │  Aggregate to InferenceMeasurement           │
    └───────────────┬──────────────────────────────┘
                    │
    ┌───────────────▼──────────────────────────────┐
    │   Statistical Aggregator                     │
    │                                              │
    │   Collect all measurements                   │
    │   Calculate distributions:                   │
    │   - Percentiles (P50, P90, P95, P99)        │
    │   - Mean, median, std deviation             │
    │   - Min, max                                │
    │                                              │
    │   Group by:                                  │
    │   - Concurrency level                        │
    │   - Token count buckets                      │
    │   - Time windows                             │
    └───────────────┬──────────────────────────────┘
                    │
    ┌───────────────▼──────────────────────────────┐
    │   Results Formatter & Exporter               │
    │                                              │
    │   Output formats:                            │
    │   - JSON (schema-compliant)                  │
    │   - CSV (for spreadsheets)                   │
    │   - Arrow IPC (for analytics)                │
    │   - HTML report (visualization)              │
    └───────────────┬──────────────────────────────┘
                    │
         ┌──────────┼──────────┐
         │          │          │
         ▼          ▼          ▼
    ┌────────┐ ┌────────┐ ┌────────┐
    │ JSON   │ │ Arrow  │ │ HTML   │
    │ File   │ │ File   │ │ Report │
    └────────┘ └────────┘ └────────┘
```

---

## 4. Optimization Feedback Loop Architecture

### 4.1 SAFLA-Style Feedback Loop

```
┌──────────────────────────────────────────────────────────────────┐
│              Continuous Optimization Loop                         │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │  Phase 1: COLLECT                                       │     │
│  │  ┌─────────────────────────────────────────────────┐    │     │
│  │  │  Metrics Aggregator                             │    │     │
│  │  │  - Real-time performance data                   │    │     │
│  │  │  - Resource utilization                         │    │     │
│  │  │  - Error rates and types                        │    │     │
│  │  │  - SLA compliance status                        │    │     │
│  │  └──────────────────┬──────────────────────────────┘    │     │
│  └────────────────────┬┘                                    │     │
│                       │                                     │     │
│  ┌────────────────────▼─────────────────────────────────┐  │     │
│  │  Phase 2: EVALUATE                                   │  │     │
│  │  ┌─────────────────────────────────────────────┐     │  │     │
│  │  │  Performance Evaluator                      │     │  │     │
│  │  │                                             │     │  │     │
│  │  │  Compare against:                           │     │  │     │
│  │  │  - SLA thresholds                           │     │  │     │
│  │  │  - Historical baselines                     │     │  │     │
│  │  │  - Optimization goals                       │     │  │     │
│  │  │                                             │     │  │     │
│  │  │  Calculate performance score:               │     │  │     │
│  │  │  Score = f(latency, throughput, cost)      │     │  │     │
│  │  └──────────────────┬──────────────────────────┘     │  │     │
│  └────────────────────┬┘                                 │  │     │
│                       │                                  │  │     │
│  ┌────────────────────▼─────────────────────────────────┐  │     │
│  │  Phase 3: LEARN                                      │  │     │
│  │  ┌─────────────────────────────────────────────┐     │  │     │
│  │  │  Causal Learning Engine                     │     │  │     │
│  │  │                                             │     │  │     │
│  │  │  Episode Storage:                           │     │  │     │
│  │  │  ┌─────────────────────────────────────┐    │     │  │     │
│  │  │  │ State: {config, metrics, context}   │    │     │  │     │
│  │  │  │ Action: {change_type, parameters}   │    │     │  │     │
│  │  │  │ Reward: performance_score_delta     │    │     │  │     │
│  │  │  │ Next_State: {new_config, new_metrics}│   │     │  │     │
│  │  │  └─────────────────────────────────────┘    │     │  │     │
│  │  │                                             │     │  │     │
│  │  │  Causal Graph Update:                       │     │  │     │
│  │  │  Action → Outcome relationships             │     │  │     │
│  │  │  - "↑ batch_size" → "+25% throughput"      │     │  │     │
│  │  │  - "↑ replicas" → "-35% latency"           │     │  │     │
│  │  │  - "enable_int8" → "+15% throughput"       │     │  │     │
│  │  └──────────────────┬──────────────────────────┘     │  │     │
│  └────────────────────┬┘                                 │  │     │
│                       │                                  │  │     │
│  ┌────────────────────▼─────────────────────────────────┐  │     │
│  │  Phase 4: DECIDE                                     │  │     │
│  │  ┌─────────────────────────────────────────────┐     │  │     │
│  │  │  Optimization Engine                        │     │  │     │
│  │  │                                             │     │  │     │
│  │  │  Trigger Evaluation:                        │     │  │     │
│  │  │  ✓ P95 latency > 100ms                     │     │  │     │
│  │  │  ✓ Throughput < 10 RPS                     │     │  │     │
│  │  │  ✗ Error rate < 1%                         │     │  │     │
│  │  │                                             │     │  │     │
│  │  │  Recommendation Generation:                 │     │  │     │
│  │  │  Using causal graph + current state        │     │  │     │
│  │  │                                             │     │  │     │
│  │  │  Ranked Recommendations:                    │     │  │     │
│  │  │  1. Scale replicas: 2→4 (confidence: 85%)  │     │  │     │
│  │  │  2. Increase batch: 4→8 (confidence: 78%) │     │  │     │
│  │  │  3. Enable int8 (confidence: 65%)          │     │  │     │
│  │  └──────────────────┬──────────────────────────┘     │  │     │
│  └────────────────────┬┘                                 │  │     │
│                       │                                  │  │     │
│  ┌────────────────────▼─────────────────────────────────┐  │     │
│  │  Phase 5: ACT                                        │  │     │
│  │  ┌─────────────────────────────────────────────┐     │  │     │
│  │  │  Configuration Adjuster                     │     │  │     │
│  │  │                                             │     │  │     │
│  │  │  Pre-flight checks:                         │     │  │     │
│  │  │  - Validate recommendation                  │     │  │     │
│  │  │  - Check cooldown period                    │     │  │     │
│  │  │  - Verify resource availability             │     │  │     │
│  │  │                                             │     │  │     │
│  │  │  Apply change:                              │     │  │     │
│  │  │  - Update deployment config                 │     │  │     │
│  │  │  - Store rollback snapshot                  │     │  │     │
│  │  │  - Tag with adjustment_id                   │     │  │     │
│  │  └──────────────────┬──────────────────────────┘     │  │     │
│  └────────────────────┬┘                                 │  │     │
│                       │                                  │  │     │
│  ┌────────────────────▼─────────────────────────────────┐  │     │
│  │  Phase 6: VERIFY                                     │  │     │
│  │  ┌─────────────────────────────────────────────┐     │  │     │
│  │  │  Verification Monitor                       │     │  │     │
│  │  │                                             │     │  │     │
│  │  │  Wait: verification_period (e.g., 5 min)   │     │  │     │
│  │  │                                             │     │  │     │
│  │  │  Compare metrics:                           │     │  │     │
│  │  │  Before: P95=120ms, RPS=8.5                │     │  │     │
│  │  │  After:  P95=75ms,  RPS=16.2               │     │  │     │
│  │  │                                             │     │  │     │
│  │  │  Improvement: +37.5% latency, +90% RPS     │     │  │     │
│  │  │                                             │     │  │     │
│  │  │  Decision:                                  │     │  │     │
│  │  │  ✓ Success → Keep change                   │     │  │     │
│  │  │  ✗ Failure → Rollback                      │     │  │     │
│  │  └──────────────────┬──────────────────────────┘     │  │     │
│  └────────────────────┬┘                                 │  │     │
│                       │                                  │  │     │
│                       └──────────────────────────────────┘  │     │
│                                │                            │     │
│                        Return to Phase 1                    │     │
│                        (Continuous Loop)                    │     │
└──────────────────────────────────────────────────────────────────┘

                    ┌────────────────────┐
                    │  External Systems  │
                    │                    │
                    │  - Kubernetes API  │
                    │  - Config Store    │
                    │  - Audit Log       │
                    └────────────────────┘
```

### 4.2 Trigger System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  Trigger Evaluation System                   │
│                                                              │
│  ┌────────────────────────────────────────────────────┐     │
│  │  Incoming Metrics Stream                           │     │
│  │  (Real-time from Collector)                        │     │
│  └───────────────────────┬────────────────────────────┘     │
│                          │                                  │
│  ┌───────────────────────▼────────────────────────────┐     │
│  │  Windowing & Aggregation                           │     │
│  │  - 1-minute window                                 │     │
│  │  - 5-minute window                                 │     │
│  │  - 15-minute window                                │     │
│  └───────────────────────┬────────────────────────────┘     │
│                          │                                  │
│  ┌───────────────────────▼────────────────────────────┐     │
│  │  Trigger Registry                                  │     │
│  │                                                     │     │
│  │  ┌───────────────────────────────────────────┐     │     │
│  │  │ Trigger 1: Latency Threshold              │     │     │
│  │  │ - Type: P95LatencyExceeds                 │     │     │
│  │  │ - Threshold: 100ms                        │     │     │
│  │  │ - Window: 5 minutes                       │     │     │
│  │  │ - Cooldown: 10 minutes                    │     │     │
│  │  │ - Last triggered: T-15min (OK to trigger) │     │     │
│  │  └───────────────────┬───────────────────────┘     │     │
│  │                      │                             │     │
│  │  ┌───────────────────▼───────────────────────┐     │     │
│  │  │ Trigger 2: Throughput Threshold           │     │     │
│  │  │ - Type: ThroughputBelow                   │     │     │
│  │  │ - Threshold: 10 RPS                       │     │     │
│  │  │ - Window: 5 minutes                       │     │     │
│  │  │ - Cooldown: 10 minutes                    │     │     │
│  │  │ - Last triggered: Never                   │     │     │
│  │  └───────────────────┬───────────────────────┘     │     │
│  │                      │                             │     │
│  │  ┌───────────────────▼───────────────────────┐     │     │
│  │  │ Trigger 3: Resource Saturation            │     │     │
│  │  │ - Type: AND(                              │     │     │
│  │  │     GpuUtilizationAbove(90%),             │     │     │
│  │  │     P95LatencyExceeds(200ms)              │     │     │
│  │  │   )                                       │     │     │
│  │  │ - Window: 5 minutes                       │     │     │
│  │  │ - Cooldown: 15 minutes                    │     │     │
│  │  │ - Last triggered: T-20min (OK to trigger) │     │     │
│  │  └───────────────────┬───────────────────────┘     │     │
│  │                      │                             │     │
│  │  ┌───────────────────▼───────────────────────┐     │     │
│  │  │ Trigger 4: Error Rate Spike               │     │     │
│  │  │ - Type: ErrorRateAbove                    │     │     │
│  │  │ - Threshold: 5%                           │     │     │
│  │  │ - Window: 1 minute                        │     │     │
│  │  │ - Cooldown: 5 minutes                     │     │     │
│  │  │ - Last triggered: Never                   │     │     │
│  │  └───────────────────────────────────────────┘     │     │
│  └────────────────────────────────────────────────────┘     │
│                          │                                  │
│  ┌───────────────────────▼────────────────────────────┐     │
│  │  Condition Evaluator                               │     │
│  │                                                     │     │
│  │  For each trigger:                                 │     │
│  │  1. Check cooldown period                          │     │
│  │  2. Evaluate condition against metrics             │     │
│  │  3. If triggered → emit TriggerEvent               │     │
│  └───────────────────────┬────────────────────────────┘     │
│                          │                                  │
│  ┌───────────────────────▼────────────────────────────┐     │
│  │  Trigger Events (Queue)                            │     │
│  │                                                     │     │
│  │  - TriggerEvent { trigger_id, timestamp, metrics } │     │
│  │  - TriggerEvent { trigger_id, timestamp, metrics } │     │
│  └───────────────────────┬────────────────────────────┘     │
└────────────────────────┬─┘                                  │
                         │                                    │
         ┌───────────────▼────────────────┐                   │
         │  Recommendation Engine         │                   │
         │  (Generates optimizations)     │                   │
         └────────────────────────────────┘                   │
```

---

## 5. Security Integration Architecture

### 5.1 Secure Inference Pipeline

```
┌────────────────────────────────────────────────────────────────┐
│                 Secure Inference Request Flow                   │
│                                                                 │
│  Client Request                                                 │
│  ┌────────────────────────────────┐                             │
│  │ POST /v1/completions           │                             │
│  │ { "prompt": "...",             │                             │
│  │   "max_tokens": 100 }          │                             │
│  └───────────────┬────────────────┘                             │
│                  │                                              │
│  ┌───────────────▼────────────────────────────────────────┐     │
│  │  LLM-Shield Security Layer                            │     │
│  │                                                        │     │
│  │  ┌──────────────────────────────────────────────┐     │     │
│  │  │  Stage 1: Input Validation                   │     │     │
│  │  │  Start: T0                                   │     │     │
│  │  │                                              │     │     │
│  │  │  Checks:                                     │     │     │
│  │  │  ✓ Prompt Injection Detection               │     │     │
│  │  │  ✓ PII Scanner                              │     │     │
│  │  │  ✓ Toxic Content Filter                     │     │     │
│  │  │  ✓ Topic Allowlist                          │     │     │
│  │  │                                              │     │     │
│  │  │  Result: PASS (validation_time = T1 - T0)   │     │     │
│  │  └──────────────┬───────────────────────────────┘     │     │
│  │                 │                                     │     │
│  │  ┌──────────────▼───────────────────────────────┐     │     │
│  │  │  Stage 2: Threat Assessment                  │     │     │
│  │  │  Start: T1                                   │     │     │
│  │  │                                              │     │     │
│  │  │  Risk Scoring:                               │     │     │
│  │  │  - Pattern matching: 0.1                     │     │     │
│  │  │  - ML classifier: 0.2                        │     │     │
│  │  │  - Context analysis: 0.15                    │     │     │
│  │  │                                              │     │     │
│  │  │  Aggregate Risk: 0.45 (MEDIUM)              │     │     │
│  │  │                                              │     │     │
│  │  │  Result: ALLOW (threat_time = T2 - T1)      │     │     │
│  │  └──────────────┬───────────────────────────────┘     │     │
│  └─────────────────┼─────────────────────────────────────┘     │
│                    │                                           │
│  ┌─────────────────▼─────────────────────────────────────┐     │
│  │  LLM-Latency-Lens Monitoring                         │     │
│  │  (Begins inference tracking)                         │     │
│  │  Inference Start: T2                                 │     │
│  └─────────────────┬─────────────────────────────────────┘     │
│                    │                                           │
│  ┌─────────────────▼─────────────────────────────────────┐     │
│  │  Inference Engine (vLLM/TGI)                         │     │
│  │                                                       │     │
│  │  Prefill → First Token (T3) → Decode → Complete (T4) │     │
│  │                                                       │     │
│  │  Inference metrics:                                  │     │
│  │  - TTFT: T3 - T2                                     │     │
│  │  - TPOT: avg(Ti+1 - Ti) for decode phase            │     │
│  │  - Total: T4 - T2                                    │     │
│  └─────────────────┬─────────────────────────────────────┘     │
│                    │                                           │
│  ┌─────────────────▼─────────────────────────────────────┐     │
│  │  LLM-Shield Security Layer                           │     │
│  │                                                       │     │
│  │  ┌──────────────────────────────────────────────┐    │     │
│  │  │  Stage 3: Output Filtering                   │    │     │
│  │  │  Start: T4                                   │    │     │
│  │  │                                              │    │     │
│  │  │  Checks:                                     │    │     │
│  │  │  ✓ PII Redaction                            │    │     │
│  │  │  ✓ Harmful Content Filter                   │    │     │
│  │  │  ✓ Compliance Check                         │    │     │
│  │  │                                              │    │     │
│  │  │  Result: PASS (filter_time = T5 - T4)       │    │     │
│  │  └──────────────┬───────────────────────────────┘    │     │
│  └─────────────────┼───────────────────────────────────┘      │
│                    │                                           │
│  ┌─────────────────▼─────────────────────────────────────┐     │
│  │  LLM-Latency-Lens                                    │     │
│  │  (Complete measurement with security overhead)       │     │
│  │                                                       │     │
│  │  SecureInferenceMeasurement {                        │     │
│  │    inference: {                                      │     │
│  │      ttft: T3 - T2,                                  │     │
│  │      tpot: ...,                                      │     │
│  │      total: T4 - T2                                  │     │
│  │    },                                                │     │
│  │    security_overhead: {                              │     │
│  │      validation: T1 - T0,                            │     │
│  │      threat_detection: T2 - T1,                      │     │
│  │      output_filtering: T5 - T4,                      │     │
│  │      total: (T1-T0) + (T2-T1) + (T5-T4)             │     │
│  │    },                                                │     │
│  │    total_time: T5 - T0,                              │     │
│  │    security_impact_percent: overhead/total * 100    │     │
│  │  }                                                   │     │
│  └───────────────────────────────────────────────────────┘     │
│                    │                                           │
│                    ▼                                           │
│  ┌──────────────────────────────────────────────────────┐      │
│  │  Response to Client                                  │      │
│  │  { "choices": [...], "usage": {...} }               │      │
│  │  X-Security-Validated: true                          │      │
│  │  X-Latency-Monitored: true                           │      │
│  └──────────────────────────────────────────────────────┘      │
└────────────────────────────────────────────────────────────────┘
```

---

## 6. Edge Deployment Architecture

### 6.1 Edge-Cloud Collaborative Inference

```
┌─────────────────────────────────────────────────────────────────┐
│                        Edge Device Layer                         │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Edge LLM Agent                                          │   │
│  │                                                           │   │
│  │  ┌────────────────────────────────────────────────┐      │   │
│  │  │  Small Language Model (SLM)                    │      │   │
│  │  │  - Quantized (INT4)                            │      │   │
│  │  │  - 1-7B parameters                             │      │   │
│  │  │  - Optimized for edge inference                │      │   │
│  │  └────────────────────────────────────────────────┘      │   │
│  │                                                           │   │
│  │  ┌────────────────────────────────────────────────┐      │   │
│  │  │  Edge Latency Monitor                          │      │   │
│  │  │  - Local measurement collection                │      │   │
│  │  │  - Lightweight storage (SQLite)                │      │   │
│  │  │  - Periodic sync to cloud                      │      │   │
│  │  └────────────────────────────────────────────────┘      │   │
│  │                                                           │   │
│  │  ┌────────────────────────────────────────────────┐      │   │
│  │  │  Offloading Decision Engine                    │      │   │
│  │  │                                                 │      │   │
│  │  │  if complexity > threshold:                    │      │   │
│  │  │      offload_to_cloud()                        │      │   │
│  │  │  else:                                          │      │   │
│  │  │      process_locally()                         │      │   │
│  │  └────────────────────────────────────────────────┘      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                               │                                 │
│                               │ Conditional Offload             │
│                               │ (Network latency aware)         │
└───────────────────────────────┼─────────────────────────────────┘
                                │
        ════════════════════════╪════════════════════════
                            Internet
        ════════════════════════╪════════════════════════
                                │
┌───────────────────────────────▼─────────────────────────────────┐
│                    Edge-Cloud Gateway Layer                      │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Load Balancer / API Gateway                             │   │
│  │  - Network latency measurement                           │   │
│  │  - Request routing (edge vs cloud)                       │   │
│  │  - Authentication & rate limiting                        │   │
│  └──────────────────┬───────────────────────────────────────┘   │
└─────────────────────┼───────────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────────┐
│                      Cloud Inference Layer                       │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Large Language Model (LLM) Cluster                      │   │
│  │  - Full-precision or FP16/BF16                           │   │
│  │  - 70B-405B parameters                                   │   │
│  │  - GPU-accelerated                                       │   │
│  │  - Distributed inference (tensor parallelism)            │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  LLM-Latency-Lens Central                               │   │
│  │  - Aggregates metrics from edge + cloud                 │   │
│  │  - Unified observability                                │   │
│  │  - Cross-location analytics                             │   │
│  └──────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘


┌──────────────────────────────────────────────────────────────────┐
│        Speculative Decoding Collaboration Pattern                │
│                                                                   │
│  Edge (Small Model)                Cloud (Large Model)           │
│  ┌──────────────────┐             ┌──────────────────┐           │
│  │ Generate k=4     │             │ Verify tokens    │           │
│  │ speculative      │─────────────▶│ in parallel      │           │
│  │ tokens           │   Network   │                  │           │
│  │                  │◀─────────────│ Accept n tokens  │           │
│  │ Fast (low        │             │ Continue from    │           │
│  │ quality)         │             │ rejection point  │           │
│  └──────────────────┘             └──────────────────┘           │
│                                                                   │
│  Metrics:                                                         │
│  - Edge generation time                                          │
│  - Network round-trip time                                       │
│  - Cloud verification time                                       │
│  - Acceptance rate (n/k)                                         │
│  - Effective speedup vs cloud-only                               │
└──────────────────────────────────────────────────────────────────┘
```

---

## 7. Data Serialization Comparison

### 7.1 Format Selection Decision Tree

```
┌──────────────────────────────────────────────┐
│  Select Serialization Format                 │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
         ┌─────────────────────┐
         │ Human-readable      │
         │ required?           │
         └─────┬─────────┬─────┘
               │         │
           Yes │         │ No
               │         │
               ▼         ▼
       ┌────────────┐   ┌────────────────────┐
       │   JSON     │   │ High throughput    │
       │            │   │ required?          │
       │ Use Cases: │   └─────┬────────┬─────┘
       │ - APIs     │         │        │
       │ - Config   │     Yes │        │ No
       │ - Debug    │         │        │
       └────────────┘         ▼        ▼
                    ┌──────────────┐  ┌──────────────┐
                    │ Columnar     │  │ Schema       │
                    │ data?        │  │ evolution    │
                    └───┬──────┬───┘  │ needed?      │
                        │      │      └───┬──────┬───┘
                    Yes │      │ No       │      │
                        │      │      Yes │      │ No
                        ▼      ▼          ▼      ▼
                  ┌─────────┐ ┌────┐ ┌─────┐ ┌──────┐
                  │ Arrow   │ │gRPC│ │Proto│ │MsgPk │
                  │         │ │+   │ │buf  │ │      │
                  │ Use:    │ │Proto│ │     │ │Use:  │
                  │ -Bulk   │ │buf │ │Use: │ │-Fast │
                  │  export │ │    │ │-OTLP│ │ srlz │
                  │ -Analyt.│ │Use:│ │-API │ │-IoT  │
                  └─────────┘ │-RPC│ └─────┘ └──────┘
                              │-Tel│
                              │-Str│
                              └────┘

Performance Characteristics:

Format      | Serialize | Deserialize | Size  | Schema | Human
            | Speed     | Speed       | (KB)  | Evol.  | Read
─────────────────────────────────────────────────────────────────
JSON        | Fast      | Medium      | 100   | No     | Yes
Protobuf    | V. Fast   | V. Fast     | 25    | Yes    | No
Arrow       | V. Fast   | Instant*    | 30    | Yes    | No
MessagePack | V. Fast   | Fast        | 60    | No     | No

* Zero-copy deserialization for compatible types
```

---

## 8. Deployment Architecture

### 8.1 Kubernetes Deployment

```
┌─────────────────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                            │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Namespace: llm-platform                                 │   │
│  │                                                           │   │
│  │  ┌────────────────────────────────────────────────┐      │   │
│  │  │  Deployment: llm-inference                     │      │   │
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐     │      │   │
│  │  │  │  vLLM    │  │  vLLM    │  │  vLLM    │     │      │   │
│  │  │  │  Pod 1   │  │  Pod 2   │  │  Pod 3   │     │      │   │
│  │  │  │          │  │          │  │          │     │      │   │
│  │  │  │ GPU: A100│  │ GPU: A100│  │ GPU: A100│     │      │   │
│  │  │  └────┬─────┘  └────┬─────┘  └────┬─────┘     │      │   │
│  │  │       │             │             │            │      │   │
│  │  │       └─────────────┼─────────────┘            │      │   │
│  │  │                     │                          │      │   │
│  │  └─────────────────────┼──────────────────────────┘      │   │
│  │                        │                                 │   │
│  │                        │ /metrics endpoint              │   │
│  │                        │                                 │   │
│  │  ┌─────────────────────▼──────────────────────────┐      │   │
│  │  │  DaemonSet: llm-latency-lens-agent             │      │   │
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐     │      │   │
│  │  │  │ Monitor  │  │ Monitor  │  │ Monitor  │     │      │   │
│  │  │  │ Node 1   │  │ Node 2   │  │ Node 3   │     │      │   │
│  │  │  │          │  │          │  │          │     │      │   │
│  │  │  │ Collects │  │ Collects │  │ Collects │     │      │   │
│  │  │  │ local    │  │ local    │  │ local    │     │      │   │
│  │  │  │ metrics  │  │ metrics  │  │ metrics  │     │      │   │
│  │  │  └────┬─────┘  └────┬─────┘  └────┬─────┘     │      │   │
│  │  │       │             │             │            │      │   │
│  │  │       └─────────────┼─────────────┘            │      │   │
│  │  │                     │                          │      │   │
│  │  └─────────────────────┼──────────────────────────┘      │   │
│  │                        │                                 │   │
│  │  ┌─────────────────────▼──────────────────────────┐      │   │
│  │  │  Service: llm-latency-lens-aggregator          │      │   │
│  │  │  ┌──────────────────────────────────────┐      │      │   │
│  │  │  │  Aggregator Pod                      │      │      │   │
│  │  │  │  - Receives metrics from agents      │      │      │   │
│  │  │  │  - Statistical processing            │      │      │   │
│  │  │  │  - Export to multiple sinks          │      │      │   │
│  │  │  └──────────────┬───────────────────────┘      │      │   │
│  │  └─────────────────┼──────────────────────────────┘      │   │
│  │                    │                                     │   │
│  └────────────────────┼─────────────────────────────────────┘   │
│                       │                                         │
│  ┌────────────────────┼─────────────────────────────────────┐   │
│  │  Namespace: observability                               │   │
│  │                    │                                     │   │
│  │       ┌────────────┼────────────┐                        │   │
│  │       │            │            │                        │   │
│  │       ▼            ▼            ▼                        │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐                  │   │
│  │  │Prometh- │  │ Jaeger  │  │ OTLP    │                  │   │
│  │  │ eus     │  │         │  │Collector│                  │   │
│  │  │         │  │         │  │         │                  │   │
│  │  │ Metrics │  │ Traces  │  │ Metrics │                  │   │
│  │  │ Store   │  │ Store   │  │ & Traces│                  │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘                  │   │
│  │       │            │            │                        │   │
│  │       └────────────┼────────────┘                        │   │
│  │                    │                                     │   │
│  │              ┌─────▼─────┐                               │   │
│  │              │  Grafana  │                               │   │
│  │              │           │                               │   │
│  │              │Dashboards │                               │   │
│  │              └───────────┘                               │   │
│  └──────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

---

This document provides comprehensive architectural diagrams for implementing LLM-Latency-Lens within the LLM DevOps ecosystem. The diagrams illustrate:

1. System-level component interactions
2. Data flow through collection, processing, and export pipelines
3. Benchmark integration patterns
4. Optimization feedback loops with SAFLA architecture
5. Security integration with guardrails
6. Edge-cloud collaborative inference
7. Serialization format decision trees
8. Kubernetes deployment topology

These diagrams should guide the Rust implementation by clarifying component boundaries, data flows, and integration points.
