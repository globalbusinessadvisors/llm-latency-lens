//! Metrics data structures for LLM Latency Lens
//!
//! Provides type-safe representations of metrics data including:
//! - Individual request metrics
//! - Aggregated statistical metrics
//! - Latency distribution data

use chrono::{DateTime, Utc};
use llm_latency_lens_core::{Provider, RequestId, SessionId};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Metrics for a single LLM request
///
/// Captures all timing and cost data for a single request including:
/// - TTFT (Time to First Token)
/// - Inter-token latencies
/// - Total request duration
/// - Token counts and throughput
/// - Cost tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Unique request identifier
    pub request_id: RequestId,

    /// Session identifier
    pub session_id: SessionId,

    /// Provider used for this request
    pub provider: Provider,

    /// Model name/ID
    pub model: String,

    /// Request timestamp
    pub timestamp: DateTime<Utc>,

    /// Time to first token (TTFT) in nanoseconds
    #[serde(with = "duration_nanos")]
    pub ttft: Duration,

    /// Total request latency (from request start to completion)
    #[serde(with = "duration_nanos")]
    pub total_latency: Duration,

    /// Inter-token latencies in nanoseconds (one per token after the first)
    #[serde(with = "duration_vec_nanos")]
    pub inter_token_latencies: Vec<Duration>,

    /// Number of input tokens
    pub input_tokens: u64,

    /// Number of output tokens generated
    pub output_tokens: u64,

    /// Number of thinking tokens (if applicable, e.g., Claude extended thinking)
    pub thinking_tokens: Option<u64>,

    /// Token generation throughput (tokens per second)
    pub tokens_per_second: f64,

    /// Estimated cost in USD (if available)
    pub cost_usd: Option<f64>,

    /// Whether the request completed successfully
    pub success: bool,

    /// Error message if request failed
    pub error: Option<String>,
}

impl RequestMetrics {
    /// Calculate the mean inter-token latency
    pub fn mean_inter_token_latency(&self) -> Option<Duration> {
        if self.inter_token_latencies.is_empty() {
            return None;
        }

        let total_nanos: u128 = self
            .inter_token_latencies
            .iter()
            .map(|d| d.as_nanos())
            .sum();
        let mean_nanos = total_nanos / self.inter_token_latencies.len() as u128;

        Some(Duration::from_nanos(mean_nanos as u64))
    }

    /// Calculate the median inter-token latency
    pub fn median_inter_token_latency(&self) -> Option<Duration> {
        if self.inter_token_latencies.is_empty() {
            return None;
        }

        let mut sorted = self.inter_token_latencies.clone();
        sorted.sort();

        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            // Even number of elements - average the two middle values
            let sum = sorted[mid - 1].as_nanos() + sorted[mid].as_nanos();
            Some(Duration::from_nanos((sum / 2) as u64))
        } else {
            // Odd number of elements - return the middle value
            Some(sorted[mid])
        }
    }

    /// Get the minimum inter-token latency
    pub fn min_inter_token_latency(&self) -> Option<Duration> {
        self.inter_token_latencies.iter().min().copied()
    }

    /// Get the maximum inter-token latency
    pub fn max_inter_token_latency(&self) -> Option<Duration> {
        self.inter_token_latencies.iter().max().copied()
    }

    /// Get total token count (input + output + thinking)
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.thinking_tokens.unwrap_or(0)
    }
}

/// Aggregated metrics across multiple requests
///
/// Provides statistical analysis of metrics collected from multiple requests:
/// - Percentile distributions (p50, p95, p99, p99.9)
/// - Mean and standard deviation
/// - Min/max values
/// - Total counts and throughput
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Session identifier
    pub session_id: SessionId,

    /// Start time of the aggregation period
    pub start_time: DateTime<Utc>,

    /// End time of the aggregation period
    pub end_time: DateTime<Utc>,

    /// Total number of requests in this aggregation
    pub total_requests: u64,

    /// Number of successful requests
    pub successful_requests: u64,

    /// Number of failed requests
    pub failed_requests: u64,

    /// TTFT (Time to First Token) distribution
    pub ttft_distribution: LatencyDistribution,

    /// Inter-token latency distribution
    pub inter_token_distribution: LatencyDistribution,

    /// Total request latency distribution
    pub total_latency_distribution: LatencyDistribution,

    /// Token throughput statistics
    pub throughput: ThroughputStats,

    /// Total input tokens processed
    pub total_input_tokens: u64,

    /// Total output tokens generated
    pub total_output_tokens: u64,

    /// Total thinking tokens (if applicable)
    pub total_thinking_tokens: Option<u64>,

    /// Total cost in USD (if available)
    pub total_cost_usd: Option<f64>,

    /// Provider breakdown (number of requests per provider)
    pub provider_breakdown: Vec<(Provider, u64)>,

    /// Model breakdown (number of requests per model)
    pub model_breakdown: Vec<(String, u64)>,
}

