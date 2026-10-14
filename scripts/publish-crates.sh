#!/usr/bin/env bash
set -e

# Publish ShadowBox to Crates.io
echo "==> Preparing shadowbox for crates.io..."

# Ensure we are at project root
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

# Run tests
echo "==> Running tests..."
cargo test --all

# Check login status
echo "==> Checking cargo credentials..."
if ! cargo login --check 2>/dev/null; then
  echo "You are not logged into crates.io."
  echo "Please generate an API token at https://crates.io/settings/tokens"
  read -r -p "Enter crates.io API Token: " CRATES_TOKEN
  cargo login "$CRATES_TOKEN"
fi

# Dry run first
echo "==> Verifying package..."
cargo package

# Publish
echo "==> Publishing to crates.io..."
cargo publish

echo "==> Successfully published to crates.io!"
