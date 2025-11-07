//! Prometheus exposition format exporter
//!
//! Exports metrics in the Prometheus text-based exposition format.
//! See: https://prometheus.io/docs/instrumenting/exposition_formats/

use crate::{Exporter, Result};
use llm_latency_lens_metrics::{AggregatedMetrics, LatencyDistribution, RequestMetrics};
use std::fmt::Write;

/// Prometheus exporter
///
/// Exports metrics in Prometheus exposition format with proper metric naming
/// conventions and labels.
#[derive(Debug, Clone)]
pub struct PrometheusExporter {
    /// Metric name prefix
    prefix: String,
    /// Whether to include help text
    include_help: bool,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter with default settings
    pub fn new() -> Self {
        Self {
            prefix: "llm_latency_lens".to_string(),
            include_help: true,
        }
    }

    /// Create a Prometheus exporter with a custom prefix
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            include_help: true,
        }
    }

    /// Disable help text in output
    pub fn without_help(mut self) -> Self {
        self.include_help = false;
        self
    }

    /// Sanitize a label value for Prometheus
    fn sanitize_label_value(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    }

    /// Write a help comment
    fn write_help(&self, output: &mut String, metric_name: &str, help_text: &str) -> Result<()> {
        if self.include_help {
            writeln!(output, "# HELP {} {}", metric_name, help_text)
                .map_err(|e| crate::ExportError::Format(e.to_string()))?;
        }
        Ok(())
    }

    /// Write a type comment
    fn write_type(&self, output: &mut String, metric_name: &str, metric_type: &str) -> Result<()> {
        writeln!(output, "# TYPE {} {}", metric_name, metric_type)
            .map_err(|e| crate::ExportError::Format(e.to_string()))?;
        Ok(())
    }

    /// Convert duration to milliseconds
    fn duration_to_ms(nanos: u128) -> f64 {
        nanos as f64 / 1_000_000.0
    }

    /// Export summary statistics as Prometheus summary metric
    fn export_summary(
        &self,
        output: &mut String,
        metric_name: &str,
        help_text: &str,
        dist: &LatencyDistribution,
        labels: &[(&str, &str)],
    ) -> Result<()> {
        let full_metric_name = format!("{}_{}", self.prefix, metric_name);

        self.write_help(output, &full_metric_name, help_text)?;
        self.write_type(output, &full_metric_name, "summary")?;

        let label_str = if labels.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = labels
                .iter()
                .map(|(k, v)| format!(r#"{}="{}""#, k, Self::sanitize_label_value(v)))
                .collect();
            format!("{{{}}}", pairs.join(","))
        };

        // Sum (mean * count - we don't have count, so just use mean)
        writeln!(
            output,
            "{}_sum{} {}",
            full_metric_name, label_str, Self::duration_to_ms(dist.mean.as_nanos())
        )
        .map_err(|e| crate::ExportError::Format(e.to_string()))?;

        writeln!(
            output,
            "{}_count{} 1",
            full_metric_name, label_str
        )
        .map_err(|e| crate::ExportError::Format(e.to_string()))?;

        // Quantiles
        for (quantile, value) in [
            ("0.5", dist.p50.as_nanos()),
            ("0.9", dist.p90.as_nanos()),
            ("0.95", dist.p95.as_nanos()),
            ("0.99", dist.p99.as_nanos()),
            ("0.999", dist.p99_9.as_nanos()),
        ] {
            let quantile_labels = if labels.is_empty() {
                format!(r#"{{quantile="{}"}}"#, quantile)
            } else {
                let mut all_labels = labels.to_vec();
                all_labels.push(("quantile", quantile));
                let pairs: Vec<String> = all_labels
                    .iter()
                    .map(|(k, v)| format!(r#"{}="{}""#, k, Self::sanitize_label_value(v)))
                    .collect();
                format!("{{{}}}", pairs.join(","))
            };

            writeln!(output, "{}{} {}", full_metric_name, quantile_labels, Self::duration_to_ms(value))
                .map_err(|e| crate::ExportError::Format(e.to_string()))?;
        }

        Ok(())
    }

    /// Export a counter metric
    fn export_counter(
        &self,
        output: &mut String,
        metric_name: &str,
        help_text: &str,
        value: u64,
    ) -> Result<()> {
        let full_metric_name = format!("{}_{}", self.prefix, metric_name);

        self.write_help(output, &full_metric_name, help_text)?;
        self.write_type(output, &full_metric_name, "counter")?;

        writeln!(output, "{} {}", full_metric_name, value)
            .map_err(|e| crate::ExportError::Format(e.to_string()))?;

        Ok(())
    }
}

impl Default for PrometheusExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for PrometheusExporter {
    fn export(&self, metrics: &AggregatedMetrics) -> Result<String> {
        let mut output = String::new();

        // Total requests
        self.export_counter(
            &mut output,
            "requests_total",
            "Total number of requests",
            metrics.total_requests,
        )?;

        // Successful requests
        self.export_counter(
            &mut output,
            "requests_successful_total",
            "Total number of successful requests",
            metrics.successful_requests,
        )?;

        // Failed requests
        self.export_counter(
            &mut output,
            "requests_failed_total",
            "Total number of failed requests",
            metrics.failed_requests,
        )?;

        // Time to first token statistics
        self.export_summary(
            &mut output,
            "ttft_milliseconds",
            "Time to first token in milliseconds",
            &metrics.ttft_distribution,
            &[],
        )?;

        // Inter-token latency statistics
        self.export_summary(
            &mut output,
            "inter_token_latency_milliseconds",
            "Inter-token latency in milliseconds",
            &metrics.inter_token_distribution,
            &[],
        )?;

        // Total duration statistics
        self.export_summary(
            &mut output,
            "request_duration_milliseconds",
            "Total request duration in milliseconds",
            &metrics.total_latency_distribution,
            &[],
        )?;

        Ok(output)
    }

    fn export_requests(&self, requests: &[RequestMetrics]) -> Result<String> {
        let mut output = String::new();

        // Export individual request metrics as gauges
        let metric_name = format!("{}_request_info", self.prefix);
        self.write_help(&mut output, &metric_name, "Individual request information")?;
        self.write_type(&mut output, &metric_name, "gauge")?;

        for req in requests {
            writeln!(
                &mut output,
                r#"{}{{request_id="{}",provider="{}",model="{}",status="{}"}} 1"#,
                metric_name,
                &req.request_id.to_string()[..8],
                req.provider.as_str(),
                Self::sanitize_label_value(&req.model),
                if req.success { "success" } else { "failure" }
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
    fn test_prometheus_export() {
        let metrics = create_test_metrics();
        let exporter = PrometheusExporter::new();

        let result = exporter.export(&metrics).unwrap();
        assert!(result.contains("# HELP"));
        assert!(result.contains("# TYPE"));
        assert!(result.contains("llm_latency_lens_requests_total"));
        assert!(result.contains("llm_latency_lens_ttft_milliseconds"));
    }

    #[test]
    fn test_prometheus_export_without_help() {
        let metrics = create_test_metrics();
        let exporter = PrometheusExporter::new().without_help();

        let result = exporter.export(&metrics).unwrap();
        assert!(!result.contains("# HELP"));
        assert!(result.contains("# TYPE"));
    }

    #[test]
    fn test_prometheus_export_custom_prefix() {
        let metrics = create_test_metrics();
        let exporter = PrometheusExporter::with_prefix("my_app");

        let result = exporter.export(&metrics).unwrap();
        assert!(result.contains("my_app_requests_total"));
        assert!(!result.contains("llm_latency_lens"));
    }

    #[test]
    fn test_prometheus_export_requests() {
        let requests = create_test_requests();
        let exporter = PrometheusExporter::new();

        let result = exporter.export_requests(&requests).unwrap();
        assert!(result.contains("request_info"));
        assert!(result.contains(r#"model="gpt-4""#));
        assert!(result.contains(r#"model="claude-3-opus""#));
    }

    #[test]
    fn test_sanitize_label_value() {
        assert_eq!(
            PrometheusExporter::sanitize_label_value(r#"test"value"#),
            r#"test\"value"#
        );
        assert_eq!(
            PrometheusExporter::sanitize_label_value("test\nvalue"),
            r"test\nvalue"
        );
        assert_eq!(
            PrometheusExporter::sanitize_label_value(r"test\value"),
            r"test\\value"
        );
    }
}
