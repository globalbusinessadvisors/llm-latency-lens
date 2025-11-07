//! Beautiful console output with colored tables
//!
//! Provides formatted, colored terminal output for metrics visualization.

use crate::{Exporter, Result};
use colored::Colorize;
use llm_latency_lens_metrics::{AggregatedMetrics, LatencyDistribution, RequestMetrics};
use tabled::{
    builder::Builder,
    settings::{object::Rows, Color, Modify, Style},
};

/// Console exporter for beautiful terminal output
#[derive(Debug, Clone)]
pub struct ConsoleExporter {
    /// Whether to use colors
    colored: bool,
}

impl ConsoleExporter {
    /// Create a new console exporter with colors enabled
    pub fn new() -> Self {
        Self { colored: true }
    }

    /// Create a console exporter without colors (for piping/logging)
    pub fn no_color() -> Self {
        Self { colored: false }
    }

    /// Format a duration in milliseconds with appropriate precision
    fn format_duration_ms(nanos: u128) -> String {
        let ms = nanos as f64 / 1_000_000.0;
        if ms < 1.0 {
            format!("{:.3} ms", ms)
        } else if ms < 10.0 {
            format!("{:.2} ms", ms)
        } else if ms < 1000.0 {
            format!("{:.1} ms", ms)
        } else {
            format!("{:.2} s", ms / 1000.0)
        }
    }

    /// Format a percentage
    fn format_percent(value: f64, total: f64) -> String {
        if total == 0.0 {
            "0.0%".to_string()
        } else {
            format!("{:.1}%", (value / total) * 100.0)
        }
    }

    /// Create a section header
    fn section_header(&self, title: &str) -> String {
        if self.colored {
            format!("\n{}\n{}", title.bold().cyan(), "=".repeat(title.len()))
        } else {
            format!("\n{}\n{}", title, "=".repeat(title.len()))
        }
    }

    /// Create a summary table from aggregated metrics
    fn create_summary_table(&self, metrics: &AggregatedMetrics) -> String {
        let mut builder = Builder::default();

        // Header
        builder.push_record(["Metric", "Value"]);

        // Basic stats
        builder.push_record([
            "Total Requests",
            &metrics.total_requests.to_string(),
        ]);
        builder.push_record([
            "Successful",
            &format!(
                "{} ({})",
                metrics.successful_requests,
                Self::format_percent(
                    metrics.successful_requests as f64,
                    metrics.total_requests as f64
                )
            ),
        ]);
        builder.push_record([
            "Failed",
            &format!(
                "{} ({})",
                metrics.failed_requests,
                Self::format_percent(
                    metrics.failed_requests as f64,
                    metrics.total_requests as f64
                )
            ),
        ]);

        // Duration
        builder.push_record([
            "Duration",
            &format!("{:.1}s", metrics.duration().as_secs_f64()),
        ]);

        // Token stats
        builder.push_record([
            "Total Tokens",
            &(metrics.total_input_tokens + metrics.total_output_tokens).to_string(),
        ]);

        if let Some(cost) = metrics.total_cost_usd {
            builder.push_record([
                "Total Cost",
                &format!("${:.4}", cost),
            ]);
            if let Some(avg_cost) = metrics.avg_cost_per_request() {
                builder.push_record([
                    "Avg Cost/Request",
                    &format!("${:.4}", avg_cost),
                ]);
            }
        }

        let mut table = builder.build();
        table.with(Style::rounded());

        if self.colored {
            table.with(Modify::new(Rows::first()).with(Color::BOLD));
        }

        table.to_string()
    }

    /// Format latency distribution for table
    fn format_latency_dist(dist: &LatencyDistribution) -> Vec<String> {
        vec![
            Self::format_duration_ms(dist.min.as_nanos()),
            Self::format_duration_ms(dist.mean.as_nanos()),
            Self::format_duration_ms(dist.p50.as_nanos()),
            Self::format_duration_ms(dist.p95.as_nanos()),
            Self::format_duration_ms(dist.p99.as_nanos()),
            Self::format_duration_ms(dist.max.as_nanos()),
        ]
    }

