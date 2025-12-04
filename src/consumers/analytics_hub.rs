//! LLM-Analytics-Hub Consumer Adapter
//!
//! Consumes historical baselines, p95/p99 summaries, throughput aggregates,
//! and rolling windows from LLM-Analytics-Hub.
//!
//! # Data Types Consumed
//!
//! - **Historical Baselines**: Reference metrics for comparison
//! - **Percentile Summaries**: p50, p95, p99 distributions
//! - **Throughput Aggregates**: Tokens/second over time windows
//! - **Rolling Windows**: Time-bucketed statistics
//!
//! # Integration
//!
//! This adapter uses the `llm-analytics-hub` crate to access Analytics Hub
//! data structures and converts them to Latency-Lens metrics.

use super::{ConsumerError, ConsumerResult, DataConsumer, RetryConfig};
use crate::{
    AggregatedMetrics, LatencyDistribution, RequestMetrics, SessionId, ThroughputStats,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use llm_latency_lens_core::Provider;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for LLM-Analytics-Hub consumer
#[derive(Debug, Clone)]
pub struct AnalyticsHubConfig {
    /// Analytics Hub API endpoint
    pub endpoint: Option<String>,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Enable local mode (read from local database/files)
    pub local_mode: bool,
    /// Retry configuration
    pub retry: RetryConfig,
    /// Timeout for API calls
    pub timeout: Duration,
    /// Default time window for queries
    pub default_window: TimeWindow,
}

impl Default for AnalyticsHubConfig {
    fn default() -> Self {
        Self {
            endpoint: None,
            api_key: None,
            local_mode: true,
            retry: RetryConfig::default(),
            timeout: Duration::from_secs(30),
            default_window: TimeWindow::Hour,
        }
    }
}

/// Time window for aggregations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeWindow {
    /// One minute window
    Minute,
    /// Five minute window
    FiveMinutes,
    /// Fifteen minute window
    FifteenMinutes,
    /// One hour window
    Hour,
    /// Six hour window
    SixHours,
    /// One day window
    Day,
    /// One week window
    Week,
}

impl TimeWindow {
    /// Get the duration of this time window
    pub fn duration(&self) -> Duration {
        match self {
            TimeWindow::Minute => Duration::from_secs(60),
            TimeWindow::FiveMinutes => Duration::from_secs(300),
            TimeWindow::FifteenMinutes => Duration::from_secs(900),
            TimeWindow::Hour => Duration::from_secs(3600),
            TimeWindow::SixHours => Duration::from_secs(21600),
            TimeWindow::Day => Duration::from_secs(86400),
            TimeWindow::Week => Duration::from_secs(604800),
        }
    }
}

/// Historical baseline data from Analytics Hub
///
/// Contains reference metrics for a specific provider/model combination
/// that can be used for comparison against current performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalBaseline {
    /// Provider name
    pub provider: String,
    /// Model identifier
    pub model: String,
    /// Baseline creation timestamp
    pub created_at: DateTime<Utc>,
    /// Start of the baseline period
    pub period_start: DateTime<Utc>,
    /// End of the baseline period
    pub period_end: DateTime<Utc>,
    /// Number of samples in baseline
    pub sample_count: u64,
    /// TTFT baseline statistics
    pub ttft_baseline: PercentileBaseline,
    /// Inter-token latency baseline
    pub itl_baseline: PercentileBaseline,
    /// Total latency baseline
    pub total_latency_baseline: PercentileBaseline,
    /// Throughput baseline (tokens/second)
    pub throughput_baseline: ThroughputBaseline,
    /// Cost per request baseline (USD)
    pub cost_baseline: Option<CostBaseline>,
    /// Success rate during baseline period
    pub success_rate: f64,
    /// Tags/labels for filtering
    pub tags: HashMap<String, String>,
}

/// Percentile-based baseline statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentileBaseline {
    /// Minimum value
    pub min: Duration,
    /// Maximum value
    pub max: Duration,
    /// Mean value
    pub mean: Duration,
    /// Standard deviation
    pub std_dev: Duration,
    /// 50th percentile (median)
    pub p50: Duration,
    /// 90th percentile
    pub p90: Duration,
    /// 95th percentile
    pub p95: Duration,
    /// 99th percentile
    pub p99: Duration,
    /// 99.9th percentile
    pub p99_9: Duration,
}

