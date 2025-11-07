//! JSON exporter for metrics
//!
//! Provides both human-readable (pretty-printed) and compact JSON output.

use crate::{Exporter, Result};
use llm_latency_lens_metrics::{AggregatedMetrics, RequestMetrics};
use serde::Serialize;

/// JSON exporter with configurable formatting
#[derive(Debug, Clone)]
pub struct JsonExporter {
    /// Whether to pretty-print the output
    pretty: bool,
}

impl JsonExporter {
    /// Create a new JSON exporter
    ///
    /// # Arguments
    ///
    /// * `pretty` - If true, output will be pretty-printed with indentation
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }

    /// Create a compact JSON exporter (no pretty printing)
    pub fn compact() -> Self {
        Self::new(false)
    }

    /// Create a pretty-printed JSON exporter
    pub fn pretty() -> Self {
        Self::new(true)
    }

    /// Serialize any serializable value to JSON
    fn serialize<T: Serialize + ?Sized>(&self, value: &T) -> Result<String> {
        if self.pretty {
            Ok(serde_json::to_string_pretty(value)?)
        } else {
            Ok(serde_json::to_string(value)?)
        }
    }
}

impl Default for JsonExporter {
    fn default() -> Self {
        Self::new(true)
    }
}

impl Exporter for JsonExporter {
    fn export(&self, metrics: &AggregatedMetrics) -> Result<String> {
        self.serialize(metrics)
    }

    fn export_requests(&self, requests: &[RequestMetrics]) -> Result<String> {
        self.serialize(requests)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{create_test_metrics, create_test_requests};

    #[test]
    fn test_json_export_compact() {
        let metrics = create_test_metrics();
        let exporter = JsonExporter::compact();

        let result = exporter.export(&metrics).unwrap();
        assert!(!result.contains('\n')); // Compact format should be single line
        assert!(result.contains("session_id"));
        assert!(result.contains("total_requests"));
    }

    #[test]
    fn test_json_export_pretty() {
        let metrics = create_test_metrics();
        let exporter = JsonExporter::pretty();

        let result = exporter.export(&metrics).unwrap();
        assert!(result.contains('\n')); // Pretty format should have newlines
        assert!(result.contains("  ")); // Should have indentation
        assert!(result.contains("session_id"));
    }

    #[test]
    fn test_json_export_requests() {
        let requests = create_test_requests();
        let exporter = JsonExporter::pretty();

        let result = exporter.export_requests(&requests).unwrap();
        assert!(result.contains("request_id"));
        assert!(result.contains("gpt-4"));
        assert!(result.contains("claude-3-opus"));
    }

    #[test]
    fn test_json_roundtrip() {
        let metrics = create_test_metrics();
        let exporter = JsonExporter::compact();

        let json = exporter.export(&metrics).unwrap();
        let deserialized: AggregatedMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(metrics.session_id, deserialized.session_id);
        assert_eq!(metrics.total_requests, deserialized.total_requests);
        assert_eq!(
            metrics.successful_requests,
            deserialized.successful_requests
        );
    }

    #[test]
    fn test_json_default() {
        let exporter = JsonExporter::default();
        assert!(exporter.pretty);
    }
}
