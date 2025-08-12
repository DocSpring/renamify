#!/usr/bin/env bash
set -euo pipefail

if [ -d /tmp/refaktor-e2e-test ]; then
  echo "Removing existing /tmp/refaktor-e2e-test"
  rm -rf /tmp/refaktor-e2e-test
fi

echo "Cloning refaktor to /tmp/refaktor-e2e-test"
git clone . /tmp/refaktor-e2e-test
cd /tmp/refaktor-e2e-test

# For these tests, we want ripgrep to ignore the same files as refaktor
cp .rfignore .rgignore
echo .rgignore >> .git/info/exclude

cargo build --release
./target/release/refaktor --version

echo "=== Testing refaktor rename to smart_search_and_replace ==="

# Use refaktor to rename itself using plan/apply
./target/release/refaktor plan refaktor smart_search_and_replace --preview summary
./target/release/refaktor apply

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

# Undo the rename
./target/release/refaktor undo latest

# Verify the undo worked
if [ -f "smart-search-and-replace-core/Cargo.toml" ]; then
  echo "ERROR: smart-search-and-replace-core/Cargo.toml still exists!"
  exit 1
fi

# The working directory should be clean after the undo
if [ -n "$(git status --porcelain)" ]; then
  echo "ERROR: Working directory is not clean after undo!"
  git status
  git diff
  exit 1
fi

echo "✓ Working directory is clean - undo successful!"
        
      
echo "=== Testing redo functionality ==="

# Redo the rename
./target/release/refaktor redo latest

# Verify no instances of "refaktor" remain in the codebase (case-insensitive)
echo "Checking for remaining instances of 'refaktor'..."
if rg -i "refaktor"; then
  echo "ERROR: Found remaining instances of 'refaktor' in the codebase!"
  exit 1
fi
echo "✓ No instances of 'refaktor' found"

rm -rf target
cargo build --release
./target/release/smart_search_and_replace --version


echo "=== Testing smart_search_and_replace rename back to refaktor ==="

# Use smart_search_and_replace to rename itself back
./target/release/smart_search_and_replace rename smart_search_and_replace refaktor --preview summary

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

# Clean and rebuild with original name
rm -rf target
cargo build --release

# Final verification
./target/release/refaktor --version
./target/release/refaktor --help

# The working directory should be clean after the round-trip
if [ -n "$(git status --porcelain)" ]; then
  echo "ERROR: Working directory is not clean after round-trip!"
  git status
  git diff
  exit 1
fi
echo "✓ Working directory is clean - round-trip successful!"
