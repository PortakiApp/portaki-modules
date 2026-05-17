#!/usr/bin/env node
/**
 * Idempotent scaffold: Vitest + @portaki/module-test-support per module package.
 * Run from portaki-modules/: node scripts/scaffold-module-unit-tests.mjs
 */
import { existsSync, readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const modulesDir = join(root, 'modules')
const testSupportVersion = '^1.0.0'

const vitestConfig = `import { portakiModuleVitestConfig } from '@portaki/module-test-support/vitest'

export default portakiModuleVitestConfig(import.meta.url)
`

const manifestTest = `import { describe, it } from 'vitest'
import { validateSiblingManifest } from '@portaki/module-test-support'

describe('portaki.module.json', () => {
  it('matches module.v1 schema', () => {
    validateSiblingManifest(import.meta.url)
  })
})
`

const moduleTestTemplate = (id) => `import { describe, expect, it } from 'vitest'
import { waitFor } from '@testing-library/react'
import {
  assertGuestSurface,
  assertHostSurface,
  renderGuestModule,
  renderHostModule,
} from '@portaki/module-test-support'

import moduleDef from './index'

describe('@portaki/module-${id}', () => {
  it('exposes a valid module definition', () => {
    if (moduleDef.surface === 'host') {
      assertHostSurface(moduleDef)
      return
    }
    assertGuestSurface(moduleDef)
  })

  it('renders without crashing', async () => {
    const view =
      moduleDef.surface === 'host' ? renderHostModule(moduleDef) : renderGuestModule(moduleDef)
    await waitFor(() => {
      expect(view.container).toBeTruthy()
    })
    view.unmount()
  })
})
`

function patchPackageJson(dir, id) {
  const pkgPath = join(dir, 'package.json')
  const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'))
  pkg.scripts = pkg.scripts ?? {}
  pkg.scripts.test = 'vitest run'
  pkg.scripts['test:watch'] = 'vitest'
  pkg.dependencies = pkg.dependencies ?? {}
  pkg.dependencies = pkg.dependencies ?? {}
  if (
    !pkg.dependencies['@portaki/module-sdk'] ||
    String(pkg.dependencies['@portaki/module-sdk']).startsWith('file:')
  ) {
    pkg.dependencies['@portaki/module-sdk'] = '^1.0.0'
  }
  if (!pkg.dependencies['@portaki/sdk']) {
    pkg.dependencies['@portaki/sdk'] = '^0.5.1'
  }
  pkg.devDependencies = pkg.devDependencies ?? {}
  pkg.devDependencies['@portaki/module-test-support'] = testSupportVersion
  pkg.devDependencies.vitest = '^3.0.5'
  if (existsSync(join(dir, 'src', 'index.tsx')) || existsSync(join(dir, 'src', 'index.ts'))) {
    pkg.devDependencies['@testing-library/react'] = '^16.3.0'
    pkg.devDependencies['@testing-library/jest-dom'] = '^6.6.3'
    pkg.devDependencies.jsdom = '^26.0.0'
  }
  writeFileSync(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`)
}

function scaffoldModule(id) {
  const dir = join(modulesDir, id)
  if (!existsSync(join(dir, 'portaki.module.json'))) {
    return
  }
  if (!existsSync(join(dir, 'package.json'))) {
    return
  }

  patchPackageJson(dir, id)

  const vitestPath = join(dir, 'vitest.config.ts')
  if (!existsSync(vitestPath)) {
    writeFileSync(vitestPath, vitestConfig)
  }

  const srcDir = join(dir, 'src')
  if (!existsSync(srcDir)) {
    return
  }

  const manifestTestPath = join(srcDir, 'manifest.test.ts')
  if (!existsSync(manifestTestPath)) {
    writeFileSync(manifestTestPath, manifestTest)
  }

  const entryTsx = join(srcDir, 'index.tsx')
  const entryTs = join(srcDir, 'index.ts')
  const hasEntry = existsSync(entryTsx) || existsSync(entryTs)
  const moduleTestPath = join(srcDir, 'module.test.tsx')
  if (hasEntry && !existsSync(moduleTestPath)) {
    writeFileSync(moduleTestPath, moduleTestTemplate(id))
  }
}

const ids = readdirSync(modulesDir, { withFileTypes: true })
  .filter((d) => d.isDirectory())
  .map((d) => d.name)

for (const id of ids) {
  scaffoldModule(id)
  console.log(`scaffolded tests: ${id}`)
}

console.log('done — run: pnpm install && pnpm -r test')
