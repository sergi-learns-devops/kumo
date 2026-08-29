#!/bin/bash
# kumo installer — https://github.com/sergi-learns-devops/kumo
# Usage: curl -fsSL https://raw.githubusercontent.com/sergi-learns-devops/kumo/main/install.sh | bash
set -euo pipefail

REPO="sergi-learns-devops/kumo"
BINARY="kumo"
INSTALL_DIR="${KUMO_INSTALL_DIR:-/usr/local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
DIM='\033[2m'
RESET='\033[0m'

info() { echo -e "${CYAN}$1${RESET}"; }
success() { echo -e "${GREEN}$1${RESET}"; }
error() { echo -e "${RED}Error: $1${RESET}" >&2; exit 1; }

# Detect OS and architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)  os="linux" ;;
        Darwin*) os="macos" ;;
        *)       error "Unsupported OS: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)             error "Unsupported architecture: $(uname -m)" ;;
    esac

    echo "${os}-${arch}"
}

# Get latest release tag from GitHub
get_latest_version() {
    local url="https://api.github.com/repos/${REPO}/releases/latest"
    if command -v curl &>/dev/null; then
        curl -fsSL "$url" | grep '"tag_name"' | sed -E 's/.*"v?([^"]+)".*/\1/'
    elif command -v wget &>/dev/null; then
        wget -qO- "$url" | grep '"tag_name"' | sed -E 's/.*"v?([^"]+)".*/\1/'
    else
        error "Neither curl nor wget found. Install one and retry."
    fi
}

# Download and install
install() {
    local platform version download_url tmp_dir

    info "☁  kumo installer"
    echo ""

    platform=$(detect_platform)
    info "  Platform: ${platform}"

    version=$(get_latest_version 2>/dev/null || echo "")

    if [ -z "$version" ]; then
        info "  Could not fetch latest release. Building from source..."
        install_from_source
        return
    fi

    info "  Version:  v${version}"
    echo ""

    download_url="https://github.com/${REPO}/releases/download/v${version}/${BINARY}-${platform}.tar.gz"

    tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT

    info "  Downloading..."
    if command -v curl &>/dev/null; then
        curl -fsSL "$download_url" -o "${tmp_dir}/${BINARY}.tar.gz" 2>/dev/null
    else
        wget -q "$download_url" -O "${tmp_dir}/${BINARY}.tar.gz" 2>/dev/null
    fi

    if [ ! -f "${tmp_dir}/${BINARY}.tar.gz" ]; then
        info "  Pre-built binary not available. Building from source..."
        install_from_source
        return
    fi

    info "  Extracting..."
    tar -xzf "${tmp_dir}/${BINARY}.tar.gz" -C "${tmp_dir}"

    info "  Installing to ${INSTALL_DIR}..."
    if [ -w "$INSTALL_DIR" ]; then
        mv "${tmp_dir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    else
        sudo mv "${tmp_dir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    fi
    chmod +x "${INSTALL_DIR}/${BINARY}"

    echo ""
    success "  ✓ kumo v${version} installed to ${INSTALL_DIR}/${BINARY}"
    echo -e "  ${DIM}Run 'kumo --help' to get started${RESET}"
    echo ""
}

# Fallback: build from source
install_from_source() {
    if ! command -v cargo &>/dev/null; then
        error "Rust toolchain not found. Install it from https://rustup.rs and retry."
    fi

    info "  Building from source (this may take a minute)..."

    local src_dir
    src_dir=$(mktemp -d)
    trap 'rm -rf "$src_dir"' EXIT

    if command -v git &>/dev/null; then
        git clone --depth 1 "https://github.com/${REPO}.git" "$src_dir" 2>/dev/null
    else
        error "git not found. Install git and retry."
    fi

    cd "$src_dir"
    cargo build --release --quiet

    if [ -w "$INSTALL_DIR" ]; then
        cp "target/release/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    else
        sudo cp "target/release/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    fi
    chmod +x "${INSTALL_DIR}/${BINARY}"

    echo ""
    success "  ✓ kumo installed from source to ${INSTALL_DIR}/${BINARY}"
    echo -e "  ${DIM}Run 'kumo --help' to get started${RESET}"
    echo ""
}

install
