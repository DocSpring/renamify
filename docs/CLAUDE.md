# Astro Starlight Documentation Guidelines

## Critical Rules

### Never Use H1 Headers in Content

**NEVER** start MDX pages with an H1 (`# Header`) - Starlight automatically
generates the H1 from the frontmatter `title`.

❌ **Wrong:**

```mdx
---
title: MCP Server Configuration
---

# MCP Server Configuration

Content here...
```

✅ **Correct:**

```mdx
---
title: MCP Server Configuration
---

Content here starts immediately, or with H2 (## Section)
```

### Component Imports

Always import Starlight components at the top:

```mdx
import {
  Card,
  CardGrid,
  LinkCard,
  Tabs,
  TabItem,
  Steps,
  Code,
  Aside,
} from '@astrojs/starlight/components';

;
```

### Best Practices

1. **Structure**: Start content immediately after frontmatter, use H2 for main
   sections
2. **Components**: Use `<Steps>` for numbered procedures, `<Tabs>` for
   platform-specific content
3. **Cards**: Don't overuse CardGrid - simple lists are often better for
   readability
4. **Code blocks**: Use proper language tags for syntax highlighting
5. **Links**: Use LinkCard for prominent external links, regular markdown links
   for inline references

### Common Components

- `<Steps>` - Numbered step-by-step instructions
- `<Tabs>` and `<TabItem>` - Platform or option-specific content
- `<LinkCard>` - Prominent navigation to other pages
- `<Aside type="note|tip|caution|danger">` - Callout boxes
- `<Code>` - Inline code with syntax highlighting

### File Structure

- Use kebab-case for file names: `mcp-server-config.mdx`
- Organize in logical directories: `mcp/`, `commands/`, `examples/`
- Use `overview.mdx` for main landing pages in sections

### Frontmatter Requirements

Always include both title and description:

```yaml
---
title: Clear, Descriptive Title
description: Brief description for SEO and navigation
---
```

### Preview Formats

When showing renamify CLI output, use the appropriate format:

- `summary` - For AI agents (structured, minimal)
- `table` - For human-readable overviews
- `diff` - For detailed change previews
- `json` - For programmatic processing

### Testing Guidelines

- Always use `--dry-run` flag when testing renamify commands in docs
- Use "renamed_renaming_tool" as replacement target in examples (not the
  alternative protected string)
- Test all links and ensure they work in the local dev server

### Sidebar Configuration

- Keep sidebar items concise and well-organized
- Use autogenerate for large directories:
  `autogenerate: { directory: "commands" }`
- Group related items under clear labels
- Put most important sections (like MCP Server) near the top
