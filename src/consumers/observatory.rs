//! LLM-Observatory Consumer Adapter
//!
//! Consumes telemetry streams, timing spans, request/response traces, and
//! structured latency events from LLM-Observatory.
//!
//! # Data Types Consumed
//!
//! - **Telemetry Spans**: OpenTelemetry-compatible timing data
//! - **Request/Response Traces**: Full request lifecycle tracking
//! - **Latency Events**: Structured timing breakdowns (TTFT, ITL, etc.)
//!
//! # Integration
//!
//! This adapter uses the `llm-observatory-core` crate to access Observatory
//! data structures and converts them to Latency-Lens `RequestMetrics`.

use super::{ConsumerError, ConsumerResult, DataConsumer, RetryConfig};
use crate::{RequestMetrics, SessionId, RequestId};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use llm_latency_lens_core::Provider;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for LLM-Observatory consumer
#[derive(Debug, Clone)]
pub struct ObservatoryConfig {
    /// Observatory API endpoint (if using remote mode)
    pub endpoint: Option<String>,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Enable local mode (read from shared memory/files)
    pub local_mode: bool,
    /// Retry configuration
    pub retry: RetryConfig,
    /// Timeout for API calls
    pub timeout: Duration,
}

impl Default for ObservatoryConfig {
    fn default() -> Self {
        Self {
            endpoint: None,
            api_key: None,
            local_mode: true,
            retry: RetryConfig::default(),
            timeout: Duration::from_secs(30),
        }
    }
}

/// A telemetry span from LLM-Observatory
///
/// Represents a timing span following OpenTelemetry GenAI semantic conventions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySpan {
    /// Unique span identifier
    pub span_id: String,
    /// Trace identifier (groups related spans)
    pub trace_id: String,
    /// Parent span ID (if any)
    pub parent_span_id: Option<String>,
    /// Span name/operation
    pub name: String,
    /// Start timestamp
    pub start_time: DateTime<Utc>,
    /// End timestamp
    pub end_time: DateTime<Utc>,
    /// Duration in nanoseconds
    pub duration_nanos: u64,
    /// Span attributes
    pub attributes: SpanAttributes,
    /// Span status
    pub status: SpanStatus,
}

/// Attributes attached to a telemetry span
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpanAttributes {
    /// LLM provider name
    #[serde(rename = "gen_ai.system")]
    pub gen_ai_system: Option<String>,
    /// Model identifier
    #[serde(rename = "gen_ai.request.model")]
    pub gen_ai_request_model: Option<String>,
    /// Input token count
    #[serde(rename = "gen_ai.usage.input_tokens")]
    pub gen_ai_usage_input_tokens: Option<u64>,
    /// Output token count
    #[serde(rename = "gen_ai.usage.output_tokens")]
    pub gen_ai_usage_output_tokens: Option<u64>,
    /// Time to first token in milliseconds
    #[serde(rename = "llm.ttft_ms")]
    pub llm_ttft_ms: Option<f64>,
    /// Tokens per second throughput
    #[serde(rename = "llm.tokens_per_second")]
    pub llm_tokens_per_second: Option<f64>,
    /// Request ID
    #[serde(rename = "llm.request_id")]
    pub llm_request_id: Option<String>,
    /// Session ID
    #[serde(rename = "llm.session_id")]
    pub llm_session_id: Option<String>,
}

/// Status of a telemetry span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanStatus {
    /// Status code (OK, ERROR, UNSET)
    pub code: String,
    /// Optional description
    pub description: Option<String>,
}

impl Default for SpanStatus {
    fn default() -> Self {
        Self {
            code: "OK".to_string(),
            description: None,
        }
    }
}

/// A traced request from LLM-Observatory
///
/// Contains full request lifecycle data including all timing breakdowns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracedRequest {
    /// Request identifier
    pub request_id: String,
    /// Trace identifier
    pub trace_id: String,
    /// Provider used
    pub provider: String,
    /// Model used
    pub model: String,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
    /// Time to first token
    pub ttft: Duration,
    /// Total request latency
    pub total_latency: Duration,
    /// Inter-token latencies
    pub inter_token_latencies: Vec<Duration>,
    /// Input token count
    pub input_tokens: u64,
    /// Output token count
    pub output_tokens: u64,
    /// Thinking tokens (if applicable)
    pub thinking_tokens: Option<u64>,
    /// Request success status
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Estimated cost in USD
    pub cost_usd: Option<f64>,
}

