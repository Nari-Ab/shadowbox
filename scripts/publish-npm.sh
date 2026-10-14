#!/usr/bin/env bash
set -e

# Publish ShadowBox to npm
echo "==> Preparing shadowbox-cli for npm..."

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR/npm"

# Check npm whoami
echo "==> Checking npm authentication..."
if ! npm whoami 2>/dev/null; then
  echo "You are not logged into npm."
  echo "Please run 'npm login' or provide an auth token."
  npm login
fi

# Verify package
echo "==> Verifying package payload..."
npm pack --dry-run

# Publish public package
echo "==> Publishing to npm registry..."
npm publish --access public

echo "==> Successfully published shadowbox-cli to npm!"
