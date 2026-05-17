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
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

# download and extract
curl -sL "$URL" -o "$TMPDIR/${TARGET}.tar.gz"
tar -xzf "$TMPDIR/${TARGET}.tar.gz" -C "$TMPDIR"

# install
if [ -w "$INSTALL_DIR" ]; then
  mv "$TMPDIR/${BINARY}" "$INSTALL_DIR/${BINARY}"
else
  sudo mv "$TMPDIR/${BINARY}" "$INSTALL_DIR/${BINARY}"
fi

chmod +x "$INSTALL_DIR/${BINARY}"

echo "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"
echo "Run 'jav --help' to get started"
