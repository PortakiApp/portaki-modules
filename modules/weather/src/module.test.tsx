import { describe, expect, it } from 'vitest'
import { waitFor } from '@testing-library/react'
import { assertGuestSurface, renderGuestModule } from '@portaki/module-test-support'

import moduleDef from './index'

describe('@portaki/module-weather', () => {
  it('exposes a valid guest module definition', () => {
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
