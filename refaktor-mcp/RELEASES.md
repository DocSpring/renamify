# MCP Server Release Guide

This guide explains how to publish the Refaktor MCP Server to npm.

## NPM Publishing Setup

The MCP server is published to npm as `@docspring/refaktor-mcp` as part of the main Refaktor release process.

### Prerequisites

1. **NPM Account**: You need an npm account with access to the @docspring organization
2. **GitHub Repository Access**: Admin access to set up secrets

### Setting Up Automated NPM Publishing

#### Step 1: Create an NPM Access Token

1. Log in to [npmjs.com](https://www.npmjs.com) with your account
2. Click your profile icon → **Access Tokens**
3. Click **Generate New Token** → **Classic Token**
4. Select token type:
   - **Automation** (recommended) - for CI/CD, never expires
   - Or **Publish** - can publish packages
5. Give it a descriptive name like "GitHub Actions - Refaktor"
6. Click **Generate Token**
7. **Copy the token immediately** (you won't see it again!)

#### Step 2: Add Token to GitHub Repository

1. Go to the GitHub repository: https://github.com/DocSpring/refaktor
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Add the secret:
   - **Name:** `NPM_TOKEN`
   - **Secret:** Paste your npm token
5. Click **Add secret**

### First-Time Manual Publishing (Optional)

If you want to publish manually before setting up automation:

```bash
cd refaktor-mcp

# Login to npm
npm login

# Test the publish (dry run)
npm publish --access public --dry-run

# Actually publish
npm publish --access public
```

## Release Process

The MCP server is automatically published when you create a new Refaktor release:

### Automatic Publishing (via GitHub Release)

When you create a new tag and push it:

```bash
# Update version in refaktor-mcp/package.json
cd refaktor-mcp
npm version 0.1.0 --no-git-tag-version
git add package.json
git commit -m "Bump MCP server version to 0.1.0"

# Create and push tag from repo root
cd ..
git tag v0.1.0
git push origin v0.1.0
```

The GitHub Actions workflow will:

1. Build Refaktor binaries for all platforms
2. Create a GitHub release
3. **Automatically publish the MCP server to npm**

### Manual Publishing (Independent Release)

To publish just the MCP server without a full Refaktor release:

```bash
cd refaktor-mcp

# Update version
npm version patch  # or minor, or major

# Build
pnpm build

# Publish
npm publish --access public
```

## Package Information

- **Package Name**: `@docspring/refaktor-mcp`
- **Registry**: https://registry.npmjs.org
- **Installation**: `npx @docspring/refaktor-mcp`
- **Repository**: https://github.com/DocSpring/refaktor/tree/main/refaktor-mcp

## Version Management

The MCP server version should generally match the Refaktor CLI version for compatibility:

- Refaktor CLI: `v0.1.0` → MCP Server: `0.1.0`
- Version is set in `refaktor-mcp/package.json`
- The release workflow automatically strips the `v` prefix

## Troubleshooting

### NPM Publishing Fails in GitHub Actions

1. **Check the Actions logs** for specific error messages
2. **Verify NPM_TOKEN secret** is set correctly in repository settings
3. **Ensure token has publish permissions** - use "Automation" or "Publish" token type
4. **Check organization exists** at https://www.npmjs.com/org/docspring
5. **Verify package.json version** matches the tag (without the 'v' prefix)

### 403 Forbidden Error

- Token might be expired or lack permissions
- Organization might not exist
- You might not have access to the organization

### 402 Payment Required

- The @docspring organization might need a paid plan for private packages
- Ensure you're publishing with `--access public`

### Package Already Exists

- The version number might already be published
- Bump the version in package.json and try again

## Testing

Before releasing, test the MCP server locally:

```bash
cd refaktor-mcp

# Install dependencies
pnpm install

# Build
pnpm build

# Test
pnpm test

# Test the built package
npm pack --dry-run
```

## Support

For issues with:

- **NPM Publishing**: Check npm documentation or contact npm support
- **GitHub Actions**: Check the workflow logs in the Actions tab
- **MCP Server Code**: Open an issue on GitHub
