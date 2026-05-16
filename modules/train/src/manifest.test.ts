import { describe, it } from 'vitest'
import { validateSiblingManifest } from '@portaki/module-test-support'

describe('portaki.module.json', () => {
  it('matches module.v1 schema', () => {
    validateSiblingManifest(import.meta.url)
  })
})
