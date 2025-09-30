#!/usr/bin/env node
/* eslint-disable no-console */
const { execSync } = require('child_process')
const fs = require('fs')
const path = require('path')

const repoRoot = path.resolve(__dirname, '..')
const docsDistRoot = path.join(repoRoot, 'docs', 'dist')

if (!fs.existsSync(docsDistRoot)) {
  console.error(
    'docs/dist not found. Please build the docs before validating links.'
  )
  process.exit(1)
}

function resolveDocPath(pathname) {
  if (!pathname.startsWith('/renamify')) return null

  const decoded = decodeURI(pathname)
  const withoutPrefix = decoded.replace(/^\/renamify/, '') || '/'

  if (withoutPrefix === '/' || withoutPrefix === '') {
    return path.join(docsDistRoot, 'index.html')
  }

  const trimmed = withoutPrefix.replace(/^\/+/, '')

  if (trimmed.endsWith('/')) {
    return path.join(docsDistRoot, trimmed, 'index.html')
  }

  const ext = path.extname(trimmed)
  if (ext) {
    if (ext === '.html') {
      return 'HTML_EXTENSION'
    }
    return path.join(docsDistRoot, trimmed)
  }

  const withIndex = path.join(docsDistRoot, trimmed, 'index.html')
  if (fs.existsSync(withIndex)) {
    return withIndex
  }

  // Fall back to direct path in case the site was built differently
  return path.join(docsDistRoot, trimmed)
}

function isBinaryFile(filePath) {
  const binaryExtensions = new Set([
    '.png',
    '.jpg',
    '.jpeg',
    '.gif',
    '.webp',
    '.ico',
    '.svg',
    '.mp4',
    '.mp3',
    '.pdf',
    '.zip',
    '.gz',
    '.bz2',
    '.7z',
    '.woff',
    '.woff2',
    '.ttf',
    '.eot',
    '.icns',
  ])

  const ext = path.extname(filePath).toLowerCase()
  return binaryExtensions.has(ext)
}

const trailingCharacters = new Set([
  '.',
  ',',
  ')',
  ';',
  '"',
  "'",
  ']',
  '>',
  '?',
])

function normalizeMatch(raw) {
  let value = raw
  while (value.length > 0 && trailingCharacters.has(value[value.length - 1])) {
    value = value.slice(0, -1)
  }
  return value
}

const htmlContentCache = new Map()
function loadHtmlContent(filePath) {
  if (!htmlContentCache.has(filePath)) {
    const content = fs.readFileSync(filePath, 'utf-8')
    htmlContentCache.set(filePath, content)
  }
  return htmlContentCache.get(filePath)
}

function anchorExists(filePath, anchor) {
  const decoded = decodeURIComponent(anchor)
  const content = loadHtmlContent(filePath)

  // Exact match against common id formats
  if (content.includes(`id="${decoded}"`)) return true
  if (content.includes(`id='${decoded}'`)) return true

  // Starlight also generates slugs using percent-encoding sometimes (e.g. spaces → %20)
  const encoded = encodeURIComponent(decoded)
  if (encoded !== decoded) {
    if (
      content.includes(`id="${encoded}"`) ||
      content.includes(`id='${encoded}'`)
    ) {
      return true
    }
  }

  return false
}

function collectFiles() {
  const output = execSync('git ls-files', { cwd: repoRoot, encoding: 'utf8' })
  return output
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
}

const ignoredPathPrefixes = []

const ignoredExactPaths = new Set([])

const ignoredLinks = new Set([
  // Base documentation URL - used in markdown link format [url](url)
  'https://docspring.github.io/renamify/',
  // MCP documentation URL
  'https://docspring.github.io/renamify/mcp/',
])

function shouldIgnoreFile(file) {
  if (ignoredExactPaths.has(file)) return true
  return ignoredPathPrefixes.some((prefix) => file.startsWith(prefix))
}

const files = collectFiles()
const errors = []
// Match URLs but stop at markdown link boundaries like ](
const linkRegex =
  /https?:\/\/docspring\.github\.io\/renamify\/[^\s"'<>\])]*/g

files.forEach((file) => {
  if (file.startsWith('docs/dist/')) {
    return
  }
  if (shouldIgnoreFile(file)) {
    return
  }
  if (isBinaryFile(file)) {
    return
  }

  const absolutePath = path.join(repoRoot, file)
  let content
  try {
    content = fs.readFileSync(absolutePath, 'utf-8')
  } catch (error) {
    return
  }

  const matches = content.matchAll(linkRegex)

  for (const match of matches) {
    const rawUrl = normalizeMatch(match[0])

    if (ignoredLinks.has(rawUrl)) {
      continue
    }
    const url = new URL(rawUrl)

    const pathname = url.pathname
    const hash = url.hash
    const [hashBase] = hash ? hash.split('?') : ['']

    const resolvedPath = resolveDocPath(pathname)

    if (resolvedPath === 'HTML_EXTENSION') {
      errors.push(
        `${file}: ${rawUrl} references a legacy .html path. Update this link to the new trailing-slash URL.`
      )
      continue
    }

    if (!resolvedPath || !fs.existsSync(resolvedPath)) {
      errors.push(
        `${file}: ${rawUrl} points to ${pathname}, but docs/dist does not contain the expected file.`
      )
      continue
    }

    if (hash) {
      const anchor = (hashBase || hash).slice(1)
      const ext = path.extname(resolvedPath).toLowerCase()
      if (ext === '.html' || resolvedPath.endsWith('/index.html')) {
        if (!anchorExists(resolvedPath, anchor)) {
          errors.push(
            `${file}: ${rawUrl} references #${anchor}, but that anchor was not found in ${path.relative(
              repoRoot,
              resolvedPath
            )}.`
          )
        }
      }
    }
  }
})

if (errors.length > 0) {
  console.error('Found invalid docspring.github.io/renamify/ links:')
  errors.forEach((error) => console.error(` - ${error}`))
  process.exit(1)
}

console.log('All docspring.github.io/renamify/ links are valid ✅')
