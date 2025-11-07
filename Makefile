# Makefile for LLM-Latency-Lens
# Provides convenient commands for development, testing, and deployment

.PHONY: help
help: ## Show this help message
	@echo "LLM-Latency-Lens - Makefile Commands"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# =============================================================================
# Development
# =============================================================================

.PHONY: dev
dev: ## Run in development mode
	cargo run -- --help

.PHONY: build
build: ## Build debug binary
	cargo build

.PHONY: build-release
build-release: ## Build optimized release binary
	cargo build --release

.PHONY: clean
clean: ## Clean build artifacts
	cargo clean
	rm -rf target/

.PHONY: fmt
fmt: ## Format code
	cargo fmt --all

.PHONY: fmt-check
fmt-check: ## Check code formatting
	cargo fmt --all -- --check

.PHONY: clippy
clippy: ## Run Clippy lints
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: fix
fix: ## Auto-fix Clippy warnings
	cargo clippy --fix --all-targets --all-features

# =============================================================================
# Testing
# =============================================================================

.PHONY: test
test: ## Run all tests
	cargo test --all-features --workspace --verbose

.PHONY: test-unit
test-unit: ## Run unit tests only
	cargo test --lib --all-features --workspace

.PHONY: test-integration
test-integration: ## Run integration tests only
	cargo test --test '*' --all-features --workspace

.PHONY: test-doc
test-doc: ## Run documentation tests
	cargo test --doc --all-features --workspace

.PHONY: bench
bench: ## Run benchmarks
	cargo bench --all-features --workspace

.PHONY: coverage
coverage: ## Generate code coverage report
	cargo install cargo-llvm-cov --locked
	cargo llvm-cov --all-features --workspace --html
	@echo "Coverage report generated at: target/llvm-cov/html/index.html"

# =============================================================================
# Quality & Security
# =============================================================================

.PHONY: audit
audit: ## Run security audit
	cargo install cargo-audit --locked
	cargo audit

.PHONY: deny
deny: ## Check dependencies with cargo-deny
	cargo install cargo-deny --locked
	cargo deny check

.PHONY: outdated
outdated: ## Check for outdated dependencies
	cargo install cargo-outdated --locked
	cargo outdated

.PHONY: update
update: ## Update dependencies
	cargo update

.PHONY: sbom
sbom: ## Generate Software Bill of Materials
	cargo install cargo-sbom --locked
	cargo sbom --output-format json > sbom.json
	@echo "SBOM generated: sbom.json"

# =============================================================================
# Docker
# =============================================================================

.PHONY: docker-build
docker-build: ## Build Docker image
	docker build -t llm-latency-lens:latest .

.PHONY: docker-build-no-cache
docker-build-no-cache: ## Build Docker image without cache
	docker build --no-cache -t llm-latency-lens:latest .

.PHONY: docker-run
docker-run: ## Run Docker container
	docker run --rm llm-latency-lens:latest --help

.PHONY: docker-shell
docker-shell: ## Open shell in Docker container
	docker run -it --rm --entrypoint /bin/sh llm-latency-lens:latest

.PHONY: docker-size
docker-size: ## Show Docker image size
	@docker images llm-latency-lens:latest --format "Size: {{.Size}}"

.PHONY: docker-inspect
docker-inspect: ## Inspect Docker image
	docker inspect llm-latency-lens:latest

.PHONY: docker-history
docker-history: ## Show Docker image layer history
	docker history llm-latency-lens:latest

# =============================================================================
# Docker Compose
# =============================================================================

.PHONY: up
up: ## Start all services with Docker Compose
	docker-compose up -d

.PHONY: down
down: ## Stop all services
	docker-compose down

.PHONY: restart
restart: ## Restart all services
	docker-compose restart

.PHONY: logs
logs: ## Show logs from all services
	docker-compose logs -f

.PHONY: logs-app
logs-app: ## Show logs from application
	docker-compose logs -f llm-latency-lens

.PHONY: ps
ps: ## Show running services
	docker-compose ps