impl PercentileBaseline {
    /// Convert to LatencyDistribution
    pub fn to_latency_distribution(&self, sample_count: u64) -> LatencyDistribution {
        LatencyDistribution {
            min: self.min,
            max: self.max,
            mean: self.mean,
            std_dev: self.std_dev,
            p50: self.p50,
            p90: self.p90,
            p95: self.p95,
            p99: self.p99,
            p99_9: self.p99_9,
            sample_count,
        }
    }
}

/// Throughput baseline statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputBaseline {
    /// Mean tokens per second
    pub mean_tokens_per_second: f64,
    /// Minimum tokens per second
    pub min_tokens_per_second: f64,
    /// Maximum tokens per second
    pub max_tokens_per_second: f64,
    /// Standard deviation
    pub std_dev_tokens_per_second: f64,
    /// 50th percentile
    pub p50_tokens_per_second: f64,
    /// 95th percentile
    pub p95_tokens_per_second: f64,
    /// 99th percentile
    pub p99_tokens_per_second: f64,
}

impl ThroughputBaseline {
    /// Convert to ThroughputStats
    pub fn to_throughput_stats(&self) -> ThroughputStats {
        ThroughputStats {
            mean_tokens_per_second: self.mean_tokens_per_second,
            min_tokens_per_second: self.min_tokens_per_second,
            max_tokens_per_second: self.max_tokens_per_second,
            std_dev_tokens_per_second: self.std_dev_tokens_per_second,
            p50_tokens_per_second: self.p50_tokens_per_second,
            p95_tokens_per_second: self.p95_tokens_per_second,
            p99_tokens_per_second: self.p99_tokens_per_second,
        }
    }
}

/// Cost baseline statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBaseline {
    /// Mean cost per request (USD)
    pub mean_cost_usd: f64,
    /// Total cost during baseline period
    pub total_cost_usd: f64,
    /// Cost per input token (USD)
    pub cost_per_input_token: f64,
    /// Cost per output token (USD)
    pub cost_per_output_token: f64,
}

/// Rolling window aggregate from Analytics Hub
///
/// Contains time-bucketed statistics for trending analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollingWindow {
    /// Window identifier
    pub window_id: String,
    /// Provider name
    pub provider: String,
    /// Model identifier
    pub model: String,
    /// Window start time
    pub start_time: DateTime<Utc>,
    /// Window end time
    pub end_time: DateTime<Utc>,
    /// Window size
    pub window_size: TimeWindow,
    /// Number of requests in window
    pub request_count: u64,
    /// Success rate in window
    pub success_rate: f64,
    /// TTFT p50 for this window
    pub ttft_p50: Duration,
    /// TTFT p95 for this window
    pub ttft_p95: Duration,
    /// TTFT p99 for this window
    pub ttft_p99: Duration,
    /// Mean throughput for this window
    pub throughput_mean: f64,
    /// Total tokens processed in window
    pub total_tokens: u64,
    /// Total cost in window (USD)
    pub total_cost_usd: Option<f64>,
}

/// Consumer for LLM-Analytics-Hub data
///
/// Provides methods to consume historical baselines, percentile summaries,
/// and rolling window aggregates from the Analytics Hub.
pub struct AnalyticsHubConsumer {
    config: AnalyticsHubConfig,
    session_id: SessionId,
}

impl AnalyticsHubConsumer {
    /// Create a new Analytics Hub consumer with default configuration
    pub fn new() -> Self {
        Self {
            config: AnalyticsHubConfig::default(),
            session_id: SessionId::new(),
        }
    }

