#!/usr/bin/env bash
set -euo pipefail

# Release script for doxx
# Usage: ./scripts/release.sh [major|minor|patch]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check if we're on main branch
current_branch=$(git rev-parse --abbrev-ref HEAD)
if [[ "$current_branch" != "main" ]]; then
    error "Must be on main branch to release. Current branch: $current_branch"
fi

# Check if working directory is clean
if [[ -n $(git status --porcelain) ]]; then
    error "Working directory must be clean to release"
fi

# Check if we have the required tools
command -v cargo >/dev/null 2>&1 || error "cargo is required"
command -v git >/dev/null 2>&1 || error "git is required"

# Get current version
current_version=$(grep '^version = ' "$PROJECT_DIR/Cargo.toml" | sed 's/version = "\(.*\)"/\1/')
log "Current version: $current_version"

# Parse version parts
IFS='.' read -r -a version_parts <<< "$current_version"
major=${version_parts[0]}
minor=${version_parts[1]}
patch=${version_parts[2]}

# Determine bump type
bump_type=${1:-patch}
case "$bump_type" in
    major)
        new_version="$((major + 1)).0.0"
        ;;
    minor)
        new_version="$major.$((minor + 1)).0"
        ;;
    patch)
        new_version="$major.$minor.$((patch + 1))"
        ;;
    *)
        error "Invalid bump type: $bump_type. Use major, minor, or patch"
        ;;
esac

log "Bumping $bump_type version: $current_version -> $new_version"

# Update version in Cargo.toml
sed -i.bak "s/^version = \"$current_version\"/version = \"$new_version\"/" "$PROJECT_DIR/Cargo.toml"
rm "$PROJECT_DIR/Cargo.toml.bak"

# Run tests to make sure everything still works
log "Running tests..."
cd "$PROJECT_DIR"
cargo test --all-features

# Update Cargo.lock
log "Updating Cargo.lock..."
cargo build --release

# Create changelog entry (if CHANGELOG.md exists)
if [[ -f "$PROJECT_DIR/CHANGELOG.md" ]]; then
    log "Please update CHANGELOG.md with release notes for v$new_version"
    read -p "Press enter when ready to continue..."
fi

# Commit version bump
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to v$new_version"

# Create and push tag
tag_name="v$new_version"
log "Creating tag: $tag_name"
git tag -a "$tag_name" -m "Release $tag_name"

log "Pushing to origin..."
git push origin main
git push origin "$tag_name"

log "Release $tag_name has been pushed!"
log "GitHub Actions will now:"
log "  1. Build cross-platform binaries"
log "  2. Create GitHub release (draft)"
log "  3. Publish to crates.io"
log "  4. Update Homebrew formula"
log ""
log "Next steps:"
log "  1. Go to GitHub releases and edit the draft release"
log "  2. Add release notes and publish the release"
log "  3. Verify package installations work correctly"