impl AggregatedMetrics {
    /// Calculate the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Calculate the duration of the aggregation period
    pub fn duration(&self) -> Duration {
        let diff = self.end_time - self.start_time;
        Duration::from_millis(diff.num_milliseconds().max(0) as u64)
    }

    /// Calculate average cost per request
    pub fn avg_cost_per_request(&self) -> Option<f64> {
        self.total_cost_usd
            .map(|cost| cost / self.successful_requests as f64)
    }

    /// Calculate average tokens per request
    pub fn avg_tokens_per_request(&self) -> f64 {
        if self.successful_requests == 0 {
            return 0.0;
        }
        let total = self.total_input_tokens + self.total_output_tokens;
        total as f64 / self.successful_requests as f64
    }
}

/// Latency distribution statistics
///
/// Contains percentile-based statistics calculated using HDR Histogram
/// for accurate representation of latency distributions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyDistribution {
    /// Minimum value observed
    #[serde(with = "duration_nanos")]
    pub min: Duration,

    /// Maximum value observed
    #[serde(with = "duration_nanos")]
    pub max: Duration,

    /// Mean (average) value
    #[serde(with = "duration_nanos")]
    pub mean: Duration,

    /// Standard deviation
    #[serde(with = "duration_nanos")]
    pub std_dev: Duration,

    /// 50th percentile (median)
    #[serde(with = "duration_nanos")]
    pub p50: Duration,

    /// 90th percentile
    #[serde(with = "duration_nanos")]
    pub p90: Duration,

    /// 95th percentile
    #[serde(with = "duration_nanos")]
    pub p95: Duration,

    /// 99th percentile
    #[serde(with = "duration_nanos")]
    pub p99: Duration,

    /// 99.9th percentile
    #[serde(with = "duration_nanos")]
    pub p99_9: Duration,

    /// Number of samples in this distribution
    pub sample_count: u64,
}

impl LatencyDistribution {
    /// Create a new empty latency distribution
    pub fn empty() -> Self {
        Self {
            min: Duration::ZERO,
            max: Duration::ZERO,
            mean: Duration::ZERO,
            std_dev: Duration::ZERO,
            p50: Duration::ZERO,
            p90: Duration::ZERO,
            p95: Duration::ZERO,
            p99: Duration::ZERO,
            p99_9: Duration::ZERO,
            sample_count: 0,
        }
    }

    /// Check if this distribution has any samples
    pub fn is_empty(&self) -> bool {
        self.sample_count == 0
    }

    /// Get the range (max - min)
    pub fn range(&self) -> Duration {
        self.max.saturating_sub(self.min)
    }
}

/// Token throughput statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputStats {
    /// Mean tokens per second
    pub mean_tokens_per_second: f64,

    /// Minimum tokens per second observed
    pub min_tokens_per_second: f64,

    /// Maximum tokens per second observed
    pub max_tokens_per_second: f64,

    /// Standard deviation of tokens per second
    pub std_dev_tokens_per_second: f64,

    /// 50th percentile tokens per second
    pub p50_tokens_per_second: f64,

    /// 95th percentile tokens per second
    pub p95_tokens_per_second: f64,

    /// 99th percentile tokens per second
    pub p99_tokens_per_second: f64,
}

impl ThroughputStats {
    /// Create a new empty throughput stats
    pub fn empty() -> Self {
        Self {
            mean_tokens_per_second: 0.0,
            min_tokens_per_second: 0.0,
            max_tokens_per_second: 0.0,
            std_dev_tokens_per_second: 0.0,
            p50_tokens_per_second: 0.0,
            p95_tokens_per_second: 0.0,
            p99_tokens_per_second: 0.0,
        }
    }
}

/// Serde module for Duration serialization to nanoseconds
mod duration_nanos {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_nanos() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nanos = u64::deserialize(deserializer)?;
        Ok(Duration::from_nanos(nanos))
    }
}

