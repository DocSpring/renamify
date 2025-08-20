#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');

const amdLoader = fs.readFileSync(
  path.join('webview', 'amd-loader.js'),
  'utf8'
);
const webview = fs.readFileSync(path.join('media', 'webview.js'), 'utf8');

fs.writeFileSync(path.join('media', 'bundle.js'), amdLoader + webview);
