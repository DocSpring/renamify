#!/usr/bin/env bash
set -euo pipefail

# Faster: build debug for the loop, build release once at the end
export CARGO_TARGET_DIR=/tmp/refaktor-target

WORKDIR=./refaktor-e2e-test

if [ -d "$WORKDIR" ]; then
  echo "Removing existing $WORKDIR"
  rm -rf "$WORKDIR"
fi
if [ -d "$CARGO_TARGET_DIR" ]; then
  echo "Removing existing $CARGO_TARGET_DIR"
  rm -rf "$CARGO_TARGET_DIR"
fi

function ensure_working_directory_is_clean() {
  if [ -n "$(git status --porcelain)" ]; then
    echo "ERROR: Working directory is not clean at stage: ${1}"
    git status
    git diff | head -n 100 
    exit 1
  fi
}

echo "Cloning refaktor to $WORKDIR"
git clone . "$WORKDIR"
cd "$WORKDIR"

# Make ripgrep respect the same ignores as the tool
cp .rfignore .rgignore
echo .rgignore >> .git/info/exclude

# Make sure working directory is clean before we start
ensure_working_directory_is_clean "before test"

echo "=== Initial debug build ==="
cargo build
DEBUG_REFAKTOR="$CARGO_TARGET_DIR/debug/refaktor"
"$DEBUG_REFAKTOR" --version

ensure_working_directory_is_clean "after initial build"

echo "=== Testing refaktor rename to awesome_file_renaming_tool ==="
# Use refaktor to rename itself using plan/apply
"$DEBUG_REFAKTOR" plan refaktor awesome_file_renaming_tool --preview summary
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
if [ ! -f "awesome-file-renaming-tool-core/Cargo.toml" ]; then
  echo "ERROR: awesome-file-renaming-tool-core/Cargo.toml not found!"
  exit 1
fi
echo "✓ No instances of 'refaktor' found"

echo "=== Testing undo functionality ==="
"$DEBUG_REFAKTOR" undo latest

# Verify the undo worked
if [ -f "awesome-file-renaming-tool-core/Cargo.toml" ]; then
  echo "ERROR: awesome-file-renaming-tool-core/Cargo.toml still exists!"
  exit 1
fi
ensure_working_directory_is_clean "after undo"
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

echo "=== Build debug for awesome_file_renaming_tool and check ==="
cargo build
DEBUG_SSAR="$CARGO_TARGET_DIR/debug/awesome_file_renaming_tool"
"$DEBUG_SSAR" --version

echo "=== Running tests with new name ==="
cargo test

echo "=== Testing awesome_file_renaming_tool rename back to refaktor ==="
# Use awesome_file_renaming_tool to rename itself back
"$DEBUG_SSAR" rename awesome_file_renaming_tool refaktor --preview summary

# Verify the rename worked
if [ -f "awesome-file-renaming-tool-core/Cargo.toml" ]; then
  echo "ERROR: awesome-file-renaming-tool-core/Cargo.toml still exists!"
  exit 1
fi
if [ ! -f "refaktor-core/Cargo.toml" ]; then
  echo "ERROR: refaktor-core/Cargo.toml not found!"
  exit 1
fi

# Verify no instances of "awesome_file_renaming_tool" or "awesome-file-renaming-tool" remain
echo "Checking for remaining instances of 'awesome_file_renaming_tool' or 'awesome-file-renaming-tool'..."
if rg -i "(awesome_file_renaming_tool|awesome-file-renaming-tool|smartsearchandreplace)"; then
  echo "ERROR: Found remaining instances of 'awesome_file_renaming_tool' in the codebase!"
  exit 1
fi
echo "✓ No instances of 'awesome_file_renaming_tool' found"

echo "=== Final release build and verification ==="
cargo build --release
REL_REFAKTOR="$CARGO_TARGET_DIR/release/refaktor"

"$REL_REFAKTOR" --version
"$REL_REFAKTOR" --help

ensure_working_directory_is_clean "after round-trip"
echo "✓ Working directory is clean - round-trip successful!"
