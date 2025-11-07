//! CSV exporter for data analysis
//!
//! Exports metrics in CSV format suitable for spreadsheet analysis and data processing.

use crate::{Exporter, Result};
use llm_latency_lens_metrics::{AggregatedMetrics, LatencyDistribution, RequestMetrics};
use std::fmt::Write;

/// CSV exporter
///
/// Exports metrics in CSV format with proper escaping and headers.
#[derive(Debug, Clone)]
pub struct CsvExporter {
    /// CSV delimiter
    delimiter: char,
    /// Whether to include headers
    include_headers: bool,
}

impl CsvExporter {
    /// Create a new CSV exporter with comma delimiter
    pub fn new() -> Self {
        Self {
            delimiter: ',',
            include_headers: true,
        }
    }

    /// Create a CSV exporter with tab delimiter (TSV)
    pub fn tab_separated() -> Self {
        Self {
            delimiter: '\t',
            include_headers: true,
        }
    }

    /// Create a CSV exporter with custom delimiter
    pub fn with_delimiter(delimiter: char) -> Self {
        Self {
            delimiter,
            include_headers: true,
        }
    }

    /// Disable headers in output
    pub fn without_headers(mut self) -> Self {
        self.include_headers = false;
        self
    }

    /// Escape a CSV field
    fn escape_field(&self, field: &str) -> String {
        if field.contains(self.delimiter)
            || field.contains('"')
            || field.contains('\n')
            || field.contains('\r')
        {
            format!(r#""{}""#, field.replace('"', r#""""#))
        } else {
            field.to_string()
        }
    }

    /// Join fields with delimiter
    fn join_fields(&self, fields: &[String]) -> String {
        fields.join(&self.delimiter.to_string())
    }

    /// Convert duration to milliseconds
    fn duration_to_ms(nanos: u128) -> f64 {
        nanos as f64 / 1_000_000.0
    }

    /// Export latency statistics to CSV rows
    fn export_latency_stats_csv(&self, metrics: &AggregatedMetrics) -> Result<String> {
        let mut output = String::new();

        if self.include_headers {
            writeln!(
                output,
                "{}",
                self.join_fields(&[
                    "metric".to_string(),
                    "min_ms".to_string(),
                    "mean_ms".to_string(),
                    "p50_ms".to_string(),
                    "p90_ms".to_string(),
                    "p95_ms".to_string(),
                    "p99_ms".to_string(),
                    "p999_ms".to_string(),
                    "max_ms".to_string(),
                    "std_dev_ms".to_string(),
                ])
            )
            .map_err(|e| crate::ExportError::Format(e.to_string()))?;
        }

        // Helper to export a distribution
        let export_dist = |name: &str, dist: &LatencyDistribution| -> Vec<String> {
            vec![
                name.to_string(),
                format!("{:.3}", Self::duration_to_ms(dist.min.as_nanos())),
                format!("{:.3}", Self::duration_to_ms(dist.mean.as_nanos())),
                format!("{:.3}", Self::duration_to_ms(dist.p50.as_nanos())),
                format!("{:.3}", Self::duration_to_ms(dist.p90.as_nanos())),
                format!("{:.3}", Self::duration_to_ms(dist.p95.as_nanos())),
                format!("{:.3}", Self::duration_to_ms(dist.p99.as_nanos())),
                format!("{:.3}", Self::duration_to_ms(dist.p99_9.as_nanos())),
                format!("{:.3}", Self::duration_to_ms(dist.max.as_nanos())),
                format!("{:.3}", Self::duration_to_ms(dist.std_dev.as_nanos())),
            ]
        };

        // TTFT
        writeln!(
            output,
            "{}",
            self.join_fields(&export_dist("ttft", &metrics.ttft_distribution))
        )
        .map_err(|e| crate::ExportError::Format(e.to_string()))?;

        // Inter-token
        writeln!(
            output,
            "{}",
            self.join_fields(&export_dist("inter_token", &metrics.inter_token_distribution))
        )
        .map_err(|e| crate::ExportError::Format(e.to_string()))?;

        // Total latency
        writeln!(
            output,
            "{}",
            self.join_fields(&export_dist("total_latency", &metrics.total_latency_distribution))
        )
        .map_err(|e| crate::ExportError::Format(e.to_string()))?;

        Ok(output)
    }
}

impl Default for CsvExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for CsvExporter {
    fn export(&self, metrics: &AggregatedMetrics) -> Result<String> {
        // For aggregated metrics, export latency statistics
        self.export_latency_stats_csv(metrics)
    }

