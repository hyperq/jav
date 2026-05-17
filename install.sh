#!/bin/sh
set -e

REPO="hyperq/jav"
BINARY="jav"
INSTALL_DIR="${JAV_INSTALL_DIR:-/usr/local/bin}"

# detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  darwin) OS="apple-darwin" ;;
  linux)  OS="unknown-linux-gnu" ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

TARGET="${BINARY}-${ARCH}-${OS}"

# get latest version
if [ -z "$JAV_VERSION" ]; then
  VERSION=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
  if [ -z "$VERSION" ]; then
    echo "Failed to get latest version"
    exit 1
  fi
else
  VERSION="$JAV_VERSION"
fi

echo "Installing ${BINARY} ${VERSION} (${ARCH}-${OS})..."

URL="https://github.com/${REPO}/releases/download/${VERSION}/${TARGET}.tar.gz"
TMP=$(mktemp -d)
trap "rm -rf $TMP" EXIT

# download and extract
curl -fSL "$URL" -o "$TMP/${TARGET}.tar.gz"
tar -xzf "$TMP/${TARGET}.tar.gz" -C "$TMP"

# ensure install dir exists and install
if [ -w "$INSTALL_DIR" ] 2>/dev/null; then
  mkdir -p "$INSTALL_DIR"
  mv "$TMP/${BINARY}" "$INSTALL_DIR/${BINARY}"
else
  sudo mkdir -p "$INSTALL_DIR"
  sudo mv "$TMP/${BINARY}" "$INSTALL_DIR/${BINARY}"
  sudo chmod +x "$INSTALL_DIR/${BINARY}"
fi

chmod +x "$INSTALL_DIR/${BINARY}" 2>/dev/null || true

echo ""
echo "✅ Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"
echo "   Run 'jav --help' to get started"
