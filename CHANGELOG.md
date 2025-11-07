# Changelog

All notable changes to LLM-Latency-Lens will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Google Gemini provider support
- Azure OpenAI support
- Cohere integration
- Prometheus metrics export
- Grafana dashboard templates
- Distributed execution support
- Historical trend analysis
- Real-time monitoring dashboard

---

## [0.1.0] - 2025-11-07

### Added

#### Core Features
- **High-Precision Timing Engine**
  - Nanosecond-accurate timing using `quanta` hardware counters
  - Sub-millisecond precision for TTFT measurements
  - Minimal overhead (<100ns per measurement)

- **Provider Support**
  - OpenAI integration (GPT-4, GPT-4o, GPT-3.5 Turbo)
  - Anthropic integration (Claude 3 Opus, Sonnet, Haiku)
  - Google provider (stub implementation)

- **Streaming Support**
  - Server-Sent Events (SSE) parsing
  - Token-by-token latency tracking
  - Inter-token latency measurements
  - Real-time token throughput

- **Metrics Collection**
  - Time to First Token (TTFT)
  - Total request duration
  - Token throughput (tokens/second)
  - Inter-token latency distribution
  - Statistical aggregation (min, max, mean, median, p50, p95, p99, p999)
  - HDR histogram for accurate percentiles

- **Cost Tracking**
  - Real-time cost calculation
  - Per-request cost breakdown
  - Provider-specific pricing tables
  - Cost projections

#### CLI Interface
- `profile` command for single provider/model profiling
- `compare` command for multi-provider comparison
- `validate` command for credential verification
- `providers` command for listing supported providers
- `analyze` command for post-processing results

#### Configuration
- YAML/JSON/TOML configuration file support
- Environment variable support
- CLI argument overrides
- Configuration validation
- Default presets

#### Export Formats
- JSON export (pretty-printed)
- CSV export
- Binary export (MessagePack)
- Console table output
- Structured logging

#### Performance Features
- Concurrent request execution (1000+ requests)
- Connection pooling with HTTP/2
- Automatic retry with exponential backoff
- Per-provider rate limiting
- Request timeout management
- Warmup requests

#### Documentation
- Comprehensive README
- User guide
- API documentation
- Architecture documentation
- Contributing guide
- Code of conduct
- Security policy
- Marketing materials

#### Testing
- Unit tests across all modules
- Integration tests for providers
- Benchmark tests for performance validation
- Mock HTTP server for testing
- Property-based tests

#### Development Tools
- Rustfmt configuration
- Clippy lints
- Pre-commit hooks
- CI/CD pipeline (GitHub Actions)
- Automated releases

### Changed
- N/A (initial release)

### Deprecated
- N/A (initial release)

### Removed
- N/A (initial release)

### Fixed
- N/A (initial release)

### Security
- Secure API key handling via environment variables
- TLS/SSL certificate verification
- Input validation and sanitization
- Rate limiting to prevent abuse
- Audit logging
- Dependency vulnerability scanning

---

## Release Notes

### Version 0.1.0 - Initial Release

This is the first public release of LLM-Latency-Lens! 🎉

**Highlights:**
- Nanosecond-precision timing for LLM APIs
- Support for OpenAI and Anthropic providers
- Comprehensive metrics including TTFT, throughput, and cost
- High concurrency support (1000+ requests)
- Multiple export formats (JSON, CSV, binary)
- Production-ready reliability with retry logic and rate limiting

**Getting Started:**
```bash
cargo install llm-latency-lens
export OPENAI_API_KEY=sk-...
llm-latency-lens profile --provider openai --model gpt-4 --prompt "Hello"
```

**Documentation:**
- [README](README.md) - Overview and quick start
- [User Guide](docs/USER_GUIDE.md) - Comprehensive usage guide
- [API Documentation](docs/API.md) - Library usage

**Known Limitations:**
- Google provider is stub implementation only
- No GUI/dashboard (CLI only)
- Limited to HTTP-based providers
- No distributed execution support

**Feedback Welcome:**
Please report issues or suggest features at https://github.com/llm-devops/llm-latency-lens/issues

---

## Upgrade Guide

### From Pre-release to 0.1.0

This is the first official release, so there is no upgrade path from previous versions.

---

## Breaking Changes

### Version 0.1.0
- N/A (initial release)

