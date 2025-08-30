#!/bin/bash

# Generate changelog for a release based on tag prefix
# Usage: ./generate-changelog.sh <prefix> <version>
# Example: ./generate-changelog.sh cli 0.1.1

set -e

PREFIX=$1
VERSION=$2

if [ -z "$PREFIX" ] || [ -z "$VERSION" ]; then
    echo "Usage: $0 <prefix> <version>"
    echo "Example: $0 cli 0.1.1"
    exit 1
fi

TAG="${PREFIX}-v${VERSION}"

# Check if the target tag exists
if ! git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Error: Tag '$TAG' does not exist" >&2
    echo "Please create the tag first or check the version number" >&2
    exit 1
fi

echo "## What's Changed"
echo ""

# Get the previous tag with same prefix (need to fetch all tags first)
git fetch --tags >/dev/null 2>&1
PREV_TAG=$(git tag -l "${PREFIX}-v*" --sort=-version:refname | grep -v "^${TAG}$" | head -n 1)

if [ -n "$PREV_TAG" ]; then
    echo "### Commits since $PREV_TAG"
    git log --pretty=format:"- %s (%h)" "$PREV_TAG..$TAG"
else
    echo "### Commits"
    # Show last 10 commits up to current tag if no previous tag
    git log --pretty=format:"- %s (%h)" -n 10 "$TAG"
fi

echo ""
echo ""
