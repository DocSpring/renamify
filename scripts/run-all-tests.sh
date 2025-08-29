#!/usr/bin/env bash
set -euo pipefail

# Colors for output
# RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🔧 Running all tests...${NC}"

echo -e "${YELLOW}Running cargo tests...${NC}"
cargo test

echo -e "${YELLOW}Running renamify-mcp tests...${NC}"
(cd renamify-mcp && pnpm check && pnpm test)

echo -e "${YELLOW}Running renamify-vscode tests...${NC}"
(cd renamify-vscode && pnpm check && pnpm test)

echo -e "${YELLOW}Running e2e tests...${NC}"
./scripts/e2e-test.sh

echo -e "${GREEN}All tests passed!${NC}"
