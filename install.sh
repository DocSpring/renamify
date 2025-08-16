#!/usr/bin/env bash
set -e

echo "Installing renamify..."
cargo install --path renamify-cli --force

echo ""
echo "✅ renamify installed successfully!"
echo "Location: $HOME/.cargo/bin/renamify"
echo ""
echo "Make sure $HOME/.cargo/bin is in your PATH"