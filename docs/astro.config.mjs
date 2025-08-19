// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import starlightImageZoom from "starlight-image-zoom";

// https://astro.build/config
export default defineConfig({
  site: "https://docspring.github.io",
  base: "/renamify",

  integrations: [
    starlight({
      title: "Renamify",
      description:
        "Smart search & replace for code and files with case-aware transformations",
      logo: {
        src: "./src/assets/logo.svg",
      },
      editLink: {
        baseUrl: "https://github.com/DocSpring/renamify/edit/main/docs",
      },
      plugins: [starlightImageZoom()],
      components: {
        Footer: "./src/components/Footer.astro",
      },
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/DocSpring/renamify",
        },
      ],
      sidebar: [
        {
          label: "Getting Started",
          items: [
            { label: "Installation", slug: "installation" },
            { label: "Self-Hosting Demo", slug: "self-hosting-demo" },
            { label: "Quick Start", slug: "quick-start" },
            {
              label: "Why Not Search and Replace?",
              slug: "why-not-search-and-replace",
            },
            { label: "FAQ", link: "/#frequently-asked-questions" },
          ],
        },
        {
          label: "MCP Server",
          items: [
            { label: "Overview", slug: "mcp/overview" },
            { label: "Installation", slug: "mcp/installation" },
            { label: "Configuration", slug: "mcp/configuration" },
            { label: "Tools Reference", slug: "mcp/tools" },
            { label: "Usage Examples", slug: "mcp/examples" },
            { label: "AI Agent Guide", slug: "mcp/ai-guide" },
          ],
        },
        {
          label: "Features",
          items: [
            {
              label: "Case-Aware Transformations",
              slug: "features/case-transformations",
            },
            {
              label: "File and Directory Renaming",
              slug: "features/file-renaming",
            },
            { label: "Safety Features", slug: "features/safety" },
            { label: "Filtering and Ignore Rules", slug: "features/filtering" },
          ],
        },
        {
          label: "Commands",
          items: [
            { label: "init", slug: "commands/init" },
            { label: "rename", slug: "commands/rename" },
            { label: "plan", slug: "commands/plan" },
            { label: "apply", slug: "commands/apply" },
            { label: "dry-run", slug: "commands/dry-run" },
            { label: "status", slug: "commands/status" },
            { label: "history", slug: "commands/history" },
            { label: "undo", slug: "commands/undo" },
            { label: "redo", slug: "commands/redo" },
          ],
        },
        {
          label: "Examples",
          autogenerate: { directory: "examples" },
        },
        {
          label: "Case Studies",
          autogenerate: { directory: "case-studies" },
        },
        {
          label: "VS Code Extension",
          items: [{ label: "Overview", slug: "vscode/overview" }],
        },
        {
          label: "Reference",
          items: [
            { label: "Configuration", slug: "reference/configuration" },
            { label: "Platform Support", slug: "reference/platform-support" },
          ],
        },
      ],
    }),
  ],
});
