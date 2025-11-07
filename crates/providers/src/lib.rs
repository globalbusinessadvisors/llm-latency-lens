//! Provider adapters for LLM Latency Lens
//!
//! This crate provides production-ready adapters for various LLM providers,
//! enabling high-precision latency measurements and streaming token analysis.
//!
//! # Features
//!
//! - **OpenAI**: Full implementation with GPT-4, GPT-4o, and GPT-3.5 support
//! - **Anthropic**: Complete Claude integration with extended thinking support
//! - **Google**: Stub implementation for Gemini (coming soon)
//! - **Streaming**: Server-Sent Events (SSE) with fine-grained token timing
//! - **Retries**: Automatic retry logic with exponential backoff
//! - **Cost Calculation**: Accurate pricing for all supported models
//!
//! # Example
//!
//! ```no_run
//! use llm_latency_lens_providers::{
//!     openai::OpenAIProvider,
//!     traits::{Provider, StreamingRequest, MessageRole},
//! };
//! use llm_latency_lens_core::TimingEngine;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create provider
//! let provider = OpenAIProvider::new("sk-...");
//!
//! // Create timing engine
//! let timing = TimingEngine::new();
//!
//! // Build request
//! let request = StreamingRequest::builder()
//!     .model("gpt-4o")
//!     .message(MessageRole::User, "Explain quantum computing")
//!     .max_tokens(500)
//!     .temperature(0.7)
//!     .build();
//!
//! // Stream response
//! let mut response = provider.stream(request, &timing).await?;
//!
//! // Process tokens
//! use futures::StreamExt;
//! while let Some(token) = response.token_stream.next().await {
//!     let event = token?;
//!     println!("Token {}: {:?} (latency: {:?})",
//!         event.sequence,
//!         event.content,
//!         event.inter_token_latency
//!     );
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Provider Implementations
//!
//! ## OpenAI
//!
//! The OpenAI provider supports all GPT models with comprehensive streaming
//! and timing capabilities:
//!
//! ```no_run
//! use llm_latency_lens_providers::openai::OpenAIProvider;
//!
//! let provider = OpenAIProvider::builder()
//!     .api_key("sk-...")
//!     .organization("org-...")  // Optional
//!     .max_retries(3)
//!     .build();
//! ```
//!
//! ## Anthropic
//!
//! The Anthropic provider supports all Claude models including extended
//! thinking mode:
//!
//! ```no_run
//! use llm_latency_lens_providers::anthropic::AnthropicProvider;
//!
//! let provider = AnthropicProvider::builder()
//!     .api_key("sk-ant-...")
//!     .api_version("2023-06-01")
//!     .max_retries(3)
//!     .build();
//! ```
//!
//! ## Google
//!
//! The Google provider is currently a stub. Full implementation coming soon:
//!
//! ```no_run
//! use llm_latency_lens_providers::google::GoogleProvider;
//!
//! let provider = GoogleProvider::new("AIza...");
//! // Note: stream() will return an error until implemented
//! ```
//!
//! # Error Handling
//!
//! All providers use a comprehensive error type that distinguishes between
//! retryable and non-retryable errors:
//!
//! ```no_run
//! use llm_latency_lens_providers::error::ProviderError;
//!
//! # async fn example() -> Result<(), ProviderError> {
//! # use llm_latency_lens_providers::openai::OpenAIProvider;
//! # use llm_latency_lens_providers::traits::{Provider, StreamingRequest, MessageRole};
//! # use llm_latency_lens_core::TimingEngine;
//! # let provider = OpenAIProvider::new("sk-...");
//! # let timing = TimingEngine::new();
//! # let request = StreamingRequest::builder()
//! #     .model("gpt-4o")
//! #     .message(MessageRole::User, "test")
//! #     .build();
//! match provider.stream(request, &timing).await {
//!     Ok(response) => {
//!         // Process response
//!     }
//!     Err(e) => {
//!         if e.is_retryable() {
//!             eprintln!("Retryable error: {}", e);
//!             if let Some(delay) = e.retry_delay() {
//!                 eprintln!("Retry after {} seconds", delay);
//!             }
//!         } else {
//!             eprintln!("Fatal error: {}", e);
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Timing Measurements
//!
//! All providers integrate with the core timing engine to provide
//! nanosecond-precision measurements:
//!
//! - **TTFT** (Time to First Token): Critical for perceived responsiveness
//! - **Inter-token latency**: Consistency of token generation
//! - **Total generation time**: Overall performance
//! - **Network timing**: DNS, TLS, connection establishment
//!
//! # Cost Calculation
//!
//! Each provider implements accurate cost calculation based on current
//! pricing (as of 2024):
//!
//! ```no_run
//! use llm_latency_lens_providers::openai::OpenAIProvider;
//! use llm_latency_lens_providers::traits::Provider;
//!
//! let provider = OpenAIProvider::new("sk-...");
//!
//! // Calculate cost for GPT-4o
//! let cost = provider.calculate_cost("gpt-4o", 1000, 2000);
//! if let Some(usd) = cost {
//!     println!("Estimated cost: ${:.6}", usd);
//! }
//! ```

