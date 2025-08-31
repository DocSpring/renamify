#!/usr/bin/env bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to show usage
usage() {
    echo "Usage: $0 {cli|mcp|vscode|all} {major|minor|patch}"
    echo ""
    echo "Components:"
    echo "  cli     - Bump CLI version (Cargo.toml workspace)"
    echo "  mcp     - Bump MCP server version (package.json)"
    echo "  vscode  - Bump VS Code extension version (package.json)"
    echo "  all     - Bump all components together"
    echo ""
    echo "Bump types:"
    echo "  major   - Bump major version (1.0.0 -> 2.0.0)"
    echo "  minor   - Bump minor version (1.0.0 -> 1.1.0)"
    echo "  patch   - Bump patch version (1.0.0 -> 1.0.1)"
    exit 1
}

# Function to bump semantic version
bump_version() {
    local version=$1
    local bump_type=$2

    # Parse version
    IFS='.' read -r major minor patch <<< "$version"

    case "$bump_type" in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
    esac

    echo "${major}.${minor}.${patch}"
}

# Function to get current CLI version
get_cli_version() {
    grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Function to get current package.json version
get_package_version() {
    local file=$1
    grep '"version":' "$file" | head -1 | sed 's/.*"version": *"\(.*\)".*/\1/'
}

# Function to update CLI version (Cargo.toml)
update_cli_version() {
    local new_version=$1
    echo -e "${BLUE}Updating CLI to v${new_version}...${NC}"

    # Update workspace Cargo.toml
    sed -i.bak "s/^version = \".*\"/version = \"${new_version}\"/" Cargo.toml
    rm Cargo.toml.bak

    # Update Cargo.lock
    cargo update -w

    echo -e "${GREEN}✓ CLI updated to v${new_version}${NC}"
}

# Function to update MCP version
update_mcp_version() {
    local new_version=$1
    echo -e "${BLUE}Updating MCP server to v${new_version}...${NC}"

    cd renamify-mcp
    # Update package.json
    sed -i.bak "s/\"version\": \".*\"/\"version\": \"${new_version}\"/" package.json
    rm package.json.bak

    # Update lockfile
    pnpm install
    cd ..

    echo -e "${GREEN}✓ MCP server updated to v${new_version}${NC}"
}

# Function to update VS Code extension version
update_vscode_version() {
    local new_version=$1
    echo -e "${BLUE}Updating VS Code extension to v${new_version}...${NC}"

    cd renamify-vscode
    # Update package.json
    sed -i.bak "s/\"version\": \".*\"/\"version\": \"${new_version}\"/" package.json
    rm package.json.bak

    # Update lockfile
    pnpm install
    cd ..

    echo -e "${GREEN}✓ VS Code extension updated to v${new_version}${NC}"
}

# Check arguments
if [ $# -ne 2 ]; then
    usage
fi

COMPONENT=$1
BUMP_TYPE=$2

# Validate bump type
if [[ ! "$BUMP_TYPE" =~ ^(major|minor|patch)$ ]]; then
    echo -e "${RED}Error: Invalid bump type: $BUMP_TYPE${NC}"
    usage
fi

# Handle components
case "$COMPONENT" in
    cli)
        current_version=$(get_cli_version)
        new_version=$(bump_version "$current_version" "$BUMP_TYPE")
        echo -e "${YELLOW}CLI: ${current_version} -> ${new_version}${NC}"
        update_cli_version "$new_version"
        ;;
    mcp)
        current_version=$(get_package_version "renamify-mcp/package.json")
        new_version=$(bump_version "$current_version" "$BUMP_TYPE")
        echo -e "${YELLOW}MCP: ${current_version} -> ${new_version}${NC}"
        update_mcp_version "$new_version"
        ;;
    vscode)
        current_version=$(get_package_version "renamify-vscode/package.json")
        new_version=$(bump_version "$current_version" "$BUMP_TYPE")
        echo -e "${YELLOW}VS Code: ${current_version} -> ${new_version}${NC}"
        update_vscode_version "$new_version"
        ;;
    all)
        # For 'all', use the CLI version as the base
        current_version=$(get_cli_version)
        new_version=$(bump_version "$current_version" "$BUMP_TYPE")
        echo -e "${YELLOW}All components: ${current_version} -> ${new_version}${NC}"
        update_cli_version "$new_version"
        update_mcp_version "$new_version"
        update_vscode_version "$new_version"
        ;;
    *)
        echo -e "${RED}Error: Invalid component: $COMPONENT${NC}"
        usage
        ;;
esac

echo ""
echo -e "${GREEN}Version bump complete!${NC}"
echo ""
echo "Next steps:"
echo "1. Review the changes: git diff"
echo "2. Commit: git commit -am \"chore: bump $COMPONENT version to v${new_version}\""
echo "3. Tag (if releasing): git tag ${COMPONENT}-v${new_version}"
echo "4. Push: git push && git push --tags"