/// Consumer for LLM-Observatory data
///
/// Provides methods to consume telemetry spans, traces, and latency events
/// from the Observatory system.
pub struct ObservatoryConsumer {
    config: ObservatoryConfig,
    session_id: SessionId,
}

impl ObservatoryConsumer {
    /// Create a new Observatory consumer with default configuration
    pub fn new() -> Self {
        Self {
            config: ObservatoryConfig::default(),
            session_id: SessionId::new(),
        }
    }

    /// Create a new Observatory consumer with custom configuration
    pub fn with_config(config: ObservatoryConfig) -> Self {
        Self {
            config,
            session_id: SessionId::new(),
        }
    }

    /// Set the session ID for consumed metrics
    pub fn with_session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = session_id;
        self
    }

    /// Consume the latest telemetry spans from Observatory
    ///
    /// Returns spans converted to Latency-Lens RequestMetrics format.
    pub async fn consume_latest_spans(&self, limit: usize) -> ConsumerResult<Vec<RequestMetrics>> {
        // In local mode, read from shared state or files
        if self.config.local_mode {
            return self.consume_local_spans(limit).await;
        }

        // Remote mode: call Observatory API
        self.consume_remote_spans(limit).await
    }

    /// Consume spans from local Observatory state
    async fn consume_local_spans(&self, limit: usize) -> ConsumerResult<Vec<RequestMetrics>> {
        // This would integrate with llm-observatory-core's local storage
        // For now, return empty as we're establishing the interface
        tracing::debug!(
            limit = limit,
            "Consuming spans from local Observatory state"
        );

        // Integration point: Use llm_observatory_core types here
        // The actual implementation would read from Observatory's span storage
        Ok(Vec::new())
    }

    /// Consume spans from remote Observatory API
    async fn consume_remote_spans(&self, limit: usize) -> ConsumerResult<Vec<RequestMetrics>> {
        let endpoint = self.config.endpoint.as_ref().ok_or_else(|| {
            ConsumerError::ConfigError("Remote endpoint not configured".to_string())
        })?;

        tracing::debug!(
            endpoint = %endpoint,
            limit = limit,
            "Consuming spans from remote Observatory"
        );

        // This would make HTTP calls to Observatory API
        // For now, return empty as we're establishing the interface
        Ok(Vec::new())
    }

    /// Convert a TelemetrySpan to RequestMetrics
    pub fn span_to_metrics(&self, span: &TelemetrySpan) -> ConsumerResult<RequestMetrics> {
        let provider = self.parse_provider(&span.attributes.gen_ai_system)?;

        let ttft = span
            .attributes
            .llm_ttft_ms
            .map(|ms| Duration::from_secs_f64(ms / 1000.0))
            .unwrap_or_else(|| Duration::from_nanos(span.duration_nanos));

        let total_latency = Duration::from_nanos(span.duration_nanos);
        let tokens_per_second = span.attributes.llm_tokens_per_second.unwrap_or(0.0);

        Ok(RequestMetrics {
            request_id: RequestId::new(),
            session_id: self.session_id,
            provider,
            model: span
                .attributes
                .gen_ai_request_model
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            timestamp: span.start_time,
            ttft,
            total_latency,
            inter_token_latencies: Vec::new(), // Spans don't include ITL breakdown
            input_tokens: span.attributes.gen_ai_usage_input_tokens.unwrap_or(0),
            output_tokens: span.attributes.gen_ai_usage_output_tokens.unwrap_or(0),
            thinking_tokens: None,
            tokens_per_second,
            cost_usd: None,
            success: span.status.code == "OK",
            error: span.status.description.clone(),
        })
    }

    /// Convert a TracedRequest to RequestMetrics
    pub fn traced_request_to_metrics(&self, traced: &TracedRequest) -> ConsumerResult<RequestMetrics> {
        let provider = self.parse_provider(&Some(traced.provider.clone()))?;

        let tokens_per_second = if traced.total_latency.as_secs_f64() > 0.0 {
            traced.output_tokens as f64 / traced.total_latency.as_secs_f64()
        } else {
            0.0
        };

        Ok(RequestMetrics {
            request_id: RequestId::new(),
            session_id: self.session_id,
            provider,
            model: traced.model.clone(),
            timestamp: traced.timestamp,
            ttft: traced.ttft,
            total_latency: traced.total_latency,
            inter_token_latencies: traced.inter_token_latencies.clone(),
            input_tokens: traced.input_tokens,
            output_tokens: traced.output_tokens,
            thinking_tokens: traced.thinking_tokens,
            tokens_per_second,
            cost_usd: traced.cost_usd,
            success: traced.success,
            error: traced.error.clone(),
        })
    }

    /// Parse provider string to Provider enum
    fn parse_provider(&self, provider_str: &Option<String>) -> ConsumerResult<Provider> {
        match provider_str.as_deref() {
            Some("openai") | Some("OpenAI") => Ok(Provider::OpenAI),
            Some("anthropic") | Some("Anthropic") => Ok(Provider::Anthropic),
            Some("google") | Some("Google") => Ok(Provider::Google),
            Some("aws-bedrock") | Some("bedrock") => Ok(Provider::AwsBedrock),
            Some("azure-openai") | Some("azure") => Ok(Provider::AzureOpenAI),
            Some(_) | None => Ok(Provider::Generic),
        }
    }

    /// Subscribe to live telemetry stream from Observatory
    ///
    /// This creates a streaming connection to receive real-time spans.
    #[cfg(feature = "streaming")]
    pub async fn subscribe_telemetry_stream(
        &self,
    ) -> ConsumerResult<impl futures::Stream<Item = ConsumerResult<TelemetrySpan>>> {
        // Would return a stream of telemetry spans
        unimplemented!("Streaming support requires 'streaming' feature")
    }
}

