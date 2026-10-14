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

# Check if 0.1.0 is already live on npm
PUBLISHED_VER="$(npm view shadowbox-cli version 2>/dev/null || true)"
if [ "$PUBLISHED_VER" = "0.1.0" ]; then
  echo "shadowbox-cli@0.1.0 is already published and live on npmjs.com!"
else
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
fi
echo ""

# ----------------------------------------------------------------------
# 2. CRATES.IO PUBLISH (shadowbox)
# ----------------------------------------------------------------------
echo "[2/2] CRATES.IO (shadowbox)"
CARGO_CREDS="$HOME/.cargo/credentials.toml"

HAS_CARGO_AUTH=false
if [ -f "$CARGO_CREDS" ] && grep -q "token" "$CARGO_CREDS" 2>/dev/null; then
  HAS_CARGO_AUTH=true
fi

if [ "$HAS_CARGO_AUTH" = false ]; then
  echo "Crates.io token is needed to publish 'shadowbox' to crates.io."
  echo "Get it in 10 seconds:"
  echo "  1. Open: https://crates.io/settings/tokens"
  echo "  2. Click 'New Token' (name: 'shadowbox', scopes: publish-update, publish-new)"
  echo "  3. Copy the token and paste it below (or press ENTER to skip):"
  read -r -p "Enter crates.io token: " CRATES_TOKEN
  
  if [ -n "$CRATES_TOKEN" ]; then
    echo "$CRATES_TOKEN" | cargo login
    HAS_CARGO_AUTH=true
    
    # Also sync to GitHub Secrets immediately
    if command -v gh >/dev/null 2>&1; then
      echo "$CRATES_TOKEN" | gh secret set CARGO_REGISTRY_TOKEN --repo Nari-Ab/shadowbox 2>/dev/null || true
      echo "CARGO_REGISTRY_TOKEN saved to GitHub Secrets."
    fi
  else
    echo "Skipping crates.io publishing (token was empty)."
  fi
fi

if [ "$HAS_CARGO_AUTH" = true ]; then
  echo "Publishing shadowbox to crates.io..."
  cargo publish
  echo "shadowbox is now live on crates.io!"
fi

echo ""
# ----------------------------------------------------------------------
# 3. SYNC NPM SECRET TO GITHUB IF NOT ALREADY DONE
# ----------------------------------------------------------------------
if command -v gh >/dev/null 2>&1; then
  NPM_TOKEN="$(grep -o 'authToken=.*' ~/.npmrc 2>/dev/null | head -n 1 | cut -d'=' -f2 || true)"
  if [ -n "$NPM_TOKEN" ]; then
    echo "$NPM_TOKEN" | gh secret set NPM_TOKEN --repo Nari-Ab/shadowbox 2>/dev/null || true
  fi
fi

echo "=== All tasks complete ==="
