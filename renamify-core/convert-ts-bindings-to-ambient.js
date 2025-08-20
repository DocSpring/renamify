#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const { execSync } = require('node:child_process');

// Convert ts-rs generated exports to ambient declarations
const bindingsDir = path.join(__dirname, 'bindings');

// Read all .ts files in the bindings directory
const files = fs.readdirSync(bindingsDir).filter(file =>
  file.endsWith('.ts') && !file.endsWith('.d.ts')
);

for (const file of files) {
  const filePath = path.join(bindingsDir, file);
  const content = fs.readFileSync(filePath, 'utf8');

  // Remove import lines and replace 'export type' with 'declare type'
  const modifiedContent = content
    .split('\n')
    .filter(line => !line.startsWith('import type'))
    .map(line => line.replace(/^export type/, 'declare type'))
    .join('\n');

  // Write to .d.ts file
  const dtsPath = filePath.replace(/\.ts$/, '.d.ts');
  fs.writeFileSync(dtsPath, modifiedContent);

  // Remove original .ts file
  fs.unlinkSync(filePath);
}

// Format the bindings
try {
  execSync('pnpm run format-bindings', {
    stdio: 'inherit',
    cwd: __dirname
  });
} catch (error) {
  console.error('Failed to format bindings/*.d.ts. Please run pnpm install in ./renamify-core');
  process.exit(1);
}

console.log('Converted all TypeScript bindings to ambient .d.ts declarations');
