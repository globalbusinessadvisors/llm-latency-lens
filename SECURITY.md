# Security Policy

## Overview

Security is a top priority for LLM-Latency-Lens. This document outlines our security policies, vulnerability reporting process, and best practices.

---

## Supported Versions

We provide security updates for the following versions:

| Version | Supported          | End of Support |
| ------- | ------------------ | -------------- |
| 0.1.x   | :white_check_mark: | TBD            |
| < 0.1   | :x:                | -              |

---

## Reporting a Vulnerability

### Where to Report

**DO NOT** open public GitHub issues for security vulnerabilities.

Instead, report security vulnerabilities to:

**Email**: security@llm-devops.com

**PGP Key**: Available at https://llm-latency-lens.dev/pgp-key.asc

### What to Include

Please include the following information:

1. **Description**: Clear description of the vulnerability
2. **Impact**: Potential security impact
3. **Reproduction**: Step-by-step instructions to reproduce
4. **Environment**: Version, OS, configuration details
5. **Proof of Concept**: Code or commands demonstrating the issue (if applicable)
6. **Suggested Fix**: Your thoughts on how to fix it (optional)

### Response Timeline

- **Acknowledgment**: Within 24 hours
- **Initial Assessment**: Within 72 hours
- **Status Update**: Weekly until resolved
- **Fix Timeline**: Depends on severity (see below)

### Severity Levels

| Severity | Description | Fix Timeline | Example |
|----------|-------------|--------------|---------|
| **Critical** | Remote code execution, data breach | 24-48 hours | API key exposure in logs |
| **High** | Privilege escalation, DoS | 1-2 weeks | Authentication bypass |
| **Medium** | Information disclosure | 2-4 weeks | Timing attack vulnerability |
| **Low** | Minor issues with limited impact | 4-8 weeks | Verbose error messages |

---

## Security Best Practices

### API Key Management

#### DO NOT

```rust
// ❌ BAD: Hardcoded API keys
let api_key = "sk-1234567890abcdef";
let provider = OpenAIProvider::new(api_key);
```

```rust
// ❌ BAD: Keys in version control
// config.toml
api_key = "sk-1234567890abcdef"
```

```rust
// ❌ BAD: Keys in logs
println!("Using API key: {}", api_key);
```

#### DO

```rust
// ✅ GOOD: Environment variables
use std::env;

let api_key = env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY not set");
let provider = OpenAIProvider::new(api_key);
```

```rust
// ✅ GOOD: .env file (not in git)
// .env
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

// .gitignore
.env
```

```rust
// ✅ GOOD: Redacted logging
println!("Using API key: {}***", &api_key[..8]);
```

### Configuration Security

#### Secure File Permissions

```bash
# Restrict config file permissions
chmod 600 config.yaml
chmod 600 .env

# Verify permissions
ls -la config.yaml
# Should show: -rw------- (600)
```

#### Configuration Validation

```rust
// Validate configuration before use
pub fn validate_config(config: &Config) -> Result<(), ConfigError> {
    // Check for suspicious values
    if config.providers.is_empty() {
        return Err(ConfigError::NoProviders);
    }

    // Validate URLs
    for provider in &config.providers {
        if !provider.endpoint.starts_with("https://") {
            return Err(ConfigError::InsecureEndpoint(
                provider.endpoint.clone()
            ));
        }
    }

    Ok(())
}
```

### Network Security

#### TLS/SSL Configuration

```rust
// Always use HTTPS
let provider = OpenAIProvider::builder()
    .api_key(api_key)
    .base_url("https://api.openai.com/v1")  // ✅ HTTPS
    .verify_ssl(true)  // ✅ Verify certificates
    .build();
```

#### Certificate Validation

```yaml
# config.yaml
execution:
  http:
    verify_ssl: true  # Always verify SSL certificates
    ca_bundle: /etc/ssl/certs/ca-certificates.crt  # Custom CA bundle if needed
```

### Dependency Security

#### Regular Updates

```bash
# Check for security vulnerabilities
cargo audit

# Update dependencies
cargo update

# Check for outdated dependencies
cargo outdated
```

#### Dependency Pinning

```toml
# Cargo.toml - Pin major versions
[dependencies]
tokio = "1.41"  # Lock to 1.x
reqwest = "0.12"  # Lock to 0.12.x
```

### Secrets Management

#### Using Secret Stores

```rust
// Example: AWS Secrets Manager
use aws_sdk_secretsmanager::Client;

async fn get_api_key() -> Result<String, Box<dyn std::error::Error>> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    let secret = client
        .get_secret_value()
        .secret_id("llm-api-keys/openai")
        .send()
        .await?;

    Ok(secret.secret_string().unwrap().to_string())
}
```

#### Using HashiCorp Vault

```rust
// Example: HashiCorp Vault
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

async fn get_api_key() -> Result<String, Box<dyn std::error::Error>> {
    let client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address("https://vault.example.com")
            .token(env::var("VAULT_TOKEN")?)
            .build()?
    )?;

    let secret = client
        .read("secret/data/llm-api-keys")
        .await?;

    Ok(secret["data"]["openai_key"].as_str().unwrap().to_string())
}
```

### Input Validation

```rust
// Validate user inputs
pub fn validate_prompt(prompt: &str) -> Result<(), ValidationError> {
    // Check length
    if prompt.is_empty() {
        return Err(ValidationError::EmptyPrompt);
    }

    if prompt.len() > MAX_PROMPT_LENGTH {
        return Err(ValidationError::PromptTooLong);
    }

    // Check for suspicious content
    if contains_injection_patterns(prompt) {
        return Err(ValidationError::SuspiciousContent);
    }

    Ok(())
}
```

