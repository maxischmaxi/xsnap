#!/bin/bash
set -euo pipefail

REPO="maxischmaxi/xsnap"
INSTALL_DIR="${XSNAP_INSTALL_DIR:-/usr/local/bin}"
VERSION="${1:-latest}"

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

if [ "$VERSION" = "latest" ]; then
    URL="https://github.com/${REPO}/releases/latest/download/${BINARY}"
else
    # Strip leading 'v' if present, then add it back
    VERSION="${VERSION#v}"
    URL="https://github.com/${REPO}/releases/download/v${VERSION}/${BINARY}"
fi

echo "Downloading xsnap ${VERSION} for ${OS}/${ARCH}..."
curl -fsSL "$URL" -o /tmp/xsnap

echo "Installing to ${INSTALL_DIR}/xsnap..."
SUDO=""
if [ ! -w "$INSTALL_DIR" ] 2>/dev/null && command -v sudo >/dev/null 2>&1; then
    SUDO="sudo"
fi
$SUDO install -d "$INSTALL_DIR"
$SUDO install -m 755 /tmp/xsnap "${INSTALL_DIR}/xsnap"
rm -f /tmp/xsnap

echo "Verifying installation..."
if xsnap --version; then
    echo "xsnap installed successfully!"
else
    echo "Installation complete. You may need to add ${INSTALL_DIR} to your PATH."
fi
