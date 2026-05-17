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
  'checklist',
  'train',
  'events',
])
const GATEWAY_IDS = new Set([
  'sections',
  'rules',
  'appliances',
  'checklist',
  'train',
  'events',
  'trmnl',
])
const WASM_BACKEND_IDS = new Set(
  (process.env.PORTAKI_WASM_BACKEND_MODULES ??
    [...BACKEND_IDS].join(','))
    .split(',')
    .map((s) => s.trim())
    .filter(Boolean),
)

for (const dir of fs.readdirSync(root)) {
  const manifestPath = path.join(root, dir, 'portaki.module.json')
  if (!fs.existsSync(manifestPath)) {
    continue
  }
  const raw = JSON.parse(fs.readFileSync(manifestPath, 'utf8'))
  const version = raw.version ?? '1.0.0'
  const npmPackage = raw.catalog?.npmPackage ?? `@portaki/module-${raw.id}`
  const hasBackend = BACKEND_IDS.has(raw.id)
  const wasmBackend = WASM_BACKEND_IDS.has(raw.id)

  raw.runtime = {
    backend: wasmBackend ? 'wasm' : hasBackend ? 'jar' : 'none',
    guest: 'remote-esm',
  }
  raw.artifacts = {
    guestEsmUrl: `https://esm.sh/${npmPackage}@${version}`,
    wasmUrl: wasmBackend ? `artifacts://${raw.id}/${version}.wasm` : '',
    jarMaven: raw.catalog?.javaArtifact ?? '',
  }

  fs.writeFileSync(manifestPath, JSON.stringify(raw, null, 2) + '\n')
  console.log('patched', raw.id, raw.runtime, hasBackend && GATEWAY_IDS.has(raw.id) ? '(gateway)' : '')
}