    fn export_requests(&self, requests: &[RequestMetrics]) -> Result<String> {
        let mut output = String::new();

        if self.include_headers {
            writeln!(
                output,
                "{}",
                self.join_fields(&[
                    "request_id".to_string(),
                    "session_id".to_string(),
                    "provider".to_string(),
                    "model".to_string(),
                    "timestamp".to_string(),
                    "success".to_string(),
                    "ttft_ms".to_string(),
                    "total_latency_ms".to_string(),
                    "input_tokens".to_string(),
                    "output_tokens".to_string(),
                    "thinking_tokens".to_string(),
                    "tokens_per_second".to_string(),
                    "cost_usd".to_string(),
                    "error".to_string(),
                ])
            )
            .map_err(|e| crate::ExportError::Format(e.to_string()))?;
        }

        for req in requests {
            writeln!(
                output,
                "{}",
                self.join_fields(&[
                    self.escape_field(&req.request_id.to_string()),
                    self.escape_field(&req.session_id.to_string()),
                    self.escape_field(req.provider.as_str()),
                    self.escape_field(&req.model),
                    self.escape_field(&req.timestamp.to_rfc3339()),
                    req.success.to_string(),
                    format!("{:.3}", Self::duration_to_ms(req.ttft.as_nanos())),
                    format!("{:.3}", Self::duration_to_ms(req.total_latency.as_nanos())),
                    req.input_tokens.to_string(),
                    req.output_tokens.to_string(),
                    req.thinking_tokens.map_or(String::new(), |t| t.to_string()),
                    format!("{:.3}", req.tokens_per_second),
                    req.cost_usd.map_or(String::new(), |c| format!("{:.4}", c)),
                    self.escape_field(&req.error.as_deref().unwrap_or("")),
                ])
            )
            .map_err(|e| crate::ExportError::Format(e.to_string()))?;
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{create_test_metrics, create_test_requests};

    #[test]
    fn test_csv_export() {
        let metrics = create_test_metrics();
        let exporter = CsvExporter::new();

        let result = exporter.export(&metrics).unwrap();
        assert!(result.contains("metric,min_ms"));
        assert!(result.contains("ttft,"));
        assert!(result.contains("inter_token,"));
    }

    #[test]
    fn test_csv_export_requests() {
        let requests = create_test_requests();
        let exporter = CsvExporter::new();

        let result = exporter.export_requests(&requests).unwrap();
        assert!(result.contains("request_id,session_id,provider"));
        assert!(result.contains("gpt-4"));
        assert!(result.contains("claude-3-opus"));
        assert!(result.contains("openai"));
        assert!(result.contains("anthropic"));
    }

    #[test]
    fn test_csv_export_tab_separated() {
        let requests = create_test_requests();
        let exporter = CsvExporter::tab_separated();

        let result = exporter.export_requests(&requests).unwrap();
        assert!(result.contains('\t'));
        assert!(!result.contains(',') || result.contains("request_id"));
    }

    #[test]
    fn test_csv_export_without_headers() {
        let metrics = create_test_metrics();
        let exporter = CsvExporter::new().without_headers();

        let result = exporter.export(&metrics).unwrap();
        assert!(!result.contains("metric,min_ms"));
        assert!(result.contains("ttft,"));
    }

    #[test]
    fn test_csv_escape_field() {
        let exporter = CsvExporter::new();

        // No escaping needed
        assert_eq!(exporter.escape_field("simple"), "simple");

        // Contains comma
        assert_eq!(exporter.escape_field("hello,world"), r#""hello,world""#);

        // Contains quotes
        assert_eq!(
            exporter.escape_field(r#"hello"world"#),
            r#""hello""world""#
        );

        // Contains newline
        assert_eq!(exporter.escape_field("hello\nworld"), r#""hello
world""#);
    }

    #[test]
    fn test_csv_custom_delimiter() {
        let requests = create_test_requests();
        let exporter = CsvExporter::with_delimiter(';');

        let result = exporter.export_requests(&requests).unwrap();
        assert!(result.contains(';'));
    }
}
