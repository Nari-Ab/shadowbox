#!/usr/bin/env bash
set -e

# ShadowBox One-Click Release & Registry Publisher
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

echo "=== ShadowBox Registry Publisher ==="
echo ""

# ----------------------------------------------------------------------
# 1. NPM PUBLISH (shadowbox-cli)
# ----------------------------------------------------------------------
echo "[1/2] NPM (shadowbox-cli)"
echo "Checking npm login status..."

if ! npm whoami --registry=https://registry.npmjs.org/ >/dev/null 2>&1; then
  echo "You are not logged in to npm."
  echo "A browser window will open for quick authentication."
  npm login --registry=https://registry.npmjs.org/
fi

NPM_USER="$(npm whoami --registry=https://registry.npmjs.org/)"
echo "Authenticated as npm user: $NPM_USER"

echo "Publishing shadowbox-cli to npm..."
cd "$ROOT_DIR/npm"
npm publish --access public
cd "$ROOT_DIR"
echo "shadowbox-cli is now live on npmjs.com!"
echo ""

# ----------------------------------------------------------------------
# 2. CRATES.IO PUBLISH (shadowbox)
# ----------------------------------------------------------------------
echo "[2/2] CRATES.IO (shadowbox)"
CARGO_CREDS="$HOME/.cargo/credentials.toml"

if [ ! -f "$CARGO_CREDS" ] || ! grep -q "token" "$CARGO_CREDS" 2>/dev/null; then
  echo "Crates.io token not found."
  echo "1. Open https://crates.io/settings/tokens in your browser"
  echo "2. Click 'New Token' (name it 'shadowbox', scopes: publish-update, publish-new)"
  echo "3. Copy the token and paste it below:"
  read -r -p "Enter crates.io token: " CRATES_TOKEN
  cargo login "$CRATES_TOKEN"
fi

echo "Publishing shadowbox to crates.io..."
cargo publish

echo "shadowbox is now live on crates.io!"
echo ""

# ----------------------------------------------------------------------
# 3. SYNC TO GITHUB SECRETS (FOR FULL CI/CD AUTOMATION)
# ----------------------------------------------------------------------
if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  echo "Syncing tokens to GitHub Secrets for future automatic releases..."
  
  # Extract npm token from ~/.npmrc
  NPM_TOKEN="$(grep -o 'authToken=.*' ~/.npmrc | head -n 1 | cut -d'=' -f2 || true)"
  if [ -n "$NPM_TOKEN" ]; then
    echo "$NPM_TOKEN" | gh secret set NPM_TOKEN --repo Nari-Ab/shadowbox 2>/dev/null || true
  fi

  # Extract crates.io token from ~/.cargo/credentials.toml
  if [ -f "$CARGO_CREDS" ]; then
    CRATES_SECRET="$(grep -o 'token = ".*"' "$CARGO_CREDS" | head -n 1 | cut -d'"' -f2 || true)"
    if [ -n "$CRATES_SECRET" ]; then
      echo "$CRATES_SECRET" | gh secret set CARGO_REGISTRY_TOKEN --repo Nari-Ab/shadowbox 2>/dev/null || true
    fi
  fi
  echo "GitHub Secrets updated. All future git tags will release automatically."
fi

echo ""
echo "=== All publications completed successfully ==="
