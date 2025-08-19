#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Checking for whitespace issues..."

FOUND_ISSUES=0

# Use git ls-files to only check tracked files
for file in $(git ls-files); do
  # Skip if file doesn't exist
  [[ -f "$file" ]] || continue

  # Check if it's a text file using the file command
  if ! file "$file" | grep -q "text\|ASCII\|UTF"; then
    continue
  fi

  # Check for trailing whitespace
  if grep -q '[[:space:]]$' "$file"; then
    echo -e "${RED}✗ Trailing whitespace found in: $file${NC}"
    FOUND_ISSUES=1
  fi

  # Check if file ends with newline
  if [[ -s "$file" ]] && [[ $(tail -c1 "$file" | wc -l) -eq 0 ]]; then
    echo -e "${RED}✗ Missing newline at end of file: $file${NC}"
    FOUND_ISSUES=1
  fi
done

if [ $FOUND_ISSUES -eq 1 ]; then
  echo -e "${RED}✗ Found whitespace issues!${NC}"
  echo -e "${YELLOW}Run './scripts/format.sh' to fix these issues${NC}"
  exit 1
else
  echo -e "${GREEN}✅ No whitespace issues found${NC}"
fi
