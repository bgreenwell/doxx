#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/release.sh [major|minor|patch]
# Update CHANGELOG.md before running.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

red='\033[0;31m'; green='\033[0;32m'; nc='\033[0m'
log()   { echo -e "${green}[INFO]${nc} $1"; }
error() { echo -e "${red}[ERROR]${nc} $1"; exit 1; }

cd "$PROJECT_DIR"

# Guards
[[ "$(git rev-parse --abbrev-ref HEAD)" == "main" ]] || error "Must be on main branch"
[[ -z "$(git status --porcelain)" ]] || error "Working directory must be clean"
command -v cargo >/dev/null 2>&1 || error "cargo is required"

# Version bump
current_version=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
IFS='.' read -r -a v <<< "$current_version"
bump_type=${1:-patch}
case "$bump_type" in
    major) new_version="$((v[0]+1)).0.0" ;;
    minor) new_version="${v[0]}.$((v[1]+1)).0" ;;
    patch) new_version="${v[0]}.${v[1]}.$((v[2]+1))" ;;
    *)     error "Invalid bump type: $bump_type. Use major, minor, or patch" ;;
esac
log "Bumping $bump_type: $current_version -> $new_version"

sed -i.bak "s/^version = \"$current_version\"/version = \"$new_version\"/" Cargo.toml
rm Cargo.toml.bak

# Validate
log "Running checks..."
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-features
cargo update --workspace

# Commit and tag
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: release $new_version"
git tag "v$new_version"
git push origin main
git push origin "v$new_version"

log "v$new_version tagged and pushed — GitHub Actions will handle the rest."
