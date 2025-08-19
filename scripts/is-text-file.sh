#!/bin/bash
# Shared script to determine if a file should be treated as text
# Usage: is-text-file.sh <filename>
# Returns: 0 if text file, 1 if not

file="$1"

# Check by extension for common text files
case "$file" in
  *.txt|*.md|*.markdown|*.rst|*.yml|*.yaml|*.json|*.toml|*.xml|*.html|*.htm|\
  *.css|*.scss|*.sass|*.less|*.js|*.jsx|*.ts|*.tsx|*.mjs|*.cjs|*.vue|*.svelte|\
  *.py|*.rb|*.go|*.rs|*.c|*.cpp|*.h|*.hpp|*.java|*.kt|*.swift|*.m|*.php|\
  *.sh|*.bash|*.zsh|*.fish|*.ps1|*.bat|*.cmd|Makefile|Dockerfile|*.dockerfile|\
  *.gitignore|*.dockerignore|*.editorconfig|*.env|*.env.*|*.conf|*.ini|*.cfg|\
  *.sql|*.graphql|*.proto|*.thrift|*.avsc)
    exit 0
    ;;
  *)
    # For other files, use the file command as fallback
    if file "$file" | grep -q "text\|ASCII\|UTF"; then
      exit 0
    else
      exit 1
    fi
    ;;
esac