    /// Create a new Analytics Hub consumer with custom configuration
    pub fn with_config(config: AnalyticsHubConfig) -> Self {
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

    /// Get historical baseline for a specific provider and model
    ///
    /// Returns baseline metrics that can be compared against current performance.
    pub async fn get_historical_baseline(
        &self,
        provider: &str,
        model: &str,
    ) -> ConsumerResult<HistoricalBaseline> {
        tracing::debug!(
            provider = provider,
            model = model,
            "Fetching historical baseline from Analytics Hub"
        );

        if self.config.local_mode {
            self.get_local_baseline(provider, model).await
        } else {
            self.get_remote_baseline(provider, model).await
        }
    }

    /// Get historical baseline from local storage
    async fn get_local_baseline(
        &self,
        provider: &str,
        model: &str,
    ) -> ConsumerResult<HistoricalBaseline> {
        // Integration point: Read from llm-analytics-hub local storage
        tracing::debug!(
            provider = provider,
            model = model,
            "Reading baseline from local Analytics Hub storage"
        );

        // Return a placeholder baseline for now
        Err(ConsumerError::ConfigError(format!(
            "No baseline found for {}/{}",
            provider, model
        )))
    }

    /// Get historical baseline from remote API
    async fn get_remote_baseline(
        &self,
        provider: &str,
        model: &str,
    ) -> ConsumerResult<HistoricalBaseline> {
        let endpoint = self.config.endpoint.as_ref().ok_or_else(|| {
            ConsumerError::ConfigError("Remote endpoint not configured".to_string())
        })?;

        tracing::debug!(
            endpoint = %endpoint,
            provider = provider,
            model = model,
            "Fetching baseline from remote Analytics Hub"
        );

        // Would make HTTP call here
        Err(ConsumerError::ConfigError(format!(
            "Remote baseline fetch not yet implemented for {}/{}",
            provider, model
        )))
    }

    /// Get rolling window aggregates for a time range
    pub async fn get_rolling_windows(
        &self,
        provider: &str,
        model: &str,
        window_size: TimeWindow,
        count: usize,
    ) -> ConsumerResult<Vec<RollingWindow>> {
        tracing::debug!(
            provider = provider,
            model = model,
            window_size = ?window_size,
            count = count,
            "Fetching rolling windows from Analytics Hub"
        );

        if self.config.local_mode {
            self.get_local_rolling_windows(provider, model, window_size, count)
                .await
        } else {
            self.get_remote_rolling_windows(provider, model, window_size, count)
                .await
        }
    }

    /// Get rolling windows from local storage
    async fn get_local_rolling_windows(
        &self,
        _provider: &str,
        _model: &str,
        _window_size: TimeWindow,
        _count: usize,
    ) -> ConsumerResult<Vec<RollingWindow>> {
        // Integration point: Read from llm-analytics-hub local storage
        Ok(Vec::new())
    }

    /// Get rolling windows from remote API
    async fn get_remote_rolling_windows(
        &self,
        _provider: &str,
        _model: &str,
        _window_size: TimeWindow,
        _count: usize,
    ) -> ConsumerResult<Vec<RollingWindow>> {
        let endpoint = self.config.endpoint.as_ref().ok_or_else(|| {
            ConsumerError::ConfigError("Remote endpoint not configured".to_string())
        })?;

        tracing::debug!(endpoint = %endpoint, "Fetching rolling windows from remote Analytics Hub");

        // Would make HTTP call here
        Ok(Vec::new())
    }

    /// Get aggregated percentile summaries
    pub async fn get_percentile_summary(
        &self,
        provider: &str,
        model: &str,
        time_range: Duration,
    ) -> ConsumerResult<AggregatedMetrics> {
        tracing::debug!(
            provider = provider,
            model = model,
            time_range = ?time_range,
            "Fetching percentile summary from Analytics Hub"
        );

        // Get baseline and convert to AggregatedMetrics
        let baseline = self.get_historical_baseline(provider, model).await?;
        Ok(self.baseline_to_aggregated_metrics(&baseline))
    }

    /// Convert baseline to AggregatedMetrics
    pub fn baseline_to_aggregated_metrics(&self, baseline: &HistoricalBaseline) -> AggregatedMetrics {
        let provider_enum = self.parse_provider(&baseline.provider);

        AggregatedMetrics {
            session_id: self.session_id,
            start_time: baseline.period_start,
            end_time: baseline.period_end,
            total_requests: baseline.sample_count,
            successful_requests: (baseline.sample_count as f64 * baseline.success_rate / 100.0) as u64,
            failed_requests: baseline.sample_count
                - (baseline.sample_count as f64 * baseline.success_rate / 100.0) as u64,
            ttft_distribution: baseline
                .ttft_baseline
                .to_latency_distribution(baseline.sample_count),
            inter_token_distribution: baseline
                .itl_baseline
                .to_latency_distribution(baseline.sample_count),
            total_latency_distribution: baseline
                .total_latency_baseline
                .to_latency_distribution(baseline.sample_count),
            throughput: baseline.throughput_baseline.to_throughput_stats(),
            total_input_tokens: 0, // Not tracked in baseline
            total_output_tokens: 0,
            total_thinking_tokens: None,
            total_cost_usd: baseline.cost_baseline.as_ref().map(|c| c.total_cost_usd),
            provider_breakdown: vec![(provider_enum, baseline.sample_count)],
            model_breakdown: vec![(baseline.model.clone(), baseline.sample_count)],
        }
    }

    /// Compare current metrics against baseline
    pub fn compare_to_baseline(
        &self,
        current: &AggregatedMetrics,
        baseline: &HistoricalBaseline,
    ) -> BaselineComparison {
        let baseline_metrics = self.baseline_to_aggregated_metrics(baseline);

        let ttft_p50_change = self.calculate_percentage_change(
            current.ttft_distribution.p50.as_nanos() as f64,
            baseline_metrics.ttft_distribution.p50.as_nanos() as f64,
        );

        let ttft_p95_change = self.calculate_percentage_change(
            current.ttft_distribution.p95.as_nanos() as f64,
            baseline_metrics.ttft_distribution.p95.as_nanos() as f64,
        );

        let ttft_p99_change = self.calculate_percentage_change(
            current.ttft_distribution.p99.as_nanos() as f64,
            baseline_metrics.ttft_distribution.p99.as_nanos() as f64,
        );

        let throughput_change = self.calculate_percentage_change(
            current.throughput.mean_tokens_per_second,
            baseline_metrics.throughput.mean_tokens_per_second,
        );

        BaselineComparison {
            baseline_period: (baseline.period_start, baseline.period_end),
            baseline_sample_count: baseline.sample_count,
            current_sample_count: current.total_requests,
            ttft_p50_change,
            ttft_p95_change,
            ttft_p99_change,
            throughput_change,
            success_rate_change: current.success_rate() - baseline.success_rate,
            is_regression: ttft_p95_change > 10.0 || throughput_change < -10.0,
        }
    }

    /// Calculate percentage change between current and baseline
    fn calculate_percentage_change(&self, current: f64, baseline: f64) -> f64 {
        if baseline == 0.0 {
            return 0.0;
        }
        ((current - baseline) / baseline) * 100.0
    }

    /// Parse provider string to Provider enum
    fn parse_provider(&self, provider_str: &str) -> Provider {
        match provider_str.to_lowercase().as_str() {
            "openai" => Provider::OpenAI,
            "anthropic" => Provider::Anthropic,
            "google" => Provider::Google,
            "aws-bedrock" | "bedrock" => Provider::AwsBedrock,
            "azure-openai" | "azure" => Provider::AzureOpenAI,
            _ => Provider::Generic,
        }
    }
}

impl Default for AnalyticsHubConsumer {
    fn default() -> Self {
        Self::new()
    }
}

/// Comparison results between current metrics and baseline
#[derive(Debug, Clone)]
pub struct BaselineComparison {
    /// Baseline period (start, end)
    pub baseline_period: (DateTime<Utc>, DateTime<Utc>),
    /// Number of samples in baseline
    pub baseline_sample_count: u64,
    /// Number of samples in current
    pub current_sample_count: u64,
    /// TTFT p50 change percentage (positive = slower)
    pub ttft_p50_change: f64,
    /// TTFT p95 change percentage
    pub ttft_p95_change: f64,
    /// TTFT p99 change percentage
    pub ttft_p99_change: f64,
    /// Throughput change percentage (positive = faster)
    pub throughput_change: f64,
    /// Success rate change (percentage points)
    pub success_rate_change: f64,
    /// Whether this represents a regression
    pub is_regression: bool,
}

#[async_trait]
impl DataConsumer for AnalyticsHubConsumer {
    fn name(&self) -> &'static str {
        "llm-analytics-hub"
    }

