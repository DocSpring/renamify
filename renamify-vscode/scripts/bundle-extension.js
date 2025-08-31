#!/usr/bin/env node

const esbuild = require('esbuild');

async function bundle() {
  try {
    await esbuild.build({
      entryPoints: ['extension/src/extension.ts'],
      bundle: true,
      outfile: 'dist/extension.js',
      external: ['vscode'],
      format: 'cjs',
      platform: 'node',
      sourcemap: true,
      target: 'node18',
      logLevel: 'info',
    });

    console.log('Extension bundled successfully');
  } catch (error) {
    console.error('Bundle failed:', error);
    process.exit(1);
  }
}

bundle();
