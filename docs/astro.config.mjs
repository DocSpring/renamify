// @ts-check

import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';
import starlightImageZoom from 'starlight-image-zoom';

// https://astro.build/config
export default defineConfig({
  site: 'https://docspring.github.io',
  base: '/renamify',

  integrations: [
    starlight({
      title: 'Renamify',
      description:
        'Smart search & replace for code and files with case-aware transformations',
      logo: {
        src: './src/assets/logo.svg',
      },
      customCss: ['./src/styles/custom.css'],
      editLink: {
        baseUrl: 'https://github.com/DocSpring/renamify/edit/main/docs',
      },
      plugins: [starlightImageZoom()],
      components: {
        Footer: './src/components/Footer.astro',
      },
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: 'https://github.com/DocSpring/renamify',
        },
      ],
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Installation', slug: 'installation' },
            { label: 'Self-Hosting Demo', slug: 'self-hosting-demo' },
            { label: 'Quick Start', slug: 'quick-start' },
            {
              label: 'Rename vs. Replace',
              slug: 'rename-vs-replace',
            },
            { label: 'FAQ', link: '/#frequently-asked-questions' },
          ],
        },
        {
          label: 'Features',
          items: [
            {
              label: 'Case-Aware Transformations',
              slug: 'features/case-transformations',
            },
            {
              label: 'File and Directory Renaming',
              slug: 'features/file-renaming',
            },
            { label: 'Safety Features', slug: 'features/safety' },
            { label: 'Filtering and Ignore Rules', slug: 'features/filtering' },
            {
              label: 'Resolving Case Ambiguity',
              slug: 'features/ambiguity-resolution',
            },
            {
              label: 'Atomic Identifiers',
              slug: 'features/atomic-identifiers',
            },
          ],
        },
        {
          label: 'MCP Server',
          items: [
            { label: 'Overview', slug: 'mcp/overview' },
            { label: 'Installation', slug: 'mcp/installation' },
            { label: 'Configuration', slug: 'mcp/configuration' },
            { label: 'Tools Reference', slug: 'mcp/tools' },
            { label: 'Usage Examples', slug: 'mcp/examples' },
            { label: 'AI Agent Guide', slug: 'mcp/ai-guide' },
          ],
        },
        {
          label: 'VS Code Extension',
          items: [{ label: 'Overview', slug: 'vscode/overview' }],
        },
        {
          label: 'Commands',
          items: [
            { label: 'init', slug: 'commands/init' },
            { label: 'search', slug: 'commands/search' },
            { label: 'rename', slug: 'commands/rename' },
            { label: 'replace', slug: 'commands/replace' },
            { label: 'plan', slug: 'commands/plan' },
            { label: 'apply', slug: 'commands/apply' },
            { label: 'undo', slug: 'commands/undo' },
            { label: 'redo', slug: 'commands/redo' },
            { label: 'status', slug: 'commands/status' },
            { label: 'history', slug: 'commands/history' },
          ],
        },
        {
          label: 'Examples',
          autogenerate: { directory: 'examples' },
        },
        {
          label: 'Case Studies',
          autogenerate: { directory: 'case-studies' },
        },
        {
          label: 'Reference',
          items: [
            { label: 'Configuration', slug: 'reference/configuration' },
            { label: 'Platform Support', slug: 'reference/platform-support' },
          ],
        },
      ],
    }),
  ],
});