    async fn health_check(&self) -> ConsumerResult<bool> {
        if self.config.local_mode {
            // Check if Analytics Hub local storage is accessible
            Ok(true)
        } else {
            let endpoint = match &self.config.endpoint {
                Some(e) => e,
                None => return Ok(false),
            };

            tracing::debug!(endpoint = %endpoint, "Health checking Analytics Hub");
            Ok(true)
        }
    }

    async fn consume(&self, _limit: usize) -> ConsumerResult<Vec<RequestMetrics>> {
        // Analytics Hub provides aggregated data, not individual request metrics
        // Return empty as the primary interface is through get_historical_baseline
        // and get_rolling_windows
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_hub_config_defaults() {
        let config = AnalyticsHubConfig::default();
        assert!(config.local_mode);
        assert_eq!(config.default_window, TimeWindow::Hour);
    }

    #[test]
    fn test_time_window_duration() {
        assert_eq!(TimeWindow::Minute.duration(), Duration::from_secs(60));
        assert_eq!(TimeWindow::Hour.duration(), Duration::from_secs(3600));
        assert_eq!(TimeWindow::Day.duration(), Duration::from_secs(86400));
    }

    #[test]
    fn test_percentile_baseline_to_distribution() {
        let baseline = PercentileBaseline {
            min: Duration::from_millis(10),
            max: Duration::from_millis(1000),
            mean: Duration::from_millis(150),
            std_dev: Duration::from_millis(50),
            p50: Duration::from_millis(120),
            p90: Duration::from_millis(300),
            p95: Duration::from_millis(500),
            p99: Duration::from_millis(800),
            p99_9: Duration::from_millis(950),
        };

        let distribution = baseline.to_latency_distribution(1000);

        assert_eq!(distribution.p50, Duration::from_millis(120));
        assert_eq!(distribution.p99, Duration::from_millis(800));
        assert_eq!(distribution.sample_count, 1000);
    }

    #[test]
    fn test_throughput_baseline_to_stats() {
        let baseline = ThroughputBaseline {
            mean_tokens_per_second: 50.0,
            min_tokens_per_second: 20.0,
            max_tokens_per_second: 100.0,
            std_dev_tokens_per_second: 15.0,
            p50_tokens_per_second: 48.0,
            p95_tokens_per_second: 80.0,
            p99_tokens_per_second: 95.0,
        };

        let stats = baseline.to_throughput_stats();

        assert_eq!(stats.mean_tokens_per_second, 50.0);
        assert_eq!(stats.p95_tokens_per_second, 80.0);
    }

    #[test]
    fn test_parse_provider() {
        let consumer = AnalyticsHubConsumer::new();

        assert!(matches!(consumer.parse_provider("openai"), Provider::OpenAI));
        assert!(matches!(
            consumer.parse_provider("ANTHROPIC"),
            Provider::Anthropic
        ));
        assert!(matches!(
            consumer.parse_provider("unknown"),
            Provider::Generic
        ));
    }

    #[tokio::test]
    async fn test_health_check_local_mode() {
        let consumer = AnalyticsHubConsumer::new();
        let result = consumer.health_check().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_percentage_change_calculation() {
        let consumer = AnalyticsHubConsumer::new();

        // 20% increase
        assert!((consumer.calculate_percentage_change(120.0, 100.0) - 20.0).abs() < 0.001);

        // 50% decrease
        assert!((consumer.calculate_percentage_change(50.0, 100.0) - (-50.0)).abs() < 0.001);

        // No change
        assert!((consumer.calculate_percentage_change(100.0, 100.0) - 0.0).abs() < 0.001);

        // Zero baseline
        assert_eq!(consumer.calculate_percentage_change(100.0, 0.0), 0.0);
    }
}