pub mod anthropic;
pub mod error;
pub mod google;
pub mod openai;
pub mod traits;

// Re-export commonly used types
pub use error::{ProviderError, Result};
pub use traits::{
    CompletionResult, Message, MessageRole, Provider, ResponseMetadata, StreamingRequest,
    StreamingResponse,
};

// Re-export provider implementations
pub use anthropic::AnthropicProvider;
pub use google::GoogleProvider;
pub use openai::OpenAIProvider;

/// Version of the providers crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Create a provider from a string identifier
///
/// This is a convenience function for dynamically selecting providers.
///
/// # Arguments
///
/// * `provider` - Provider identifier ("openai", "anthropic", "google")
/// * `api_key` - API key for the provider
///
/// # Example
///
/// ```no_run
/// use llm_latency_lens_providers::create_provider;
///
/// let provider = create_provider("openai", "sk-...").unwrap();
/// ```
pub fn create_provider(
    provider: &str,
    api_key: impl Into<String>,
) -> Result<Box<dyn Provider>> {
    match provider.to_lowercase().as_str() {
        "openai" => Ok(Box::new(OpenAIProvider::new(api_key))),
        "anthropic" => Ok(Box::new(AnthropicProvider::new(api_key))),
        "google" => Ok(Box::new(GoogleProvider::new(api_key))),
        _ => Err(ProviderError::ConfigError(format!(
            "Unknown provider: {}. Supported providers: openai, anthropic, google",
            provider
        ))),
    }
}

/// List all supported providers
pub fn supported_providers() -> Vec<&'static str> {
    vec!["openai", "anthropic", "google"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_provider_openai() {
        let provider = create_provider("openai", "test-key");
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn test_create_provider_anthropic() {
        let provider = create_provider("anthropic", "test-key");
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.name(), "anthropic");
    }

    #[test]
    fn test_create_provider_google() {
        let provider = create_provider("google", "test-key");
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.name(), "google");
    }

    #[test]
    fn test_create_provider_unknown() {
        let provider = create_provider("unknown", "test-key");
        assert!(provider.is_err());
    }

    #[test]
    fn test_create_provider_case_insensitive() {
        assert!(create_provider("OpenAI", "test-key").is_ok());
        assert!(create_provider("ANTHROPIC", "test-key").is_ok());
        assert!(create_provider("Google", "test-key").is_ok());
    }

    #[test]
    fn test_supported_providers() {
        let providers = supported_providers();
        assert_eq!(providers.len(), 3);
        assert!(providers.contains(&"openai"));
        assert!(providers.contains(&"anthropic"));
        assert!(providers.contains(&"google"));
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
