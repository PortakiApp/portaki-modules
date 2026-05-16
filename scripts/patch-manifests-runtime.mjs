import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'modules')
const BACKEND_IDS = new Set([
  'sections',
  'rules',
  'appliances',
  'ical-sync',
  'trmnl',
  'pre-arrival-form',
])
const GATEWAY_IDS = new Set(['sections', 'rules', 'appliances'])

for (const dir of fs.readdirSync(root)) {
  const manifestPath = path.join(root, dir, 'portaki.module.json')
  if (!fs.existsSync(manifestPath)) {
    continue
  }
  const raw = JSON.parse(fs.readFileSync(manifestPath, 'utf8'))
  const version = raw.version ?? '1.0.0'
  const npmPackage = raw.catalog?.npmPackage ?? `@portaki/module-${raw.id}`
  const hasBackend = BACKEND_IDS.has(raw.id)
  const hasGateway = GATEWAY_IDS.has(raw.id)

  raw.runtime = {
    backend: hasBackend ? 'jar' : 'none',
    guest: 'bundled',
  }
  raw.artifacts = {
    guestEsmUrl: `https://esm.sh/${npmPackage}@${version}`,
    wasmUrl: '',
    jarMaven: raw.catalog?.javaArtifact ?? '',
  }
  if (!hasGateway && !hasBackend) {
    raw.runtime.guest = 'remote-esm'
  }

  fs.writeFileSync(manifestPath, JSON.stringify(raw, null, 2) + '\n')
  console.log('patched', raw.id, raw.runtime)
}
