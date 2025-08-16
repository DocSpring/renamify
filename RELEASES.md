# Release Guide

This guide explains how to create a new release of Renamify.

## GitHub Releases Overview

GitHub releases work by:

1. Creating a git tag (e.g., `v1.0.0`)
2. Pushing the tag to GitHub
3. GitHub Actions automatically builds binaries for all platforms
4. A release is created with the binaries attached

## Prerequisites

1. Ensure you have an NPM account and are logged in:

   ```bash
   pnpm login
   ```

2. Add the NPM_TOKEN secret to your GitHub repository:
   - Go to Settings → Secrets and variables → Actions
   - Add a new secret named `NPM_TOKEN` with your npm token
   - Generate a token at: https://www.npmjs.com/settings/YOUR_USERNAME/tokens

## Creating a Release

### Option 1: Automated Release (Recommended)

1. Update version numbers in:

   - `renamify-cli/Cargo.toml`
   - `renamify-core/Cargo.toml`
   - `renamify-mcp/package.json`

2. Commit the version changes:

   ```bash
   git add -A
   git commit -m "Bump version to v1.0.0"
   git push
   ```

3. Create and push a tag:

   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

4. GitHub Actions will automatically:
   - Build binaries for macOS (Intel & Apple Silicon) and Linux (x86_64 & ARM64)
   - Create a GitHub release with the binaries
   - Publish the MCP server to npm

### Option 2: Manual Workflow Dispatch

You can also trigger a release manually from the GitHub Actions tab:

1. Go to Actions → Release workflow
2. Click "Run workflow"
3. Enter the tag name (e.g., `v1.0.0`)
4. Click "Run workflow"

### Option 3: Local Build (for testing)

For testing release builds locally:

```bash
# Build for your current platform
./scripts/build-release.sh v1.0.0

# Output will be in release/v1.0.0/
```

## Version Numbering

We follow Semantic Versioning (SemVer):

- **Major** (1.0.0): Breaking changes
- **Minor** (0.1.0): New features, backward compatible
- **Patch** (0.0.1): Bug fixes, backward compatible

Pre-release versions:

- Alpha: `v1.0.0-alpha.1`
- Beta: `v1.0.0-beta.1`
- Release Candidate: `v1.0.0-rc.1`

## Platform Support

The release workflow builds for:

- **macOS Intel** (x86_64-apple-darwin)
- **macOS Apple Silicon** (aarch64-apple-darwin)
- **Linux x86_64** (x86_64-unknown-linux-gnu)
- **Linux ARM64** (aarch64-unknown-linux-gnu)

## Release Artifacts

Each release includes:

- `renamify-macos-amd64.tar.gz` - macOS Intel binary
- `renamify-macos-arm64.tar.gz` - macOS Apple Silicon binary
- `renamify-linux-amd64.tar.gz` - Linux x86_64 binary
- `renamify-linux-arm64.tar.gz` - Linux ARM64 binary
- Source code (automatically included by GitHub)

## NPM Package

The MCP server is published to npm as `@renamify/mcp-server`.

Users can install it with:

```bash
npx @renamify/mcp-server
```

## Troubleshooting

### Build Failures

If the release build fails:

1. Check the GitHub Actions logs for errors
2. Ensure all tests pass locally
3. Verify Cargo.toml versions match

### NPM Publishing Issues

If npm publishing fails:

1. Verify the NPM_TOKEN secret is set correctly
2. Check that the package name is available
3. Ensure package.json is valid

### Cross-Compilation Issues

For Linux ARM64 builds from x86_64:

- The workflow uses `cross` for cross-compilation
- This requires Docker to be available in the CI environment

## Post-Release

After a successful release:

1. Update the documentation if needed
2. Announce the release (optional)
3. Start working on the next version!