    /// Create a latency statistics table
    fn create_latency_table(&self, metrics: &AggregatedMetrics) -> String {
        let mut builder = Builder::default();

        // Header
        builder.push_record(["Metric", "Min", "Mean", "P50", "P95", "P99", "Max"]);

        // TTFT row
        let ttft_values = Self::format_latency_dist(&metrics.ttft_distribution);
        let mut ttft_row = vec!["Time to First Token"];
        ttft_row.extend(ttft_values.iter().map(|s| s.as_str()));
        builder.push_record(ttft_row);

        // Inter-token latency row
        let inter_values = Self::format_latency_dist(&metrics.inter_token_distribution);
        let mut inter_row = vec!["Inter-token Latency"];
        inter_row.extend(inter_values.iter().map(|s| s.as_str()));
        builder.push_record(inter_row);

        // Total duration row
        let total_values = Self::format_latency_dist(&metrics.total_latency_distribution);
        let mut total_row = vec!["Total Duration"];
        total_row.extend(total_values.iter().map(|s| s.as_str()));
        builder.push_record(total_row);

        let mut table = builder.build();
        table.with(Style::rounded());

        if self.colored {
            table.with(Modify::new(Rows::first()).with(Color::BOLD));
        }

        table.to_string()
    }

    /// Create throughput statistics table
    fn create_throughput_table(&self, metrics: &AggregatedMetrics) -> String {
        let mut builder = Builder::default();

        // Header
        builder.push_record(["Metric", "Min", "Mean", "P50", "P95", "P99", "Max"]);

        // Tokens per second
        builder.push_record([
            "Tokens/Second",
            &format!("{:.1}", metrics.throughput.min_tokens_per_second),
            &format!("{:.1}", metrics.throughput.mean_tokens_per_second),
            &format!("{:.1}", metrics.throughput.p50_tokens_per_second),
            &format!("{:.1}", metrics.throughput.p95_tokens_per_second),
            &format!("{:.1}", metrics.throughput.p99_tokens_per_second),
            &format!("{:.1}", metrics.throughput.max_tokens_per_second),
        ]);

        let mut table = builder.build();
        table.with(Style::rounded());

        if self.colored {
            table.with(Modify::new(Rows::first()).with(Color::BOLD));
        }

        table.to_string()
    }

    /// Create per-model breakdown table
    fn create_model_table(&self, metrics: &AggregatedMetrics) -> String {
        if metrics.model_breakdown.is_empty() {
            return String::new();
        }

        let mut builder = Builder::default();

        // Header
        builder.push_record([
            "Model",
            "Requests",
            "Percentage",
        ]);

        for (model, count) in &metrics.model_breakdown {
            builder.push_record([
                model,
                &count.to_string(),
                &Self::format_percent(*count as f64, metrics.total_requests as f64),
            ]);
        }

        let mut table = builder.build();
        table.with(Style::rounded());

        if self.colored {
            table.with(Modify::new(Rows::first()).with(Color::BOLD));
        }

        table.to_string()
    }

    /// Create per-provider breakdown table
    fn create_provider_table(&self, metrics: &AggregatedMetrics) -> String {
        if metrics.provider_breakdown.is_empty() {
            return String::new();
        }

        let mut builder = Builder::default();

        // Header
        builder.push_record([
            "Provider",
            "Requests",
            "Percentage",
        ]);

        for (provider, count) in &metrics.provider_breakdown {
            builder.push_record([
                provider.as_str(),
                &count.to_string(),
                &Self::format_percent(*count as f64, metrics.total_requests as f64),
            ]);
        }

        let mut table = builder.build();
        table.with(Style::rounded());

        if self.colored {
            table.with(Modify::new(Rows::first()).with(Color::BOLD));
        }

        table.to_string()
    }

    /// Create individual requests table
    fn create_requests_table(&self, requests: &[RequestMetrics]) -> String {
        if requests.is_empty() {
            return String::new();
        }

        let mut builder = Builder::default();

        // Header
        builder.push_record([
            "Request ID",
            "Provider",
            "Model",
            "Status",
            "TTFT",
            "Tokens",
            "TPS",
        ]);

        for req in requests {
            builder.push_record([
                &req.request_id.to_string()[..8],
                req.provider.as_str(),
                &req.model,
                if req.success { "OK" } else { "FAIL" },
                &Self::format_duration_ms(req.ttft.as_nanos()),
                &(req.input_tokens + req.output_tokens).to_string(),
                &format!("{:.1}", req.tokens_per_second),
            ]);
        }

        let mut table = builder.build();
        table.with(Style::rounded());

        if self.colored {
            table.with(Modify::new(Rows::first()).with(Color::BOLD));
        }

        table.to_string()
    }
}

