#!/usr/bin/env bash
# Infrastructure verification script for LLM-Latency-Lens

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

ERRORS=0

check_file() {
    if [ -f "$1" ]; then
        echo -e "${GREEN}✓${NC} $1"
    else
        echo -e "${RED}✗${NC} $1 (missing)"
        ERRORS=$((ERRORS + 1))
    fi
}

check_dir() {
    if [ -d "$1" ]; then
        echo -e "${GREEN}✓${NC} $1/"
    else
        echo -e "${RED}✗${NC} $1/ (missing)"
        ERRORS=$((ERRORS + 1))
    fi
}

echo "🔍 Verifying LLM-Latency-Lens Infrastructure"
echo ""

echo "📁 Core Files:"
check_file "Dockerfile"
check_file ".dockerignore"
check_file "docker-compose.yml"
check_file "docker-compose.prod.yml"
check_file "Makefile"
check_file ".env.example"
echo ""

echo "⚙️  Configuration Files:"
check_file "deny.toml"
check_file "cliff.toml"
echo ""

echo "🔄 CI/CD Workflows:"
check_file ".github/workflows/ci.yml"
check_file ".github/workflows/security.yml"
check_file ".github/workflows/release.yml"
check_file ".github/workflows/docker-build.yml"
check_file ".github/dependabot.yml"
echo ""

echo "📊 Monitoring Configuration:"
check_dir "monitoring"
check_file "monitoring/prometheus/prometheus.yml"
check_file "monitoring/prometheus/alerts.yml"
check_file "monitoring/grafana/provisioning/datasources/prometheus.yml"
check_file "monitoring/grafana/provisioning/dashboards/dashboards.yml"
check_file "monitoring/grafana/dashboards/llm-latency-overview.json"
check_file "monitoring/alertmanager/alertmanager.yml"
echo ""

echo "📜 Scripts:"
check_file "scripts/deploy.sh"
check_file "scripts/release.sh"
check_file "scripts/verify-infrastructure.sh"
echo ""

echo "📚 Documentation:"
check_file "docs/DOCKER.md"
check_file "docs/CI-CD.md"
check_file "docs/DEPLOYMENT.md"
check_file "INFRASTRUCTURE.md"
echo ""

# Check file permissions
echo "🔐 Checking Permissions:"
for script in scripts/*.sh; do
    if [ -x "$script" ]; then
        echo -e "${GREEN}✓${NC} $script (executable)"
    else
        echo -e "${YELLOW}⚠${NC} $script (not executable)"
        chmod +x "$script"
        echo -e "${GREEN}  → Fixed${NC}"
    fi
done
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}✅ All infrastructure files are present!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Copy .env.example to .env and configure"
    echo "  2. Run 'make up' to start the development stack"
    echo "  3. View documentation in docs/"
    exit 0
else
    echo -e "${RED}❌ Found $ERRORS missing files${NC}"
    exit 1
fi
