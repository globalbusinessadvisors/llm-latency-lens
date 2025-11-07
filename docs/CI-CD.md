# CI/CD Guide for LLM-Latency-Lens

This guide covers the Continuous Integration and Continuous Deployment (CI/CD) pipelines for LLM-Latency-Lens.

## Table of Contents

- [Overview](#overview)
- [GitHub Actions Workflows](#github-actions-workflows)
- [CI Pipeline](#ci-pipeline)
- [Security Pipeline](#security-pipeline)
- [Release Pipeline](#release-pipeline)
- [Docker Build Pipeline](#docker-build-pipeline)
- [Configuration](#configuration)
- [Secrets Management](#secrets-management)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

LLM-Latency-Lens uses GitHub Actions for automated CI/CD with the following workflows:

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | Push, PR | Code quality, tests, builds |
| `security.yml` | Push, PR, Daily | Security scanning |
| `release.yml` | Tag push | Release automation |
| `docker-build.yml` | PR | Docker image validation |

## GitHub Actions Workflows

### Workflow Files

All workflows are located in `.github/workflows/`:

```
.github/workflows/
├── ci.yml              # Main CI pipeline
├── security.yml        # Security scanning
├── release.yml         # Release automation
└── docker-build.yml    # Docker validation
```

## CI Pipeline

The main CI pipeline (`ci.yml`) runs on every push and pull request.

### Jobs

#### 1. Format Check

Ensures code follows Rust formatting standards:

```yaml
- cargo fmt --all -- --check
```

**Fix formatting issues:**
```bash
cargo fmt --all
```

#### 2. Clippy Lints

Runs Clippy linter with strict warnings:

```yaml
- cargo clippy --all-targets --all-features -- -D warnings
```

**Fix clippy warnings:**
```bash
cargo clippy --fix --all-targets --all-features
```

#### 3. Unit Tests

Runs tests on multiple platforms:

- **Platforms**: Ubuntu, macOS, Windows
- **Rust versions**: stable, beta

```yaml
- cargo test --all-features --workspace --verbose
- cargo test --doc --all-features --workspace
```

**Run tests locally:**
```bash
cargo test --all-features --workspace
```

#### 4. Integration Tests

Runs integration tests:

```yaml
- cargo test --test '*' --all-features --workspace
```

#### 5. Security Audit

Checks for security vulnerabilities:

```yaml
- cargo audit --deny warnings
```

**Run audit locally:**
```bash
cargo install cargo-audit
cargo audit
```

#### 6. Dependency Check

Validates dependencies with cargo-deny:

```yaml
- cargo deny check
```

**Configure in `deny.toml`**

#### 7. Code Coverage

Generates code coverage reports:

```yaml
- cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

Uploads to Codecov for tracking.

#### 8. Build Release Binaries

Builds release binaries for all platforms:

- Linux (x86_64, ARM64, MUSL)
- macOS (x86_64, ARM64)
- Windows (x86_64)

Artifacts are uploaded for each platform.

#### 9. Docker Build

Builds and pushes Docker images (on main/develop):

- Multi-platform: linux/amd64, linux/arm64
- Registries: Docker Hub, GHCR
- Tags: branch name, SHA

#### 10. Benchmarks

Runs performance benchmarks (on main branch):

```yaml
- cargo bench --all-features --workspace
```

### Caching Strategy

The CI uses aggressive caching for speed:

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
```

**Cache keys are based on:**
- OS
- Rust version
- Cargo.lock hash

### Matrix Testing

Tests run on multiple configurations:

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, beta]
```

## Security Pipeline

The security pipeline (`security.yml`) runs daily and on every push/PR.

### Security Jobs

#### 1. Cargo Audit

Checks for known security vulnerabilities:

```bash
cargo audit --deny warnings
```

#### 2. Dependency Review

Reviews dependency changes in PRs:

```yaml
- uses: actions/dependency-review-action@v3
  with:
    fail-on-severity: moderate
    deny-licenses: GPL-2.0, GPL-3.0, AGPL-3.0
```

#### 3. Semgrep SAST

Static Application Security Testing:

```yaml
- semgrep scan --config=auto --error
```

#### 4. CodeQL Analysis

Advanced code analysis for vulnerabilities:

```yaml
- uses: github/codeql-action/analyze@v2
  with:
    queries: security-extended,security-and-quality
```

#### 5. Gitleaks Secret Scanning

Scans for exposed secrets:

```yaml
- uses: gitleaks/gitleaks-action@v2
```

#### 6. Trivy Vulnerability Scanner

Scans filesystem and Docker images:

```yaml
- uses: aquasecurity/trivy-action@master
  with:
    severity: 'CRITICAL,HIGH'
```

#### 7. SBOM Generation

Generates Software Bill of Materials:

```bash
cargo sbom --output-format json > sbom.json
```

### Security Alerts

Security findings are:
- Uploaded to GitHub Security tab
- Visible in PR checks
- Sent to configured channels

## Release Pipeline

The release pipeline (`release.yml`) automates the release process.

### Triggering Releases

#### Manual Release

```bash
gh workflow run release.yml -f version=1.0.0
```

#### Tag-based Release

```bash
git tag v1.0.0
git push origin v1.0.0
```

### Release Jobs

#### 1. Create GitHub Release

- Generates changelog with git-cliff
- Creates GitHub release
- Attaches release notes

#### 2. Build Release Binaries

Builds optimized binaries for all platforms:

```yaml
profile.release:
  opt-level = 3
  lto = "fat"
  codegen-units = 1
  strip = true
```

#### 3. Upload Assets

Uploads to GitHub release:
- Binary archives (tar.gz, zip)
- Checksums (SHA256)
- README and LICENSE

#### 4. Docker Release

Builds and pushes Docker images with version tags:

```
llm-devops/llm-latency-lens:1.0.0
llm-devops/llm-latency-lens:latest
ghcr.io/llm-devops/llm-latency-lens:1.0.0
ghcr.io/llm-devops/llm-latency-lens:latest
```

#### 5. Publish to Crates.io

Publishes workspace crates in dependency order:

1. llm-latency-lens-core
2. llm-latency-lens-providers
3. llm-latency-lens-metrics
4. llm-latency-lens-exporters
5. llm-latency-lens

#### 6. Update Homebrew

Updates Homebrew formula (if configured):

```ruby
class LlmLatencyLens < Formula
  desc "Enterprise-grade LLM profiler"
  homepage "https://github.com/llm-devops/llm-latency-lens"
  url "https://github.com/.../releases/download/v1.0.0/..."
  sha256 "..."
end
```

#### 7. Generate SBOM

Generates and uploads SBOM for the release.

#### 8. Notifications

Sends release notifications to:
- Slack
- Email
- Other configured channels

### Changelog Generation

Uses git-cliff for conventional commit-based changelogs:

```toml
# cliff.toml
[git]
conventional_commits = true
commit_parsers = [
  { message = "^feat", group = "Features" },
  { message = "^fix", group = "Bug Fixes" },
  { message = "^doc", group = "Documentation" },
  ...
]
```

## Docker Build Pipeline

The Docker build pipeline (`docker-build.yml`) validates Docker images on PRs.

### Jobs

#### 1. Build Image

Builds for multiple platforms:
- linux/amd64
- linux/arm64

#### 2. Test Image

Tests Docker image functionality:

```bash
docker run --rm llm-latency-lens:test --help
docker run --rm llm-latency-lens:test --version
```

#### 3. Check Image Size

Ensures image size is under 100MB:

```bash
SIZE_MB=$((SIZE / 1024 / 1024))
if [ $SIZE_MB -gt 100 ]; then
  echo "Image too large!"
  exit 1
fi
```

#### 4. Security Scan

Scans image for vulnerabilities:

```yaml
- uses: aquasecurity/trivy-action@master
  with:
    severity: 'CRITICAL,HIGH'
    exit-code: '1'
```

#### 5. Best Practices Check

Validates Dockerfile with Dockle:

```yaml
- uses: goodwithtech/dockle-action@main
  with:
    exit-level: 'warn'
```

#### 6. Performance Benchmark

Measures container startup time:

```bash
START=$(date +%s%N)
docker run --rm llm-latency-lens:test --version
END=$(date +%s%N)
DURATION=$((($END - $START) / 1000000))
```

## Configuration

### Required Secrets

Configure in GitHub Settings → Secrets and variables → Actions:

| Secret | Purpose | Required |
|--------|---------|----------|
| `DOCKER_USERNAME` | Docker Hub username | Yes (for Docker push) |
| `DOCKER_PASSWORD` | Docker Hub password/token | Yes (for Docker push) |
| `CARGO_REGISTRY_TOKEN` | Crates.io API token | Yes (for publishing) |
| `HOMEBREW_TAP_TOKEN` | GitHub PAT for Homebrew | No (optional) |
| `SLACK_WEBHOOK_URL` | Slack webhook for notifications | No (optional) |
| `SNYK_TOKEN` | Snyk API token | No (optional) |
| `GITLEAKS_LICENSE` | Gitleaks license | No (optional) |

### Generating Tokens

#### Docker Hub Token

1. Login to Docker Hub
2. Account Settings → Security → New Access Token
3. Copy token

#### Crates.io Token

```bash
cargo login
# Follow prompts to get token
```

#### GitHub PAT

1. Settings → Developer settings → Personal access tokens
2. Generate new token (classic)
3. Select scopes: `repo`, `write:packages`

### Environment Variables

Set in workflow files or repository variables:

```yaml
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUSTFLAGS: -D warnings
```

## Best Practices

### Commit Messages

Use conventional commits for changelog generation:

```
feat: add new latency metric
fix: correct TTFT calculation
docs: update Docker guide
chore: bump dependencies
```

### Versioning

Follow Semantic Versioning:

```
MAJOR.MINOR.PATCH
- MAJOR: Breaking changes
- MINOR: New features
- PATCH: Bug fixes
```

### Branch Protection

Configure branch protection rules:

- ✅ Require pull request reviews
- ✅ Require status checks to pass
- ✅ Require branches to be up to date
- ✅ Include administrators

### PR Checks

All PRs must pass:

- ✅ Format check
- ✅ Clippy lints
- ✅ Unit tests (all platforms)
- ✅ Integration tests
- ✅ Security audit
- ✅ Dependency check
- ✅ Docker build (if Dockerfile changed)

### Dependency Updates

Dependabot is configured to:

- Check weekly
- Group related updates
- Auto-label PRs
- Limit open PRs to 10

### Security Scanning

Security scans run:

- ✅ On every push
- ✅ On every PR
- ✅ Daily at 2 AM UTC
- ✅ Before releases

## Troubleshooting

### CI Failures

#### Format Check Failed

```bash
# Fix locally
cargo fmt --all

# Commit and push
git add .
git commit -m "chore: fix formatting"
git push
```

#### Clippy Failed

```bash
# See warnings
cargo clippy --all-targets --all-features

# Auto-fix
cargo clippy --fix --all-targets --all-features

# Commit and push
git add .
git commit -m "fix: address clippy warnings"
git push
```

#### Tests Failed

```bash
# Run tests locally
cargo test --all-features --workspace --verbose

# Debug specific test
cargo test --test test_name -- --nocapture

# Fix and push
```

#### Build Failed

```bash
# Clean and rebuild
cargo clean
cargo build --release

# Check for platform-specific issues
cargo build --release --target x86_64-unknown-linux-gnu
```

### Cache Issues

If builds are slow or failing due to cache:

```yaml
# In workflow file, update cache key
key: ${{ runner.os }}-cargo-v2-${{ hashFiles('**/Cargo.lock') }}
```

Or manually clear cache:
1. Actions → Caches → Delete cache

### Docker Build Issues

#### Image Too Large

```bash
# Check layers
docker history llm-latency-lens

# Optimize Dockerfile
# - Use smaller base image
# - Combine RUN commands
# - Remove unnecessary files
```

#### Build Timeout

Increase timeout:

```yaml
- name: Build
  timeout-minutes: 30
```

#### Platform Build Fails

```bash
# Test locally with QEMU
docker buildx create --use
docker buildx build --platform linux/arm64 -t test .
```

### Release Issues

#### Publish to Crates.io Failed

```bash
# Verify version number
cargo publish --dry-run

# Check dependencies are published
cargo tree
```

#### GitHub Release Failed

Ensure:
- Tag format is correct (v1.0.0)
- GITHUB_TOKEN has permissions
- Release doesn't already exist

### Security Scan Failures

#### False Positives

Add to ignore list:

```toml
# deny.toml
[advisories]
ignore = [
    "RUSTSEC-2021-0124",  # Description of why ignored
]
```

#### Dependency Vulnerabilities

```bash
# Update vulnerable dependency
cargo update -p vulnerable-crate

# Or upgrade to newer version
cargo upgrade vulnerable-crate
```

## Additional Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Semantic Versioning](https://semver.org/)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny)
- [git-cliff](https://github.com/orhun/git-cliff)

## Support

For CI/CD issues:
1. Check workflow logs
2. Review this documentation
3. Search existing issues
4. Open a new issue with:
   - Workflow name
   - Error message
   - Relevant logs
