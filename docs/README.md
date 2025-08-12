# Refaktor Documentation Site

This directory contains the Refaktor documentation website built with [Starlight](https://starlight.astro.build).

## 📖 Documentation Structure

- **Guide** - User guides and tutorials
- **Reference** - CLI reference and configuration options
- **Demo** - Interactive demonstration of Refaktor's capabilities
- **Case Studies** - Real-world usage examples

## 🛠️ Development

We use [pnpm](https://pnpm.io/) to manage dependencies.

```bash
# Install dependencies
pnpm install

# Start development server
pnpm dev

# Build for production
pnpm build
```

The documentation is automatically built and deployed from the `main` branch.

## 📝 Content

Documentation content is written in Markdown/MDX and located in `src/content/docs/`. The site includes:

- Getting started guides
- CLI command reference
- Configuration examples
- Live demo with self-referential testing
- Real-world case studies

Visit the live documentation at https://docspring.github.io/refaktor/ to learn more about Refaktor.