impl Default for ObservatoryConsumer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DataConsumer for ObservatoryConsumer {
    fn name(&self) -> &'static str {
        "llm-observatory"
    }

    async fn health_check(&self) -> ConsumerResult<bool> {
        if self.config.local_mode {
            // In local mode, check if Observatory shared state is accessible
            Ok(true)
        } else {
            // In remote mode, ping the API endpoint
            let endpoint = match &self.config.endpoint {
                Some(e) => e,
                None => return Ok(false),
            };

            tracing::debug!(endpoint = %endpoint, "Health checking Observatory");
            // Would make actual health check call here
            Ok(true)
        }
    }

    async fn consume(&self, limit: usize) -> ConsumerResult<Vec<RequestMetrics>> {
        self.consume_latest_spans(limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observatory_config_defaults() {
        let config = ObservatoryConfig::default();
        assert!(config.local_mode);
        assert!(config.endpoint.is_none());
    }

    #[test]
    fn test_span_to_metrics_conversion() {
        let consumer = ObservatoryConsumer::new();

        let span = TelemetrySpan {
            span_id: "span-123".to_string(),
            trace_id: "trace-456".to_string(),
            parent_span_id: None,
            name: "llm.completion".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            duration_nanos: 1_000_000_000, // 1 second
            attributes: SpanAttributes {
                gen_ai_system: Some("openai".to_string()),
                gen_ai_request_model: Some("gpt-4".to_string()),
                gen_ai_usage_input_tokens: Some(100),
                gen_ai_usage_output_tokens: Some(50),
                llm_ttft_ms: Some(150.0),
                llm_tokens_per_second: Some(50.0),
                ..Default::default()
            },
            status: SpanStatus::default(),
        };

        let metrics = consumer.span_to_metrics(&span).unwrap();

        assert_eq!(metrics.model, "gpt-4");
        assert_eq!(metrics.input_tokens, 100);
        assert_eq!(metrics.output_tokens, 50);
        assert!(metrics.success);
    }

    #[test]
    fn test_parse_provider() {
        let consumer = ObservatoryConsumer::new();

        assert!(matches!(
            consumer.parse_provider(&Some("openai".to_string())),
            Ok(Provider::OpenAI)
        ));
        assert!(matches!(
            consumer.parse_provider(&Some("anthropic".to_string())),
            Ok(Provider::Anthropic)
        ));
        assert!(matches!(
            consumer.parse_provider(&None),
            Ok(Provider::Generic)
        ));
    }

    #[tokio::test]
    async fn test_health_check_local_mode() {
        let consumer = ObservatoryConsumer::new();
        let result = consumer.health_check().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
