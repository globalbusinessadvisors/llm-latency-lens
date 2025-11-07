//! Core types for LLM Latency Lens

use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Unique identifier for a profiling session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Create a new session ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a single request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(Uuid);

impl RequestId {
    /// Create a new request ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// LLM provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// OpenAI (GPT models)
    OpenAI,
    /// Anthropic (Claude models)
    Anthropic,
    /// Google (Gemini models)
    Google,
    /// AWS Bedrock
    AwsBedrock,
    /// Azure OpenAI
    AzureOpenAI,
    /// Generic OpenAI-compatible endpoint
    Generic,
}

impl Provider {
    /// Get the provider name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::OpenAI => "openai",
            Provider::Anthropic => "anthropic",
            Provider::Google => "google",
            Provider::AwsBedrock => "aws-bedrock",
            Provider::AzureOpenAI => "azure-openai",
            Provider::Generic => "generic",
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Request configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestConfig {
    /// Provider to use
    pub provider: Provider,
    /// Model name/ID
    pub model: String,
    /// API endpoint URL (optional, uses provider default if not specified)
    pub endpoint: Option<String>,
    /// API key or authentication token
    pub api_key: String,
    /// Request timeout
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
    /// Maximum number of retries
    pub max_retries: u32,
}

/// Streaming token event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEvent {
    /// Request ID this token belongs to
    pub request_id: RequestId,
    /// Token sequence number (0 = first token)
    pub sequence: u64,
    /// Token content (if available)
    pub content: Option<String>,
    /// Timestamp when token was received
    pub timestamp_nanos: u64,
    /// Time since request start
    #[serde(with = "duration_serde")]
    pub time_since_start: Duration,
    /// Time since previous token (None for first token)
    #[serde(with = "option_duration_serde")]
    pub inter_token_latency: Option<Duration>,
}

/// Request metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    /// Unique request ID
    pub request_id: RequestId,
    /// Session ID
    pub session_id: SessionId,
    /// Provider used
    pub provider: Provider,
    /// Model used
    pub model: String,
    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Number of input tokens
    pub input_tokens: Option<u64>,
    /// Number of output tokens
    pub output_tokens: Option<u64>,
    /// Number of thinking tokens (Claude specific)
    pub thinking_tokens: Option<u64>,
}

/// Serde module for Duration serialization
mod duration_serde {
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

/// Serde module for Option<Duration> serialization
mod option_duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(d) => serializer.serialize_some(&(d.as_nanos() as u64)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nanos = Option::<u64>::deserialize(deserializer)?;
        Ok(nanos.map(Duration::from_nanos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_creation() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_request_id_creation() {
        let id1 = RequestId::new();
        let id2 = RequestId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_provider_display() {
        assert_eq!(Provider::OpenAI.to_string(), "openai");
        assert_eq!(Provider::Anthropic.to_string(), "anthropic");
        assert_eq!(Provider::Google.to_string(), "google");
    }

    #[test]
    fn test_request_config_serialization() {
        let config = RequestConfig {
            provider: Provider::OpenAI,
            model: "gpt-4".to_string(),
            endpoint: None,
            api_key: "test-key".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RequestConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.provider, deserialized.provider);
        assert_eq!(config.model, deserialized.model);
        assert_eq!(config.timeout, deserialized.timeout);
    }

    #[test]
    fn test_token_event_serialization() {
        let event = TokenEvent {
            request_id: RequestId::new(),
            sequence: 0,
            content: Some("Hello".to_string()),
            timestamp_nanos: 1000000,
            time_since_start: Duration::from_millis(10),
            inter_token_latency: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: TokenEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.sequence, deserialized.sequence);
        assert_eq!(event.content, deserialized.content);
    }
}