impl Default for ConsoleExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for ConsoleExporter {
    fn export(&self, metrics: &AggregatedMetrics) -> Result<String> {
        let mut output = String::new();

        // Title
        if self.colored {
            output.push_str(&format!(
                "\n{}\n",
                "LLM Latency Lens - Performance Report".bold().green()
            ));
        } else {
            output.push_str("\nLLM Latency Lens - Performance Report\n");
        }

        output.push_str(&format!("Session ID: {}\n", metrics.session_id));
        output.push_str(&format!("Period: {} to {}\n",
            metrics.start_time.format("%Y-%m-%d %H:%M:%S UTC"),
            metrics.end_time.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Summary section
        output.push_str(&self.section_header("Summary"));
        output.push('\n');
        output.push_str(&self.create_summary_table(metrics));

        // Latency section
        output.push_str(&self.section_header("Latency Statistics"));
        output.push('\n');
        output.push_str(&self.create_latency_table(metrics));

        // Throughput section
        output.push_str(&self.section_header("Throughput"));
        output.push('\n');
        output.push_str(&self.create_throughput_table(metrics));

        // Per-model breakdown
        let model_table = self.create_model_table(metrics);
        if !model_table.is_empty() {
            output.push_str(&self.section_header("Per-Model Breakdown"));
            output.push('\n');
            output.push_str(&model_table);
        }

        // Per-provider breakdown
        let provider_table = self.create_provider_table(metrics);
        if !provider_table.is_empty() {
            output.push_str(&self.section_header("Per-Provider Breakdown"));
            output.push('\n');
            output.push_str(&provider_table);
        }

        output.push('\n');
        Ok(output)
    }

    fn export_requests(&self, requests: &[RequestMetrics]) -> Result<String> {
        let mut output = String::new();

        if self.colored {
            output.push_str(&format!(
                "\n{}\n",
                "Individual Requests".bold().cyan()
            ));
        } else {
            output.push_str("\nIndividual Requests\n");
        }

        output.push_str(&self.create_requests_table(requests));
        output.push('\n');

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{create_test_metrics, create_test_requests};

    #[test]
    fn test_console_export() {
        let metrics = create_test_metrics();
        let exporter = ConsoleExporter::new();

        let result = exporter.export(&metrics).unwrap();
        assert!(result.contains("Performance Report"));
        assert!(result.contains("Summary"));
        assert!(result.contains("Latency Statistics"));
    }

    #[test]
    fn test_console_export_no_color() {
        let metrics = create_test_metrics();
        let exporter = ConsoleExporter::no_color();

        let result = exporter.export(&metrics).unwrap();
        assert!(result.contains("Performance Report"));
        // Should not contain ANSI color codes
        assert!(!result.contains("\x1b["));
    }

    #[test]
    fn test_console_export_requests() {
        let requests = create_test_requests();
        let exporter = ConsoleExporter::new();

        let result = exporter.export_requests(&requests).unwrap();
        assert!(result.contains("Individual Requests"));
        assert!(result.contains("gpt-4"));
        assert!(result.contains("claude-3-opus"));
    }

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(ConsoleExporter::format_duration_ms(500_000), "0.500 ms");
        assert_eq!(ConsoleExporter::format_duration_ms(5_000_000), "5.00 ms");
        assert_eq!(ConsoleExporter::format_duration_ms(50_000_000), "50.0 ms");
        assert_eq!(ConsoleExporter::format_duration_ms(5_000_000_000), "5.00 s");
    }

    #[test]
    fn test_format_percent() {
        assert_eq!(ConsoleExporter::format_percent(50.0, 100.0), "50.0%");
        assert_eq!(ConsoleExporter::format_percent(1.0, 3.0), "33.3%");
        assert_eq!(ConsoleExporter::format_percent(0.0, 0.0), "0.0%");
    }
}
