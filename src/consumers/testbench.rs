//! LLM-Test-Bench File Reader Adapter
//!
//! Optional file-based reader for ingesting benchmark output files from
//! LLM-Test-Bench without creating a compile-time dependency.
//!
//! # Supported Formats
//!
//! - **JSON**: Standard JSON benchmark output format
//! - **CSV**: Comma-separated values with header row
//! - **JSONL**: JSON Lines format (one JSON object per line)
//!
//! # Design Principles
//!
//! This adapter is intentionally file-based only:
//! - NO compile-time dependency on LLM-Test-Bench crate
//! - Reads standard file formats that Test-Bench exports
//! - Converts to Latency-Lens RequestMetrics format
//! - Supports both single-file and directory batch imports

use super::{ConsumerError, ConsumerResult};
use crate::{RequestMetrics, SessionId, RequestId};
use chrono::{DateTime, Utc};
use llm_latency_lens_core::Provider;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Duration;

/// Supported file formats for Test-Bench import
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestBenchFormat {
    /// Standard JSON format
    Json,
    /// JSON Lines format (one object per line)
    JsonLines,
    /// CSV with header row
    Csv,
    /// Auto-detect from file extension
    Auto,
}

impl TestBenchFormat {
    /// Detect format from file path
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("json") => Self::Json,
            Some("jsonl") | Some("ndjson") => Self::JsonLines,
            Some("csv") => Self::Csv,
            _ => Self::Json, // Default to JSON
        }
    }
}

/// Test-Bench benchmark result as stored in files
///
/// This structure matches the standard Test-Bench export format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestBenchMetrics {
    /// Test case identifier
    #[serde(alias = "test_id", alias = "id")]
    pub test_case_id: Option<String>,
    /// Provider name
    #[serde(alias = "llm_provider")]
    pub provider: String,
    /// Model identifier
    #[serde(alias = "llm_model")]
    pub model: String,
    /// Test timestamp
    #[serde(alias = "run_time", alias = "executed_at")]
    pub timestamp: Option<DateTime<Utc>>,
    /// Time to first token (milliseconds)
    #[serde(alias = "time_to_first_token_ms", alias = "ttft")]
    pub ttft_ms: f64,
    /// Total latency (milliseconds)
    #[serde(alias = "total_time_ms", alias = "latency_ms")]
    pub total_latency_ms: f64,
    /// Inter-token latencies (milliseconds)
    #[serde(default, alias = "itl_ms", alias = "inter_token_latencies_ms")]
    pub inter_token_latencies_ms: Vec<f64>,
    /// Input token count
    #[serde(default, alias = "prompt_tokens")]
    pub input_tokens: u64,
    /// Output token count
    #[serde(default, alias = "completion_tokens")]
    pub output_tokens: u64,
    /// Thinking tokens (Claude extended thinking)
    #[serde(default)]
    pub thinking_tokens: Option<u64>,
    /// Tokens per second
    #[serde(default, alias = "throughput")]
    pub tokens_per_second: Option<f64>,
    /// Cost in USD
    #[serde(default)]
    pub cost_usd: Option<f64>,
    /// Test passed/succeeded
    #[serde(default = "default_true", alias = "passed")]
    pub success: bool,
    /// Error message if failed
    #[serde(default)]
    pub error: Option<String>,
    /// Additional metadata
    #[serde(default, flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_true() -> bool {
    true
}

/// CSV row structure for Test-Bench CSV format
#[derive(Debug, Deserialize)]
struct TestBenchCsvRow {
    #[serde(alias = "test_id", alias = "id")]
    test_case_id: Option<String>,
    provider: String,
    model: String,
    #[serde(alias = "timestamp", alias = "run_time")]
    timestamp: Option<String>,
    #[serde(alias = "ttft", alias = "time_to_first_token_ms")]
    ttft_ms: f64,
    #[serde(alias = "latency_ms", alias = "total_time_ms")]
    total_latency_ms: f64,
    #[serde(default, alias = "prompt_tokens")]
    input_tokens: Option<u64>,
    #[serde(default, alias = "completion_tokens")]
    output_tokens: Option<u64>,
    #[serde(default)]
    tokens_per_second: Option<f64>,
    #[serde(default)]
    cost_usd: Option<f64>,
    #[serde(default = "default_true", alias = "passed")]
    success: bool,
    #[serde(default)]
    error: Option<String>,
}

/// File reader for LLM-Test-Bench benchmark outputs
///
/// Reads exported benchmark files and converts them to Latency-Lens
/// RequestMetrics format without requiring a dependency on Test-Bench.
pub struct TestBenchReader {
    session_id: SessionId,
}

