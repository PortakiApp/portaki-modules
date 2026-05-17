import { describe, expect, it } from 'vitest'
import { waitFor } from '@testing-library/react'
import { assertHostSurface, renderHostModule } from '@portaki/module-test-support'

import moduleDef from './index'

describe('@portaki/module-ical-sync', () => {
  it('exposes a valid host module definition', () => {
    assertHostSurface(moduleDef)
  })

  it('renders host surface without crashing', async () => {
    const view = renderHostModule(moduleDef)
    await waitFor(() => {
      expect(view.container).toBeTruthy()
    })
    view.unmount()
  })
})
