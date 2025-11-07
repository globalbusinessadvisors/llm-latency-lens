# LLM Latency Lens - Provider Adapters

Production-ready provider adapters for LLM Latency Lens, enabling high-precision latency measurements and streaming token analysis across multiple LLM providers.

## Features

- **OpenAI**: Full implementation with GPT-4, GPT-4o, and GPT-3.5 support
- **Anthropic**: Complete Claude integration with extended thinking support
- **Google**: Stub implementation for Gemini (coming soon)
- **Streaming**: Server-Sent Events (SSE) with fine-grained token timing
- **Retries**: Automatic retry logic with exponential backoff
- **Cost Calculation**: Accurate pricing for all supported models
- **Error Handling**: Comprehensive error types with retryable/non-retryable classification

## Architecture

### Provider Trait

All providers implement the `Provider` trait which defines:

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn health_check(&self) -> Result<()>;
    async fn stream(&self, request: StreamingRequest, timing_engine: &TimingEngine) -> Result<StreamingResponse>;
    async fn complete(&self, request: StreamingRequest, timing_engine: &TimingEngine) -> Result<CompletionResult>;
    fn calculate_cost(&self, model: &str, input_tokens: u64, output_tokens: u64) -> Option<f64>;
    fn supported_models(&self) -> Vec<String>;
    fn validate_model(&self, model: &str) -> Result<()>;
}
```

### Error Handling

The `ProviderError` enum provides comprehensive error handling:

- `HttpError`: Network-level errors from reqwest
- `ApiError`: API-specific errors with status codes
- `AuthenticationError`: Invalid API keys
- `RateLimitError`: Rate limiting with retry-after
- `TimeoutError`: Request timeouts
- `StreamingError`: SSE streaming errors
- `SseParseError`: SSE parsing errors
- And more...

Each error implements:
- `is_retryable()`: Whether the error should trigger a retry
- `retry_delay()`: Suggested delay before retry

### Streaming Architecture

All providers use Server-Sent Events (SSE) for streaming:

1. **Request Building**: Construct provider-specific request payload
2. **Timing Setup**: Initialize timing engine and checkpoints
3. **SSE Connection**: Create EventSource with reqwest
4. **Token Stream**: Convert SSE events to `TokenEvent` with timing data
5. **Error Handling**: Parse and classify streaming errors

### Timing Measurements

Each token event includes:

- `sequence`: Token position in stream
- `content`: Token text content
- `timestamp_nanos`: Absolute timestamp in nanoseconds
- `time_since_start`: Duration from request start (TTFT for first token)
- `inter_token_latency`: Time since previous token

## Modules

### `error.rs`

Comprehensive error types for all provider operations:

- Error classification (retryable vs non-retryable)
- Error parsing from API responses
- Retry delay calculation
- Status code handling

### `traits.rs`

Core trait definitions and types:

- `Provider` trait
- `StreamingRequest` and builder
- `StreamingResponse` with token stream
- `CompletionResult` with analytics
- `Message` and `MessageRole`
- `ResponseMetadata`

### `openai.rs`

OpenAI Chat Completions API implementation:

**Supported Models:**
- GPT-4o (gpt-4o, gpt-4o-mini)
- GPT-4 Turbo (gpt-4-turbo)
- GPT-4 (gpt-4, gpt-4-32k)
- GPT-3.5 Turbo (gpt-3.5-turbo)

**Features:**
- SSE streaming with delta parsing
- Retry with exponential backoff
- Organization ID support
- Custom endpoint support
- Accurate cost calculation

**API Details:**
- Endpoint: `/v1/chat/completions`
- Authentication: Bearer token
- Streaming: SSE with `data: [DONE]` terminator

### `anthropic.rs`

Anthropic Messages API implementation:

**Supported Models:**
- Claude 3.5 Sonnet (claude-3-5-sonnet-20241022, claude-3-5-sonnet-20240620)
- Claude 3.5 Haiku (claude-3-5-haiku-20241022)
- Claude 3 Opus (claude-3-opus-20240229)
- Claude 3 Sonnet (claude-3-sonnet-20240229)
- Claude 3 Haiku (claude-3-haiku-20240307)

**Features:**
- SSE streaming with content blocks
- Extended thinking token tracking
- System message handling
- Custom API version support
- Accurate cost calculation

**API Details:**
- Endpoint: `/v1/messages`
- Authentication: `x-api-key` header
- API Version: `anthropic-version` header
- Streaming: SSE with typed events

**Event Types:**
- `message_start`: Message metadata
- `content_block_start`: Content block start
- `content_block_delta`: Token deltas (text_delta)
- `content_block_stop`: Content block end
- `message_delta`: Usage statistics
- `message_stop`: Stream completion

### `google.rs`

Google Gemini API stub implementation:

**Supported Models:**
- Gemini 1.5 Pro
- Gemini 1.5 Flash
- Gemini 1.5 Flash-8B
- Gemini 1.0 Pro

**Status:** Stub implementation - returns not implemented error
**Coming Soon:** Full streaming implementation

### `lib.rs`

Main library module with:

- Module exports
- Re-exports of common types
- `create_provider()` factory function
- `supported_providers()` helper
- Comprehensive documentation

## Usage Examples

### OpenAI Provider

```rust
use llm_latency_lens_providers::{
    openai::OpenAIProvider,
    traits::{Provider, StreamingRequest, MessageRole},
};
use llm_latency_lens_core::TimingEngine;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create provider
    let provider = OpenAIProvider::builder()
        .api_key(std::env::var("OPENAI_API_KEY")?)
        .max_retries(3)
        .build();

    // Create timing engine
    let timing = TimingEngine::new();

    // Build request
    let request = StreamingRequest::builder()
        .model("gpt-4o")
        .message(MessageRole::System, "You are a helpful assistant.")
        .message(MessageRole::User, "Explain quantum computing in one paragraph.")
        .max_tokens(200)
        .temperature(0.7)
        .build();

    // Stream response
    let mut response = provider.stream(request, &timing).await?;

    // Process tokens
    while let Some(token) = response.token_stream.next().await {
        let event = token?;
        if let Some(text) = event.content {
            print!("{}", text);
        }

        // Log timing information
        if event.sequence == 0 {
            println!("\nTTFT: {:?}", event.time_since_start);
        }
        if let Some(latency) = event.inter_token_latency {
            println!("Token {} latency: {:?}", event.sequence, latency);
        }
    }

    Ok(())
}
```

### Anthropic Provider

```rust
use llm_latency_lens_providers::{
    anthropic::AnthropicProvider,
    traits::{Provider, StreamingRequest, MessageRole},
};
use llm_latency_lens_core::TimingEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create provider
    let provider = AnthropicProvider::builder()
        .api_key(std::env::var("ANTHROPIC_API_KEY")?)
        .build();

    // Create timing engine
    let timing = TimingEngine::new();

    // Build request with system message
    let request = StreamingRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .message(MessageRole::System, "You are a concise assistant.")
        .message(MessageRole::User, "What is the speed of light?")
        .max_tokens(100)
        .build();

    // Use complete() for full response
    let result = provider.complete(request, &timing).await?;

    println!("Response: {}", result.content);
    println!("TTFT: {:?}", result.ttft());
    println!("Avg inter-token latency: {:?}", result.avg_inter_token_latency());
    println!("Tokens/sec: {:.2}", result.tokens_per_second().unwrap_or(0.0));

    // Calculate cost
    if let (Some(input), Some(output)) = (result.metadata.input_tokens, result.metadata.output_tokens) {
        if let Some(cost) = provider.calculate_cost(&result.metadata.model, input, output) {
            println!("Estimated cost: ${:.6}", cost);
        }
    }

    Ok(())
}
```

### Dynamic Provider Selection

```rust
use llm_latency_lens_providers::{create_provider, traits::*};
use llm_latency_lens_core::TimingEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider_name = std::env::var("PROVIDER")?;
    let api_key = std::env::var("API_KEY")?;

    // Create provider dynamically
    let provider = create_provider(&provider_name, api_key)?;

    // Verify health
    provider.health_check().await?;

    // List supported models
    println!("{} supports: {:?}", provider.name(), provider.supported_models());

    Ok(())
}
```

## Cost Calculation

All providers implement accurate cost calculation based on current pricing (2024):

### OpenAI Pricing (per 1M tokens)

| Model | Input | Output |
|-------|-------|--------|
| gpt-4o | $2.50 | $10.00 |
| gpt-4o-mini | $0.15 | $0.60 |
| gpt-4-turbo | $10.00 | $30.00 |
| gpt-4 | $30.00 | $60.00 |
| gpt-3.5-turbo | $0.50 | $1.50 |

### Anthropic Pricing (per 1M tokens)

| Model | Input | Output |
|-------|-------|--------|
| claude-3-5-sonnet | $3.00 | $15.00 |
| claude-3-5-haiku | $0.80 | $4.00 |
| claude-3-opus | $15.00 | $75.00 |
| claude-3-haiku | $0.25 | $1.25 |

### Google Pricing (per 1M tokens)

| Model | Input | Output |
|-------|-------|--------|
| gemini-1.5-pro | $1.25 | $5.00 |
| gemini-1.5-flash | $0.075 | $0.30 |
| gemini-1.5-flash-8b | $0.0375 | $0.15 |

## Testing

The crate includes comprehensive unit tests for all components:

```bash
cargo test
```

Test coverage includes:
- Provider creation and configuration
- Model validation
- Cost calculation accuracy
- Error classification
- Request building
- Header construction

## Future Enhancements

1. **Google Provider**: Complete Gemini streaming implementation
2. **AWS Bedrock**: Support for Claude and other models on Bedrock
3. **Azure OpenAI**: Azure-specific endpoint support
4. **Metrics Collection**: Built-in Prometheus metrics
5. **Request Caching**: Optional response caching
6. **Batch Requests**: Support for batch API endpoints
7. **Function Calling**: Tool/function calling support
8. **Vision Models**: Image input support

## Dependencies

- `llm-latency-lens-core`: Core timing and types
- `tokio`: Async runtime
- `reqwest`: HTTP client
- `reqwest-eventsource`: SSE parsing
- `serde`/`serde_json`: Serialization
- `async-trait`: Async trait support
- `futures`: Stream utilities
- `thiserror`: Error handling
- `tracing`: Logging

## License

Apache-2.0