impl TestBenchReader {
    /// Create a new Test-Bench file reader
    pub fn new() -> Self {
        Self {
            session_id: SessionId::new(),
        }
    }

    /// Set the session ID for imported metrics
    pub fn with_session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = session_id;
        self
    }

    /// Read a single JSON file
    pub fn read_json_file<P: AsRef<Path>>(&self, path: P) -> ConsumerResult<Vec<RequestMetrics>> {
        let file = File::open(path.as_ref()).map_err(ConsumerError::IoError)?;
        let reader = BufReader::new(file);

        // Try parsing as array first, then as single object
        let metrics: Vec<TestBenchMetrics> = match serde_json::from_reader::<_, Vec<TestBenchMetrics>>(reader) {
            Ok(arr) => arr,
            Err(_) => {
                // Re-open file and try single object
                let file = File::open(path.as_ref()).map_err(ConsumerError::IoError)?;
                let reader = BufReader::new(file);
                let single: TestBenchMetrics = serde_json::from_reader(reader)?;
                vec![single]
            }
        };

        self.convert_metrics(metrics)
    }

    /// Read a JSON Lines file (one JSON object per line)
    pub fn read_jsonl_file<P: AsRef<Path>>(&self, path: P) -> ConsumerResult<Vec<RequestMetrics>> {
        let file = File::open(path.as_ref()).map_err(ConsumerError::IoError)?;
        let reader = BufReader::new(file);

        let mut metrics = Vec::new();
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(ConsumerError::IoError)?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<TestBenchMetrics>(&line) {
                Ok(m) => metrics.push(m),
                Err(e) => {
                    tracing::warn!(
                        line = line_num + 1,
                        error = %e,
                        "Failed to parse JSONL line, skipping"
                    );
                }
            }
        }

        self.convert_metrics(metrics)
    }

    /// Read a CSV file
    pub fn read_csv_file<P: AsRef<Path>>(&self, path: P) -> ConsumerResult<Vec<RequestMetrics>> {
        let file = File::open(path.as_ref()).map_err(ConsumerError::IoError)?;
        let mut reader = csv::Reader::from_reader(file);

        let mut metrics = Vec::new();
        for result in reader.deserialize::<TestBenchCsvRow>() {
            match result {
                Ok(row) => {
                    let tbm = TestBenchMetrics {
                        test_case_id: row.test_case_id,
                        provider: row.provider,
                        model: row.model,
                        timestamp: row.timestamp.and_then(|s| s.parse().ok()),
                        ttft_ms: row.ttft_ms,
                        total_latency_ms: row.total_latency_ms,
                        inter_token_latencies_ms: Vec::new(),
                        input_tokens: row.input_tokens.unwrap_or(0),
                        output_tokens: row.output_tokens.unwrap_or(0),
                        thinking_tokens: None,
                        tokens_per_second: row.tokens_per_second,
                        cost_usd: row.cost_usd,
                        success: row.success,
                        error: row.error,
                        metadata: HashMap::new(),
                    };
                    metrics.push(tbm);
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to parse CSV row, skipping");
                }
            }
        }

        self.convert_metrics(metrics)
    }

    /// Read a file with auto-detected format
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> ConsumerResult<Vec<RequestMetrics>> {
        self.read_file_with_format(path.as_ref(), TestBenchFormat::Auto)
    }

    /// Read a file with specified format
    pub fn read_file_with_format<P: AsRef<Path>>(
        &self,
        path: P,
        format: TestBenchFormat,
    ) -> ConsumerResult<Vec<RequestMetrics>> {
        let path = path.as_ref();
        let format = if format == TestBenchFormat::Auto {
            TestBenchFormat::from_path(path)
        } else {
            format
        };

        tracing::debug!(
            path = %path.display(),
            format = ?format,
            "Reading Test-Bench file"
        );

        match format {
            TestBenchFormat::Json => self.read_json_file(path),
            TestBenchFormat::JsonLines => self.read_jsonl_file(path),
            TestBenchFormat::Csv => self.read_csv_file(path),
            TestBenchFormat::Auto => unreachable!(),
        }
    }

    /// Read all benchmark files from a directory
    pub fn read_directory<P: AsRef<Path>>(&self, dir: P) -> ConsumerResult<Vec<RequestMetrics>> {
        let dir = dir.as_ref();
        if !dir.is_dir() {
            return Err(ConsumerError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotADirectory,
                format!("{} is not a directory", dir.display()),
            )));
        }

        let mut all_metrics = Vec::new();

        for entry in std::fs::read_dir(dir).map_err(ConsumerError::IoError)? {
            let entry = entry.map_err(ConsumerError::IoError)?;
            let path = entry.path();

            // Skip directories and hidden files
            if path.is_dir() || path.file_name().map(|n| n.to_string_lossy().starts_with('.')).unwrap_or(false) {
                continue;
            }

            // Only process known formats
            let ext = path.extension().and_then(|e| e.to_str());
            if !matches!(ext, Some("json") | Some("jsonl") | Some("ndjson") | Some("csv")) {
                continue;
            }

            match self.read_file(&path) {
                Ok(metrics) => {
                    tracing::debug!(
                        path = %path.display(),
                        count = metrics.len(),
                        "Imported metrics from file"
                    );
                    all_metrics.extend(metrics);
                }
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "Failed to read file, skipping"
                    );
                }
            }
        }

        // Sort by timestamp
        all_metrics.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(all_metrics)
    }

    /// Convert TestBenchMetrics to RequestMetrics
    fn convert_metrics(
        &self,
        metrics: Vec<TestBenchMetrics>,
    ) -> ConsumerResult<Vec<RequestMetrics>> {
        metrics
            .into_iter()
            .map(|m| self.testbench_to_request_metrics(&m))
            .collect()
    }

    /// Convert a single TestBenchMetrics to RequestMetrics
    fn testbench_to_request_metrics(
        &self,
        tbm: &TestBenchMetrics,
    ) -> ConsumerResult<RequestMetrics> {
        let provider = self.parse_provider(&tbm.provider);

        let ttft = Duration::from_secs_f64(tbm.ttft_ms / 1000.0);
        let total_latency = Duration::from_secs_f64(tbm.total_latency_ms / 1000.0);

        let inter_token_latencies: Vec<Duration> = tbm
            .inter_token_latencies_ms
            .iter()
            .map(|&ms| Duration::from_secs_f64(ms / 1000.0))
            .collect();

        let tokens_per_second = tbm.tokens_per_second.unwrap_or_else(|| {
            if total_latency.as_secs_f64() > 0.0 {
                tbm.output_tokens as f64 / total_latency.as_secs_f64()
            } else {
                0.0
            }
        });

        Ok(RequestMetrics {
            request_id: RequestId::new(),
            session_id: self.session_id,
            provider,
            model: tbm.model.clone(),
            timestamp: tbm.timestamp.unwrap_or_else(Utc::now),
            ttft,
            total_latency,
            inter_token_latencies,
            input_tokens: tbm.input_tokens,
            output_tokens: tbm.output_tokens,
            thinking_tokens: tbm.thinking_tokens,
            tokens_per_second,
            cost_usd: tbm.cost_usd,
            success: tbm.success,
            error: tbm.error.clone(),
        })
    }

    /// Parse provider string to Provider enum
    fn parse_provider(&self, provider_str: &str) -> Provider {
        match provider_str.to_lowercase().as_str() {
            "openai" | "gpt" => Provider::OpenAI,
            "anthropic" | "claude" => Provider::Anthropic,
            "google" | "gemini" => Provider::Google,
            "aws-bedrock" | "bedrock" | "aws" => Provider::AwsBedrock,
            "azure-openai" | "azure" => Provider::AzureOpenAI,
            _ => Provider::Generic,
        }
    }

    /// Validate a Test-Bench file without importing
    pub fn validate_file<P: AsRef<Path>>(&self, path: P) -> ConsumerResult<ValidationResult> {
        let path = path.as_ref();
        let format = TestBenchFormat::from_path(path);

        let start = std::time::Instant::now();
        let metrics = self.read_file_with_format(path, format)?;
        let parse_duration = start.elapsed();

        let mut providers = HashMap::new();
        let mut models = HashMap::new();
        let mut success_count = 0u64;
        let mut failed_count = 0u64;

        for m in &metrics {
            *providers.entry(m.provider.clone()).or_insert(0u64) += 1;
            *models.entry(m.model.clone()).or_insert(0u64) += 1;
            if m.success {
                success_count += 1;
            } else {
                failed_count += 1;
            }
        }

        Ok(ValidationResult {
            file_path: path.to_path_buf(),
            format,
            record_count: metrics.len(),
            parse_duration,
            providers,
            models,
            success_count,
            failed_count,
            is_valid: true,
            errors: Vec::new(),
        })
    }
}

