#!/usr/bin/env bash
set -e

# ShadowBox Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/Nari-Ab/shadowbox/main/install.sh | bash

REPO="Nari-Ab/shadowbox"
VERSION="v0.1.0"

echo "📦 Installing ShadowBox ($VERSION)..."

# Detect OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin)
    case "$ARCH" in
      arm64)
        ASSET="shadowbox-macos-arm64.tar.gz"
        ;;
      x86_64)
        ASSET="shadowbox-macos-x86_64.tar.gz"
        ;;
      *)
        echo "❌ Unsupported architecture: $ARCH"
        exit 1
        ;;
    esac
    ;;
  Linux)
    case "$ARCH" in
      x86_64)
        ASSET="shadowbox-linux-x86_64.tar.gz"
        ;;
      aarch64|arm64)
        ASSET="shadowbox-linux-arm64.tar.gz"
        ;;
      *)
        echo "❌ Unsupported architecture: $ARCH"
        exit 1
        ;;
    esac
    ;;
  *)
    echo "❌ Unsupported operating system: $OS"
    exit 1
    ;;
esac

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"
# Fallback to direct binary if asset tarball not found
FALLBACK_URL="https://github.com/$REPO/releases/download/$VERSION/shadowbox"

INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "⬇️  Downloading from $DOWNLOAD_URL..."
if curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/asset.tar.gz" 2>/dev/null; then
  tar -xzf "$TMP_DIR/asset.tar.gz" -C "$TMP_DIR"
elif curl -fsSL "$FALLBACK_URL" -o "$TMP_DIR/shadowbox" 2>/dev/null; then
  chmod +x "$TMP_DIR/shadowbox"
else
  echo "❌ Failed to download release asset."
  exit 1
fi

chmod +x "$TMP_DIR/shadowbox"
mv "$TMP_DIR/shadowbox" "$INSTALL_DIR/shadowbox"

echo "✅ ShadowBox installed successfully to $INSTALL_DIR/shadowbox"

# Check PATH
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo "⚠️  Note: Add $INSTALL_DIR to your PATH by adding this line to your ~/.zshrc or ~/.bashrc:"
    echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
    ;;
esac

echo ""
echo "🚀 Try running:"
echo "    shadowbox claude --dangerously-skip-permissions"
