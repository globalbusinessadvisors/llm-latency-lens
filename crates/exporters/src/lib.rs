//! Multi-format export system for LLM Latency Lens
//!
//! This crate provides exporters for various output formats including:
//! - JSON (human-readable and compact)
//! - Console (beautiful colored table output)
//! - Prometheus (exposition format)
//! - CSV (for data analysis)
//!
//! # Example
//!
//! ```no_run
//! use llm_latency_lens_exporters::{Exporter, JsonExporter, ConsoleExporter};
//! use llm_latency_lens_metrics::AggregatedMetrics;
//!
//! # fn get_metrics() -> AggregatedMetrics { unimplemented!() }
//! let metrics = get_metrics();
//!
//! // Export to JSON
//! let json_exporter = JsonExporter::new(true); // pretty print
//! let json_output = json_exporter.export(&metrics).unwrap();
//!
//! // Export to console
//! let console_exporter = ConsoleExporter::new();
//! console_exporter.export(&metrics).unwrap();
//! ```

use llm_latency_lens_metrics::{AggregatedMetrics, RequestMetrics};
use thiserror::Error;

pub mod console;
pub mod csv;
pub mod json;
pub mod prometheus;

pub use console::ConsoleExporter;
pub use csv::CsvExporter;
pub use json::JsonExporter;
pub use prometheus::PrometheusExporter;

/// Errors that can occur during export
#[derive(Debug, Error)]
pub enum ExportError {
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Format error
    #[error("Format error: {0}")]
    Format(String),

    /// CSV error
    #[error("CSV error: {0}")]
    Csv(String),
}

/// Result type for export operations
pub type Result<T> = std::result::Result<T, ExportError>;

/// Trait for exporting metrics to different formats
pub trait Exporter {
    /// Export aggregated metrics to a string
    fn export(&self, metrics: &AggregatedMetrics) -> Result<String>;

    /// Export individual request metrics to a string
    fn export_requests(&self, requests: &[RequestMetrics]) -> Result<String>;

    /// Write metrics to a file
    fn export_to_file(
        &self,
        metrics: &AggregatedMetrics,
        path: &std::path::Path,
    ) -> Result<()> {
        let output = self.export(metrics)?;
        std::fs::write(path, output)?;
        Ok(())
    }

    /// Write request metrics to a file
    fn export_requests_to_file(
        &self,
        requests: &[RequestMetrics],
        path: &std::path::Path,
    ) -> Result<()> {
        let output = self.export_requests(requests)?;
        std::fs::write(path, output)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_latency_lens_core::{Provider, RequestId, SessionId};
    use llm_latency_lens_metrics::{LatencyDistribution, ThroughputStats};
    use std::time::Duration;

    pub(crate) fn create_test_metrics() -> AggregatedMetrics {
        AggregatedMetrics {
            session_id: SessionId::new(),
            start_time: chrono::Utc::now(),
            end_time: chrono::Utc::now() + chrono::Duration::seconds(10),
            total_requests: 10,
            successful_requests: 9,
            failed_requests: 1,
            ttft_distribution: LatencyDistribution {
                min: Duration::from_millis(100),
                max: Duration::from_millis(300),
                mean: Duration::from_millis(150),
                std_dev: Duration::from_millis(50),
                p50: Duration::from_millis(150),
                p90: Duration::from_millis(250),
                p95: Duration::from_millis(280),
                p99: Duration::from_millis(295),
                p999: Duration::from_millis(299),
            },
            inter_token_distribution: LatencyDistribution {
                min: Duration::from_millis(5),
                max: Duration::from_millis(20),
                mean: Duration::from_millis(10),
                std_dev: Duration::from_millis(3),
                p50: Duration::from_millis(10),
                p90: Duration::from_millis(15),
                p95: Duration::from_millis(18),
                p99: Duration::from_millis(19),
                p999: Duration::from_millis(20),
            },
            total_latency_distribution: LatencyDistribution {
                min: Duration::from_secs(1),
                max: Duration::from_secs(3),
                mean: Duration::from_secs(2),
                std_dev: Duration::from_millis(500),
                p50: Duration::from_secs(2),
                p90: Duration::from_millis(2500),
                p95: Duration::from_millis(2800),
                p99: Duration::from_millis(2950),
                p999: Duration::from_millis(2990),
            },
            throughput: ThroughputStats {
                mean_tokens_per_second: 50.0,
                min_tokens_per_second: 30.0,
                max_tokens_per_second: 70.0,
                p50_tokens_per_second: 50.0,
                p95_tokens_per_second: 65.0,
                p99_tokens_per_second: 68.0,
            },
            total_input_tokens: 1000,
            total_output_tokens: 2000,
            total_thinking_tokens: Some(100),
            total_cost_usd: Some(5.50),
            provider_breakdown: vec![
                (Provider::OpenAI, 5),
                (Provider::Anthropic, 4),
            ],
            model_breakdown: vec![
                ("gpt-4".to_string(), 5),
                ("claude-3-opus".to_string(), 4),
            ],
        }
    }

    pub(crate) fn create_test_requests() -> Vec<RequestMetrics> {
        vec![
            RequestMetrics {
                request_id: RequestId::new(),
                session_id: SessionId::new(),
                provider: Provider::OpenAI,
                model: "gpt-4".to_string(),
                timestamp: chrono::Utc::now(),
                ttft: Duration::from_millis(150),
                total_latency: Duration::from_secs(2),
                inter_token_latencies: vec![
                    Duration::from_millis(10),
                    Duration::from_millis(15),
                    Duration::from_millis(12),
                ],
                input_tokens: 100,
                output_tokens: 200,
                thinking_tokens: None,
                tokens_per_second: 50.0,
                cost_usd: Some(0.50),
                success: true,
                error: None,
            },
            RequestMetrics {
                request_id: RequestId::new(),
                session_id: SessionId::new(),
                provider: Provider::Anthropic,
                model: "claude-3-opus".to_string(),
                timestamp: chrono::Utc::now(),
                ttft: Duration::from_millis(180),
                total_latency: Duration::from_secs(3),
                inter_token_latencies: vec![
                    Duration::from_millis(8),
                    Duration::from_millis(12),
                    Duration::from_millis(10),
                ],
                input_tokens: 150,
                output_tokens: 300,
                thinking_tokens: Some(20),
                tokens_per_second: 55.0,
                cost_usd: Some(0.75),
                success: true,
                error: None,
            },
        ]
    }

    #[test]
    fn test_export_trait() {
        let metrics = create_test_metrics();
        let exporter = JsonExporter::new(false);

        let result = exporter.export(&metrics);
        assert!(result.is_ok());
    }
}
