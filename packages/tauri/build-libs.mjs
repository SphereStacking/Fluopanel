#!/usr/bin/env node
/**
 * Build shared libraries for widget runtime
 * Outputs to src-tauri/libs/ which is served via arcana://lib/
 */

import { build } from 'esbuild'
import { mkdir, copyFile } from 'fs/promises'
import { dirname, join } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const outDir = join(__dirname, 'src-tauri/libs')

async function main() {
  // Ensure output directory exists
  await mkdir(outDir, { recursive: true })

  console.log('Building shared libraries for widget runtime...\n')

  // 1. Bundle @arcana/providers
  console.log('  Building @arcana/providers...')
  await build({
    entryPoints: [join(__dirname, '../providers/dist/index.js')],
    bundle: true,
    format: 'esm',
    outfile: join(outDir, 'providers.js'),
    external: ['@tauri-apps/api', '@tauri-apps/api/*'],
    minify: true,
    sourcemap: false,
  })
  console.log('  -> libs/providers.js')

  // 2. Bundle @tauri-apps/api (core module)
  console.log('  Building @tauri-apps/api...')
  await build({
    entryPoints: ['@tauri-apps/api'],
    bundle: true,
    format: 'esm',
    outfile: join(outDir, 'tauri-api.js'),
    minify: true,
    sourcemap: false,
    platform: 'browser',
  })
  console.log('  -> libs/tauri-api.js')

  // 3. Copy Vue 3 ESM from node_modules
  console.log('  Copying Vue 3 ESM...')
  const vueSource = join(__dirname, '../../node_modules/vue/dist/vue.esm-browser.prod.js')
  await copyFile(vueSource, join(outDir, 'vue.esm.js'))
  console.log('  -> libs/vue.esm.js')

  console.log('\nDone! Libraries built to packages/tauri/src-tauri/libs/')
}

main().catch(err => {
  console.error(err)
  process.exit(1)
})
