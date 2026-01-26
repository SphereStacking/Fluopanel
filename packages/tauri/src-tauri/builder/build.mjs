#!/usr/bin/env node

/**
 * Arcana Widget Builder
 *
 * Builds Vue/React widgets from source files (.vue, .jsx, .tsx)
 * into a browser-ready HTML bundle.
 *
 * Usage:
 *   node build.mjs --widget /path/to/widget
 */

import { build } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve, dirname, basename } from 'path'
import { existsSync, writeFileSync, unlinkSync } from 'fs'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))

// Parse arguments
const widgetArg = process.argv.find(a => a.startsWith('--widget'))
let widgetDir = null

if (widgetArg) {
  if (widgetArg.includes('=')) {
    widgetDir = widgetArg.split('=')[1]
  } else {
    const idx = process.argv.indexOf('--widget')
    widgetDir = process.argv[idx + 1]
  }
}

if (!widgetDir) {
  console.error('Usage: node build.mjs --widget /path/to/widget')
  process.exit(1)
}

if (!existsSync(widgetDir)) {
  console.error(`Widget directory not found: ${widgetDir}`)
  process.exit(1)
}

// Find the main entry file
function findEntryFile(dir) {
  const candidates = ['App.vue', 'App.jsx', 'App.tsx', 'main.vue', 'main.jsx', 'main.tsx']
  for (const candidate of candidates) {
    const path = resolve(dir, candidate)
    if (existsSync(path)) {
      return { path, type: candidate.endsWith('.vue') ? 'vue' : 'react' }
    }
  }
  return null
}

const entry = findEntryFile(widgetDir)
if (!entry) {
  console.error('No entry file found (App.vue, App.jsx, or App.tsx)')
  process.exit(1)
}

console.log(`[Builder] Building widget: ${basename(widgetDir)}`)
console.log(`[Builder] Entry: ${entry.path}`)
console.log(`[Builder] Type: ${entry.type}`)

// Output directory
const outDir = resolve(widgetDir, '.arcana')

// Create a temporary entry point
const tempEntryPath = resolve(widgetDir, '.arcana-entry.js')
const isVue = entry.type === 'vue'

const entryContent = isVue
  ? `
import { createApp } from 'vue'
import App from './${basename(entry.path)}'

const app = createApp(App)
app.mount('#app')
`
  : `
import React from 'react'
import { createRoot } from 'react-dom/client'
import App from './${basename(entry.path)}'

const root = createRoot(document.getElementById('app'))
root.render(<App />)
`

writeFileSync(tempEntryPath, entryContent)

// External modules - these will be resolved via importmap at runtime
const externalModules = [
  'vue',
  '@arcana/providers',
  '@tauri-apps/api/core',
  '@tauri-apps/api/event',
]

// Vite configuration
const plugins = [vue()]

try {
  await build({
    root: widgetDir,
    plugins,
    build: {
      outDir: '.arcana',
      emptyOutDir: true,
      rollupOptions: {
        input: tempEntryPath,
        external: externalModules,
        output: {
          format: 'es',
          entryFileNames: 'bundle.js',
          chunkFileNames: '[name].js',
          assetFileNames: '[name].[ext]',
        },
      },
    },
    logLevel: 'warn',
    clearScreen: false,
  })

  // Create final index.html with the bundle
  const finalHtml = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Widget</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    html, body, #app { width: 100%; height: 100%; }
  </style>
</head>
<body>
  <div id="app"></div>
  <script type="module" src="./bundle.js"></script>
</body>
</html>`

  writeFileSync(resolve(outDir, 'index.html'), finalHtml)

  // Clean up temp file
  if (existsSync(tempEntryPath)) {
    unlinkSync(tempEntryPath)
  }

  console.log(`[Builder] Build complete: ${outDir}`)
} catch (error) {
  console.error('[Builder] Build failed:', error)

  // Clean up temp file on error
  if (existsSync(tempEntryPath)) {
    try { unlinkSync(tempEntryPath) } catch {}
  }

  process.exit(1)
}
