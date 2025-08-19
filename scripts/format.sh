#!/bin/bash
set -e

# Colors for output
# RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🔧 Running formatters...${NC}"

# Run cargo fmt
echo -e "${YELLOW}Running cargo fmt...${NC}"
cargo fmt --all

# Run biome format on JavaScript/TypeScript projects
for dir in renamify-mcp renamify-vscode; do
  if [ -d "$dir" ]; then
    echo -e "${YELLOW}Running biome format in $dir...${NC}"
    (cd "$dir" && pnpm biome format --write .)
  fi
done

# Fix trailing whitespace and ensure newlines at end of files
echo -e "${YELLOW}Fixing whitespace issues...${NC}"

# Use git ls-files to only format tracked files
for file in $(git ls-files); do
  # Skip if file doesn't exist
  [[ -f "$file" ]] || continue

  # Check if it's a text file using our shared script
  if ! ./scripts/is-text-file.sh "$file"; then
    continue
  fi

  # Remove trailing whitespace
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' 's/[[:space:]]*$//' "$file"
  else
    sed -i 's/[[:space:]]*$//' "$file"
  fi

  # Ensure file ends with newline
  if [[ -s "$file" ]] && [[ $(tail -c1 "$file" | wc -l) -eq 0 ]]; then
    echo >> "$file"
  fi
done

echo -e "${GREEN}✅ Formatting complete!${NC}"
