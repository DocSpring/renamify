#!/bin/bash
# Convert ts-rs generated exports to ambient declarations

set -euo pipefail

for file in bindings/*.ts; do
  # Skip .d.ts files
  [[ "$file" == *.d.ts ]] && continue

  if [ -f "$file" ]; then
    # Remove import lines and replace 'export type' with 'declare type'
    sed -e '/^import type/d' -e 's/^export type/declare type/' "$file" > "${file%.ts}.d.ts"
    rm "$file"
  fi
done

pnpm run format-bindings

echo "Converted all TypeScript bindings to ambient .d.ts declarations"
