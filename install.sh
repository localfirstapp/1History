#!/bin/sh

set -e

REPO="localfirstapp/1History"
BINARY_NAME="onehistory"
GITHUB_URL="https://github.com/${REPO}"

# Default values
VERSION="latest"
INSTALL_DIR="${HOME}/.local/bin"
CHINA=false

# Help message
usage() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  -v, --version <ver>      Release version (e.g. v0.4.0), default is latest"
    echo "  -p, --prefix <dir>       Directory to install binary, default is ~/.local/bin"
    echo "  --china                  Use proxy for downloads (for users in China)"
    echo "  -h, --help               Show this help message"
    exit 1
}

# Parse arguments
while [ "$#" -gt 0 ]; do
    case "$1" in
        --version|-v)
            if [ -n "$2" ]; then
                VERSION="$2"
                shift 2
            else
                echo "Error: --version requires an argument"
                usage
            fi
            ;;
        --prefix|-p)
            if [ -n "$2" ]; then
                INSTALL_DIR="$2"
                shift 2
            else
                echo "Error: --prefix requires an argument"
                usage
            fi
            ;;
        --china)
            CHINA=true
            shift
            ;;
        --help|-h)
            usage
            ;;
        *)
            echo "Unknown argument: $1"
            usage
            ;;
    esac
done

# Detect OS
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$OS" in
    linux*)  OS="linux" ;;
    darwin*) OS="darwin" ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

# Detect Architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)        ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Map to Rust target triple
if [ "$OS" = "linux" ]; then
    TARGET="${ARCH}-unknown-linux-musl"
elif [ "$OS" = "darwin" ]; then
    TARGET="${ARCH}-apple-darwin"
else
    echo "Unsupported platform: $OS/$ARCH"; exit 1
fi

# Resolve 'latest' version if needed
if [ "$VERSION" = "latest" ]; then
    echo "Resolving latest version..."
    VERSION=$(curl -sI "${GITHUB_URL}/releases/latest" | grep -i '^location:' | sed -E 's|.*/tag/([^ ]+).*|\1|' | tr -d '\r')
    if [ -z "$VERSION" ]; then
        AUTH_HEADER=""
        if [ -n "$GITHUB_TOKEN" ]; then
            AUTH_HEADER="Authorization: token ${GITHUB_TOKEN}"
        fi
        VERSION=$(curl -s ${AUTH_HEADER:+-H "$AUTH_HEADER"} "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    fi
fi

if [ -z "$VERSION" ]; then
    echo "Could not resolve latest version for ${REPO}. You can:"
    echo "  1. Specify a version explicitly: $0 --version v0.4.0"
    echo "  2. Set GITHUB_TOKEN to avoid API rate limits"
    exit 1
fi

ARCHIVE_NAME="${BINARY_NAME}-${VERSION}-${TARGET}.tar.gz"
DOWNLOAD_URL="${GITHUB_URL}/releases/download/${VERSION}/${ARCHIVE_NAME}"

if [ "$CHINA" = true ]; then
    DOWNLOAD_URL="https://gh-proxy.com/${DOWNLOAD_URL}"
fi

echo "Downloading ${BINARY_NAME} ${VERSION} for ${OS}/${ARCH} (${TARGET})..."
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

curl -fL "$DOWNLOAD_URL" -o "${TMP_DIR}/${ARCHIVE_NAME}"
tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "${TMP_DIR}"
chmod +x "${TMP_DIR}/${BINARY_NAME}"

mkdir -p "${INSTALL_DIR}"

if [ -w "${INSTALL_DIR}" ]; then
    mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/"
else
    echo "Need sudo permissions to move binary to ${INSTALL_DIR}"
    sudo mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/"
fi

echo "Successfully installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"

case ":${PATH}:" in
    *:"${INSTALL_DIR}":*) ;;
    *) echo "Warning: ${INSTALL_DIR} is not in your PATH. You may need to add it to your shell profile." ;;
esac

"${INSTALL_DIR}/${BINARY_NAME}" --version
