#!/usr/bin/env bash
set -e

# Build release binaries for macOS and Linux
# Usage: ./scripts/build-release.sh [version]

VERSION=${1:-$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")}

echo "Building Renamify release $VERSION"

# Create release directory
RELEASE_DIR="release/$VERSION"
mkdir -p "$RELEASE_DIR"

# Build for current platform (likely macOS)
echo "Building for native platform..."
cargo build --release --bin renamify

# Detect current platform
if [[ "$OSTYPE" == "darwin"* ]]; then
    ARCH=$(uname -m)
    if [[ "$ARCH" == "x86_64" ]]; then
        NATIVE_TARGET="renamify-macos-amd64"
    else
        NATIVE_TARGET="renamify-macos-arm64"
    fi
else
    ARCH=$(uname -m)
    if [[ "$ARCH" == "x86_64" ]]; then
        NATIVE_TARGET="renamify-linux-amd64"
    else
        NATIVE_TARGET="renamify-linux-arm64"
    fi
fi

# Package native build
echo "Packaging $NATIVE_TARGET..."
cp target/release/renamify "$RELEASE_DIR/renamify"
cd "$RELEASE_DIR"
tar czf "$NATIVE_TARGET.tar.gz" renamify
rm renamify
cd ../..

# Cross-compile for other platforms (requires rustup targets)
echo ""
echo "To build for other platforms, you'll need to:"
echo ""
echo "For macOS cross-compilation:"
echo "  rustup target add x86_64-apple-darwin"
echo "  rustup target add aarch64-apple-darwin"
echo "  cargo build --release --target x86_64-apple-darwin"
echo "  cargo build --release --target aarch64-apple-darwin"
echo ""
echo "For Linux cross-compilation (from macOS):"
echo "  # Install cross"
echo "  cargo install cross"
echo "  # Build"
echo "  cross build --release --target x86_64-unknown-linux-gnu"
echo "  cross build --release --target aarch64-unknown-linux-gnu"
echo ""
echo "Native build complete: $RELEASE_DIR/$NATIVE_TARGET.tar.gz"

# Generate checksums
cd "$RELEASE_DIR"
if command -v sha256sum > /dev/null; then
    sha256sum ./*.tar.gz > SHA256SUMS
elif command -v shasum > /dev/null; then
    shasum -a 256 ./*.tar.gz > SHA256SUMS
fi
cd ../..

echo ""
echo "Release artifacts in: $RELEASE_DIR"
ls -la "$RELEASE_DIR"
