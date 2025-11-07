# Contributing to LLM-Latency-Lens

Thank you for your interest in contributing to LLM-Latency-Lens! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Setup](#development-setup)
4. [How to Contribute](#how-to-contribute)
5. [Coding Standards](#coding-standards)
6. [Testing Guidelines](#testing-guidelines)
7. [Documentation](#documentation)
8. [Pull Request Process](#pull-request-process)
9. [Community](#community)

---

## Code of Conduct

This project adheres to the Contributor Covenant [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to conduct@llm-devops.com.

---

## Getting Started

### Prerequisites

- **Rust**: Version 1.75 or higher
- **Git**: For version control
- **GitHub Account**: For submitting pull requests
- **API Keys**: For testing (OpenAI, Anthropic, etc.)

### Finding Ways to Contribute

There are many ways to contribute:

- **Bug Reports**: Found a bug? [Open an issue](https://github.com/llm-devops/llm-latency-lens/issues/new)
- **Feature Requests**: Have an idea? [Start a discussion](https://github.com/llm-devops/llm-latency-lens/discussions)
- **Code Contributions**: Fix bugs or implement features
- **Documentation**: Improve docs, add examples, fix typos
- **Testing**: Write tests, improve test coverage
- **Community**: Answer questions, help others

### Good First Issues

Look for issues labeled [`good first issue`](https://github.com/llm-devops/llm-latency-lens/labels/good%20first%20issue) - these are great for newcomers!

---

## Development Setup

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR-USERNAME/llm-latency-lens.git
cd llm-latency-lens

# Add upstream remote
git remote add upstream https://github.com/llm-devops/llm-latency-lens.git
```

### 2. Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
rustup component add rustfmt clippy

# Build the project
cargo build
```

### 3. Set Up Environment

```bash
# Copy example environment file
cp .env.example .env

# Add your API keys
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
```

### 4. Verify Setup

```bash
# Run tests
cargo test

# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt

# Build documentation
cargo doc --open
```

---

## How to Contribute

### Reporting Bugs

Before creating a bug report:

1. **Check existing issues** to avoid duplicates
2. **Verify the bug** in the latest version
3. **Collect information**: version, OS, steps to reproduce

Create a detailed bug report including:

```markdown
**Description**
A clear description of the bug.

**To Reproduce**
Steps to reproduce:
1. Run command '...'
2. See error '...'

**Expected Behavior**
What you expected to happen.

**Actual Behavior**
What actually happened.

**Environment**
- OS: [e.g., Ubuntu 22.04]
- Rust version: [e.g., 1.75.0]
- LLM-Latency-Lens version: [e.g., 0.1.0]

**Additional Context**
Any other relevant information.
```

### Suggesting Features

Feature requests are welcome! Please include:

1. **Use Case**: What problem does this solve?
2. **Proposed Solution**: How should it work?
3. **Alternatives**: Other approaches you considered
4. **Additional Context**: Examples, mockups, etc.

### Contributing Code

1. **Pick an Issue**: Choose an issue to work on or create one
2. **Discuss**: Comment on the issue to discuss your approach
3. **Create Branch**: Create a feature branch from `main`
4. **Make Changes**: Implement your changes
5. **Test**: Add tests and ensure all tests pass
6. **Document**: Update documentation as needed
7. **Submit PR**: Create a pull request

---

## Coding Standards

### Rust Style Guidelines

We follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/README.html).

**Key Points:**

- Use `rustfmt` for formatting (run `cargo fmt`)
- Use `clippy` for linting (run `cargo clippy`)
- Limit line length to 100 characters
- Use meaningful variable names
- Add comments for complex logic
- Keep functions focused and small

**Example:**

```rust
/// Calculate the mean of a vector of values.
///
/// # Arguments
///
/// * `values` - A slice of f64 values
///
/// # Returns
///
/// The arithmetic mean, or None if the slice is empty
///
/// # Example
///
/// ```
/// let values = vec![1.0, 2.0, 3.0];
/// let mean = calculate_mean(&values);
/// assert_eq!(mean, Some(2.0));
/// ```
pub fn calculate_mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    let sum: f64 = values.iter().sum();
    Some(sum / values.len() as f64)
}
```

### Error Handling

Use `Result<T, E>` for recoverable errors and `panic!` only for unrecoverable errors.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, MyError>;
```

### Async Code

Follow async best practices:

```rust
use tokio;

#[tokio::test]
async fn test_async_function() {
    let result = my_async_function().await;
    assert!(result.is_ok());
}
```

### Naming Conventions

- **Types**: `PascalCase` (e.g., `StreamingRequest`)
- **Functions**: `snake_case` (e.g., `calculate_mean`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RETRIES`)
- **Modules**: `snake_case` (e.g., `timing_engine`)

---

## Testing Guidelines

### Test Coverage

We aim for >80% test coverage. All new code should include tests.

### Test Types

**Unit Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_mean() {
        let values = vec![1.0, 2.0, 3.0];
        let mean = calculate_mean(&values);
        assert_eq!(mean, Some(2.0));
    }

    #[test]
    fn test_empty_vector() {
        let values = vec![];
        let mean = calculate_mean(&values);
        assert_eq!(mean, None);
    }
}
```

**Integration Tests:**

```rust
// tests/integration_test.rs
use llm_latency_lens_providers::{OpenAIProvider, Provider};

#[tokio::test]
async fn test_openai_integration() {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap();
    let provider = OpenAIProvider::new(api_key);

    let models = provider.list_models().await;
    assert!(models.is_ok());
}
```

**Benchmark Tests:**

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("calculate_mean", |b| {
        let values = vec![1.0; 1000];
        b.iter(|| calculate_mean(black_box(&values)));
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run integration tests
cargo test --test integration_test

# Run benchmarks
cargo bench

# Run with coverage
cargo tarpaulin --out Html
```

---

## Documentation

### Code Documentation

All public APIs must be documented with rustdoc:

```rust
/// Create a new timing engine.
///
/// The timing engine uses hardware counters for nanosecond-precision
/// timing measurements.
///
/// # Example
///
/// ```
/// use llm_latency_lens_core::TimingEngine;
///
/// let engine = TimingEngine::new();
/// let start = engine.now();
/// // ... perform operation ...
/// let elapsed = engine.elapsed_nanos(start);
/// ```
pub fn new() -> Self {
    // Implementation
}
```

### User Documentation

Update documentation when adding features:

- `README.md` - Overview and quick start
- `docs/USER_GUIDE.md` - Comprehensive user guide
- `docs/API.md` - API documentation
- `CHANGELOG.md` - Version history

### Examples

Add examples for new features:

```rust
// examples/my_feature.rs
use llm_latency_lens::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example code here
    Ok(())
}
```

---

## Pull Request Process

### 1. Create a Feature Branch

```bash
# Update your fork
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/my-feature
```

### 2. Make Changes

- Write code following our standards
- Add tests for new functionality
- Update documentation
- Ensure all tests pass

### 3. Commit Changes

Use clear, descriptive commit messages:

```bash
# Good commit messages
git commit -m "Add support for Google Gemini provider"
git commit -m "Fix: Handle rate limiting in retry logic"
git commit -m "Docs: Update API documentation for streaming"

# Follow conventional commits format
# type(scope): description
#
# Types: feat, fix, docs, style, refactor, test, chore
```

### 4. Push and Create PR

```bash
# Push to your fork
git push origin feature/my-feature

# Create pull request on GitHub
```

### 5. PR Description

Include in your PR description:

```markdown
## Description
Brief description of changes.

## Motivation
Why is this change needed?

## Changes
- Added X
- Fixed Y
- Updated Z

## Testing
How was this tested?

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Changelog updated
- [ ] Code formatted (`cargo fmt`)
- [ ] Lints pass (`cargo clippy`)
- [ ] All tests pass (`cargo test`)
```

### 6. Review Process

- **Automated Checks**: CI/CD runs tests and lints
- **Code Review**: Maintainers review your code
- **Feedback**: Address review comments
- **Approval**: Once approved, maintainers will merge

### 7. After Merge

```bash
# Update your fork
git checkout main
git pull upstream main
git push origin main

# Delete feature branch
git branch -d feature/my-feature
git push origin --delete feature/my-feature
```

---

## Community

### Getting Help

- **GitHub Discussions**: [Ask questions](https://github.com/llm-devops/llm-latency-lens/discussions)
- **Discord**: [Join our community](https://discord.gg/llm-latency-lens)
- **Stack Overflow**: Tag questions with `llm-latency-lens`

### Community Calls

We host monthly community calls. See the [calendar](https://llm-latency-lens.dev/calendar) for dates.

### Recognition

Contributors are recognized in:

- `CONTRIBUTORS.md` file
- Release notes
- Annual contributor report

---

## Release Process

Releases are handled by maintainers:

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create release tag
4. Publish to crates.io
5. Create GitHub release
6. Announce on social media

---

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

---

## Questions?

If you have questions about contributing, feel free to:

- Open a [GitHub Discussion](https://github.com/llm-devops/llm-latency-lens/discussions)
- Ask in [Discord](https://discord.gg/llm-latency-lens)
- Email: contribute@llm-devops.com

Thank you for contributing to LLM-Latency-Lens! Every contribution, no matter how small, helps make the project better.

---

**Version**: 1.0
**Last Updated**: 2025-11-07
