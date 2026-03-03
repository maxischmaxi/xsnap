#!/bin/bash
set -euo pipefail

REPO="maxischmaxi/xsnap"
INSTALL_DIR="${XSNAP_INSTALL_DIR:-/usr/local/bin}"

# Detect OS
case "$(uname -s)" in
    Linux)  OS="linux" ;;
    Darwin) OS="darwin" ;;
    *)
        echo "Error: Unsupported operating system: $(uname -s)" >&2
        echo "Only Linux and macOS are supported. For Windows, see the README." >&2
        exit 1
        ;;
esac

# Detect architecture
case "$(uname -m)" in
    x86_64)         ARCH="x64" ;;
    aarch64|arm64)  ARCH="arm64" ;;
    *)
        echo "Error: Unsupported architecture: $(uname -m)" >&2
        exit 1
        ;;
esac

BINARY="xsnap-${OS}-${ARCH}"
URL="https://github.com/${REPO}/releases/latest/download/${BINARY}"

echo "Downloading xsnap for ${OS}/${ARCH}..."
curl -fsSL "$URL" -o /tmp/xsnap

echo "Installing to ${INSTALL_DIR}/xsnap..."
install -d "$INSTALL_DIR"
install -m 755 /tmp/xsnap "${INSTALL_DIR}/xsnap"
rm -f /tmp/xsnap

echo "Verifying installation..."
if xsnap --version; then
    echo "xsnap installed successfully!"
else
    echo "Installation complete. You may need to add ${INSTALL_DIR} to your PATH."
fi
