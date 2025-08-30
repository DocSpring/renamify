#!/bin/bash

# Script to create release tags for cli, mcp, and vscode if they don't already exist

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get versions from each component
CLI_VERSION=$(grep -E '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
MCP_VERSION=$(grep '"version":' renamify-mcp/package.json | head -1 | sed 's/.*"version": "\(.*\)".*/\1/')
VSCODE_VERSION=$(grep '"version":' renamify-vscode/package.json | head -1 | sed 's/.*"version": "\(.*\)".*/\1/')

echo -e "${BLUE}Checking release tags for current versions...${NC}"
echo "CLI version: $CLI_VERSION"
echo "MCP version: $MCP_VERSION"
echo "VS Code version: $VSCODE_VERSION"
echo ""

# Function to check and create tag
create_tag_if_missing() {
    local tag_name=$1
    local component=$2

    if git tag --list | grep -q "^${tag_name}$"; then
        echo -e "${YELLOW}✓ Tag ${tag_name} already exists${NC}"
        # Show which commit it points to
        commit=$(git rev-list -n 1 "${tag_name}")
        commit_short=$(git rev-parse --short "${commit}")
        commit_msg=$(git log --format=%s -n 1 "${commit}")
        echo "  Points to: ${commit_short} - ${commit_msg}"
    else
        echo -e "${GREEN}Creating tag ${tag_name} for ${component}${NC}"
        git tag -a "${tag_name}" -m "Release ${component} v${tag_name#*-v}"
        echo "  Created at current commit: $(git rev-parse --short HEAD)"
    fi
}

# Check and create tags
create_tag_if_missing "cli-v${CLI_VERSION}" "CLI"
create_tag_if_missing "mcp-v${MCP_VERSION}" "MCP server"
create_tag_if_missing "vscode-v${VSCODE_VERSION}" "VS Code extension"

echo ""
echo -e "${BLUE}Tag status complete!${NC}"
echo ""
echo "To push new tags to remote, run:"
echo "  git push origin --tags"
echo ""
echo "To push a specific tag:"
echo "  git push origin cli-v${CLI_VERSION}"
echo "  git push origin mcp-v${MCP_VERSION}"
echo "  git push origin vscode-v${VSCODE_VERSION}"
