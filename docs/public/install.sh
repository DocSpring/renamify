#!/usr/bin/env bash
set -e

# Renamify Installation Script
# https://github.com/DocSpring/renamify

REPO="DocSpring/renamify"

# Default to user-local installation
DEFAULT_INSTALL_DIR="$HOME/.local/bin"
INSTALL_DIR="$DEFAULT_INSTALL_DIR"
INSTALL_MODE="local"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --system)
            INSTALL_DIR="/usr/local/bin"
            INSTALL_MODE="system"
            shift
            ;;
        --prefix)
            INSTALL_DIR="$2"
            INSTALL_MODE="custom"
            shift 2
            ;;
        --uninstall)
            UNINSTALL=true
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Usage: $0 [--system | --prefix <dir> | --uninstall]"
            exit 1
            ;;
    esac
done

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
            echo "Renamify currently supports Linux and macOS."
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
            echo "Renamify currently supports x86_64/amd64 and aarch64/arm64."
            exit 1
            ;;
    esac

    ASSET_NAME="renamify-${PLATFORM}-${ARCH}.tar.gz"
}

# Uninstall function
uninstall_renamify() {
    echo "🗑️  Uninstalling Renamify..."

    # Check common locations
    LOCATIONS=("$HOME/.local/bin/renamify" "/usr/local/bin/renamify" "$HOME/bin/renamify")
    FOUND=false

    for loc in "${LOCATIONS[@]}"; do
        if [ -f "$loc" ]; then
            echo "Found renamify at: $loc"
            if [ -w "$loc" ]; then
                rm "$loc"
            else
                sudo rm "$loc"
            fi
            echo -e "${GREEN}✓ Removed $loc${NC}"
            FOUND=true
        fi
    done

    if [ "$FOUND" = false ]; then
        echo "Renamify not found in standard locations."
        echo "If installed elsewhere, please remove manually."
    fi

    exit 0
}

# Check for Homebrew on macOS
check_homebrew() {
    if [ "$PLATFORM" = "macos" ] && command -v brew &> /dev/null; then
        echo -e "${BLUE}ℹ️  Homebrew detected${NC}"
        echo ""
        echo "For macOS, we recommend installing via Homebrew (when available):"
        echo -e "  ${GREEN}brew install renamify${NC}  # Coming soon"
        echo ""
        echo "Continuing with manual installation to $INSTALL_DIR..."
        echo ""
    fi
}

# Detect user's shell
detect_shell() {
    if [ -n "$SHELL" ]; then
        case "$SHELL" in
            */bash)
                USER_SHELL="bash"
                SHELL_RC="$HOME/.bashrc"
                # On macOS, .bash_profile is often used instead
                if [ "$PLATFORM" = "macos" ] && [ -f "$HOME/.bash_profile" ]; then
                    SHELL_RC="$HOME/.bash_profile"
                fi
                ;;
            */zsh)
                USER_SHELL="zsh"
                # Use .zshrc for interactive shells
                SHELL_RC="$HOME/.zshrc"
                ;;
            */fish)
                USER_SHELL="fish"
                SHELL_RC="$HOME/.config/fish/config.fish"
                ;;
            *)
                USER_SHELL="unknown"
                SHELL_RC=""
                ;;
        esac
    else
        USER_SHELL="unknown"
        SHELL_RC=""
    fi
}

# Check if directory is in PATH
check_path() {
    local dir="$1"
    if [[ ":$PATH:" == *":$dir:"* ]]; then
        return 0
    else
        return 1
    fi
}

# Download and install
install_renamify() {
    echo "📦 Installing Renamify..."
    echo "  Platform: $PLATFORM"
    echo "  Architecture: $ARCH"
    echo "  Destination: $INSTALL_DIR"
    echo ""

    # Create install directory if needed
    if [ ! -d "$INSTALL_DIR" ]; then
        echo "Creating directory: $INSTALL_DIR"
        mkdir -p "$INSTALL_DIR"
    fi

    # Check write permissions
    if [ ! -w "$INSTALL_DIR" ]; then
        if [ "$INSTALL_MODE" = "system" ]; then
            echo -e "${YELLOW}System installation requires sudo${NC}"
            NEED_SUDO=true
        else
            echo -e "${RED}Error: Cannot write to $INSTALL_DIR${NC}"
            echo "Please check permissions or choose a different directory."
            exit 1
        fi
    fi

    # Get the latest release URL
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"

    echo "Downloading from: $DOWNLOAD_URL"

    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TEMP_DIR"' EXIT

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
    if [ "$NEED_SUDO" = true ]; then
        sudo mv "$TEMP_DIR/renamify" "$INSTALL_DIR/"
        sudo chmod 755 "$INSTALL_DIR/renamify"
    else
        mv "$TEMP_DIR/renamify" "$INSTALL_DIR/"
        chmod 755 "$INSTALL_DIR/renamify"
    fi

    echo -e "${GREEN}✓ Renamify installed successfully!${NC}"
    echo -e "  Installed to: ${BLUE}$INSTALL_DIR/renamify${NC}"
    echo ""
}

# Check installation and PATH
verify_installation() {
    # Check if renamify is accessible
    if command -v renamify &> /dev/null; then
        VERSION=$(renamify --version 2>&1 | head -n1)
        echo -e "${GREEN}✓ Installation verified${NC}"
        echo "  Version: $VERSION"
        echo ""
        echo "Get started with:"
        echo "  renamify --help"
        echo ""
        echo "Quick example:"
        echo "  renamify rename old_name new_name"
    else
        # Check if the binary exists but PATH needs updating
        if [ -f "$INSTALL_DIR/renamify" ]; then
            echo -e "${YELLOW}⚠️  Renamify installed but not in PATH${NC}"
            echo ""

            if ! check_path "$INSTALL_DIR"; then
                detect_shell
                echo "Add $INSTALL_DIR to your PATH:"
                echo ""

                case "$USER_SHELL" in
                    bash)
                        echo "Run this command:"
                        echo -e "  ${GREEN}echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> $SHELL_RC${NC}"
                        echo ""
                        echo "Then reload your shell:"
                        echo -e "  ${GREEN}source $SHELL_RC${NC}"
                        ;;
                    zsh)
                        echo "Run this command:"
                        echo -e "  ${GREEN}echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> $SHELL_RC${NC}"
                        echo ""
                        echo "Then reload your shell:"
                        echo -e "  ${GREEN}source $SHELL_RC${NC}"
                        ;;
                    fish)
                        echo "Run this command:"
                        echo -e "  ${GREEN}fish_add_path \$HOME/.local/bin${NC}"
                        echo ""
                        echo "Or manually:"
                        echo -e "  ${GREEN}set -U fish_user_paths \$HOME/.local/bin \$fish_user_paths${NC}"
                        ;;
                    *)
                        echo "Add this line to your shell configuration:"
                        echo -e "  ${GREEN}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
                        ;;
                esac
            fi
        else
            echo -e "${RED}Error: Installation may have failed${NC}"
            echo "Binary not found at: $INSTALL_DIR/renamify"
            exit 1
        fi
    fi
}

# Main installation flow
main() {
    echo "🔧 Renamify Installer"
    echo "===================="
    echo ""

    # Handle uninstall
    if [ "$UNINSTALL" = true ]; then
        uninstall_renamify
    fi

    detect_platform

    # Check for Homebrew on macOS (but continue with install)
    if [ "$PLATFORM" = "macos" ] && [ "$INSTALL_MODE" = "local" ]; then
        check_homebrew
    fi

    install_renamify
    verify_installation
}

# Run main function
main
