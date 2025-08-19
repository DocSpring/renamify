#!/usr/bin/env bash
set -euo pipefail

if ! command -v rg &> /dev/null; then
  echo "ripgrep not found. Please install it."
  exit 1
fi

echo "=== Renamify E2E Test ==="

# Faster: build debug for the loop, build release once at the end
export CARGO_TARGET_DIR=/tmp/renamify-e2e-target
WORKDIR=./renamify-e2e-test

if [ -z "${CI:-}" ]; then
  RUSTC_WRAPPER="$(command -v sccache || true)"
  if [ -n "$RUSTC_WRAPPER" ]; then
    SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.cache/sccache}"
    export RUSTC_WRAPPER SCCACHE_DIR
  fi
fi

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

echo "Cloning renamify to $WORKDIR"
git clone . "$WORKDIR"
cd "$WORKDIR"

# Make ripgrep respect the same ignores as the tool
cp .rnignore .rgignore
echo .rgignore >> .git/info/exclude

# Make sure working directory is clean before we start
ensure_working_directory_is_clean "before test"

echo "=== Initial debug build ==="
cargo build
DEBUG_RENAMIFY="$CARGO_TARGET_DIR/debug/renamify"
"$DEBUG_RENAMIFY" --version

ensure_working_directory_is_clean "after initial build"

echo "=== Testing renamify rename to awesome_file_renaming_tool ==="
# Use renamify to rename itself using plan/apply
"$DEBUG_RENAMIFY" plan renamify awesome_file_renaming_tool --preview summary
"$DEBUG_RENAMIFY" apply

# Verify no instances of "renamify" remain in the codebase (case-insensitive)
echo "Checking for remaining instances of 'renamify'..."
if rg -i renamify; then
  echo "ERROR: Found remaining instances of 'renamify' in the codebase!"
  exit 1
fi

# Verify the rename worked by checking key dirs / files
if [ -d "renamify-core" ]; then
  echo "ERROR: renamify-core directory still exists!"
  exit 1
fi
if [ ! -f "awesome-file-renaming-tool-core/Cargo.toml" ]; then
  echo "ERROR: awesome-file-renaming-tool-core/Cargo.toml not found!"
  exit 1
fi
echo "✓ No instances of 'renamify' found"

echo "=== Testing undo functionality ==="
"$DEBUG_RENAMIFY" undo latest

# Verify the undo worked
if [ -f "awesome-file-renaming-tool-core/Cargo.toml" ]; then
  echo "ERROR: awesome-file-renaming-tool-core/Cargo.toml still exists!"
  exit 1
fi
ensure_working_directory_is_clean "after undo"
echo "✓ Working directory is clean - undo successful!"

echo "=== Testing redo functionality ==="
"$DEBUG_RENAMIFY" redo latest

# Verify no instances of "renamify" remain in the codebase (case-insensitive)
echo "Checking for remaining instances of 'renamify'..."
if rg -i "renamify"; then
  echo "ERROR: Found remaining instances of 'renamify' in the codebase!"
  exit 1
fi
echo "✓ No instances of 'renamify' found"

echo "=== Build debug for awesome_file_renaming_tool and check ==="
cargo build
DEBUG_AFRT="$CARGO_TARGET_DIR/debug/awesome_file_renaming_tool"
"$DEBUG_AFRT" --version

echo "=== Running tests with new name ==="
cargo test

echo "=== Running $DEBUG_AFRT init to add .awesome_file_renaming_tool/ to .gitignore"
"$DEBUG_AFRT" init

if ! rg .awesome_file_renaming_tool/ .gitignore; then 
  echo "ERROR: Did not find .awesome_file_renaming_tool/ in .gitignore!"
  cat .gitignore
  exit 1
fi
echo "✓ Found .awesome_file_renaming_tool/ in .gitignore"

echo "=== Committing change to .gitignore"
# Set git user config if not already set
if ! git config user.email > /dev/null 2>&1; then
  git config --global user.email "e2e.test@example.com"
fi
if ! git config user.name > /dev/null 2>&1; then
  git config --global user.name "renamify e2e test"
fi
git add .gitignore
git commit -m "Added .awesome_file_renaming_tool/ to .gitignore"

echo "=== Testing awesome_file_renaming_tool rename back to renamify ==="
# Use awesome_file_renaming_tool to rename itself back
"$DEBUG_AFRT" rename awesome_file_renaming_tool renamify --preview summary --yes

# Verify the rename worked
if [ -f "awesome-file-renaming-tool-core/Cargo.toml" ]; then
  echo "ERROR: awesome-file-renaming-tool-core/Cargo.toml still exists!"
  exit 1
fi
if [ ! -f "renamify-core/Cargo.toml" ]; then
  echo "ERROR: renamify-core/Cargo.toml not found!"
  exit 1
fi

# Verify no instances of "awesome_file_renaming_tool" or "awesome-file-renaming-tool" remain
echo "Checking for remaining instances of 'awesome_file_renaming_tool' or 'awesome-file-renaming-tool'..."
if rg -i "(awesome_file_renaming_tool|awesome-file-renaming-tool|awesomefilerenamingtool)"; then
  echo "ERROR: Found remaining instances of 'awesome_file_renaming_tool' in the codebase!"
  exit 1
fi
echo "✓ No instances of 'awesome_file_renaming_tool' found"

echo "=== Final release build and verification ==="
cargo build --release
REL_RENAMIFY="$CARGO_TARGET_DIR/release/renamify"

"$REL_RENAMIFY" --version

ensure_working_directory_is_clean "after round-trip"
echo "✓ Working directory is clean - round-trip successful!"
