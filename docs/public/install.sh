#!/usr/bin/env bash
set -e

# Refaktor Installation Script
# https://github.com/DocSpring/refaktor

REPO="DocSpring/refaktor"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="refaktor"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Detect OS and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    case "$OS" in
        linux)
            PLATFORM="linux"
            ;;
        darwin)
            PLATFORM="macos"
            ;;
        *)
            echo -e "${RED}Error: Unsupported operating system: $OS${NC}"
            echo "Refaktor currently supports Linux and macOS."
            exit 1
            ;;
    esac
    
    case "$ARCH" in
        x86_64|amd64)
            ARCH="amd64"
            ;;
        aarch64|arm64)
            ARCH="arm64"
            ;;
        *)
            echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
            echo "Refaktor currently supports x86_64/amd64 and aarch64/arm64."
            exit 1
            ;;
    esac
    
    ASSET_NAME="refaktor-${PLATFORM}-${ARCH}.tar.gz"
}

# Check if running with sudo when needed
check_permissions() {
    if [ ! -w "$INSTALL_DIR" ]; then
        if [ "$EUID" -ne 0 ]; then
            echo -e "${YELLOW}Permission denied. Please run with sudo:${NC}"
            echo "  curl -fsSL https://docspring.github.io/refaktor/install.sh | sudo bash"
            exit 1
        fi
    fi
}

# Download and install
install_refaktor() {
    echo "Installing Refaktor..."
    echo "  Platform: $PLATFORM"
    echo "  Architecture: $ARCH"
    echo ""
    
    # Get the latest release URL
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"
    
    echo "Downloading from: $DOWNLOAD_URL"
    
    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT
    
    # Download and extract
    if command -v curl &> /dev/null; then
        curl -fsSL "$DOWNLOAD_URL" | tar xz -C "$TEMP_DIR"
    elif command -v wget &> /dev/null; then
        wget -qO- "$DOWNLOAD_URL" | tar xz -C "$TEMP_DIR"
    else
        echo -e "${RED}Error: Neither curl nor wget is available.${NC}"
        echo "Please install curl or wget and try again."
        exit 1
    fi
    
    # Move binary to install directory
    if [ -w "$INSTALL_DIR" ]; then
        mv "$TEMP_DIR/refaktor" "$INSTALL_DIR/"
        chmod 755 "$INSTALL_DIR/refaktor"
    else
        sudo mv "$TEMP_DIR/refaktor" "$INSTALL_DIR/"
        sudo chmod 755 "$INSTALL_DIR/refaktor"
    fi
    
    echo -e "${GREEN}✓ Refaktor installed successfully!${NC}"
    echo ""
    
    # Verify installation
    if command -v refaktor &> /dev/null; then
        VERSION=$(refaktor --version 2>&1 | head -n1)
        echo "Installed: $VERSION"
        echo ""
        echo "Get started with:"
        echo "  refaktor --help"
        echo ""
        echo "Quick example:"
        echo "  refaktor rename old_name new_name"
    else
        echo -e "${YELLOW}Warning: refaktor was installed but is not in your PATH.${NC}"
        echo "Add $INSTALL_DIR to your PATH:"
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
    fi
}

# Alternative installation directory
use_local_install() {
    echo "Installing to ~/.local/bin (no sudo required)..."
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
    install_refaktor
    
    # Check if ~/.local/bin is in PATH
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        echo ""
        echo -e "${YELLOW}Note: ~/.local/bin is not in your PATH.${NC}"
        echo "Add this to your shell configuration file (.bashrc, .zshrc, etc.):"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        echo "Then reload your shell or run:"
        echo "  source ~/.bashrc  # or ~/.zshrc"
    fi
}

# Main installation flow
main() {
    echo "🔧 Refaktor Installer"
    echo "===================="
    echo ""
    
    detect_platform
    
    # Check if user wants local installation
    if [ "$1" = "--local" ] || [ "$1" = "-l" ]; then
        use_local_install
    else
        check_permissions
        install_refaktor
    fi
}

# Run main function
main "$@"