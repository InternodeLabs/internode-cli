#!/bin/sh
set -e

REPO="InternodeLabs/internode-cli"
COMMAND_NAME="internode"
INSTALL_BINARY="${COMMAND_NAME}"
INSTALL_DIR="${HOME}/.local/bin"

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)  os="linux" ;;
        Darwin) os="macos" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *) echo "Unsupported OS: $OS" >&2; exit 1 ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   arch="amd64" ;;
        aarch64|arm64)   arch="arm64" ;;
        *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
    esac

    echo "${os}-${arch}"
}

resolve_release_asset() {
    platform="$1"

    case "$platform" in
        linux-amd64)
            target="x86_64-unknown-linux-gnu"
            archive_ext="tar.gz"
            ;;
        macos-arm64)
            target="aarch64-apple-darwin"
            archive_ext="tar.gz"
            ;;
        windows-amd64)
            target="x86_64-pc-windows-msvc"
            archive_ext="zip"
            INSTALL_BINARY="${COMMAND_NAME}.exe"
            ;;
        windows-arm64)
            echo "Unsupported platform: ${platform} (no release artifact published yet)" >&2
            exit 1
            ;;
        *)
            echo "Unsupported platform: ${platform}" >&2
            exit 1
            ;;
    esac

    echo "${target}|${archive_ext}"
}

fetch_text() {
    url="$1"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "$url"
    else
        echo "Error: curl or wget required." >&2
        exit 1
    fi
}

download_file() {
    url="$1"
    output="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fSL "$url" -o "$output"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$output"
    else
        echo "Error: curl or wget required." >&2
        exit 1
    fi
}

PLATFORM="$(detect_platform)"
echo "Detected platform: ${PLATFORM}"
ASSET_INFO="$(resolve_release_asset "${PLATFORM}")"
TARGET="${ASSET_INFO%%|*}"
ARCHIVE_EXT="${ASSET_INFO##*|}"

RELEASE_JSON="$(fetch_text "https://api.github.com/repos/${REPO}/releases/latest")"
DOWNLOAD_URL="$(printf "%s\n" "$RELEASE_JSON" | awk -F '"' -v target="$TARGET" -v ext="$ARCHIVE_EXT" '
    $2 == "browser_download_url" {
        url = $4
        suffix = "-" target "." ext
        if (length(url) >= length(suffix) && substr(url, length(url) - length(suffix) + 1) == suffix) {
            print url
            exit
        }
    }
')"

if [ -z "$DOWNLOAD_URL" ]; then
    # Backward compatibility for older releases that published raw binaries.
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${INSTALL_BINARY}-${PLATFORM}"
    ARCHIVE_EXT="raw"
fi

echo "Downloading from ${DOWNLOAD_URL}..."

TMP_FILE="$(mktemp)"
TMP_DIR="$(mktemp -d)"
trap 'rm -f "$TMP_FILE"; rm -rf "$TMP_DIR"' EXIT
download_file "$DOWNLOAD_URL" "$TMP_FILE"

EXTRACTED_FILE="${TMP_FILE}"

if [ "$ARCHIVE_EXT" = "tar.gz" ]; then
    tar -xzf "$TMP_FILE" -C "$TMP_DIR"
    if [ -f "${TMP_DIR}/${INSTALL_BINARY}" ]; then
        EXTRACTED_FILE="${TMP_DIR}/${INSTALL_BINARY}"
    elif [ -f "${TMP_DIR}/dist/${INSTALL_BINARY}" ]; then
        EXTRACTED_FILE="${TMP_DIR}/dist/${INSTALL_BINARY}"
    else
        echo "Could not find ${INSTALL_BINARY} in downloaded tarball." >&2
        exit 1
    fi
elif [ "$ARCHIVE_EXT" = "zip" ]; then
    if ! command -v unzip >/dev/null 2>&1; then
        echo "Error: unzip is required to extract Windows release assets." >&2
        exit 1
    fi
    unzip -q "$TMP_FILE" -d "$TMP_DIR"
    if [ -f "${TMP_DIR}/${INSTALL_BINARY}" ]; then
        EXTRACTED_FILE="${TMP_DIR}/${INSTALL_BINARY}"
    elif [ -f "${TMP_DIR}/dist/${INSTALL_BINARY}" ]; then
        EXTRACTED_FILE="${TMP_DIR}/dist/${INSTALL_BINARY}"
    else
        echo "Could not find ${INSTALL_BINARY} in downloaded zip archive." >&2
        exit 1
    fi
fi

mkdir -p "$INSTALL_DIR"
mv "$EXTRACTED_FILE" "${INSTALL_DIR}/${INSTALL_BINARY}"
chmod +x "${INSTALL_DIR}/${INSTALL_BINARY}"
trap - EXIT
rm -f "$TMP_FILE"
rm -rf "$TMP_DIR"

echo ""
echo "Installed ${INSTALL_BINARY} to ${INSTALL_DIR}/${INSTALL_BINARY}"

case ":$PATH:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        echo ""
        echo "Add ${INSTALL_DIR} to your PATH:"
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        echo ""
        echo "Add that line to your ~/.bashrc or ~/.zshrc to make it permanent."
        ;;
esac

echo ""
echo "Quick start:"
echo "  1. Get an API key from Settings > CLI API Key at https://app.internode.ai"
echo "  2. Run: ${COMMAND_NAME} configure <your-api-key>"
echo "  3. Verify: ${COMMAND_NAME} auth status"
echo ""
