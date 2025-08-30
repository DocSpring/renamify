#!/bin/bash

# Script to bump cache timestamps in GitHub workflow files
# Excludes release-*.yml files

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get current date and time
CURRENT_DATE=$(date +"%Y-%m-%d %H:%M:%S")

echo -e "${BLUE}Bumping workflow cache timestamps to: ${CURRENT_DATE}${NC}"
echo ""

# Find all workflow files except release-*.yml
WORKFLOW_DIR=".github/workflows"

if [ ! -d "$WORKFLOW_DIR" ]; then
    echo -e "${YELLOW}Warning: $WORKFLOW_DIR directory not found${NC}"
    exit 1
fi

# Process each workflow file
updated_count=0
skipped_count=0

for file in "$WORKFLOW_DIR"/*.yml "$WORKFLOW_DIR"/*.yaml; do
    # Skip if no files match the pattern
    [ -e "$file" ] || continue

    # Get basename
    basename=$(basename "$file")

    # Skip release-*.yml files
    if [[ "$basename" == release-*.yml ]] || [[ "$basename" == release-*.yaml ]]; then
        echo -e "${YELLOW}Skipping: $basename (release workflow)${NC}"
        ((skipped_count++))
        continue
    fi

    # Check if file has a cache comment at the top
    if head -n 5 "$file" | grep -q "^# Cache:"; then
        # Update the cache timestamp
        # Use a temporary file to avoid issues with sed -i differences between macOS and Linux
        temp_file=$(mktemp)
        sed "s/^# Cache: .*$/# Cache: $CURRENT_DATE/" "$file" > "$temp_file"
        mv "$temp_file" "$file"
        echo -e "${GREEN}✓ Updated: $basename${NC}"
        ((updated_count++))
    else
        echo -e "${YELLOW}No cache comment found in: $basename${NC}"
    fi
done

echo ""
echo -e "${BLUE}Summary:${NC}"
echo "  Updated: $updated_count workflow(s)"
echo "  Skipped: $skipped_count release workflow(s)"
echo ""
echo -e "${GREEN}Cache timestamps bumped to: $CURRENT_DATE${NC}"
