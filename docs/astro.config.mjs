// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import starlightImageZoom from "starlight-image-zoom";

// https://astro.build/config
export default defineConfig({
  site: "https://docspring.github.io",
  base: "/refaktor",
  integrations: [
    starlight({
      title: "Refaktor",
      description:
        "Smart search & replace for code and files with case-aware transformations",
      plugins: [starlightImageZoom()],
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/DocSpring/refaktor",
        },
      ],
      sidebar: [
        {
          label: "Getting Started",
          items: [
            { label: "Installation", slug: "installation" },
            { label: "Quick Start", slug: "quick-start" },
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
          autogenerate: { directory: "commands" },
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
          label: "Integration",
          items: [
            { label: "VS Code Extension", slug: "integration/vscode" },
            { label: "MCP Server", slug: "integration/mcp" },
          ],
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