---

## Deprecation Warnings

### Version 0.1.0
- N/A (initial release)

---

## Migration Guide

### Version 0.1.0
- N/A (initial release)

---

## Version History

| Version | Release Date | Status | Support |
|---------|--------------|--------|---------|
| 0.1.0 | 2025-11-07 | Current | Active |

---

## Changelog Guidelines

We follow these principles for our changelog:

1. **Keep a Changelog Format**
   - One section per version
   - Sections: Added, Changed, Deprecated, Removed, Fixed, Security
   - Most recent version at top

2. **Semantic Versioning**
   - MAJOR.MINOR.PATCH (e.g., 1.2.3)
   - MAJOR: Breaking changes
   - MINOR: New features (backward compatible)
   - PATCH: Bug fixes (backward compatible)

3. **Clear Communication**
   - Write for users, not developers
   - Explain impact, not implementation
   - Link to issues/PRs for details
   - Highlight breaking changes

4. **Release Notes**
   - Comprehensive notes for each release
   - Getting started instructions
   - Known issues and limitations
   - Upgrade instructions

---

## Contributing to Changelog

When contributing, please update this file with your changes:

1. Add entry under `[Unreleased]` section
2. Use appropriate category (Added, Changed, Fixed, etc.)
3. Write clear, user-focused descriptions
4. Link to relevant issues or PRs
5. Follow existing format and style

**Example:**

```markdown
## [Unreleased]

### Added
- Google Gemini provider support ([#123](https://github.com/llm-devops/llm-latency-lens/pull/123))
  - Full implementation with streaming
  - Cost calculation for Gemini Pro and Ultra
  - Integration tests

### Fixed
- Handle rate limiting correctly for Anthropic ([#124](https://github.com/llm-devops/llm-latency-lens/issues/124))
  - Parse retry-after header
  - Exponential backoff with jitter
```

---

## Future Releases

### Version 0.2.0 (Planned: Q1 2026)
- Google Gemini provider (full implementation)
- Azure OpenAI support
- Cohere integration
- Prometheus metrics export
- Grafana dashboard templates
- Enhanced error handling
- Performance improvements

### Version 0.3.0 (Planned: Q2 2026)
- InfluxDB integration
- Datadog integration
- Custom provider support
- Advanced rate limiting
- Request replay functionality
- Distributed tracing

### Version 1.0.0 (Planned: Q3 2026)
- Distributed execution
- Real-time monitoring dashboard
- Historical trend analysis
- AI-powered optimization suggestions
- Multi-region testing
- Enterprise features
- Stable API guarantee

---

## Versioning Policy

### Semantic Versioning

We follow [Semantic Versioning 2.0.0](https://semver.org/):

- **Major Version (X.0.0)**: Breaking changes
- **Minor Version (0.X.0)**: New features, backward compatible
- **Patch Version (0.0.X)**: Bug fixes, backward compatible

### Pre-releases

Pre-release versions use suffixes:
- **Alpha**: `0.1.0-alpha.1` - Early development, unstable
- **Beta**: `0.1.0-beta.1` - Feature complete, testing
- **RC**: `0.1.0-rc.1` - Release candidate, final testing

### Support Policy

- **Current Version**: Full support, active development
- **Previous Minor**: Security fixes only (6 months)
- **Older Versions**: No support

---

## Release Process

1. **Update Version**
   - Update `Cargo.toml` version
   - Update `CHANGELOG.md`
   - Update documentation

2. **Testing**
   - Run full test suite
   - Run benchmarks
   - Manual testing

3. **Documentation**
   - Update README if needed
   - Update API docs
   - Update examples

4. **Release**
   - Create git tag
   - Publish to crates.io
   - Create GitHub release
   - Update website

5. **Announcement**
   - Blog post
   - Social media
   - Discord/Slack
   - Email newsletter

---

## Contact

- **Issues**: https://github.com/llm-devops/llm-latency-lens/issues
- **Discussions**: https://github.com/llm-devops/llm-latency-lens/discussions
- **Email**: support@llm-devops.com

---

**Note**: This changelog is maintained by the LLM-Latency-Lens team and community contributors.

[Unreleased]: https://github.com/llm-devops/llm-latency-lens/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/llm-devops/llm-latency-lens/releases/tag/v0.1.0
