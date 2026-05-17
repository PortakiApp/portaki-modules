#!/usr/bin/env node
/**
 * Replace local file: portaki-sdk deps with npm registry versions (CI-safe).
 */
import { existsSync, readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const modulesDir = join(root, 'modules')

const REGISTRY = {
  '@portaki/module-sdk': '^1.0.0',
  '@portaki/sdk': '^0.5.1',
}

function fixPackageJson(path) {
  const pkg = JSON.parse(readFileSync(path, 'utf8'))
  let changed = false

  for (const section of ['dependencies', 'devDependencies', 'optionalDependencies', 'peerDependencies']) {
    if (!pkg[section] || typeof pkg[section] !== 'object') {
      continue
    }
    for (const [name, value] of Object.entries(pkg[section])) {
      if (typeof value !== 'string' || !value.startsWith('file:') || !value.includes('portaki-sdk')) {
        continue
      }
      if (name === '@portaki/module-test-support') {
        delete pkg[section][name]
        changed = true
        continue
      }
      if (REGISTRY[name]) {
        pkg[section][name] = REGISTRY[name]
        changed = true
      }
    }
    if (Object.keys(pkg[section]).length === 0) {
      delete pkg[section]
      changed = true
    }
  }

  if (changed) {
    writeFileSync(path, `${JSON.stringify(pkg, null, 2)}\n`)
    console.log('fixed', path)
  }
}

for (const ent of readdirSync(modulesDir, { withFileTypes: true })) {
  if (!ent.isDirectory()) {
    continue
  }
  const base = join(modulesDir, ent.name)
  const main = join(base, 'package.json')
  if (existsSync(main)) {
    fixPackageJson(main)
  }
  const fe = join(base, 'frontend', 'package.json')
  if (existsSync(fe)) {
    fixPackageJson(fe)
  }
}