impl Default for TestBenchReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of file validation
#[derive(Debug)]
pub struct ValidationResult {
    /// File path that was validated
    pub file_path: std::path::PathBuf,
    /// Detected file format
    pub format: TestBenchFormat,
    /// Number of records in file
    pub record_count: usize,
    /// Time taken to parse
    pub parse_duration: Duration,
    /// Provider breakdown
    pub providers: HashMap<Provider, u64>,
    /// Model breakdown
    pub models: HashMap<String, u64>,
    /// Number of successful records
    pub success_count: u64,
    /// Number of failed records
    pub failed_count: u64,
    /// Whether the file is valid
    pub is_valid: bool,
    /// Validation errors (if any)
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_format_detection() {
        assert_eq!(
            TestBenchFormat::from_path(Path::new("test.json")),
            TestBenchFormat::Json
        );
        assert_eq!(
            TestBenchFormat::from_path(Path::new("test.jsonl")),
            TestBenchFormat::JsonLines
        );
        assert_eq!(
            TestBenchFormat::from_path(Path::new("test.csv")),
            TestBenchFormat::Csv
        );
        assert_eq!(
            TestBenchFormat::from_path(Path::new("test.unknown")),
            TestBenchFormat::Json
        );
    }

    #[test]
    fn test_parse_provider() {
        let reader = TestBenchReader::new();

        assert!(matches!(reader.parse_provider("openai"), Provider::OpenAI));
        assert!(matches!(reader.parse_provider("OpenAI"), Provider::OpenAI));
        assert!(matches!(reader.parse_provider("gpt"), Provider::OpenAI));
        assert!(matches!(
            reader.parse_provider("anthropic"),
            Provider::Anthropic
        ));
        assert!(matches!(reader.parse_provider("claude"), Provider::Anthropic));
        assert!(matches!(reader.parse_provider("google"), Provider::Google));
        assert!(matches!(
            reader.parse_provider("unknown"),
            Provider::Generic
        ));
    }

