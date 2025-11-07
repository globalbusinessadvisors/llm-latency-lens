# LLM-Latency-Lens Ecosystem Integration Specifications

## Executive Summary

This document provides comprehensive integration specifications for LLM-Latency-Lens within the LLM DevOps ecosystem, detailing data schemas, API patterns, and architectural designs optimized for Rust implementation. The specifications cover integration with benchmarking systems, observability platforms, optimization feedback loops, and other LLM DevOps modules.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [LLM-Test-Bench Integration](#llm-test-bench-integration)
3. [LLM-Observatory Telemetry Integration](#llm-observatory-telemetry-integration)
4. [LLM-Auto-Optimizer Feedback Loops](#llm-auto-optimizer-feedback-loops)
5. [Security and Edge Integration](#security-and-edge-integration)
6. [Data Serialization Formats](#data-serialization-formats)
7. [Rust Implementation Guide](#rust-implementation-guide)
8. [Reference Implementations](#reference-implementations)

---

## 1. Architecture Overview

### 1.1 System Context

```
┌─────────────────────────────────────────────────────────────────┐
│                    LLM DevOps Control Plane                      │
│                      (Kubernetes/Orchestrator)                   │
└────────────┬────────────────────────────────────┬────────────────┘
             │                                    │
             ▼                                    ▼
┌────────────────────────┐           ┌────────────────────────────┐
│   LLM-Test-Bench       │◄──────────┤  LLM-Latency-Lens          │
│   (Benchmarking)       │  Results  │  (Performance Monitor)     │
└────────────────────────┘           └────────────────────────────┘
             │                                    │
             │                                    ▼
             │                       ┌────────────────────────────┐
             │                       │  LLM-Observatory           │
             │                       │  (Telemetry System)        │
             │                       └────────────────────────────┘
             │                                    │
             │                                    │
             ▼                                    ▼
┌────────────────────────┐           ┌────────────────────────────┐
│  LLM-Auto-Optimizer    │◄──────────┤  Time-Series Database      │
│  (Feedback Control)    │  Metrics  │  (Prometheus/Grafana)      │
└────────────────────────┘           └────────────────────────────┘
             │
             │
┌────────────┴────────────┐           ┌────────────────────────────┐
│   LLM-Shield            │           │  LLM-Edge-Agent            │
│   (Security Layer)      │◄──────────┤  (Edge Deployment)         │
└─────────────────────────┘           └────────────────────────────┘
```

### 1.2 Data Flow Architecture

```
┌─────────────────┐
│  LLM Inference  │
│     Engine      │
│   (vLLM, etc)   │
└────────┬────────┘
         │ Raw latency metrics
         ▼
┌─────────────────────────────────┐
│   LLM-Latency-Lens              │
│   ┌─────────────────────────┐   │
│   │  Metrics Collector      │   │
│   │  (Async Tokio Streams)  │   │
│   └──────────┬──────────────┘   │
│              ▼                   │
│   ┌─────────────────────────┐   │
│   │  Data Processor         │   │
│   │  (Statistical Analysis) │   │
│   └──────────┬──────────────┘   │
│              ▼                   │
│   ┌─────────────────────────┐   │
│   │  Multi-Format Exporter  │   │
│   │  - OpenTelemetry/OTLP  │   │
│   │  - Prometheus Metrics  │   │
│   │  - JSON/Arrow IPC      │   │
│   └──────────┬──────────────┘   │
└──────────────┼──────────────────┘
               │
       ┌───────┼───────┐
       │       │       │
       ▼       ▼       ▼
   [OTLP]  [Prom]  [Bench]
```

---

## 2. LLM-Test-Bench Integration

### 2.1 Overview

Integration with benchmarking systems enables LLM-Latency-Lens to:
- Consume benchmark test configurations
- Export detailed latency measurements
- Provide reproducible performance baselines
- Support comparative analysis across models

### 2.2 Benchmark Results Schema

#### JSON Schema for Benchmark Results

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "LLM Performance Benchmark Results",
  "type": "object",
  "required": ["metadata", "configuration", "results"],
  "properties": {
    "metadata": {
      "type": "object",
      "required": ["benchmark_id", "timestamp", "version"],
      "properties": {
        "benchmark_id": {
          "type": "string",
          "format": "uuid",
          "description": "Unique identifier for this benchmark run"
        },
        "timestamp": {
          "type": "string",
          "format": "date-time",
          "description": "ISO 8601 timestamp of benchmark execution"
        },
        "version": {
          "type": "string",
          "pattern": "^\\d+\\.\\d+\\.\\d+$",
          "description": "Semantic version of benchmark schema"
        },
        "tool": {
          "type": "string",
          "default": "llm-latency-lens"
        }
      }
    },
    "configuration": {
      "type": "object",
      "required": ["model", "hardware", "test_parameters"],
      "properties": {
        "model": {
          "type": "object",
          "properties": {
            "name": {
              "type": "string",
              "description": "Model identifier (e.g., 'llama-3.1-70b')"
            },
            "provider": {
              "type": "string",
              "enum": ["openai", "anthropic", "together", "vllm", "local"]
            },
            "version": {
              "type": "string"
            },
            "quantization": {
              "type": "string",
              "enum": ["none", "int8", "int4", "fp16", "bf16"]
            }
          }
        },
        "hardware": {
          "type": "object",
          "properties": {
            "gpu_type": {
              "type": "string",
              "description": "GPU model (e.g., 'A100-80GB', 'H100')"
            },
            "gpu_count": {
              "type": "integer",
              "minimum": 1
            },
            "cpu_cores": {
              "type": "integer"
            },
            "memory_gb": {
              "type": "number"
            },
            "batch_size": {
              "type": "integer",
              "minimum": 1
            }
          }
        },
        "test_parameters": {
          "type": "object",
          "properties": {
            "input_tokens": {
              "type": "object",
              "properties": {
                "mean": {
                  "type": "integer"
                },
                "stddev": {
                  "type": "integer"
                },
                "distribution": {
                  "type": "string",
                  "enum": ["normal", "uniform", "fixed"]
                }
              }
            },
            "output_tokens": {
              "type": "object",
              "properties": {
                "mean": {
                  "type": "integer"
                },
                "stddev": {
                  "type": "integer"
                },
                "distribution": {
                  "type": "string",
                  "enum": ["normal", "uniform", "fixed"]
                }
              }
            },
            "concurrency_levels": {
              "type": "array",
              "items": {
                "type": "integer"
              },
              "description": "Concurrency levels tested (e.g., [1, 2, 4, 8, 16])"
            },
            "warmup_requests": {
              "type": "integer",
              "minimum": 0
            },
            "total_requests": {
              "type": "integer",
              "minimum": 1
            }
          }
        }
      }
    },
    "results": {
      "type": "object",
      "required": ["latency_metrics", "throughput_metrics"],
      "properties": {
        "latency_metrics": {
          "type": "object",
          "properties": {
            "time_to_first_token": {
              "$ref": "#/definitions/latency_distribution"
            },
            "time_per_output_token": {
              "$ref": "#/definitions/latency_distribution"
            },
            "end_to_end_latency": {
              "$ref": "#/definitions/latency_distribution"
            },
            "prefill_time": {
              "$ref": "#/definitions/latency_distribution"
            },
            "decode_time": {
              "$ref": "#/definitions/latency_distribution"
            }
          }
        },
        "throughput_metrics": {
          "type": "object",
          "properties": {
            "tokens_per_second": {
              "type": "number",
              "description": "Average tokens generated per second"
            },
            "requests_per_second": {
              "type": "number",
              "description": "Average requests processed per second"
            },
            "tokens_per_second_per_gpu": {
              "type": "number"
            }
          }
        },
        "resource_utilization": {
          "type": "object",
          "properties": {
            "gpu_utilization_percent": {
              "$ref": "#/definitions/metric_distribution"
            },
            "memory_utilization_percent": {
              "$ref": "#/definitions/metric_distribution"
            },
            "kv_cache_utilization_percent": {
              "$ref": "#/definitions/metric_distribution"
            }
          }
        },
        "error_metrics": {
          "type": "object",
          "properties": {
            "total_errors": {
              "type": "integer"
            },
            "error_rate": {
              "type": "number",
              "minimum": 0,
              "maximum": 1
            },
            "errors_by_type": {
              "type": "object",
              "additionalProperties": {
                "type": "integer"
              }
            }
          }
        }
      }
    }
  },
  "definitions": {
    "latency_distribution": {
      "type": "object",
      "required": ["mean_ms", "median_ms", "p50_ms", "p95_ms", "p99_ms"],
      "properties": {
        "mean_ms": {
          "type": "number",
          "description": "Mean latency in milliseconds"
        },
        "median_ms": {
          "type": "number"
        },
        "p50_ms": {
          "type": "number"
        },
        "p90_ms": {
          "type": "number"
        },
        "p95_ms": {
          "type": "number"
        },
        "p99_ms": {
          "type": "number"
        },
        "p999_ms": {
          "type": "number"
        },
        "min_ms": {
          "type": "number"
        },
        "max_ms": {
          "type": "number"
        },
        "stddev_ms": {
          "type": "number"
        },
        "sample_count": {
          "type": "integer"
        }
      }
    },
    "metric_distribution": {
      "type": "object",
      "properties": {
        "mean": {
          "type": "number"
        },
        "median": {
          "type": "number"
        },
        "p95": {
          "type": "number"
        },
        "p99": {
          "type": "number"
        },
        "min": {
          "type": "number"
        },
        "max": {
          "type": "number"
        }
      }
    }
  }
}
```

### 2.3 Benchmark Configuration Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "LLM Benchmark Test Configuration",
  "type": "object",
  "required": ["test_suite", "models", "workloads"],
  "properties": {
    "test_suite": {
      "type": "object",
      "properties": {
        "name": {
          "type": "string"
        },
        "description": {
          "type": "string"
        },
        "version": {
          "type": "string"
        }
      }
    },
    "models": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "endpoint"],
        "properties": {
          "name": {
            "type": "string"
          },
          "endpoint": {
            "type": "string",
            "format": "uri"
          },
          "api_type": {
            "type": "string",
            "enum": ["openai", "vllm", "tgi", "custom"]
          },
          "auth": {
            "type": "object",
            "properties": {
              "type": {
                "type": "string",
                "enum": ["bearer", "api_key", "none"]
              },
              "credentials_path": {
                "type": "string"
              }
            }
          }
        }
      }
    },
    "workloads": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": {
            "type": "string"
          },
          "input_dataset": {
            "type": "string",
            "description": "Path or reference to input dataset"
          },
          "input_token_range": {
            "type": "object",
            "properties": {
              "min": {
                "type": "integer"
              },
              "max": {
                "type": "integer"
              }
            }
          },
          "output_token_target": {
            "type": "integer"
          },
          "concurrency": {
            "type": "integer"
          },
          "duration_seconds": {
            "type": "integer"
          }
        }
      }
    }
  }
}
```

### 2.4 API Integration Patterns

#### REST API Endpoints

```rust
// Rust API endpoint definitions for benchmark integration

use axum::{Router, Json, extract::Path};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// POST /api/v1/benchmark/start
/// Start a new benchmark run with configuration
#[derive(Deserialize)]
pub struct StartBenchmarkRequest {
    pub configuration: BenchmarkConfiguration,
    pub callback_url: Option<String>,
}

#[derive(Serialize)]
pub struct StartBenchmarkResponse {
    pub benchmark_id: Uuid,
    pub status: String,
    pub estimated_duration_seconds: u64,
}

/// GET /api/v1/benchmark/{id}/status
/// Get current status of benchmark run
#[derive(Serialize)]
pub struct BenchmarkStatus {
    pub benchmark_id: Uuid,
    pub status: BenchmarkState,
    pub progress_percent: f32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub enum BenchmarkState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// GET /api/v1/benchmark/{id}/results
/// Retrieve benchmark results in specified format
#[derive(Serialize)]
pub struct BenchmarkResults {
    pub benchmark_id: Uuid,
    pub results: serde_json::Value, // Full schema-compliant JSON
    pub format_version: String,
}

/// GET /api/v1/benchmark/{id}/export?format={json|arrow|csv}
/// Export results in various formats
pub async fn export_benchmark_results(
    Path(id): Path<Uuid>,
    format: ExportFormat,
) -> Result<Vec<u8>, ApiError> {
    // Implementation
    todo!()
}

#[derive(Deserialize)]
pub enum ExportFormat {
    Json,
    Arrow,
    Csv,
    Parquet,
}
```

### 2.5 LLMPerf-Style Integration

Based on LLMPerf (Ray Project) patterns:

```rust
use tokio::time::{Duration, Instant};
use std::collections::HashMap;

pub struct LLMPerfCompatibleCollector {
    results_dir: PathBuf,
    model: String,
    mean_input_tokens: usize,
    stddev_input_tokens: usize,
    mean_output_tokens: usize,
    stddev_output_tokens: usize,
}

impl LLMPerfCompatibleCollector {
    pub async fn run_benchmark(&self) -> BenchmarkResults {
        let mut latencies = Vec::new();
        let mut ttft_measurements = Vec::new();
        let mut token_throughputs = Vec::new();

        // Generate requests with specified token distributions
        let requests = self.generate_requests();

        for request in requests {
            let start = Instant::now();
            let mut first_token_received = false;
            let mut first_token_time = None;

            // Stream response and measure
            let mut stream = self.send_request(request).await?;

            while let Some(token) = stream.next().await {
                if !first_token_received {
                    first_token_time = Some(start.elapsed());
                    first_token_received = true;
                }
            }

            let total_latency = start.elapsed();

            latencies.push(total_latency);
            if let Some(ttft) = first_token_time {
                ttft_measurements.push(ttft);
            }
        }

        // Calculate statistics and format results
        self.format_results(latencies, ttft_measurements, token_throughputs)
    }

    fn format_results(
        &self,
        latencies: Vec<Duration>,
        ttft: Vec<Duration>,
        throughput: Vec<f64>,
    ) -> BenchmarkResults {
        // Convert to LLMPerf-compatible output format
        todo!()
    }
}
```

---

## 3. LLM-Observatory Telemetry Integration

### 3.1 Overview

Integration with observability platforms using industry-standard telemetry protocols:
- OpenTelemetry (OTLP) for traces and metrics
- Prometheus for time-series metrics
- Grafana for visualization
- Support for distributed tracing

### 3.2 OpenTelemetry Integration

#### 3.2.1 Semantic Conventions

Following OpenTelemetry GenAI semantic conventions (2025 standard):

```rust
use opentelemetry::{
    metrics::{Counter, Histogram, Meter},
    KeyValue,
};

pub struct OtelMetrics {
    // Token usage metrics
    gen_ai_token_usage: Counter<u64>,

    // Latency metrics
    gen_ai_time_to_first_token: Histogram<f64>,
    gen_ai_time_per_output_token: Histogram<f64>,

    // Request metrics
    gen_ai_request_duration: Histogram<f64>,
    gen_ai_request_errors: Counter<u64>,
}

impl OtelMetrics {
    pub fn new(meter: Meter) -> Self {
        Self {
            gen_ai_token_usage: meter
                .u64_counter("gen_ai.token.usage")
                .with_description("Number of tokens used in operation")
                .with_unit("tokens")
                .build(),

            gen_ai_time_to_first_token: meter
                .f64_histogram("gen_ai.server.time_to_first_token")
                .with_description("Time to generate first token")
                .with_unit("s")
                .build(),

            gen_ai_time_per_output_token: meter
                .f64_histogram("gen_ai.server.time_per_output_token")
                .with_description("Time per output token after first")
                .with_unit("s")
                .build(),

            gen_ai_request_duration: meter
                .f64_histogram("gen_ai.client.operation.duration")
                .with_description("Full request duration")
                .with_unit("s")
                .build(),

            gen_ai_request_errors: meter
                .u64_counter("gen_ai.client.operation.errors")
                .with_description("Number of errors")
                .with_unit("errors")
                .build(),
        }
    }

    pub fn record_inference(&self, measurement: &InferenceMeasurement) {
        let attributes = vec![
            KeyValue::new("gen_ai.operation.name", "completion"),
            KeyValue::new("gen_ai.system", measurement.provider.clone()),
            KeyValue::new("gen_ai.request.model", measurement.model.clone()),
            KeyValue::new("server.address", measurement.endpoint.clone()),
        ];

        // Record token usage
        self.gen_ai_token_usage.add(
            measurement.input_tokens as u64,
            &[
                attributes.clone(),
                vec![KeyValue::new("gen_ai.token.type", "input")],
            ].concat(),
        );

        self.gen_ai_token_usage.add(
            measurement.output_tokens as u64,
            &[
                attributes.clone(),
                vec![KeyValue::new("gen_ai.token.type", "output")],
            ].concat(),
        );

        // Record latency metrics
        self.gen_ai_time_to_first_token.record(
            measurement.ttft.as_secs_f64(),
            &attributes,
        );

        self.gen_ai_time_per_output_token.record(
            measurement.tpot.as_secs_f64(),
            &attributes,
        );

        self.gen_ai_request_duration.record(
            measurement.total_duration.as_secs_f64(),
            &attributes,
        );
    }
}
```

#### 3.2.2 Span Structure for Distributed Tracing

```rust
use opentelemetry::trace::{Span, Tracer, SpanKind, Status};
use opentelemetry::Context;

pub async fn trace_llm_request<T: Tracer>(
    tracer: &T,
    model: &str,
    input_tokens: usize,
) -> Result<InferenceMeasurement, Error> {
    let mut span = tracer
        .span_builder("llm.inference")
        .with_kind(SpanKind::Client)
        .with_attributes(vec![
            KeyValue::new("gen_ai.system", "vllm"),
            KeyValue::new("gen_ai.request.model", model.to_string()),
            KeyValue::new("gen_ai.operation.name", "completion"),
            KeyValue::new("gen_ai.request.max_tokens", input_tokens as i64),
        ])
        .start(tracer);

    let cx = Context::current_with_span(span.clone());

    // Execute the actual LLM request
    let result = match execute_llm_request(&cx, model, input_tokens).await {
        Ok(measurement) => {
            // Add result attributes
            span.set_attribute(KeyValue::new(
                "gen_ai.response.finish_reasons",
                vec!["stop"],
            ));
            span.set_attribute(KeyValue::new(
                "gen_ai.usage.input_tokens",
                measurement.input_tokens as i64,
            ));
            span.set_attribute(KeyValue::new(
                "gen_ai.usage.output_tokens",
                measurement.output_tokens as i64,
            ));

            span.set_status(Status::Ok);
            Ok(measurement)
        }
        Err(e) => {
            span.set_status(Status::error(e.to_string()));
            span.record_error(&e);
            Err(e)
        }
    };

    span.end();
    result
}
```

#### 3.2.3 OTLP Export Configuration

```rust
use opentelemetry_otlp::{
    WithExportConfig, WithTonicConfig,
    new_exporter, new_pipeline,
};
use opentelemetry_sdk::{
    metrics::{PeriodicReader, SdkMeterProvider},
    runtime,
    Resource,
};
use tonic::metadata::MetadataMap;

pub fn init_otlp_exporter(
    endpoint: &str,
    headers: Option<HashMap<String, String>>,
) -> Result<SdkMeterProvider, Box<dyn std::error::Error>> {
    let mut exporter = new_exporter()
        .tonic()
        .with_endpoint(endpoint);

    // Add custom headers (e.g., authentication)
    if let Some(headers) = headers {
        let mut metadata = MetadataMap::new();
        for (key, value) in headers {
            metadata.insert(
                key.parse()?,
                value.parse()?,
            );
        }
        exporter = exporter.with_metadata(metadata);
    }

    let meter_provider = new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(exporter)
        .with_resource(Resource::new(vec![
            KeyValue::new("service.name", "llm-latency-lens"),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ]))
        .with_period(std::time::Duration::from_secs(10))
        .build()?;

    Ok(meter_provider)
}
```

### 3.3 Prometheus Integration

#### 3.3.1 Metrics Exposition Format

```rust
use prometheus::{
    Registry, HistogramOpts, HistogramVec, CounterVec, GaugeVec,
    Opts, Encoder, TextEncoder,
};

pub struct PrometheusMetrics {
    registry: Registry,

    // Latency metrics (histograms)
    ttft_histogram: HistogramVec,
    tpot_histogram: HistogramVec,
    e2e_latency_histogram: HistogramVec,

    // Throughput metrics
    tokens_per_second: GaugeVec,
    requests_per_second: GaugeVec,

    // Request counters
    total_requests: CounterVec,
    failed_requests: CounterVec,

    // Resource utilization
    gpu_utilization: GaugeVec,
    memory_utilization: GaugeVec,
}

impl PrometheusMetrics {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Registry::new();

        // Time to First Token histogram
        let ttft_histogram = HistogramVec::new(
            HistogramOpts::new(
                "llm_time_to_first_token_seconds",
                "Time to generate first token in seconds"
            )
            .namespace("llm_latency_lens")
            .buckets(vec![
                0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0,
                2.5, 5.0, 7.5, 10.0, 25.0, 50.0, 75.0, 100.0
            ]),
            &["model", "provider", "endpoint"]
        )?;

        // Time Per Output Token histogram
        let tpot_histogram = HistogramVec::new(
            HistogramOpts::new(
                "llm_time_per_output_token_seconds",
                "Time per output token after first in seconds"
            )
            .namespace("llm_latency_lens")
            .buckets(vec![
                0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.075, 0.1,
                0.25, 0.5, 0.75, 1.0, 2.5
            ]),
            &["model", "provider", "endpoint"]
        )?;

        // End-to-end latency histogram
        let e2e_latency_histogram = HistogramVec::new(
            HistogramOpts::new(
                "llm_request_duration_seconds",
                "Total request duration in seconds"
            )
            .namespace("llm_latency_lens")
            .buckets(vec![
                0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0
            ]),
            &["model", "provider", "endpoint", "status"]
        )?;

        // Throughput gauges
        let tokens_per_second = GaugeVec::new(
            Opts::new(
                "llm_tokens_per_second",
                "Average tokens generated per second"
            )
            .namespace("llm_latency_lens"),
            &["model", "provider", "token_type"]
        )?;

        let requests_per_second = GaugeVec::new(
            Opts::new(
                "llm_requests_per_second",
                "Average requests processed per second"
            )
            .namespace("llm_latency_lens"),
            &["model", "provider"]
        )?;

        // Request counters
        let total_requests = CounterVec::new(
            Opts::new(
                "llm_requests_total",
                "Total number of LLM requests"
            )
            .namespace("llm_latency_lens"),
            &["model", "provider", "endpoint"]
        )?;

        let failed_requests = CounterVec::new(
            Opts::new(
                "llm_requests_failed_total",
                "Total number of failed LLM requests"
            )
            .namespace("llm_latency_lens"),
            &["model", "provider", "error_type"]
        )?;

        // Resource utilization
        let gpu_utilization = GaugeVec::new(
            Opts::new(
                "llm_gpu_utilization_percent",
                "GPU utilization percentage"
            )
            .namespace("llm_latency_lens"),
            &["gpu_id", "gpu_type"]
        )?;

        let memory_utilization = GaugeVec::new(
            Opts::new(
                "llm_memory_utilization_percent",
                "Memory utilization percentage"
            )
            .namespace("llm_latency_lens"),
            &["memory_type"]
        )?;

        // Register all metrics
        registry.register(Box::new(ttft_histogram.clone()))?;
        registry.register(Box::new(tpot_histogram.clone()))?;
        registry.register(Box::new(e2e_latency_histogram.clone()))?;
        registry.register(Box::new(tokens_per_second.clone()))?;
        registry.register(Box::new(requests_per_second.clone()))?;
        registry.register(Box::new(total_requests.clone()))?;
        registry.register(Box::new(failed_requests.clone()))?;
        registry.register(Box::new(gpu_utilization.clone()))?;
        registry.register(Box::new(memory_utilization.clone()))?;

        Ok(Self {
            registry,
            ttft_histogram,
            tpot_histogram,
            e2e_latency_histogram,
            tokens_per_second,
            requests_per_second,
            total_requests,
            failed_requests,
            gpu_utilization,
            memory_utilization,
        })
    }

    pub fn record_inference(&self, measurement: &InferenceMeasurement) {
        let labels = &[
            measurement.model.as_str(),
            measurement.provider.as_str(),
            measurement.endpoint.as_str(),
        ];

        // Record histograms
        self.ttft_histogram
            .with_label_values(labels)
            .observe(measurement.ttft.as_secs_f64());

        self.tpot_histogram
            .with_label_values(labels)
            .observe(measurement.tpot.as_secs_f64());

        self.e2e_latency_histogram
            .with_label_values(&[
                measurement.model.as_str(),
                measurement.provider.as_str(),
                measurement.endpoint.as_str(),
                "success",
            ])
            .observe(measurement.total_duration.as_secs_f64());

        // Increment request counter
        self.total_requests
            .with_label_values(labels)
            .inc();
    }

    pub fn export_metrics(&self) -> Result<String, Box<dyn std::error::Error>> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}

// Example metrics endpoint handler
pub async fn metrics_handler(
    metrics: Arc<PrometheusMetrics>,
) -> Result<String, StatusCode> {
    metrics.export_metrics()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
```

#### 3.3.2 Example Prometheus Output

```prometheus
# HELP llm_latency_lens_ttft_seconds Time to generate first token in seconds
# TYPE llm_latency_lens_ttft_seconds histogram
llm_latency_lens_time_to_first_token_seconds_bucket{model="llama-3.1-70b",provider="vllm",endpoint="http://localhost:8000",le="0.01"} 0
llm_latency_lens_time_to_first_token_seconds_bucket{model="llama-3.1-70b",provider="vllm",endpoint="http://localhost:8000",le="0.025"} 2
llm_latency_lens_time_to_first_token_seconds_bucket{model="llama-3.1-70b",provider="vllm",endpoint="http://localhost:8000",le="0.05"} 15
llm_latency_lens_time_to_first_token_seconds_bucket{model="llama-3.1-70b",provider="vllm",endpoint="http://localhost:8000",le="0.1"} 45
llm_latency_lens_time_to_first_token_seconds_bucket{model="llama-3.1-70b",provider="vllm",endpoint="http://localhost:8000",le="+Inf"} 100
llm_latency_lens_time_to_first_token_seconds_sum{model="llama-3.1-70b",provider="vllm",endpoint="http://localhost:8000"} 5.234
llm_latency_lens_time_to_first_token_seconds_count{model="llama-3.1-70b",provider="vllm",endpoint="http://localhost:8000"} 100

# HELP llm_latency_lens_tpot_seconds Time per output token after first in seconds
# TYPE llm_latency_lens_tpot_seconds histogram
llm_latency_lens_time_per_output_token_seconds_bucket{model="llama-3.1-70b",provider="vllm",endpoint="http://localhost:8000",le="0.01"} 85
llm_latency_lens_time_per_output_token_seconds_bucket{model="llama-3.1-70b",provider="vllm",endpoint="http://localhost:8000",le="0.025"} 95
llm_latency_lens_time_per_output_token_seconds_bucket{model="llama-3.1-70b",provider="vllm",endpoint="http://localhost:8000",le="+Inf"} 100
llm_latency_lens_time_per_output_token_seconds_sum{model="llama-3.1-70b",provider="vllm",endpoint="http://localhost:8000"} 1.234
llm_latency_lens_time_per_output_token_seconds_count{model="llama-3.1-70b",provider="vllm",endpoint="http://localhost:8000"} 100

# HELP llm_latency_lens_tokens_per_second Average tokens generated per second
# TYPE llm_latency_lens_tokens_per_second gauge
llm_latency_lens_tokens_per_second{model="llama-3.1-70b",provider="vllm",token_type="input"} 2450.5
llm_latency_lens_tokens_per_second{model="llama-3.1-70b",provider="vllm",token_type="output"} 125.3

# HELP llm_latency_lens_requests_total Total number of LLM requests
# TYPE llm_latency_lens_requests_total counter
llm_latency_lens_requests_total{model="llama-3.1-70b",provider="vllm",endpoint="http://localhost:8000"} 1547
```

### 3.4 Grafana Dashboard Configuration

```json
{
  "dashboard": {
    "title": "LLM Latency Lens - Performance Monitoring",
    "uid": "llm-latency-lens",
    "version": 1,
    "panels": [
      {
        "id": 1,
        "title": "Time to First Token (P50, P95, P99)",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(llm_latency_lens_time_to_first_token_seconds_bucket[5m]))",
            "legendFormat": "P50 - {{model}}"
          },
          {
            "expr": "histogram_quantile(0.95, rate(llm_latency_lens_time_to_first_token_seconds_bucket[5m]))",
            "legendFormat": "P95 - {{model}}"
          },
          {
            "expr": "histogram_quantile(0.99, rate(llm_latency_lens_time_to_first_token_seconds_bucket[5m]))",
            "legendFormat": "P99 - {{model}}"
          }
        ]
      },
      {
        "id": 2,
        "title": "Tokens Per Second by Model",
        "type": "graph",
        "targets": [
          {
            "expr": "llm_latency_lens_tokens_per_second{token_type=\"output\"}",
            "legendFormat": "{{model}} - {{provider}}"
          }
        ]
      },
      {
        "id": 3,
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(llm_latency_lens_requests_total[5m])",
            "legendFormat": "{{model}}"
          }
        ]
      },
      {
        "id": 4,
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(llm_latency_lens_requests_failed_total[5m]) / rate(llm_latency_lens_requests_total[5m])",
            "legendFormat": "{{model}} - {{error_type}}"
          }
        ]
      }
    ]
  }
}
```

### 3.5 Real-time Streaming Integration

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub struct RealtimeMetricsStream {
    tx: mpsc::Sender<MetricEvent>,
}

#[derive(Clone, Debug)]
pub struct MetricEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metric_type: MetricType,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub enum MetricType {
    TimeToFirstToken,
    TimePerOutputToken,
    TokensPerSecond,
    RequestLatency,
    ErrorRate,
}

impl RealtimeMetricsStream {
    pub fn new(buffer_size: usize) -> (Self, ReceiverStream<MetricEvent>) {
        let (tx, rx) = mpsc::channel(buffer_size);
        let stream = ReceiverStream::new(rx);
        (Self { tx }, stream)
    }

    pub async fn emit(&self, event: MetricEvent) -> Result<(), SendError<MetricEvent>> {
        self.tx.send(event).await
    }

    pub async fn emit_inference(&self, measurement: &InferenceMeasurement) {
        let base_labels = HashMap::from([
            ("model".to_string(), measurement.model.clone()),
            ("provider".to_string(), measurement.provider.clone()),
        ]);

        // Emit TTFT
        let _ = self.emit(MetricEvent {
            timestamp: chrono::Utc::now(),
            metric_type: MetricType::TimeToFirstToken,
            value: measurement.ttft.as_secs_f64(),
            labels: base_labels.clone(),
        }).await;

        // Emit TPOT
        let _ = self.emit(MetricEvent {
            timestamp: chrono::Utc::now(),
            metric_type: MetricType::TimePerOutputToken,
            value: measurement.tpot.as_secs_f64(),
            labels: base_labels.clone(),
        }).await;

        // Calculate and emit throughput
        let tokens_per_second = measurement.output_tokens as f64
            / measurement.total_duration.as_secs_f64();
        let _ = self.emit(MetricEvent {
            timestamp: chrono::Utc::now(),
            metric_type: MetricType::TokensPerSecond,
            value: tokens_per_second,
            labels: base_labels,
        }).await;
    }
}

// WebSocket endpoint for real-time streaming
pub async fn websocket_metrics_handler(
    ws: WebSocketUpgrade,
    metrics_stream: Arc<RealtimeMetricsStream>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_metrics_websocket(socket, metrics_stream))
}

async fn handle_metrics_websocket(
    mut socket: WebSocket,
    metrics_stream: Arc<RealtimeMetricsStream>,
) {
    let (_, mut stream) = RealtimeMetricsStream::new(1000);

    while let Some(event) = stream.next().await {
        let json = serde_json::to_string(&event).unwrap();
        if socket.send(Message::Text(json)).await.is_err() {
            break;
        }
    }
}
```

---

## 4. LLM-Auto-Optimizer Feedback Loops

### 4.1 Overview

Integration with optimization systems enables automatic performance tuning based on latency measurements:
- Performance threshold triggers
- Configuration recommendation
- A/B testing support
- Continuous optimization loops

### 4.2 Feedback Loop Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Feedback Control Loop                     │
│                                                              │
│  ┌──────────────┐      ┌──────────────┐      ┌───────────┐ │
│  │   Monitor    │─────▶│   Analyze    │─────▶│  Decide   │ │
│  │  (Measure)   │      │  (Evaluate)  │      │ (Optimize)│ │
│  └──────────────┘      └──────────────┘      └───────┬───┘ │
│         ▲                                              │     │
│         │                                              ▼     │
│  ┌──────┴──────┐                            ┌─────────────┐ │
│  │   Verify    │◀───────────────────────────│    Act      │ │
│  │  (Confirm)  │                            │  (Apply)    │ │
│  └─────────────┘                            └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 4.3 Performance Data Format for Optimization

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Performance Feedback Data",
  "type": "object",
  "required": ["timestamp", "current_state", "performance_metrics", "recommendations"],
  "properties": {
    "timestamp": {
      "type": "string",
      "format": "date-time"
    },
    "observation_window": {
      "type": "object",
      "properties": {
        "start_time": {
          "type": "string",
          "format": "date-time"
        },
        "end_time": {
          "type": "string",
          "format": "date-time"
        },
        "sample_count": {
          "type": "integer"
        }
      }
    },
    "current_state": {
      "type": "object",
      "properties": {
        "model_config": {
          "type": "object",
          "properties": {
            "model_name": {
              "type": "string"
            },
            "batch_size": {
              "type": "integer"
            },
            "max_tokens": {
              "type": "integer"
            },
            "temperature": {
              "type": "number"
            },
            "tensor_parallel_size": {
              "type": "integer"
            },
            "gpu_memory_utilization": {
              "type": "number"
            },
            "quantization": {
              "type": "string"
            }
          }
        },
        "deployment_config": {
          "type": "object",
          "properties": {
            "replica_count": {
              "type": "integer"
            },
            "gpu_allocation": {
              "type": "string"
            },
            "node_selector": {
              "type": "object"
            }
          }
        }
      }
    },
    "performance_metrics": {
      "type": "object",
      "properties": {
        "latency_analysis": {
          "type": "object",
          "properties": {
            "ttft_p50_ms": {
              "type": "number"
            },
            "ttft_p95_ms": {
              "type": "number"
            },
            "ttft_p99_ms": {
              "type": "number"
            },
            "tpot_p50_ms": {
              "type": "number"
            },
            "tpot_p95_ms": {
              "type": "number"
            },
            "e2e_p95_ms": {
              "type": "number"
            }
          }
        },
        "throughput_analysis": {
          "type": "object",
          "properties": {
            "requests_per_second": {
              "type": "number"
            },
            "tokens_per_second": {
              "type": "number"
            },
            "utilization_efficiency": {
              "type": "number",
              "description": "Ratio of actual vs theoretical throughput"
            }
          }
        },
        "resource_analysis": {
          "type": "object",
          "properties": {
            "avg_gpu_utilization": {
              "type": "number"
            },
            "avg_memory_utilization": {
              "type": "number"
            },
            "bottleneck_indicator": {
              "type": "string",
              "enum": ["compute", "memory", "network", "none"]
            }
          }
        },
        "sla_compliance": {
          "type": "object",
          "properties": {
            "latency_sla_met": {
              "type": "boolean"
            },
            "throughput_sla_met": {
              "type": "boolean"
            },
            "availability_sla_met": {
              "type": "boolean"
            },
            "violations_count": {
              "type": "integer"
            }
          }
        }
      }
    },
    "anomalies": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "detected_at": {
            "type": "string",
            "format": "date-time"
          },
          "anomaly_type": {
            "type": "string",
            "enum": ["latency_spike", "throughput_drop", "error_burst", "resource_saturation"]
          },
          "severity": {
            "type": "string",
            "enum": ["low", "medium", "high", "critical"]
          },
          "description": {
            "type": "string"
          },
          "affected_metric": {
            "type": "string"
          },
          "deviation_magnitude": {
            "type": "number",
            "description": "Standard deviations from baseline"
          }
        }
      }
    },
    "recommendations": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "recommendation_id": {
            "type": "string",
            "format": "uuid"
          },
          "priority": {
            "type": "integer",
            "minimum": 1,
            "maximum": 10
          },
          "category": {
            "type": "string",
            "enum": [
              "scaling",
              "configuration",
              "resource_allocation",
              "caching",
              "batching",
              "quantization"
            ]
          },
          "action": {
            "type": "object",
            "properties": {
              "type": {
                "type": "string",
                "enum": ["scale_up", "scale_down", "config_change", "optimize"]
              },
              "target": {
                "type": "string",
                "description": "What to change (e.g., 'batch_size', 'replica_count')"
              },
              "current_value": {
                "type": ["string", "number", "boolean"]
              },
              "recommended_value": {
                "type": ["string", "number", "boolean"]
              },
              "rollback_strategy": {
                "type": "string"
              }
            }
          },
          "expected_impact": {
            "type": "object",
            "properties": {
              "latency_improvement_percent": {
                "type": "number"
              },
              "throughput_improvement_percent": {
                "type": "number"
              },
              "cost_impact_percent": {
                "type": "number"
              },
              "confidence_score": {
                "type": "number",
                "minimum": 0,
                "maximum": 1
              }
            }
          },
          "rationale": {
            "type": "string",
            "description": "Explanation for why this recommendation was made"
          }
        }
      }
    }
  }
}
```

### 4.4 Optimization Trigger Conditions

```rust
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct OptimizationTrigger {
    pub trigger_type: TriggerType,
    pub condition: TriggerCondition,
    pub evaluation_window: Duration,
    pub cooldown_period: Duration,
}

#[derive(Debug, Clone)]
pub enum TriggerType {
    LatencyThreshold,
    ThroughputThreshold,
    ErrorRateThreshold,
    ResourceUtilization,
    SLAViolation,
    CostEfficiency,
    Scheduled,
}

#[derive(Debug, Clone)]
pub enum TriggerCondition {
    // Latency conditions
    P95LatencyExceeds { threshold_ms: f64 },
    P99LatencyExceeds { threshold_ms: f64 },
    MeanLatencyExceeds { threshold_ms: f64 },

    // Throughput conditions
    ThroughputBelow { threshold_rps: f64 },
    TokensPerSecondBelow { threshold_tps: f64 },

    // Resource conditions
    GpuUtilizationAbove { threshold_percent: f64 },
    GpuUtilizationBelow { threshold_percent: f64 },
    MemoryUtilizationAbove { threshold_percent: f64 },

    // Error conditions
    ErrorRateAbove { threshold_percent: f64 },
    ConsecutiveFailures { count: usize },

    // SLA conditions
    SLAViolationCount { max_violations: usize, window: Duration },

    // Composite conditions
    And(Vec<TriggerCondition>),
    Or(Vec<TriggerCondition>),
}

pub struct OptimizationEngine {
    triggers: Vec<OptimizationTrigger>,
    last_trigger_time: HashMap<String, chrono::DateTime<chrono::Utc>>,
    performance_history: PerformanceHistory,
}

impl OptimizationEngine {
    pub async fn evaluate_triggers(
        &mut self,
        current_metrics: &PerformanceMetrics,
    ) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        for trigger in &self.triggers {
            // Check cooldown period
            if let Some(last_time) = self.last_trigger_time.get(&trigger.trigger_type.to_string()) {
                if chrono::Utc::now().signed_duration_since(*last_time).to_std().unwrap()
                    < trigger.cooldown_period
                {
                    continue;
                }
            }

            // Evaluate condition
            if self.evaluate_condition(&trigger.condition, current_metrics) {
                let recs = self.generate_recommendations(trigger, current_metrics).await;
                recommendations.extend(recs);

                self.last_trigger_time.insert(
                    trigger.trigger_type.to_string(),
                    chrono::Utc::now(),
                );
            }
        }

        // Rank recommendations by priority and expected impact
        recommendations.sort_by(|a, b| {
            b.expected_impact.confidence_score
                .partial_cmp(&a.expected_impact.confidence_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        recommendations
    }

    fn evaluate_condition(
        &self,
        condition: &TriggerCondition,
        metrics: &PerformanceMetrics,
    ) -> bool {
        match condition {
            TriggerCondition::P95LatencyExceeds { threshold_ms } => {
                metrics.latency_analysis.ttft_p95_ms > *threshold_ms
            }
            TriggerCondition::P99LatencyExceeds { threshold_ms } => {
                metrics.latency_analysis.ttft_p99_ms > *threshold_ms
            }
            TriggerCondition::ThroughputBelow { threshold_rps } => {
                metrics.throughput_analysis.requests_per_second < *threshold_rps
            }
            TriggerCondition::GpuUtilizationAbove { threshold_percent } => {
                metrics.resource_analysis.avg_gpu_utilization > *threshold_percent
            }
            TriggerCondition::ErrorRateAbove { threshold_percent } => {
                let error_rate = metrics.error_metrics.error_rate * 100.0;
                error_rate > *threshold_percent
            }
            TriggerCondition::And(conditions) => {
                conditions.iter().all(|c| self.evaluate_condition(c, metrics))
            }
            TriggerCondition::Or(conditions) => {
                conditions.iter().any(|c| self.evaluate_condition(c, metrics))
            }
            _ => false,
        }
    }

    async fn generate_recommendations(
        &self,
        trigger: &OptimizationTrigger,
        metrics: &PerformanceMetrics,
    ) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        match trigger.trigger_type {
            TriggerType::LatencyThreshold => {
                // Analyze latency bottleneck
                if metrics.resource_analysis.bottleneck_indicator == "compute" {
                    // Recommend scaling or quantization
                    recommendations.push(OptimizationRecommendation {
                        category: RecommendationCategory::Scaling,
                        action: OptimizationAction::ScaleUp {
                            target: "replica_count".to_string(),
                            current_value: 2,
                            recommended_value: 4,
                        },
                        expected_impact: ExpectedImpact {
                            latency_improvement_percent: 35.0,
                            throughput_improvement_percent: 80.0,
                            cost_impact_percent: 100.0,
                            confidence_score: 0.85,
                        },
                        rationale: "High GPU utilization indicates compute bottleneck. \
                                   Increasing replicas will distribute load and reduce latency."
                            .to_string(),
                    });
                } else if metrics.resource_analysis.bottleneck_indicator == "memory" {
                    // Recommend quantization or KV cache optimization
                    recommendations.push(OptimizationRecommendation {
                        category: RecommendationCategory::Quantization,
                        action: OptimizationAction::ConfigChange {
                            target: "quantization".to_string(),
                            current_value: "none".to_string(),
                            recommended_value: "int8".to_string(),
                        },
                        expected_impact: ExpectedImpact {
                            latency_improvement_percent: 15.0,
                            throughput_improvement_percent: 25.0,
                            cost_impact_percent: -20.0, // Cost reduction
                            confidence_score: 0.78,
                        },
                        rationale: "Memory bottleneck detected. INT8 quantization can \
                                   reduce memory usage and improve throughput."
                            .to_string(),
                    });
                }
            }
            TriggerType::ThroughputThreshold => {
                // Check if batch size can be increased
                recommendations.push(OptimizationRecommendation {
                    category: RecommendationCategory::Batching,
                    action: OptimizationAction::ConfigChange {
                        target: "batch_size".to_string(),
                        current_value: "4".to_string(),
                        recommended_value: "8".to_string(),
                    },
                    expected_impact: ExpectedImpact {
                        latency_improvement_percent: -10.0, // Slight increase
                        throughput_improvement_percent: 60.0,
                        cost_impact_percent: 0.0,
                        confidence_score: 0.82,
                    },
                    rationale: "Low throughput with available GPU capacity. \
                               Increasing batch size will improve throughput."
                        .to_string(),
                });
            }
            _ => {}
        }

        recommendations
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub category: RecommendationCategory,
    pub action: OptimizationAction,
    pub expected_impact: ExpectedImpact,
    pub rationale: String,
}

#[derive(Debug, Clone)]
pub enum RecommendationCategory {
    Scaling,
    Configuration,
    ResourceAllocation,
    Caching,
    Batching,
    Quantization,
}

#[derive(Debug, Clone)]
pub enum OptimizationAction {
    ScaleUp { target: String, current_value: u32, recommended_value: u32 },
    ScaleDown { target: String, current_value: u32, recommended_value: u32 },
    ConfigChange { target: String, current_value: String, recommended_value: String },
}

#[derive(Debug, Clone)]
pub struct ExpectedImpact {
    pub latency_improvement_percent: f64,
    pub throughput_improvement_percent: f64,
    pub cost_impact_percent: f64,
    pub confidence_score: f64,
}
```

### 4.5 Configuration Adjustment Protocol

```rust
use async_trait::async_trait;

#[async_trait]
pub trait ConfigurationAdjuster: Send + Sync {
    async fn apply_recommendation(
        &self,
        recommendation: &OptimizationRecommendation,
    ) -> Result<AdjustmentResult, AdjustmentError>;

    async fn rollback(
        &self,
        adjustment_id: &str,
    ) -> Result<(), AdjustmentError>;

    async fn verify_adjustment(
        &self,
        adjustment_id: &str,
        verification_period: Duration,
    ) -> Result<VerificationResult, AdjustmentError>;
}

pub struct KubernetesConfigAdjuster {
    kube_client: kube::Client,
    namespace: String,
}

#[async_trait]
impl ConfigurationAdjuster for KubernetesConfigAdjuster {
    async fn apply_recommendation(
        &self,
        recommendation: &OptimizationRecommendation,
    ) -> Result<AdjustmentResult, AdjustmentError> {
        match &recommendation.action {
            OptimizationAction::ScaleUp { target, recommended_value, .. } => {
                if target == "replica_count" {
                    self.scale_deployment(*recommended_value).await?;
                }
            }
            OptimizationAction::ConfigChange { target, recommended_value, .. } => {
                self.update_config(target, recommended_value).await?;
            }
            _ => {}
        }

        Ok(AdjustmentResult {
            adjustment_id: uuid::Uuid::new_v4().to_string(),
            applied_at: chrono::Utc::now(),
            recommendation: recommendation.clone(),
        })
    }

    async fn rollback(
        &self,
        adjustment_id: &str,
    ) -> Result<(), AdjustmentError> {
        // Retrieve original configuration and restore
        todo!()
    }

    async fn verify_adjustment(
        &self,
        adjustment_id: &str,
        verification_period: Duration,
    ) -> Result<VerificationResult, AdjustmentError> {
        // Wait for verification period
        tokio::time::sleep(verification_period).await;

        // Collect metrics and compare with baseline
        let post_adjustment_metrics = self.collect_metrics().await?;

        // Determine if adjustment was successful
        let improvement = self.calculate_improvement(&post_adjustment_metrics);

        Ok(VerificationResult {
            adjustment_id: adjustment_id.to_string(),
            success: improvement > 0.0,
            actual_improvement_percent: improvement,
            should_rollback: improvement < -5.0, // Worse than 5%
        })
    }
}

#[derive(Debug, Clone)]
pub struct AdjustmentResult {
    pub adjustment_id: String,
    pub applied_at: chrono::DateTime<chrono::Utc>,
    pub recommendation: OptimizationRecommendation,
}

#[derive(Debug)]
pub struct VerificationResult {
    pub adjustment_id: String,
    pub success: bool,
    pub actual_improvement_percent: f64,
    pub should_rollback: bool,
}
```

### 4.6 SAFLA-Style Feedback Architecture

Based on Self-Adaptive Feedback Loop Architecture (SAFLA) pattern:

```rust
pub struct SAFLAOptimizationLoop {
    collector: MetricsCollector,
    evaluator: PerformanceEvaluator,
    learner: CausalLearner,
    adjuster: Box<dyn ConfigurationAdjuster>,
}

impl SAFLAOptimizationLoop {
    pub async fn run_cycle(&mut self) -> Result<CycleResult, Error> {
        // 1. Collect: Gather performance data
        let metrics = self.collector.collect_metrics().await?;

        // 2. Evaluate: Analyze performance against goals
        let evaluation = self.evaluator.evaluate(&metrics).await?;

        // 3. Learn: Update causal relationships
        let learning_episode = LearningEpisode {
            state: metrics.current_state.clone(),
            action: metrics.last_action.clone(),
            reward: evaluation.performance_score,
            next_state: metrics.current_state.clone(),
        };
        self.learner.store_episode(learning_episode).await?;

        // 4. Decide: Generate optimized recommendations
        let recommendations = self.learner
            .generate_recommendations(&metrics, &evaluation)
            .await?;

        // 5. Act: Apply best recommendation
        if let Some(best_rec) = recommendations.first() {
            let result = self.adjuster.apply_recommendation(best_rec).await?;

            // 6. Verify: Monitor impact
            tokio::spawn({
                let adjuster = self.adjuster.clone();
                let adjustment_id = result.adjustment_id.clone();
                async move {
                    let verification = adjuster
                        .verify_adjustment(&adjustment_id, Duration::from_secs(300))
                        .await?;

                    if verification.should_rollback {
                        adjuster.rollback(&adjustment_id).await?;
                    }

                    Ok::<(), Error>(())
                }
            });

            Ok(CycleResult::ActionTaken(result))
        } else {
            Ok(CycleResult::NoActionNeeded)
        }
    }
}

#[derive(Debug)]
pub struct LearningEpisode {
    pub state: serde_json::Value,
    pub action: Option<serde_json::Value>,
    pub reward: f64,
    pub next_state: serde_json::Value,
}

pub struct CausalLearner {
    episode_store: Vec<LearningEpisode>,
    causal_graph: HashMap<String, Vec<(String, f64)>>, // Action -> [(Outcome, Weight)]
}

impl CausalLearner {
    pub async fn store_episode(&mut self, episode: LearningEpisode) {
        self.episode_store.push(episode);

        // Update causal relationships
        if self.episode_store.len() >= 10 {
            self.update_causal_graph().await;
        }
    }

    async fn update_causal_graph(&mut self) {
        // Analyze recent episodes to identify causal patterns
        // e.g., "increasing batch_size -> higher throughput"
        // Use statistical methods or simple heuristics

        for window in self.episode_store.windows(2) {
            if let [prev, curr] = window {
                if let Some(action) = &prev.action {
                    let reward_delta = curr.reward - prev.reward;
                    // Update causal graph based on reward delta
                    // ... implementation details
                }
            }
        }
    }

    pub async fn generate_recommendations(
        &self,
        metrics: &PerformanceMetrics,
        evaluation: &PerformanceEvaluation,
    ) -> Result<Vec<OptimizationRecommendation>, Error> {
        // Use causal graph to recommend actions most likely to improve performance
        let mut recommendations = Vec::new();

        for (action_type, outcomes) in &self.causal_graph {
            let avg_impact: f64 = outcomes.iter().map(|(_, weight)| weight).sum::<f64>()
                / outcomes.len() as f64;

            if avg_impact > 0.1 { // Threshold for meaningful impact
                // Generate recommendation based on learned causal relationship
                // ... implementation details
            }
        }

        Ok(recommendations)
    }
}
```

---

## 5. Security and Edge Integration

### 5.1 LLM-Shield Security Layer

#### 5.1.1 Guardrails Integration

```rust
use async_trait::async_trait;

#[async_trait]
pub trait SecurityGuardrail: Send + Sync {
    async fn validate_input(&self, input: &str) -> ValidationResult;
    async fn filter_output(&self, output: &str) -> FilterResult;
    async fn detect_threats(&self, request: &InferenceRequest) -> ThreatAssessment;
}

#[derive(Debug)]
pub struct ValidationResult {
    pub passed: bool,
    pub violations: Vec<PolicyViolation>,
    pub risk_score: f64,
}

#[derive(Debug)]
pub struct PolicyViolation {
    pub policy_type: PolicyType,
    pub severity: Severity,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum PolicyType {
    PromptInjection,
    PIILeakage,
    ToxicContent,
    DisallowedTopic,
    Jailbreak,
}

#[derive(Debug, Clone)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct CompositeGuardrail {
    guardrails: Vec<Box<dyn SecurityGuardrail>>,
}

#[async_trait]
impl SecurityGuardrail for CompositeGuardrail {
    async fn validate_input(&self, input: &str) -> ValidationResult {
        let mut all_violations = Vec::new();
        let mut max_risk_score = 0.0;

        for guardrail in &self.guardrails {
            let result = guardrail.validate_input(input).await;
            all_violations.extend(result.violations);
            max_risk_score = max_risk_score.max(result.risk_score);
        }

        ValidationResult {
            passed: all_violations.is_empty(),
            violations: all_violations,
            risk_score: max_risk_score,
        }
    }

    async fn filter_output(&self, output: &str) -> FilterResult {
        // Similar implementation
        todo!()
    }

    async fn detect_threats(&self, request: &InferenceRequest) -> ThreatAssessment {
        // Similar implementation
        todo!()
    }
}

// Integration with latency monitoring
pub struct SecureLatencyMonitor {
    guardrail: Box<dyn SecurityGuardrail>,
    latency_monitor: LatencyMonitor,
}

impl SecureLatencyMonitor {
    pub async fn monitor_secure_inference(
        &self,
        request: InferenceRequest,
    ) -> Result<SecureInferenceMeasurement, Error> {
        let overall_start = Instant::now();

        // Security validation (measured separately)
        let validation_start = Instant::now();
        let validation = self.guardrail.validate_input(&request.prompt).await;
        let validation_time = validation_start.elapsed();

        if !validation.passed {
            return Err(Error::SecurityViolation(validation.violations));
        }

        // Threat detection (measured separately)
        let threat_start = Instant::now();
        let threats = self.guardrail.detect_threats(&request).await;
        let threat_detection_time = threat_start.elapsed();

        if threats.risk_level > RiskLevel::High {
            return Err(Error::ThreatDetected(threats));
        }

        // Actual inference with latency monitoring
        let inference_start = Instant::now();
        let measurement = self.latency_monitor.monitor_inference(request).await?;
        let inference_time = inference_start.elapsed();

        // Output filtering (measured separately)
        let filter_start = Instant::now();
        let filtered = self.guardrail.filter_output(&measurement.response).await;
        let filter_time = filter_start.elapsed();

        let total_time = overall_start.elapsed();

        Ok(SecureInferenceMeasurement {
            measurement,
            security_overhead: SecurityOverhead {
                validation_time,
                threat_detection_time,
                filter_time,
                total_security_overhead: validation_time + threat_detection_time + filter_time,
            },
            total_time,
            security_impact_percent: (validation_time + threat_detection_time + filter_time)
                .as_secs_f64() / total_time.as_secs_f64() * 100.0,
        })
    }
}

#[derive(Debug)]
pub struct SecureInferenceMeasurement {
    pub measurement: InferenceMeasurement,
    pub security_overhead: SecurityOverhead,
    pub total_time: Duration,
    pub security_impact_percent: f64,
}

#[derive(Debug)]
pub struct SecurityOverhead {
    pub validation_time: Duration,
    pub threat_detection_time: Duration,
    pub filter_time: Duration,
    pub total_security_overhead: Duration,
}
```

#### 5.1.2 Security Metrics Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Security-Aware Performance Metrics",
  "type": "object",
  "properties": {
    "inference_metrics": {
      "$ref": "#/definitions/inference_measurement"
    },
    "security_metrics": {
      "type": "object",
      "properties": {
        "guardrail_latency_ms": {
          "type": "object",
          "properties": {
            "input_validation_ms": {
              "type": "number"
            },
            "threat_detection_ms": {
              "type": "number"
            },
            "output_filtering_ms": {
              "type": "number"
            },
            "total_overhead_ms": {
              "type": "number"
            }
          }
        },
        "security_impact_percent": {
          "type": "number",
          "description": "Percentage of total latency attributed to security"
        },
        "violations_detected": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "type": {
                "type": "string"
              },
              "severity": {
                "type": "string"
              },
              "blocked": {
                "type": "boolean"
              }
            }
          }
        },
        "risk_assessment": {
          "type": "object",
          "properties": {
            "risk_score": {
              "type": "number",
              "minimum": 0,
              "maximum": 1
            },
            "risk_level": {
              "type": "string",
              "enum": ["low", "medium", "high", "critical"]
            }
          }
        }
      }
    }
  }
}
```

### 5.2 LLM-Edge-Agent Integration

#### 5.2.1 Edge Deployment Patterns

```rust
pub struct EdgeLatencyMonitor {
    central_aggregator: Option<CentralAggregatorClient>,
    local_storage: LocalMetricsStore,
    sync_policy: SyncPolicy,
}

#[derive(Debug, Clone)]
pub struct SyncPolicy {
    pub sync_interval: Duration,
    pub batch_size: usize,
    pub compression: bool,
    pub sync_on_network_available: bool,
}

impl EdgeLatencyMonitor {
    pub async fn monitor_edge_inference(
        &mut self,
        request: InferenceRequest,
    ) -> Result<EdgeInferenceMeasurement, Error> {
        let start = Instant::now();

        // Measure edge-specific components
        let model_load_time = self.measure_model_load().await?;
        let inference_time = self.measure_inference(&request).await?;
        let quantization_overhead = self.measure_quantization_overhead().await;

        let measurement = EdgeInferenceMeasurement {
            standard_metrics: InferenceMeasurement {
                ttft: inference_time.ttft,
                tpot: inference_time.tpot,
                total_duration: start.elapsed(),
                input_tokens: request.input_tokens,
                output_tokens: inference_time.output_tokens,
                model: request.model.clone(),
                provider: "edge".to_string(),
                endpoint: "local".to_string(),
            },
            edge_metrics: EdgeSpecificMetrics {
                model_load_time,
                quantization_overhead,
                device_type: self.get_device_info(),
                memory_footprint_mb: self.get_memory_usage(),
                power_consumption_w: self.measure_power_consumption().await,
                network_latency: None, // Local inference
            },
        };

        // Store locally
        self.local_storage.store(&measurement).await?;

        // Sync with central if policy allows
        if self.should_sync().await {
            self.sync_to_central().await?;
        }

        Ok(measurement)
    }

    async fn sync_to_central(&mut self) -> Result<(), Error> {
        if let Some(aggregator) = &self.central_aggregator {
            let pending_measurements = self.local_storage
                .get_unsynced(self.sync_policy.batch_size)
                .await?;

            let payload = if self.sync_policy.compression {
                self.compress_measurements(&pending_measurements)?
            } else {
                serde_json::to_vec(&pending_measurements)?
            };

            aggregator.upload_batch(payload).await?;

            // Mark as synced
            for measurement in pending_measurements {
                self.local_storage.mark_synced(&measurement.id).await?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeInferenceMeasurement {
    pub standard_metrics: InferenceMeasurement,
    pub edge_metrics: EdgeSpecificMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeSpecificMetrics {
    pub model_load_time: Duration,
    pub quantization_overhead: Duration,
    pub device_type: DeviceInfo,
    pub memory_footprint_mb: f64,
    pub power_consumption_w: Option<f64>,
    pub network_latency: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_class: DeviceClass,
    pub accelerator: Option<String>,
    pub ram_gb: f64,
    pub compute_capability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceClass {
    Smartphone,
    Tablet,
    Laptop,
    Desktop,
    EmbeddedDevice,
    EdgeServer,
}
```

#### 5.2.2 Collaborative Inference Monitoring

```rust
pub struct CollaborativeInferenceMonitor {
    edge_monitor: EdgeLatencyMonitor,
    cloud_monitor: LatencyMonitor,
}

impl CollaborativeInferenceMonitor {
    pub async fn monitor_collaborative_inference(
        &self,
        request: InferenceRequest,
        strategy: CollaborationStrategy,
    ) -> Result<CollaborativeMeasurement, Error> {
        match strategy {
            CollaborationStrategy::SpeculativeDecoding => {
                self.monitor_speculative_decoding(request).await
            }
            CollaborationStrategy::ModelPartitioning { partition_point } => {
                self.monitor_partitioned_inference(request, partition_point).await
            }
            CollaborationStrategy::Offloading { threshold } => {
                self.monitor_adaptive_offloading(request, threshold).await
            }
        }
    }

    async fn monitor_speculative_decoding(
        &self,
        request: InferenceRequest,
    ) -> Result<CollaborativeMeasurement, Error> {
        let start = Instant::now();

        // Edge device generates speculative tokens (small model)
        let edge_start = Instant::now();
        let speculative_tokens = self.edge_monitor
            .generate_speculative_tokens(&request, 4) // Generate 4 tokens speculatively
            .await?;
        let edge_time = edge_start.elapsed();

        // Cloud verifies and continues (large model)
        let cloud_start = Instant::now();
        let verification = self.cloud_monitor
            .verify_and_continue(&request, &speculative_tokens)
            .await?;
        let cloud_time = cloud_start.elapsed();

        // Calculate efficiency metrics
        let tokens_accepted = verification.accepted_tokens;
        let acceptance_rate = tokens_accepted as f64 / speculative_tokens.len() as f64;
        let speedup = self.calculate_speedup(edge_time, cloud_time, acceptance_rate);

        Ok(CollaborativeMeasurement {
            total_duration: start.elapsed(),
            edge_time,
            cloud_time,
            network_time: verification.network_latency,
            collaboration_overhead: verification.coordination_overhead,
            efficiency_metrics: EfficiencyMetrics {
                acceptance_rate,
                speedup,
                tokens_per_second: verification.total_tokens as f64
                    / start.elapsed().as_secs_f64(),
            },
        })
    }
}

#[derive(Debug, Clone)]
pub enum CollaborationStrategy {
    SpeculativeDecoding,
    ModelPartitioning { partition_point: usize },
    Offloading { threshold: f64 },
}

#[derive(Debug)]
pub struct CollaborativeMeasurement {
    pub total_duration: Duration,
    pub edge_time: Duration,
    pub cloud_time: Duration,
    pub network_time: Duration,
    pub collaboration_overhead: Duration,
    pub efficiency_metrics: EfficiencyMetrics,
}

#[derive(Debug)]
pub struct EfficiencyMetrics {
    pub acceptance_rate: f64,
    pub speedup: f64,
    pub tokens_per_second: f64,
}
```

---

## 6. Data Serialization Formats

### 6.1 Format Comparison

| Format | Use Case | Pros | Cons | Rust Crate |
|--------|----------|------|------|------------|
| JSON | API responses, configuration | Human-readable, universal | Verbose, slow parsing | `serde_json` |
| Protocol Buffers | gRPC/OTLP telemetry | Compact, fast, schema validation | Requires compilation | `prost` |
| Apache Arrow | Bulk data transfer, analytics | Zero-copy, columnar | Complex setup | `arrow` |
| MessagePack | Binary JSON alternative | Compact, fast | Less ecosystem support | `rmp-serde` |
| CBOR | IoT, constrained environments | Compact, extensible | Less tooling | `ciborium` |

### 6.2 Protocol Buffer Definitions

```protobuf
syntax = "proto3";

package llm_latency_lens.v1;

message InferenceMeasurement {
  string measurement_id = 1;
  google.protobuf.Timestamp timestamp = 2;

  ModelInfo model_info = 3;
  LatencyMetrics latency = 4;
  TokenMetrics tokens = 5;
  ResourceMetrics resources = 6;
}

message ModelInfo {
  string name = 1;
  string provider = 2;
  string endpoint = 3;
  string version = 4;
  Quantization quantization = 5;
}

enum Quantization {
  QUANTIZATION_UNSPECIFIED = 0;
  QUANTIZATION_NONE = 1;
  QUANTIZATION_INT8 = 2;
  QUANTIZATION_INT4 = 3;
  QUANTIZATION_FP16 = 4;
  QUANTIZATION_BF16 = 5;
}

message LatencyMetrics {
  google.protobuf.Duration time_to_first_token = 1;
  google.protobuf.Duration time_per_output_token = 2;
  google.protobuf.Duration end_to_end_latency = 3;
  google.protobuf.Duration prefill_time = 4;
  google.protobuf.Duration decode_time = 5;
}

message TokenMetrics {
  uint32 input_tokens = 1;
  uint32 output_tokens = 2;
  double tokens_per_second = 3;
}

message ResourceMetrics {
  double gpu_utilization_percent = 1;
  double memory_utilization_percent = 2;
  double kv_cache_utilization_percent = 3;
  double power_consumption_watts = 4;
}

message BenchmarkResults {
  string benchmark_id = 1;
  google.protobuf.Timestamp timestamp = 2;

  BenchmarkConfiguration config = 3;
  repeated InferenceMeasurement measurements = 4;
  AggregatedMetrics aggregated = 5;
}

message AggregatedMetrics {
  LatencyDistribution ttft_distribution = 1;
  LatencyDistribution tpot_distribution = 2;
  LatencyDistribution e2e_distribution = 3;

  ThroughputMetrics throughput = 4;
  ErrorMetrics errors = 5;
}

message LatencyDistribution {
  double mean_ms = 1;
  double median_ms = 2;
  double p50_ms = 3;
  double p90_ms = 4;
  double p95_ms = 5;
  double p99_ms = 6;
  double p999_ms = 7;
  double min_ms = 8;
  double max_ms = 9;
  double stddev_ms = 10;
  uint64 sample_count = 11;
}

message ThroughputMetrics {
  double requests_per_second = 1;
  double tokens_per_second = 2;
  double tokens_per_second_per_gpu = 3;
}

message ErrorMetrics {
  uint64 total_errors = 1;
  double error_rate = 2;
  map<string, uint64> errors_by_type = 3;
}

message BenchmarkConfiguration {
  ModelInfo model = 1;
  HardwareConfig hardware = 2;
  TestParameters test_params = 3;
}

message HardwareConfig {
  string gpu_type = 1;
  uint32 gpu_count = 2;
  uint32 cpu_cores = 3;
  double memory_gb = 4;
  uint32 batch_size = 5;
}

message TestParameters {
  TokenDistribution input_tokens = 1;
  TokenDistribution output_tokens = 2;
  repeated uint32 concurrency_levels = 3;
  uint32 warmup_requests = 4;
  uint32 total_requests = 5;
}

message TokenDistribution {
  uint32 mean = 1;
  uint32 stddev = 2;
  DistributionType distribution = 3;
}

enum DistributionType {
  DISTRIBUTION_TYPE_UNSPECIFIED = 0;
  DISTRIBUTION_TYPE_NORMAL = 1;
  DISTRIBUTION_TYPE_UNIFORM = 2;
  DISTRIBUTION_TYPE_FIXED = 3;
}
```

### 6.3 Apache Arrow Schema

```rust
use arrow::datatypes::{DataType, Field, Schema};
use std::sync::Arc;

pub fn inference_measurement_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("measurement_id", DataType::Utf8, false),
        Field::new("timestamp", DataType::Timestamp(arrow::datatypes::TimeUnit::Millisecond, None), false),

        // Model info
        Field::new("model_name", DataType::Utf8, false),
        Field::new("provider", DataType::Utf8, false),
        Field::new("endpoint", DataType::Utf8, false),

        // Latency metrics (in milliseconds)
        Field::new("ttft_ms", DataType::Float64, false),
        Field::new("tpot_ms", DataType::Float64, false),
        Field::new("e2e_latency_ms", DataType::Float64, false),
        Field::new("prefill_ms", DataType::Float64, true),
        Field::new("decode_ms", DataType::Float64, true),

        // Token metrics
        Field::new("input_tokens", DataType::UInt32, false),
        Field::new("output_tokens", DataType::UInt32, false),
        Field::new("tokens_per_second", DataType::Float64, false),

        // Resource metrics
        Field::new("gpu_utilization_percent", DataType::Float64, true),
        Field::new("memory_utilization_percent", DataType::Float64, true),
        Field::new("kv_cache_utilization_percent", DataType::Float64, true),

        // Error info
        Field::new("error_occurred", DataType::Boolean, false),
        Field::new("error_type", DataType::Utf8, true),
    ]))
}

use arrow::array::{
    ArrayRef, Float64Array, StringArray, TimestampMillisecondArray,
    UInt32Array, BooleanArray,
};
use arrow::record_batch::RecordBatch;

pub fn measurements_to_arrow_batch(
    measurements: &[InferenceMeasurement],
) -> Result<RecordBatch, arrow::error::ArrowError> {
    let schema = inference_measurement_schema();

    let measurement_ids: ArrayRef = Arc::new(StringArray::from(
        measurements.iter()
            .map(|m| m.measurement_id.as_str())
            .collect::<Vec<_>>(),
    ));

    let timestamps: ArrayRef = Arc::new(TimestampMillisecondArray::from(
        measurements.iter()
            .map(|m| m.timestamp.timestamp_millis())
            .collect::<Vec<_>>(),
    ));

    let model_names: ArrayRef = Arc::new(StringArray::from(
        measurements.iter()
            .map(|m| m.model.as_str())
            .collect::<Vec<_>>(),
    ));

    let ttft_ms: ArrayRef = Arc::new(Float64Array::from(
        measurements.iter()
            .map(|m| m.ttft.as_secs_f64() * 1000.0)
            .collect::<Vec<_>>(),
    ));

    let tpot_ms: ArrayRef = Arc::new(Float64Array::from(
        measurements.iter()
            .map(|m| m.tpot.as_secs_f64() * 1000.0)
            .collect::<Vec<_>>(),
    ));

    // ... more columns

    RecordBatch::try_new(
        schema,
        vec![
            measurement_ids,
            timestamps,
            model_names,
            ttft_ms,
            tpot_ms,
            // ... more columns
        ],
    )
}

// Arrow IPC writer for efficient serialization
use arrow::ipc::writer::StreamWriter;
use std::fs::File;

pub fn write_measurements_to_arrow_file(
    measurements: &[InferenceMeasurement],
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let batch = measurements_to_arrow_batch(measurements)?;
    let file = File::create(path)?;
    let mut writer = StreamWriter::try_new(file, &batch.schema())?;

    writer.write(&batch)?;
    writer.finish()?;

    Ok(())
}
```

---

## 7. Rust Implementation Guide

### 7.1 Core Dependencies

```toml
[package]
name = "llm-latency-lens"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async runtime
tokio = { version = "1.40", features = ["full"] }
tokio-stream = "0.1"
async-trait = "0.1"

# HTTP client/server
axum = "0.7"
reqwest = { version = "0.12", features = ["json", "stream"] }
hyper = "1.0"
tower = "0.4"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
prost = "0.13"
prost-types = "0.13"

# OpenTelemetry
opentelemetry = "0.25"
opentelemetry-otlp = { version = "0.25", features = ["tonic", "metrics"] }
opentelemetry_sdk = { version = "0.25", features = ["rt-tokio"] }

# Prometheus
prometheus = "0.13"

# Apache Arrow
arrow = "53.0"
arrow-ipc = "53.0"
parquet = "53.0"

# Utilities
uuid = { version = "1.10", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
anyhow = "1.0"

# Statistics
statrs = "0.17"

# Configuration
config = "0.14"
clap = { version = "4.5", features = ["derive"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# gRPC
tonic = "0.12"
tonic-build = "0.12"

# Kubernetes client (optional)
kube = { version = "0.95", features = ["runtime", "derive"], optional = true }
k8s-openapi = { version = "0.22", features = ["latest"], optional = true }

[build-dependencies]
tonic-build = "0.12"

[features]
default = ["telemetry", "benchmarking"]
telemetry = ["opentelemetry", "opentelemetry-otlp", "prometheus"]
benchmarking = []
kubernetes = ["kube", "k8s-openapi"]
security = []
edge = []
```

### 7.2 Project Structure

```
llm-latency-lens/
├── Cargo.toml
├── build.rs                    # Protocol buffer compilation
├── proto/
│   ├── measurement.proto       # Measurement protobuf definitions
│   └── telemetry.proto        # Telemetry protobuf definitions
├── src/
│   ├── main.rs                # Binary entry point
│   ├── lib.rs                 # Library root
│   ├── config/
│   │   ├── mod.rs
│   │   └── settings.rs        # Configuration management
│   ├── collector/
│   │   ├── mod.rs
│   │   ├── metrics.rs         # Metrics collection
│   │   └── streaming.rs       # Async streaming collection
│   ├── measurement/
│   │   ├── mod.rs
│   │   ├── types.rs           # Core measurement types
│   │   └── statistics.rs      # Statistical analysis
│   ├── export/
│   │   ├── mod.rs
│   │   ├── json.rs            # JSON exporter
│   │   ├── prometheus.rs      # Prometheus exporter
│   │   ├── otlp.rs            # OpenTelemetry exporter
│   │   └── arrow.rs           # Arrow IPC exporter
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── benchmark.rs       # Benchmark integration
│   │   ├── observatory.rs     # Observatory integration
│   │   └── optimizer.rs       # Optimizer integration
│   ├── api/
│   │   ├── mod.rs
│   │   ├── rest.rs            # REST API handlers
│   │   └── websocket.rs       # WebSocket handlers
│   ├── optimization/
│   │   ├── mod.rs
│   │   ├── triggers.rs        # Optimization triggers
│   │   ├── recommendations.rs # Recommendation engine
│   │   └── feedback_loop.rs   # Feedback loop implementation
│   ├── security/
│   │   ├── mod.rs
│   │   └── guardrails.rs      # Security guardrails
│   └── edge/
│       ├── mod.rs
│       ├── monitor.rs         # Edge monitoring
│       └── sync.rs            # Sync with central
└── examples/
    ├── basic_monitoring.rs
    ├── benchmark_integration.rs
    ├── otlp_export.rs
    └── feedback_loop.rs
```

### 7.3 Build Configuration

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/proto")
        .compile(
            &[
                "proto/measurement.proto",
                "proto/telemetry.proto",
            ],
            &["proto"],
        )?;

    Ok(())
}
```

### 7.4 Core Type Definitions

```rust
// src/measurement/types.rs

use std::time::Duration;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceMeasurement {
    pub measurement_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,

    // Model info
    pub model: String,
    pub provider: String,
    pub endpoint: String,

    // Latency metrics
    pub ttft: Duration,
    pub tpot: Duration,
    pub total_duration: Duration,
    pub prefill_time: Option<Duration>,
    pub decode_time: Option<Duration>,

    // Token metrics
    pub input_tokens: usize,
    pub output_tokens: usize,

    // Resource metrics
    pub gpu_utilization: Option<f64>,
    pub memory_utilization: Option<f64>,
    pub kv_cache_utilization: Option<f64>,

    // Error info
    pub error: Option<InferenceError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceError {
    pub error_type: String,
    pub message: String,
    pub code: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub latency_analysis: LatencyAnalysis,
    pub throughput_analysis: ThroughputAnalysis,
    pub resource_analysis: ResourceAnalysis,
    pub error_metrics: ErrorMetrics,
}

#[derive(Debug, Clone)]
pub struct LatencyAnalysis {
    pub ttft_p50_ms: f64,
    pub ttft_p95_ms: f64,
    pub ttft_p99_ms: f64,
    pub tpot_p50_ms: f64,
    pub tpot_p95_ms: f64,
    pub e2e_p95_ms: f64,
}

#[derive(Debug, Clone)]
pub struct ThroughputAnalysis {
    pub requests_per_second: f64,
    pub tokens_per_second: f64,
    pub utilization_efficiency: f64,
}

#[derive(Debug, Clone)]
pub struct ResourceAnalysis {
    pub avg_gpu_utilization: f64,
    pub avg_memory_utilization: f64,
    pub bottleneck_indicator: BottleneckType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BottleneckType {
    Compute,
    Memory,
    Network,
    None,
}

#[derive(Debug, Clone)]
pub struct ErrorMetrics {
    pub total_errors: usize,
    pub error_rate: f64,
    pub errors_by_type: std::collections::HashMap<String, usize>,
}
```

### 7.5 Async Streaming Collector

```rust
// src/collector/streaming.rs

use tokio::sync::mpsc;
use tokio_stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct StreamingCollector {
    rx: mpsc::Receiver<InferenceMeasurement>,
}

impl StreamingCollector {
    pub fn new(buffer_size: usize) -> (MeasurementSender, Self) {
        let (tx, rx) = mpsc::channel(buffer_size);
        (MeasurementSender { tx }, Self { rx })
    }
}

impl Stream for StreamingCollector {
    type Item = InferenceMeasurement;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

#[derive(Clone)]
pub struct MeasurementSender {
    tx: mpsc::Sender<InferenceMeasurement>,
}

impl MeasurementSender {
    pub async fn send(&self, measurement: InferenceMeasurement) -> Result<(), Error> {
        self.tx.send(measurement).await
            .map_err(|_| Error::ChannelClosed)?;
        Ok(())
    }
}

// Example usage
pub async fn monitor_with_streaming() -> Result<(), Error> {
    let (sender, mut collector) = StreamingCollector::new(1000);

    // Spawn collection task
    tokio::spawn(async move {
        while let Some(measurement) = collector.next().await {
            // Process measurement
            println!("Received: {:?}", measurement);
        }
    });

    // Simulate measurements
    for i in 0..100 {
        let measurement = InferenceMeasurement {
            measurement_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            model: "test-model".to_string(),
            provider: "test".to_string(),
            endpoint: "http://localhost:8000".to_string(),
            ttft: Duration::from_millis(50 + i * 10),
            tpot: Duration::from_millis(10 + i),
            total_duration: Duration::from_millis(500 + i * 10),
            prefill_time: None,
            decode_time: None,
            input_tokens: 100,
            output_tokens: 50,
            gpu_utilization: Some(75.0),
            memory_utilization: Some(60.0),
            kv_cache_utilization: Some(40.0),
            error: None,
        };

        sender.send(measurement).await?;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}
```

---

## 8. Reference Implementations

### 8.1 Complete Example: Benchmark Integration

```rust
// examples/benchmark_integration.rs

use llm_latency_lens::{
    measurement::{InferenceMeasurement, PerformanceMetrics},
    collector::StreamingCollector,
    export::json::JsonExporter,
    integration::benchmark::BenchmarkRunner,
};
use tokio::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize collector
    let (sender, collector) = StreamingCollector::new(1000);

    // Spawn metrics processing
    let metrics_handle = tokio::spawn(async move {
        let mut measurements = Vec::new();

        let mut stream = collector;
        while let Some(measurement) = stream.next().await {
            measurements.push(measurement);

            if measurements.len() >= 100 {
                break;
            }
        }

        measurements
    });

    // Run benchmark
    let benchmark_config = BenchmarkConfig {
        model: "llama-3.1-70b".to_string(),
        endpoint: "http://localhost:8000/v1/completions".to_string(),
        input_token_range: (100, 500),
        output_token_target: 100,
        concurrency: 4,
        total_requests: 100,
    };

    let runner = BenchmarkRunner::new(benchmark_config, sender);
    runner.run().await?;

    // Wait for collection to complete
    let measurements = metrics_handle.await?;

    // Calculate statistics
    let metrics = calculate_performance_metrics(&measurements)?;

    // Export results
    let exporter = JsonExporter::new("./benchmark_results.json");
    exporter.export_benchmark_results(&measurements, &metrics).await?;

    println!("Benchmark complete!");
    println!("TTFT P95: {:.2}ms", metrics.latency_analysis.ttft_p95_ms);
    println!("TPOT P95: {:.2}ms", metrics.latency_analysis.tpot_p95_ms);
    println!("Throughput: {:.2} req/s", metrics.throughput_analysis.requests_per_second);

    Ok(())
}

fn calculate_performance_metrics(
    measurements: &[InferenceMeasurement],
) -> Result<PerformanceMetrics, Box<dyn std::error::Error>> {
    use statrs::statistics::{Data, OrderStatistics};

    // Extract TTFT values
    let ttft_values: Vec<f64> = measurements
        .iter()
        .map(|m| m.ttft.as_secs_f64() * 1000.0)
        .collect();

    let mut ttft_data = Data::new(ttft_values);

    let latency_analysis = LatencyAnalysis {
        ttft_p50_ms: ttft_data.percentile(50),
        ttft_p95_ms: ttft_data.percentile(95),
        ttft_p99_ms: ttft_data.percentile(99),
        // ... more calculations
    };

    // ... calculate other metrics

    Ok(PerformanceMetrics {
        latency_analysis,
        throughput_analysis: ThroughputAnalysis {
            requests_per_second: 10.0,
            tokens_per_second: 100.0,
            utilization_efficiency: 0.85,
        },
        resource_analysis: ResourceAnalysis {
            avg_gpu_utilization: 75.0,
            avg_memory_utilization: 60.0,
            bottleneck_indicator: BottleneckType::None,
        },
        error_metrics: ErrorMetrics {
            total_errors: 0,
            error_rate: 0.0,
            errors_by_type: Default::default(),
        },
    })
}
```

### 8.2 Complete Example: OTLP Telemetry Export

```rust
// examples/otlp_export.rs

use llm_latency_lens::{
    measurement::InferenceMeasurement,
    export::otlp::OtlpExporter,
};
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::Resource;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize OpenTelemetry
    let exporter = OtlpExporter::new(
        "http://localhost:4317",
        Some(vec![
            ("x-api-key".to_string(), "your-api-key".to_string()),
        ].into_iter().collect()),
    )?;

    // Simulate measurements
    for i in 0..10 {
        let measurement = InferenceMeasurement {
            measurement_id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            model: "llama-3.1-70b".to_string(),
            provider: "vllm".to_string(),
            endpoint: "http://localhost:8000".to_string(),
            ttft: Duration::from_millis(50 + i * 5),
            tpot: Duration::from_millis(10 + i),
            total_duration: Duration::from_millis(500 + i * 10),
            prefill_time: Some(Duration::from_millis(30 + i * 3)),
            decode_time: Some(Duration::from_millis(450 + i * 7)),
            input_tokens: 100,
            output_tokens: 50,
            gpu_utilization: Some(75.0 + i as f64 * 0.5),
            memory_utilization: Some(60.0 + i as f64 * 0.3),
            kv_cache_utilization: Some(40.0 + i as f64 * 0.2),
            error: None,
        };

        exporter.export_measurement(&measurement).await?;

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("Exported 10 measurements to OTLP collector");

    // Shutdown
    global::shutdown_tracer_provider();

    Ok(())
}
```

### 8.3 Complete Example: Optimization Feedback Loop

```rust
// examples/feedback_loop.rs

use llm_latency_lens::{
    measurement::PerformanceMetrics,
    optimization::{
        OptimizationEngine,
        OptimizationTrigger,
        TriggerType,
        TriggerCondition,
    },
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define optimization triggers
    let triggers = vec![
        OptimizationTrigger {
            trigger_type: TriggerType::LatencyThreshold,
            condition: TriggerCondition::P95LatencyExceeds {
                threshold_ms: 100.0,
            },
            evaluation_window: Duration::from_secs(300),
            cooldown_period: Duration::from_secs(600),
        },
        OptimizationTrigger {
            trigger_type: TriggerType::ThroughputThreshold,
            condition: TriggerCondition::ThroughputBelow {
                threshold_rps: 10.0,
            },
            evaluation_window: Duration::from_secs(300),
            cooldown_period: Duration::from_secs(600),
        },
        OptimizationTrigger {
            trigger_type: TriggerType::ResourceUtilization,
            condition: TriggerCondition::And(vec![
                TriggerCondition::GpuUtilizationAbove {
                    threshold_percent: 90.0,
                },
                TriggerCondition::P95LatencyExceeds {
                    threshold_ms: 200.0,
                },
            ]),
            evaluation_window: Duration::from_secs(300),
            cooldown_period: Duration::from_secs(600),
        },
    ];

    let mut engine = OptimizationEngine::new(triggers);

    // Simulate monitoring loop
    loop {
        // Collect current metrics
        let current_metrics = collect_current_metrics().await?;

        // Evaluate triggers
        let recommendations = engine.evaluate_triggers(&current_metrics).await;

        if !recommendations.is_empty() {
            println!("Generated {} recommendations:", recommendations.len());
            for (i, rec) in recommendations.iter().enumerate() {
                println!("  {}. {:?}", i + 1, rec);
                println!("     Expected impact: {:.1}% latency improvement",
                         rec.expected_impact.latency_improvement_percent);
                println!("     Confidence: {:.1}%",
                         rec.expected_impact.confidence_score * 100.0);
            }

            // In production, apply recommendations here
            // engine.apply_recommendation(&recommendations[0]).await?;
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

async fn collect_current_metrics() -> Result<PerformanceMetrics, Box<dyn std::error::Error>> {
    // Simulate metric collection
    Ok(PerformanceMetrics {
        latency_analysis: LatencyAnalysis {
            ttft_p50_ms: 45.0,
            ttft_p95_ms: 120.0, // Exceeds threshold
            ttft_p99_ms: 180.0,
            tpot_p50_ms: 8.0,
            tpot_p95_ms: 15.0,
            e2e_p95_ms: 500.0,
        },
        throughput_analysis: ThroughputAnalysis {
            requests_per_second: 8.5, // Below threshold
            tokens_per_second: 425.0,
            utilization_efficiency: 0.72,
        },
        resource_analysis: ResourceAnalysis {
            avg_gpu_utilization: 92.0, // High utilization
            avg_memory_utilization: 85.0,
            bottleneck_indicator: BottleneckType::Compute,
        },
        error_metrics: ErrorMetrics {
            total_errors: 3,
            error_rate: 0.02,
            errors_by_type: Default::default(),
        },
    })
}
```

---

## Conclusion

This document provides comprehensive integration specifications for LLM-Latency-Lens within the LLM DevOps ecosystem. Key takeaways:

1. **Standardized Data Formats**: Use OpenTelemetry semantic conventions for telemetry, JSON Schema for benchmark results, and Protocol Buffers for efficient data exchange.

2. **Multiple Integration Patterns**: Support REST APIs, gRPC/OTLP, Prometheus metrics, and WebSocket streaming for real-time data.

3. **Optimization Feedback Loops**: Implement SAFLA-style architectures with trigger-based optimization, causal learning, and automated configuration adjustment.

4. **Security and Edge Support**: Integrate security guardrails with latency monitoring and support edge deployment patterns with collaborative inference.

5. **Rust-First Implementation**: Leverage Tokio for async operations, multiple serialization formats, and strong type safety.

The specifications are designed to be:
- **Interoperable**: Compatible with existing LLM DevOps tools
- **Scalable**: Efficient data formats and streaming architectures
- **Observable**: Rich telemetry with OpenTelemetry and Prometheus
- **Optimizable**: Feedback loops for continuous performance improvement
- **Production-Ready**: Security, edge deployment, and robust error handling

### Next Steps

1. Implement core measurement collection with Tokio streams
2. Add OpenTelemetry and Prometheus exporters
3. Build benchmark integration following LLMPerf patterns
4. Implement optimization trigger system
5. Add security guardrails integration
6. Support edge deployment scenarios
7. Create comprehensive test suite
8. Write deployment documentation

This specification provides the foundation for a comprehensive LLM latency monitoring and optimization system that integrates seamlessly with the broader LLM DevOps ecosystem.
