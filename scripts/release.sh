#!/usr/bin/env bash
# Release automation script for LLM-Latency-Lens

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_step "Checking prerequisites..."

    # Check git
    if ! command -v git &> /dev/null; then
        log_error "git is not installed"
        exit 1
    fi

    # Check cargo
    if ! command -v cargo &> /dev/null; then
        log_error "cargo is not installed"
        exit 1
    fi

    # Check git-cliff
    if ! command -v git-cliff &> /dev/null; then
        log_warn "git-cliff is not installed. Installing..."
        cargo install git-cliff --locked
    fi

    # Check if working directory is clean
    if [ -n "$(git status --porcelain)" ]; then
        log_error "Working directory is not clean. Commit or stash changes first."
        git status --short
        exit 1
    fi

    # Check if on main branch
    CURRENT_BRANCH=$(git branch --show-current)
    if [ "$CURRENT_BRANCH" != "main" ]; then
        log_warn "Not on main branch (current: $CURRENT_BRANCH)"
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi

    log_info "Prerequisites check passed"
}

# Get current version
get_current_version() {
    grep '^version = ' Cargo.toml | head -n 1 | cut -d '"' -f 2
}

# Calculate next version
calculate_next_version() {
    local CURRENT_VERSION=$1
    local BUMP_TYPE=$2

    IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

    case "$BUMP_TYPE" in
        major)
            MAJOR=$((MAJOR + 1))
            MINOR=0
            PATCH=0
            ;;
        minor)
            MINOR=$((MINOR + 1))
            PATCH=0
            ;;
        patch)
            PATCH=$((PATCH + 1))
            ;;
        *)
            log_error "Invalid bump type: $BUMP_TYPE"
            exit 1
            ;;
    esac

    echo "$MAJOR.$MINOR.$PATCH"
}

# Update version in Cargo.toml files
update_version() {
    local NEW_VERSION=$1

    log_step "Updating version to $NEW_VERSION..."

    # Update main Cargo.toml
    sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml

    # Update workspace crate Cargo.toml files
    for CRATE_TOML in crates/*/Cargo.toml; do
        sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$CRATE_TOML"
    done

    # Remove backup files
    find . -name "Cargo.toml.bak" -delete

    # Update Cargo.lock
    cargo build --release

    log_info "Version updated to $NEW_VERSION"
}

# Generate changelog
generate_changelog() {
    local NEW_VERSION=$1

    log_step "Generating changelog..."

    git-cliff --tag "v$NEW_VERSION" --output CHANGELOG.md

    log_info "Changelog generated"
}

# Run tests
run_tests() {
    log_step "Running tests..."

    cargo test --all-features --workspace

    log_info "Tests passed"
}

# Create release commit
create_release_commit() {
    local NEW_VERSION=$1

    log_step "Creating release commit..."

    git add Cargo.toml Cargo.lock crates/*/Cargo.toml CHANGELOG.md
    git commit -m "chore(release): prepare for v$NEW_VERSION"

    log_info "Release commit created"
}

# Create git tag
create_git_tag() {
    local NEW_VERSION=$1

    log_step "Creating git tag v$NEW_VERSION..."

    git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"

    log_info "Git tag created"
}

# Push changes
push_changes() {
    local NEW_VERSION=$1

    log_step "Pushing changes to remote..."

    # Push commits
    git push origin main

    # Push tag
    git push origin "v$NEW_VERSION"

    log_info "Changes pushed to remote"
}

# Show summary
show_summary() {
    local CURRENT_VERSION=$1
    local NEW_VERSION=$2

    echo ""
    log_info "Release Summary"
    echo "  Previous version: $CURRENT_VERSION"
    echo "  New version:      $NEW_VERSION"
    echo "  Tag:              v$NEW_VERSION"
    echo ""
    log_info "Next steps:"
    echo "  1. Monitor GitHub Actions for release workflow"
    echo "  2. Check release at: https://github.com/llm-devops/llm-latency-lens/releases/tag/v$NEW_VERSION"
    echo "  3. Verify Docker images are published"
    echo "  4. Verify crates.io publication"
    echo ""
}

# Main release process
release() {
    local BUMP_TYPE=$1

    log_info "Starting release process (type: $BUMP_TYPE)..."

    # Get versions
    CURRENT_VERSION=$(get_current_version)
    NEW_VERSION=$(calculate_next_version "$CURRENT_VERSION" "$BUMP_TYPE")

    log_info "Current version: $CURRENT_VERSION"
    log_info "New version: $NEW_VERSION"

    # Confirmation
    echo ""
    read -p "Create release v$NEW_VERSION? (y/N) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_warn "Release cancelled"
        exit 0
    fi

    # Run release steps
    check_prerequisites
    update_version "$NEW_VERSION"
    generate_changelog "$NEW_VERSION"
    run_tests
    create_release_commit "$NEW_VERSION"
    create_git_tag "$NEW_VERSION"

    # Push confirmation
    echo ""
    read -p "Push changes to remote? (y/N) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        push_changes "$NEW_VERSION"
        show_summary "$CURRENT_VERSION" "$NEW_VERSION"
    else
        log_warn "Changes not pushed. You can push manually with:"
        echo "  git push origin main"
        echo "  git push origin v$NEW_VERSION"
    fi

    log_info "Release process completed!"
}

# Dry run
dry_run() {
    local BUMP_TYPE=$1

    log_info "Running dry release (type: $BUMP_TYPE)..."

    CURRENT_VERSION=$(get_current_version)
    NEW_VERSION=$(calculate_next_version "$CURRENT_VERSION" "$BUMP_TYPE")

    log_info "Current version: $CURRENT_VERSION"
    log_info "New version would be: $NEW_VERSION"

    # Dry run cargo publish
    cargo publish --dry-run

    log_info "Dry run completed"
}

# Usage
usage() {
    cat << EOF
Usage: $0 [COMMAND]

Commands:
    patch       Create a patch release (0.0.X)
    minor       Create a minor release (0.X.0)
    major       Create a major release (X.0.0)
    dry         Dry run (test release process)
    help        Show this help message

Examples:
    $0 patch    # 0.1.0 -> 0.1.1
    $0 minor    # 0.1.0 -> 0.2.0
    $0 major    # 0.1.0 -> 1.0.0
    $0 dry      # Test release process

EOF
}

# Main
main() {
    COMMAND="${1:-help}"

    case "$COMMAND" in
        patch|minor|major)
            release "$COMMAND"
            ;;
        dry)
            dry_run "patch"
            ;;
        help|--help|-h)
            usage
            ;;
        *)
            log_error "Unknown command: $COMMAND"
            usage
            exit 1
            ;;
    esac
}

main "$@"