### Error Handling

```rust
// Don't expose sensitive information in errors
pub enum Error {
    // ❌ BAD: Exposes API key
    AuthError(String),

    // ✅ GOOD: Generic error
    AuthenticationFailed,
}

// Error messages
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            // ❌ BAD: Exposes details
            Error::AuthError(key) => write!(f, "Auth failed with key: {}", key),

            // ✅ GOOD: Generic message
            Error::AuthenticationFailed => write!(f, "Authentication failed"),
        }
    }
}
```

---

## Security Features

### Rate Limiting

Built-in rate limiting prevents API abuse:

```rust
use llm_latency_lens::RateLimiter;

let limiter = RateLimiter::new()
    .requests_per_second(10)
    .burst(20)
    .build();
```

### Request Timeouts

Prevent resource exhaustion with timeouts:

```rust
let request = StreamingRequest::builder()
    .model("gpt-4")
    .prompt("Hello")
    .timeout(Duration::from_secs(30))  // 30 second timeout
    .build();
```

### Connection Pooling

Secure connection management:

```yaml
execution:
  http:
    pool_size: 100
    pool_idle_timeout: 90  # Close idle connections
    max_idle_per_host: 10
```

---

## Audit Logging

### Enable Audit Logs

```rust
use tracing_subscriber::fmt::format::FmtSpan;

tracing_subscriber::fmt()
    .with_env_filter("llm_latency_lens=info")
    .with_span_events(FmtSpan::CLOSE)
    .json()  // Structured logging
    .init();
```

### What to Log

**DO Log:**
- Authentication attempts (success/failure)
- API calls (timestamp, provider, model)
- Errors and exceptions
- Configuration changes
- Rate limit violations

**DO NOT Log:**
- API keys or credentials
- Sensitive prompt content
- User PII
- Full request/response bodies

### Example Audit Log Entry

```json
{
  "timestamp": "2024-11-07T18:30:00Z",
  "event": "api_call",
  "provider": "openai",
  "model": "gpt-4",
  "status": "success",
  "duration_ms": 1234,
  "user_id": "user_abc123",
  "request_id": "req_xyz789"
}
```

---

## Compliance

### GDPR Compliance

- **Data Minimization**: Only collect necessary data
- **Right to Erasure**: Provide data deletion capabilities
- **Data Portability**: Export data in standard formats
- **Privacy by Design**: Built-in privacy features

### SOC 2 Compliance

- **Security**: Secure credential management
- **Availability**: High availability architecture
- **Processing Integrity**: Data integrity checks
- **Confidentiality**: Encryption at rest and in transit
- **Privacy**: Privacy controls and consent management

### HIPAA Compliance (for healthcare applications)

- **Encryption**: All data encrypted
- **Audit Logs**: Comprehensive audit trails
- **Access Controls**: Role-based access
- **Data Retention**: Configurable retention policies

---

## Security Checklist

### Development

- [ ] No hardcoded credentials
- [ ] Input validation on all inputs
- [ ] Secure error messages
- [ ] Dependencies up to date
- [ ] Security lints pass (`cargo clippy`)
- [ ] No vulnerable dependencies (`cargo audit`)

### Deployment

- [ ] HTTPS only (no HTTP)
- [ ] SSL certificate verification enabled
- [ ] API keys in environment variables or secret store
- [ ] File permissions restricted (600 for sensitive files)
- [ ] Audit logging enabled
- [ ] Rate limiting configured
- [ ] Timeouts configured
- [ ] Resource limits set

### Operations

- [ ] Regular security updates
- [ ] Monitor audit logs
- [ ] Rotate credentials periodically
- [ ] Review access logs
- [ ] Test disaster recovery
- [ ] Security scanning enabled

---

## Known Security Considerations

### 1. API Key Exposure

**Risk**: API keys could be exposed in logs, error messages, or version control.

**Mitigation**:
- Never hardcode API keys
- Use environment variables or secret stores
- Implement key redaction in logs
- Add .env to .gitignore

### 2. Rate Limiting Bypass

**Risk**: Malicious users could bypass rate limits.

**Mitigation**:
- Implement both client-side and server-side rate limiting
- Use token bucket algorithm
- Monitor for suspicious patterns

### 3. Prompt Injection

**Risk**: User prompts could contain malicious content.

**Mitigation**:
- Validate and sanitize inputs
- Implement content filtering
- Use provider safety features

### 4. Denial of Service

**Risk**: Resource exhaustion from excessive requests.

**Mitigation**:
- Configure request timeouts
- Implement rate limiting
- Set connection pool limits
- Monitor resource usage

---

## Security Contact

For security issues:

- **Email**: security@llm-devops.com
- **PGP Key**: https://llm-latency-lens.dev/pgp-key.asc
- **Bug Bounty**: Contact us for details

---

## Acknowledgments

We thank the following security researchers for responsibly disclosing vulnerabilities:

*(List will be updated as vulnerabilities are reported and fixed)*

---

## Security Updates

Subscribe to security updates:

- **GitHub Watch**: Watch the repository for security advisories
- **Mailing List**: security-announce@llm-devops.com
- **RSS Feed**: https://llm-latency-lens.dev/security.rss

---

## Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [API Security Best Practices](https://owasp.org/www-project-api-security/)
- [Secrets Management Best Practices](https://www.vaultproject.io/docs/what-is-vault)

---

**Version**: 1.0
**Last Updated**: 2025-11-07

For questions about this security policy, contact security@llm-devops.com.
