#!/usr/bin/env bash
set -euo pipefail

# Faster: build debug for the loop, build release once at the end
export CARGO_TARGET_DIR=/tmp/refaktor-target

WORKDIR=/tmp/refaktor-e2e-test

if [ -d "$WORKDIR" ]; then
  echo "Removing existing $WORKDIR"
  rm -rf "$WORKDIR"
fi
if [ -d "$CARGO_TARGET_DIR" ]; then
  echo "Removing existing $CARGO_TARGET_DIR"
  rm -rf "$CARGO_TARGET_DIR"
fi


echo "Cloning refaktor to $WORKDIR"
git clone . "$WORKDIR"
cd "$WORKDIR"

# Make ripgrep respect the same ignores as the tool
cp .rfignore .rgignore
echo .rgignore >> .git/info/exclude

echo "=== Initial debug build ==="
cargo build
DEBUG_REFAKTOR="$CARGO_TARGET_DIR/debug/refaktor"
"$DEBUG_REFAKTOR" --version

echo "=== Testing refaktor rename to smart_search_and_replace ==="
# Use refaktor to rename itself using plan/apply
"$DEBUG_REFAKTOR" plan refaktor smart_search_and_replace --preview summary
"$DEBUG_REFAKTOR" apply

# Verify no instances of "refaktor" remain in the codebase (case-insensitive)
echo "Checking for remaining instances of 'refaktor'..."
if rg -i refaktor; then
  echo "ERROR: Found remaining instances of 'refaktor' in the codebase!"
  exit 1
fi

# Verify the rename worked by checking key dirs / files
if [ -d "refaktor-core" ]; then
  echo "ERROR: refaktor-core directory still exists!"
  exit 1
fi
if [ ! -f "smart-search-and-replace-core/Cargo.toml" ]; then
  echo "ERROR: smart-search-and-replace-core/Cargo.toml not found!"
  exit 1
fi
echo "✓ No instances of 'refaktor' found"

echo "=== Testing undo functionality ==="
"$DEBUG_REFAKTOR" undo latest

# Verify the undo worked
if [ -f "smart-search-and-replace-core/Cargo.toml" ]; then
  echo "ERROR: smart-search-and-replace-core/Cargo.toml still exists!"
  exit 1
fi

# The working directory should be clean after the undo
if [ -n "$(git status --porcelain)" ]; then
  echo "ERROR: Working directory is not clean after undo!"
  git status
  git diff | head -n 100 
  exit 1
fi
echo "✓ Working directory is clean - undo successful!"

echo "=== Testing redo functionality ==="
"$DEBUG_REFAKTOR" redo latest

# Verify no instances of "refaktor" remain in the codebase (case-insensitive)
echo "Checking for remaining instances of 'refaktor'..."
if rg -i "refaktor"; then
  echo "ERROR: Found remaining instances of 'refaktor' in the codebase!"
  exit 1
fi
echo "✓ No instances of 'refaktor' found"

echo "=== Build debug for smart_search_and_replace and check ==="
cargo build
DEBUG_SSAR="$CARGO_TARGET_DIR/debug/smart_search_and_replace"
"$DEBUG_SSAR" --version

echo "=== Testing smart_search_and_replace rename back to refaktor ==="
# Use smart_search_and_replace to rename itself back
"$DEBUG_SSAR" rename smart_search_and_replace refaktor --preview summary

# Verify the rename worked
if [ -f "smart-search-and-replace-core/Cargo.toml" ]; then
  echo "ERROR: smart-search-and-replace-core/Cargo.toml still exists!"
  exit 1
fi
if [ ! -f "refaktor-core/Cargo.toml" ]; then
  echo "ERROR: refaktor-core/Cargo.toml not found!"
  exit 1
fi

# Verify no instances of "smart_search_and_replace" or "smart-search-and-replace" remain
echo "Checking for remaining instances of 'smart_search_and_replace' or 'smart-search-and-replace'..."
if rg "(smart_search_and_replace|smart-search-and-replace|smartsearchandreplace)"; then
  echo "ERROR: Found remaining instances of 'smart_search_and_replace' in the codebase!"
  exit 1
fi
echo "✓ No instances of 'smart_search_and_replace' found"

echo "=== Final release build and verification ==="
cargo build --release
REL_REFAKTOR="$CARGO_TARGET_DIR/release/refaktor"

"$REL_REFAKTOR" --version
"$REL_REFAKTOR" --help

# The working directory should be clean after the round-trip
if [ -n "$(git status --porcelain)" ]; then
  echo "ERROR: Working directory is not clean after round-trip!"
  git status
  git diff | head -n 100
  exit 1
fi
echo "✓ Working directory is clean - round-trip successful!"
