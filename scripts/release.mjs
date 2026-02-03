#!/usr/bin/env node

import { execSync } from 'child_process'
import { readFileSync, writeFileSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const rootDir = join(__dirname, '..')

// Read version from root package.json
const packageJson = JSON.parse(readFileSync(join(rootDir, 'package.json'), 'utf-8'))
const version = packageJson.version

console.log(`\n🚀 Releasing v${version}\n`)

// Sync version to tauri.conf.json
const tauriConfigPath = join(rootDir, 'packages/tauri/src-tauri/tauri.conf.json')
const tauriConfig = JSON.parse(readFileSync(tauriConfigPath, 'utf-8'))
tauriConfig.version = version
writeFileSync(tauriConfigPath, JSON.stringify(tauriConfig, null, 2) + '\n')
console.log(`✓ Updated tauri.conf.json to v${version}`)

// Check if there are changes to commit
const status = execSync('git status --porcelain', { cwd: rootDir, encoding: 'utf-8' })
if (status.trim()) {
  // Stage and commit
  execSync('git add -A', { cwd: rootDir, stdio: 'inherit' })
  execSync(`git commit -m "chore: release v${version}"`, { cwd: rootDir, stdio: 'inherit' })
  console.log(`✓ Committed changes`)
}

// Create tag
const tag = `v${version}`
try {
  execSync(`git tag ${tag}`, { cwd: rootDir, stdio: 'inherit' })
  console.log(`✓ Created tag ${tag}`)
} catch (e) {
  console.error(`✗ Tag ${tag} already exists`)
  process.exit(1)
}

// Push to origin
execSync('git push origin main --tags', { cwd: rootDir, stdio: 'inherit' })
console.log(`✓ Pushed to origin`)

console.log(`\n✅ Released ${tag}!`)
console.log(`   GitHub Actions will build and publish the release.\n`)
