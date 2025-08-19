#!/usr/bin/env bash
set -euo pipefail

# ShellCheck script - runs shellcheck on all shell scripts in the project

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if shellcheck is installed
if ! command -v shellcheck &> /dev/null; then
    echo -e "${RED}Error: shellcheck is not installed${NC}"
    echo "Install it with:"
    echo "  macOS:    brew install shellcheck"
    echo "  Ubuntu:   apt-get install shellcheck"
    echo "  Fedora:   dnf install ShellCheck"
    echo "  Windows:  choco install shellcheck"
    exit 1
fi

# Find project root (where .git directory is)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "Running ShellCheck on all shell scripts..."
echo

# Find all shell scripts
# - All .sh files
# - All files in scripts/ directories (checking shebang)
# - Exclude node_modules, target, and other build directories
SCRIPTS=()

# Find all .sh files
while IFS= read -r -d '' file; do
    SCRIPTS+=("$file")
done < <(find . \
    -type f \
    -name "*.sh" \
    -not -path "./target/*" \
    -not -path "./node_modules/*" \
    -not -path "./*-e2e-test/*" \
    -not -path "./ripgrep-source/*" \
    -not -path "./refaktor-e2e-test/*" \
    -not -path "./docs/node_modules/*" \
    -not -path "./renamify-mcp/node_modules/*" \
    -print0)

# Find scripts without .sh extension in scripts directories
while IFS= read -r -d '' file; do
    # Check if it's a shell script by looking at shebang
    if head -n 1 "$file" 2>/dev/null | grep -qE '^#!/(usr/)?bin/(ba)?sh'; then
        SCRIPTS+=("$file")
    fi
done < <(find . \
    -type f \
    -path "*/scripts/*" \
    -not -name "*.sh" \
    -not -path "./target/*" \
    -not -path "./node_modules/*" \
    -not -path "./*-e2e-test/*" \
    -not -path "./ripgrep-source/*" \
    -not -path "./refaktor-e2e-test/*" \
    -not -path "./docs/node_modules/*" \
    -not -path "./renamify-mcp/node_modules/*" \
    -print0)

# Remove duplicates and sort
mapfile -t SCRIPTS < <(printf '%s\n' "${SCRIPTS[@]}" | sort -u)

if [ ${#SCRIPTS[@]} -eq 0 ]; then
    echo -e "${YELLOW}No shell scripts found${NC}"
    exit 0
fi

# echo "Found ${#SCRIPTS[@]} shell scripts to check:"
# printf '%s\n' "${SCRIPTS[@]}"
# echo

# Track failures
FAILED_SCRIPTS=()
PASSED_COUNT=0

# Run shellcheck on each script
for script in "${SCRIPTS[@]}"; do
    echo -n "Checking $script... "
    if shellcheck "$script" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        echo -e "${RED}✗${NC}"
        FAILED_SCRIPTS+=("$script")
    fi
done

echo
echo "────────────────────────────────────────"

# Summary
if [ ${#FAILED_SCRIPTS[@]} -eq 0 ]; then
    echo -e "${GREEN}All ${PASSED_COUNT} scripts passed ShellCheck!${NC}"
    exit 0
else
    echo -e "${RED}ShellCheck found issues in ${#FAILED_SCRIPTS[@]} script(s):${NC}"
    echo
    for script in "${FAILED_SCRIPTS[@]}"; do
        echo -e "${YELLOW}Issues in $script:${NC}"
        shellcheck "$script" || true
        echo
    done
    echo "────────────────────────────────────────"
    echo -e "${GREEN}Passed:${NC} $PASSED_COUNT"
    echo -e "${RED}Failed:${NC} ${#FAILED_SCRIPTS[@]}"
    exit 1
fi
