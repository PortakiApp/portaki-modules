/**
 * Bump la version dans chaque package publishable.
 * Usage:
 *   node scripts/bump-workspace-versions.mjs prerelease develop.42
 *   node scripts/bump-workspace-versions.mjs set 0.2.0
 */
import fs from 'node:fs'
import path from 'node:path'

const roots = [
  'modules/train',
  'modules/events',
  'modules/rules',
  'modules/appliances',
  'modules/checklist',
  'modules/pre-arrival-form/frontend',
]

const [mode, arg] = process.argv.slice(2)

function baseVersion(current) {
  return current.replace(/[-+].*$/, '')
}

function writeVersion(root, version) {
  const p = path.join(root, 'package.json')
  const j = JSON.parse(fs.readFileSync(p, 'utf8'))
  j.version = version
  fs.writeFileSync(p, `${JSON.stringify(j, null, 2)}\n`)
}

if (mode === 'prerelease') {
  if (!arg) {
    throw new Error('usage: prerelease <suffixe> ex. develop.42')
  }
  for (const root of roots) {
    const p = path.join(root, 'package.json')
    const j = JSON.parse(fs.readFileSync(p, 'utf8'))
    const base = baseVersion(j.version)
    writeVersion(root, `${base}-${arg}`)
  }
} else if (mode === 'set') {
  if (!arg) {
    throw new Error('usage: set <semver>')
  }
  for (const root of roots) {
    writeVersion(root, arg)
  }
} else {
  console.error('Usage: prerelease <suffixe> | set <semver>')
  process.exit(1)
}
