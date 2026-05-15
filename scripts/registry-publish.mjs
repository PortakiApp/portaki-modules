#!/usr/bin/env node
/**
 * Publication registry : compare la version locale à la dernière version publiée
 * et publie uniquement si la locale est strictement plus grande (semver).
 *
 * Usage :
 *   node scripts/registry-publish.mjs npm
 *   node scripts/registry-publish.mjs maven
 *
 * Env :
 *   REGISTRY_PUBLISH_DRY_RUN=1 — log seulement, aucun publish / deploy
 *   NPM_REGISTRY — défaut https://registry.npmjs.org
 */

import { execFileSync } from 'node:child_process'
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import semver from 'semver'

const __dirname = dirname(fileURLToPath(import.meta.url))
const ROOT = join(__dirname, '..')

const NPM_REGISTRY = process.env.NPM_REGISTRY ?? 'https://registry.npmjs.org'
const DRY = process.env.REGISTRY_PUBLISH_DRY_RUN === '1'

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'))
}

function* discoverModulePackageJsonPaths() {
  const modulesDir = join(ROOT, 'modules')
  if (!existsSync(modulesDir)) return
  for (const name of readdirSync(modulesDir)) {
    const base = join(modulesDir, name)
    if (!statSync(base).isDirectory()) continue
    const pj = join(base, 'package.json')
    if (existsSync(pj)) yield pj
  }
  const paf = join(ROOT, 'modules', 'pre-arrival-form', 'frontend', 'package.json')
  if (existsSync(paf)) yield paf
}

function npmLatestVersion(packageName) {
  try {
    const out = execFileSync(
      'npm',
      ['view', packageName, 'version', `--registry=${NPM_REGISTRY}`],
      { encoding: 'utf8', cwd: ROOT, stdio: ['ignore', 'pipe', 'pipe'] },
    ).trim()
    const v = semver.valid(semver.coerce(out))
    return v ?? '0.0.0'
  } catch {
    return '0.0.0'
  }
}

function shouldPublishLocalNewer(localRaw, remoteRaw) {
  const local = semver.valid(semver.coerce(localRaw))
  const remote = semver.valid(semver.coerce(remoteRaw)) ?? '0.0.0'
  if (!local) {
    console.warn(`[registry-publish] version locale invalide: ${localRaw}`)
    return false
  }
  return semver.gt(local, remote)
}

function runNpmPublish(filterArg) {
  const args = ['pnpm', 'publish', '--filter', filterArg, '--access', 'public', '--no-git-checks']
  if (DRY) {
    console.log(`[registry-publish:dry-run] ${args.join(' ')}`)
    return
  }
  execFileSync(args[0], args.slice(1), { stdio: 'inherit', cwd: ROOT })
}

function publishNpm() {
  let published = 0
  let skipped = 0
  for (const pjPath of discoverModulePackageJsonPaths()) {
    const meta = readJson(pjPath)
    const name = meta.name
    if (typeof name !== 'string' || !name.startsWith('@portaki/module-')) {
      continue
    }
    const local = meta.version
    const remote = npmLatestVersion(name)
    if (!shouldPublishLocalNewer(local, remote)) {
      console.log(`[registry-publish:npm] skip ${name}@${local} (registry: ${remote})`)
      skipped += 1
      continue
    }
    console.log(`[registry-publish:npm] publish ${name}@${local} (registry was ${remote})`)
    runNpmPublish(name)
    published += 1
  }
  console.log(`[registry-publish:npm] done published=${published} skipped=${skipped}`)
}

function mvnEval(javaDir, expression) {
  return execFileSync(
    'mvn',
    ['-B', '-ntp', '-q', '-DforceStdout', 'help:evaluate', `-Dexpression=${expression}`],
    { encoding: 'utf8', cwd: join(ROOT, javaDir), stdio: ['ignore', 'pipe', 'pipe'] },
  )
    .trim()
    .replace(/\r/g, '')
}

function mavenLatestRelease(groupId, artifactId) {
  const path = `${groupId.replaceAll('.', '/')}/${artifactId}/maven-metadata.xml`
  const url = `https://repo1.maven.org/maven2/${path}`
  try {
    const xml = execFileSync('curl', ['-fsSL', url], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    })
    const m = xml.match(/<latest>([^<]+)<\/latest>/)
    if (!m) return '0.0.0'
    const v = semver.valid(semver.coerce(m[1]))
    return v ?? '0.0.0'
  } catch {
    return '0.0.0'
  }
}

function publishMaven() {
  const out = execFileSync('find', ['modules', '-path', 'modules/*/backend/pom.xml', '-type', 'f'], {
    encoding: 'utf8',
    cwd: ROOT,
  })
  const poms = out
    .split('\n')
    .map((s) => s.trim())
    .filter(Boolean)
    .sort()
  let published = 0
  let skipped = 0
  for (const rel of poms) {
    const javaDir = dirname(rel)
    const version = mvnEval(javaDir, 'project.version')
    const groupId = mvnEval(javaDir, 'project.groupId')
    const artifactId = mvnEval(javaDir, 'project.artifactId')
    if (version.includes('SNAPSHOT')) {
      console.warn(`[registry-publish:maven] skip ${groupId}:${artifactId}:${version} (SNAPSHOT)`)
      skipped += 1
      continue
    }
    const remote = mavenLatestRelease(groupId, artifactId)
    if (!shouldPublishLocalNewer(version, remote)) {
      console.log(
        `[registry-publish:maven] skip ${groupId}:${artifactId}@${version} (Central latest: ${remote})`,
      )
      skipped += 1
      continue
    }
    console.log(
      `[registry-publish:maven] deploy ${groupId}:${artifactId}@${version} (Central was ${remote})`,
    )
    if (DRY) {
      console.log(`[registry-publish:dry-run] mvn deploy in ${javaDir}`)
      continue
    }
    execFileSync('mvn', ['--batch-mode', '-ntp', 'deploy', '-DskipTests', '-P', 'central-deploy'], {
      stdio: 'inherit',
      cwd: join(ROOT, javaDir),
    })
    published += 1
  }
  console.log(`[registry-publish:maven] done published=${published} skipped=${skipped}`)
}

const mode = process.argv[2]
if (mode === 'npm') {
  publishNpm()
} else if (mode === 'maven') {
  publishMaven()
} else {
  console.error('Usage: node scripts/registry-publish.mjs <npm|maven>')
  process.exit(2)
}
