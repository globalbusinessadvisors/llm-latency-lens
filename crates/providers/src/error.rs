//! Error types for LLM provider adapters
//!
//! This module defines comprehensive error handling for all provider operations,
//! including network errors, API errors, authentication issues, and rate limiting.

/// Result type alias for provider operations
pub type Result<T> = std::result::Result<T, ProviderError>;

/// Comprehensive error type for all provider operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum ProviderError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    /// API returned an error response
    #[error("API error: {message} (status: {status_code})")]
    ApiError {
        /// HTTP status code
        status_code: u16,
        /// Error message from API
        message: String,
        /// Raw error response body
        body: Option<String>,
    },

    /// Authentication failed (invalid API key)
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {message}. Retry after: {retry_after:?}s")]
    RateLimitError {
        /// Rate limit message
        message: String,
        /// Seconds to wait before retrying
        retry_after: Option<u64>,
    },

    /// Timeout occurred during request
    #[error("Request timeout after {0:?}")]
    TimeoutError(std::time::Duration),

    /// Streaming error
    #[error("Streaming error: {0}")]
    StreamingError(String),

    /// Failed to parse SSE event
    #[error("SSE parse error: {0}")]
    SseParseError(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// Model not found or invalid
    #[error("Invalid model: {0}")]
    InvalidModel(String),

    /// Request payload too large
    #[error("Request payload too large: {0}")]
    PayloadTooLarge(String),

    /// Service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Network error (DNS, connection, etc.)
    #[error("Network error: {0}")]
    NetworkError(String),

    /// TLS/SSL error
    #[error("TLS error: {0}")]
    TlsError(String),

    /// Content filter triggered (safety filters)
    #[error("Content filtered: {0}")]
    ContentFilterError(String),

    /// Context length exceeded
    #[error("Context length exceeded: {0}")]
    ContextLengthExceeded(String),

    /// Internal provider error
    #[error("Internal provider error: {0}")]
    InternalError(String),

    /// Generic error with context
    #[error("{0}")]
    Other(String),
}

impl ProviderError {
    /// Create an API error from status code and message
    pub fn api_error(status_code: u16, message: impl Into<String>) -> Self {
        Self::ApiError {
            status_code,
            message: message.into(),
            body: None,
        }
    }

    /// Create an API error with full response body
    pub fn api_error_with_body(
        status_code: u16,
        message: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self::ApiError {
            status_code,
            message: message.into(),
            body: Some(body.into()),
        }
    }

    /// Create a rate limit error
    pub fn rate_limit(message: impl Into<String>, retry_after: Option<u64>) -> Self {
        Self::RateLimitError {
            message: message.into(),
            retry_after,
        }
    }

    /// Create a streaming error
    pub fn streaming(message: impl Into<String>) -> Self {
        Self::StreamingError(message.into())
    }

    /// Create an SSE parse error
    pub fn sse_parse(message: impl Into<String>) -> Self {
        Self::SseParseError(message.into())
    }

    /// Create a JSON error from serde_json::Error
    pub fn from_json_error(error: serde_json::Error) -> Self {
        Self::JsonError(error.to_string())
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            // Retryable errors
            Self::RateLimitError { .. } => true,
            Self::TimeoutError(_) => true,
            Self::ServiceUnavailable(_) => true,
            Self::NetworkError(_) => true,
            Self::ApiError { status_code, .. } => {
                // Retry on 5xx errors and 429
                *status_code >= 500 || *status_code == 429
            }
            Self::HttpError(_) => {
                // Retry on generic HTTP errors (conservative approach)
                false
            }
            // Non-retryable errors
            Self::AuthenticationError(_) => false,
            Self::InvalidModel(_) => false,
            Self::PayloadTooLarge(_) => false,
            Self::ContentFilterError(_) => false,
            Self::ContextLengthExceeded(_) => false,
            Self::ConfigError(_) => false,
            Self::JsonError(_) => false,
            Self::TlsError(_) => false,
            Self::StreamingError(_) => false,
            Self::SseParseError(_) => false,
            Self::InternalError(_) => false,
            Self::Other(_) => false,
        }
    }

    /// Get suggested retry delay in seconds
    pub fn retry_delay(&self) -> Option<u64> {
        match self {
            Self::RateLimitError { retry_after, .. } => *retry_after,
            Self::TimeoutError(_) => Some(1),
            Self::ServiceUnavailable(_) => Some(5),
            Self::NetworkError(_) => Some(2),
            Self::ApiError { status_code, .. } if *status_code >= 500 => Some(3),
            _ => None,
        }
    }

    /// Convert reqwest error to provider error with context
    pub fn from_reqwest(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            Self::TimeoutError(std::time::Duration::from_secs(30))
        } else if error.is_connect() {
            Self::NetworkError(format!("Connection failed: {}", error))
        } else if error.is_status() {
            if let Some(status) = error.status() {
                Self::ApiError {
                    status_code: status.as_u16(),
                    message: format!("HTTP {}: {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")),
                    body: None,
                }
            } else {
                Self::HttpError(error.to_string())
            }
        } else {
            Self::HttpError(error.to_string())
        }
    }
}