    #[test]
    fn test_read_json_file() {
        let reader = TestBenchReader::new();

        // Create a temp file with test data
        let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
        writeln!(
            temp_file,
            r#"[
                {{
                    "provider": "openai",
                    "model": "gpt-4",
                    "ttft_ms": 150.0,
                    "total_latency_ms": 2000.0,
                    "input_tokens": 100,
                    "output_tokens": 50,
                    "success": true
                }}
            ]"#
        )
        .unwrap();

        let metrics = reader.read_json_file(temp_file.path()).unwrap();

        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].model, "gpt-4");
        assert_eq!(metrics[0].input_tokens, 100);
        assert!(metrics[0].success);
    }

    #[test]
    fn test_read_single_json_object() {
        let reader = TestBenchReader::new();

        let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
        writeln!(
            temp_file,
            r#"{{
                "provider": "anthropic",
                "model": "claude-3-opus",
                "ttft_ms": 200.0,
                "total_latency_ms": 3000.0,
                "input_tokens": 150,
                "output_tokens": 80
            }}"#
        )
        .unwrap();

        let metrics = reader.read_json_file(temp_file.path()).unwrap();

        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].model, "claude-3-opus");
    }

    #[test]
    fn test_read_jsonl_file() {
        let reader = TestBenchReader::new();

        let mut temp_file = NamedTempFile::with_suffix(".jsonl").unwrap();
        writeln!(
            temp_file,
            r#"{{"provider": "openai", "model": "gpt-4", "ttft_ms": 100.0, "total_latency_ms": 1000.0}}"#
        )
        .unwrap();
        writeln!(
            temp_file,
            r#"{{"provider": "anthropic", "model": "claude-3", "ttft_ms": 150.0, "total_latency_ms": 1500.0}}"#
        )
        .unwrap();

        let metrics = reader.read_jsonl_file(temp_file.path()).unwrap();

        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics[0].model, "gpt-4");
        assert_eq!(metrics[1].model, "claude-3");
    }

    #[test]
    fn test_testbench_to_request_metrics() {
        let reader = TestBenchReader::new();

        let tbm = TestBenchMetrics {
            test_case_id: Some("test-1".to_string()),
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            timestamp: Some(Utc::now()),
            ttft_ms: 150.0,
            total_latency_ms: 2000.0,
            inter_token_latencies_ms: vec![10.0, 15.0, 12.0],
            input_tokens: 100,
            output_tokens: 50,
            thinking_tokens: None,
            tokens_per_second: Some(25.0),
            cost_usd: Some(0.05),
            success: true,
            error: None,
            metadata: HashMap::new(),
        };

        let metrics = reader.testbench_to_request_metrics(&tbm).unwrap();

        assert_eq!(metrics.model, "gpt-4o");
        assert_eq!(metrics.ttft, Duration::from_millis(150));
        assert_eq!(metrics.total_latency, Duration::from_millis(2000));
        assert_eq!(metrics.inter_token_latencies.len(), 3);
        assert_eq!(metrics.input_tokens, 100);
        assert_eq!(metrics.output_tokens, 50);
        assert_eq!(metrics.tokens_per_second, 25.0);
        assert!(metrics.success);
    }
}
