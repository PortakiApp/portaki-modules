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

const ghcrOwner = (process.env.PORTAKI_GHCR_OWNER ?? 'portakiapp').toLowerCase()
/** oci (GHCR default) | artifacts (local volume) | https (CDN base URL) */
const artifactsScheme = process.env.PORTAKI_ARTIFACTS_SCHEME ?? 'oci'

function resolveWasmUrl(moduleId, version, wasmBackend) {
  if (!wasmBackend) {
    return ''
  }
  if (artifactsScheme === 'https') {
    const wasmCdnBase = (process.env.PORTAKI_WASM_CDN_BASE_URL ?? '').replace(/\/$/, '')
    return wasmCdnBase
      ? `${wasmCdnBase}/${moduleId}/${version}.wasm`
      : `artifacts://${moduleId}/${version}.wasm`
  }
  if (artifactsScheme === 'artifacts') {
    return `artifacts://${moduleId}/${version}.wasm`
  }
  return `oci://ghcr.io/${ghcrOwner}/portaki-module-${moduleId}:${version}`
}

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
  const wasmUrl = resolveWasmUrl(raw.id, version, wasmBackend)
  raw.artifacts = {
    guestEsmUrl: `https://esm.sh/${npmPackage}@${version}`,
    wasmUrl,
    jarMaven: raw.catalog?.javaArtifact ?? '',
  }

  fs.writeFileSync(manifestPath, JSON.stringify(raw, null, 2) + '\n')
  console.log('patched', raw.id, raw.runtime, hasBackend && GATEWAY_IDS.has(raw.id) ? '(gateway)' : '')
}
