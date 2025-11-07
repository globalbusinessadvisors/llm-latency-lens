//! Export command implementation

use anyhow::{Context, Result};
use colored::Colorize;
use tracing::info;

use crate::cli::ExportArgs;
use llm_latency_lens_exporters::{
    ConsoleExporter, CsvExporter, Exporter, JsonExporter, PrometheusExporter,
};
use llm_latency_lens_metrics::AggregatedMetrics;

use super::write_output;

/// Run the export command
pub async fn run(args: ExportArgs, json_output: bool, quiet: bool) -> Result<()> {
    info!("Starting export command");

    // Read input file
    let input_content = std::fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read input file: {}", args.input.display()))?;

    // Parse metrics from JSON
    let metrics: AggregatedMetrics = serde_json::from_str(&input_content)
        .with_context(|| "Failed to parse metrics JSON. Expected AggregatedMetrics format.")?;

    if !quiet {
        println!(
            "{} Exporting metrics to {} format...",
            "=>".bright_cyan().bold(),
            args.format.bright_yellow()
        );
    }

    // Create appropriate exporter
    let output = match args.format.to_lowercase().as_str() {
        "json" => {
            let exporter = JsonExporter::new(args.pretty);
            exporter
                .export(&metrics)
                .context("Failed to export to JSON")?
        }
        "csv" => {
            let exporter = CsvExporter::new();
            exporter
                .export(&metrics)
                .context("Failed to export to CSV")?
        }
        "prometheus" | "prom" => {
            let exporter = PrometheusExporter::new();
            exporter
                .export(&metrics)
                .context("Failed to export to Prometheus format")?
        }
        "console" | "table" => {
            let exporter = ConsoleExporter::new();
            exporter
                .export(&metrics)
                .context("Failed to export to console format")?
        }
        _ => {
            anyhow::bail!(
                "Unsupported format '{}'. Supported formats: json, csv, prometheus, console",
                args.format
            );
        }
    };

    // Write output
    write_output(&output, &args.output)?;

    if !quiet {
        if let Some(output_path) = &args.output {
            println!(
                "{} Exported to: {}",
                "✓".bright_green().bold(),
                output_path.display()
            );
        } else {
            // Output was printed to stdout by write_output
            if args.format != "console" && args.format != "table" {
                // Only show success message for non-console formats
                // (console format already prints to stdout)
            }
        }

        println!();
        println!("{} Export complete!", "✓".bright_green().bold());
    }

    Ok(())
}
