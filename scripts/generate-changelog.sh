#!/bin/bash

# Generate changelog for a release based on tag prefix
# Usage: ./generate-changelog.sh <prefix> <version>
# Example: ./generate-changelog.sh cli 0.1.1

set -e
set -o pipefail

PREFIX=$1
VERSION=$2

if [ -z "$PREFIX" ] || [ -z "$VERSION" ]; then
    echo "Usage: $0 <prefix> <version>"
    echo "Example: $0 cli 0.1.1"
    exit 1
fi

TAG="${PREFIX}-v${VERSION}"

# Determine which directories to filter based on prefix
case "$PREFIX" in
    vscode)
        PATH_FILTER=("renamify-vscode/")
        ;;
    mcp)
        PATH_FILTER=("renamify-mcp/")
        ;;
    cli)
        # CLI includes both renamify-core and renamify-cli
        PATH_FILTER=("renamify-core/" "renamify-cli/")
        ;;
    *)
        echo "Error: Unknown prefix '$PREFIX'. Expected: vscode, mcp, or cli" >&2
        exit 1
        ;;
esac

# Check if the target tag exists
if ! git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Error: Tag '$TAG' does not exist" >&2
    echo "Please create the tag first or check the version number" >&2
    exit 1
fi

echo "## What's Changed"
echo ""

# Get the previous tag with same prefix
# Use grep -v with || true to prevent exit on no match
PREV_TAG=$(git tag -l "${PREFIX}-v*" --sort=-version:refname | { grep -v "^${TAG}$" || true; } | head -n 1)

if [ -n "$PREV_TAG" ]; then
    echo "### Commits since $PREV_TAG"
    COMMITS=$(git log --pretty=format:"- %s (%h)" "$PREV_TAG..$TAG" -- "${PATH_FILTER[@]}" | grep -v -i "^- bump .*version" || true)
    if [ -z "$COMMITS" ]; then
        echo "- No relevant git commits found"
    else
        echo "$COMMITS"
    fi
else
    echo "### Commits"
    # Show last 10 commits up to current tag if no previous tag that touched the package
    COMMITS=$(git log --pretty=format:"- %s (%h)" -n 10 "$TAG" -- "${PATH_FILTER[@]}" | grep -v -i "^- bump .*version" || true)
    if [ -z "$COMMITS" ]; then
        echo "- No relevant git commits found"
    else
        echo "$COMMITS"
    fi
fi

echo ""