/// Serde module for Vec<Duration> serialization to nanoseconds
mod duration_vec_nanos {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(durations: &Vec<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let nanos: Vec<u64> = durations.iter().map(|d| d.as_nanos() as u64).collect();
        nanos.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nanos = Vec::<u64>::deserialize(deserializer)?;
        Ok(nanos.into_iter().map(Duration::from_nanos).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_latency_lens_core::Provider;

    #[test]
    fn test_request_metrics_serialization() {
        let metrics = RequestMetrics {
            request_id: RequestId::new(),
            session_id: SessionId::new(),
            provider: Provider::OpenAI,
            model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            ttft: Duration::from_millis(100),
            total_latency: Duration::from_millis(1000),
            inter_token_latencies: vec![
                Duration::from_millis(10),
                Duration::from_millis(15),
                Duration::from_millis(12),
            ],
            input_tokens: 100,
            output_tokens: 50,
            thinking_tokens: None,
            tokens_per_second: 50.0,
            cost_usd: Some(0.05),
            success: true,
            error: None,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: RequestMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(metrics.request_id, deserialized.request_id);
        assert_eq!(metrics.ttft, deserialized.ttft);
        assert_eq!(metrics.total_latency, deserialized.total_latency);
        assert_eq!(
            metrics.inter_token_latencies.len(),
            deserialized.inter_token_latencies.len()
        );
    }

    #[test]
    fn test_request_metrics_mean_inter_token_latency() {
        let metrics = RequestMetrics {
            request_id: RequestId::new(),
            session_id: SessionId::new(),
            provider: Provider::Anthropic,
            model: "claude-3-opus".to_string(),
            timestamp: Utc::now(),
            ttft: Duration::from_millis(100),
            total_latency: Duration::from_millis(1000),
            inter_token_latencies: vec![
                Duration::from_millis(10),
                Duration::from_millis(20),
                Duration::from_millis(30),
            ],
            input_tokens: 100,
            output_tokens: 3,
            thinking_tokens: None,
            tokens_per_second: 3.0,
            cost_usd: None,
            success: true,
            error: None,
        };

        let mean = metrics.mean_inter_token_latency().unwrap();
        assert_eq!(mean, Duration::from_millis(20));
    }

    #[test]
    fn test_request_metrics_median_inter_token_latency() {
        let metrics = RequestMetrics {
            request_id: RequestId::new(),
            session_id: SessionId::new(),
            provider: Provider::Google,
            model: "gemini-pro".to_string(),
            timestamp: Utc::now(),
            ttft: Duration::from_millis(100),
            total_latency: Duration::from_millis(1000),
            inter_token_latencies: vec![
                Duration::from_millis(10),
                Duration::from_millis(20),
                Duration::from_millis(30),
                Duration::from_millis(40),
                Duration::from_millis(50),
            ],
            input_tokens: 100,
            output_tokens: 5,
            thinking_tokens: None,
            tokens_per_second: 5.0,
            cost_usd: None,
            success: true,
            error: None,
        };

        let median = metrics.median_inter_token_latency().unwrap();
        assert_eq!(median, Duration::from_millis(30));
    }

    #[test]
    fn test_request_metrics_total_tokens() {
        let metrics = RequestMetrics {
            request_id: RequestId::new(),
            session_id: SessionId::new(),
            provider: Provider::Anthropic,
            model: "claude-3-5-sonnet-20241022".to_string(),
            timestamp: Utc::now(),
            ttft: Duration::from_millis(100),
            total_latency: Duration::from_millis(1000),
            inter_token_latencies: vec![],
            input_tokens: 100,
            output_tokens: 50,
            thinking_tokens: Some(200),
            tokens_per_second: 50.0,
            cost_usd: None,
            success: true,
            error: None,
        };

        assert_eq!(metrics.total_tokens(), 350);
    }

    #[test]
    fn test_aggregated_metrics_success_rate() {
        let metrics = AggregatedMetrics {
            session_id: SessionId::new(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            total_requests: 100,
            successful_requests: 95,
            failed_requests: 5,
            ttft_distribution: LatencyDistribution::empty(),
            inter_token_distribution: LatencyDistribution::empty(),
            total_latency_distribution: LatencyDistribution::empty(),
            throughput: ThroughputStats::empty(),
            total_input_tokens: 10000,
            total_output_tokens: 5000,
            total_thinking_tokens: None,
            total_cost_usd: Some(10.0),
            provider_breakdown: vec![],
            model_breakdown: vec![],
        };

        assert_eq!(metrics.success_rate(), 95.0);
    }

    #[test]
    fn test_latency_distribution_empty() {
        let dist = LatencyDistribution::empty();
        assert!(dist.is_empty());
        assert_eq!(dist.sample_count, 0);
        assert_eq!(dist.range(), Duration::ZERO);
    }

    #[test]
    fn test_latency_distribution_range() {
        let dist = LatencyDistribution {
            min: Duration::from_millis(10),
            max: Duration::from_millis(100),
            mean: Duration::from_millis(50),
            std_dev: Duration::from_millis(20),
            p50: Duration::from_millis(50),
            p90: Duration::from_millis(90),
            p95: Duration::from_millis(95),
            p99: Duration::from_millis(99),
            p99_9: Duration::from_millis(100),
            sample_count: 1000,
        };

        assert_eq!(dist.range(), Duration::from_millis(90));
        assert!(!dist.is_empty());
    }
}