.PHONY: stats
stats: ## Show container resource usage
	docker stats

# =============================================================================
# Production
# =============================================================================

.PHONY: prod-up
prod-up: ## Start production stack
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

.PHONY: prod-down
prod-down: ## Stop production stack
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml down

.PHONY: prod-logs
prod-logs: ## Show production logs
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml logs -f

# =============================================================================
# Monitoring
# =============================================================================

.PHONY: metrics
metrics: ## Show Prometheus metrics
	@curl -s http://localhost:9090/metrics | head -n 50

.PHONY: grafana
grafana: ## Open Grafana in browser
	@echo "Opening Grafana at http://localhost:3000"
	@command -v open >/dev/null 2>&1 && open http://localhost:3000 || xdg-open http://localhost:3000 || echo "Please open http://localhost:3000 manually"

.PHONY: prometheus
prometheus: ## Open Prometheus in browser
	@echo "Opening Prometheus at http://localhost:9091"
	@command -v open >/dev/null 2>&1 && open http://localhost:9091 || xdg-open http://localhost:9091 || echo "Please open http://localhost:9091 manually"

# =============================================================================
# CI/CD
# =============================================================================

.PHONY: ci
ci: fmt-check clippy test audit deny ## Run all CI checks locally

.PHONY: pre-commit
pre-commit: fmt clippy test ## Run pre-commit checks

.PHONY: pre-push
pre-push: ci ## Run pre-push checks

# =============================================================================
# Release
# =============================================================================

.PHONY: changelog
changelog: ## Generate changelog
	git-cliff --output CHANGELOG.md

.PHONY: version
version: ## Show current version
	@grep '^version = ' Cargo.toml | head -n 1 | cut -d '"' -f 2

.PHONY: release-dry
release-dry: ## Dry run release to crates.io
	cargo publish --dry-run

.PHONY: release-patch
release-patch: ## Release patch version
	@echo "Releasing patch version..."
	@./scripts/release.sh patch

.PHONY: release-minor
release-minor: ## Release minor version
	@echo "Releasing minor version..."
	@./scripts/release.sh minor

.PHONY: release-major
release-major: ## Release major version
	@echo "Releasing major version..."
	@./scripts/release.sh major

# =============================================================================
# Utilities
# =============================================================================

.PHONY: install
install: ## Install the binary locally
	cargo install --path .

.PHONY: uninstall
uninstall: ## Uninstall the binary
	cargo uninstall llm-latency-lens

.PHONY: check
check: ## Quick check (compile without codegen)
	cargo check --all-targets --all-features

.PHONY: doc
doc: ## Generate and open documentation
	cargo doc --all-features --workspace --no-deps --open

.PHONY: tree
tree: ## Show dependency tree
	cargo tree --all-features

.PHONY: bloat
bloat: ## Analyze binary size
	cargo install cargo-bloat --locked
	cargo bloat --release

.PHONY: watch
watch: ## Watch for changes and rebuild
	cargo install cargo-watch --locked
	cargo watch -x build

.PHONY: setup
setup: ## Install development dependencies
	rustup component add rustfmt clippy llvm-tools-preview
	cargo install cargo-audit cargo-deny cargo-outdated cargo-sbom cargo-llvm-cov --locked

.PHONY: env
env: ## Create .env file from example
	@if [ ! -f .env ]; then \
		cp .env.example .env; \
		echo ".env file created. Please edit it with your configuration."; \
	else \
		echo ".env file already exists."; \
	fi

# =============================================================================
# Cleanup
# =============================================================================

.PHONY: clean-docker
clean-docker: ## Remove all Docker containers and images
	docker-compose down -v
	docker rmi llm-latency-lens:latest || true

.PHONY: clean-all
clean-all: clean clean-docker ## Clean everything
	rm -rf target/
	rm -rf ~/.cargo/registry/cache/
	rm -rf ~/.cargo/git/db/

# =============================================================================
# Default
# =============================================================================

.DEFAULT_GOAL := help