/// Parse API error from response
///
/// This helper function attempts to extract structured error information
/// from various API error response formats.
pub async fn parse_api_error(response: reqwest::Response) -> ProviderError {
    let status = response.status();
    let status_code = status.as_u16();

    // Try to read response body
    let body = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            return ProviderError::api_error(
                status_code,
                format!("Failed to read error response: {}", e),
            );
        }
    };

    // Handle specific status codes
    match status_code {
        401 => ProviderError::AuthenticationError(
            extract_error_message(&body).unwrap_or_else(|| "Invalid API key".to_string()),
        ),
        429 => {
            let retry_after = extract_retry_after(&body);
            ProviderError::rate_limit(
                extract_error_message(&body).unwrap_or_else(|| "Rate limit exceeded".to_string()),
                retry_after,
            )
        }
        413 => ProviderError::PayloadTooLarge(
            extract_error_message(&body).unwrap_or_else(|| "Request too large".to_string()),
        ),
        503 => ProviderError::ServiceUnavailable(
            extract_error_message(&body).unwrap_or_else(|| "Service unavailable".to_string()),
        ),
        _ => ProviderError::api_error_with_body(status_code, status.to_string(), body),
    }
}

/// Extract error message from JSON response
fn extract_error_message(body: &str) -> Option<String> {
    // Try to parse as JSON and extract common error message fields
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        // OpenAI format: { "error": { "message": "..." } }
        if let Some(error) = json.get("error") {
            if let Some(message) = error.get("message") {
                if let Some(msg) = message.as_str() {
                    return Some(msg.to_string());
                }
            }
        }
        // Anthropic format: { "error": { "type": "...", "message": "..." } }
        if let Some(message) = json.get("message") {
            if let Some(msg) = message.as_str() {
                return Some(msg.to_string());
            }
        }
    }
    None
}

/// Extract retry-after value from response
fn extract_retry_after(body: &str) -> Option<u64> {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(retry) = json.get("retry_after") {
            if let Some(seconds) = retry.as_u64() {
                return Some(seconds);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_retryable() {
        assert!(ProviderError::rate_limit("test", Some(60)).is_retryable());
        assert!(ProviderError::TimeoutError(std::time::Duration::from_secs(30)).is_retryable());
        assert!(ProviderError::ServiceUnavailable("test".to_string()).is_retryable());
        assert!(ProviderError::NetworkError("test".to_string()).is_retryable());
        assert!(ProviderError::api_error(500, "server error").is_retryable());
        assert!(ProviderError::api_error(429, "rate limit").is_retryable());

        assert!(!ProviderError::AuthenticationError("test".to_string()).is_retryable());
        assert!(!ProviderError::InvalidModel("test".to_string()).is_retryable());
        assert!(!ProviderError::api_error(400, "bad request").is_retryable());
        assert!(!ProviderError::ContentFilterError("test".to_string()).is_retryable());
    }

    #[test]
    fn test_retry_delay() {
        assert_eq!(
            ProviderError::rate_limit("test", Some(60)).retry_delay(),
            Some(60)
        );
        assert_eq!(
            ProviderError::TimeoutError(std::time::Duration::from_secs(30)).retry_delay(),
            Some(1)
        );
        assert_eq!(
            ProviderError::ServiceUnavailable("test".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            ProviderError::NetworkError("test".to_string()).retry_delay(),
            Some(2)
        );
    }

    #[test]
    fn test_extract_error_message() {
        let openai_error = r#"{"error": {"message": "Invalid API key"}}"#;
        assert_eq!(
            extract_error_message(openai_error),
            Some("Invalid API key".to_string())
        );

        let anthropic_error = r#"{"message": "Rate limit exceeded"}"#;
        assert_eq!(
            extract_error_message(anthropic_error),
            Some("Rate limit exceeded".to_string())
        );

        let invalid = "not json";
        assert_eq!(extract_error_message(invalid), None);
    }

    #[test]
    fn test_extract_retry_after() {
        let json = r#"{"retry_after": 60}"#;
        assert_eq!(extract_retry_after(json), Some(60));

        let no_retry = r#"{"error": "test"}"#;
        assert_eq!(extract_retry_after(no_retry), None);
    }
}
