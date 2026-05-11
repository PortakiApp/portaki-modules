/**
 * GitHub Packages impose @<owner_github_minuscule>/<nom>.
 * Réécrit les champs `name` et dépendances scoppées dans les package.json du workspace.
 */
import fs from 'node:fs'
import path from 'node:path'

const owner = process.env.OWNER_LOWER?.toLowerCase()
if (!owner) {
  throw new Error('OWNER_LOWER est requis')
}

const roots = [
  'packages/module-sdk',
  'train',
  'events',
  'rules',
  'appliances',
  'checklist',
  'pre-arrival-form/frontend',
]

const LEGACY_SCOPE = '@portaki/'

function rewriteScopedKeys(obj) {
  if (!obj || typeof obj !== 'object') {
    return
  }
  for (const field of ['dependencies', 'devDependencies', 'peerDependencies', 'optionalDependencies']) {
    const deps = obj[field]
    if (!deps) {
      continue
    }
    const next = {}
    for (const [k, v] of Object.entries(deps)) {
      const nk = k.startsWith(LEGACY_SCOPE) ? `@${owner}/${k.slice(LEGACY_SCOPE.length)}` : k
      next[nk] = v
    }
    obj[field] = next
  }
}

for (const root of roots) {
  const p = path.join(root, 'package.json')
  const j = JSON.parse(fs.readFileSync(p, 'utf8'))
  if (j.name?.startsWith(LEGACY_SCOPE)) {
    j.name = `@${owner}/${j.name.slice(LEGACY_SCOPE.length)}`
  }
  rewriteScopedKeys(j)
  fs.writeFileSync(p, `${JSON.stringify(j, null, 2)}\n`)
}

console.log(`Scopes npm alignés sur @${owner}`)
