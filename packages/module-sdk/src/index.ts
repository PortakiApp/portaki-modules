import type { ReactNode } from 'react'

export type Lang = 'fr' | 'en'

export interface PortakiProperty {
  id: string
  trainStationCode?: string
  checklistItems?: ReadonlyArray<{
    id: string
    labelFr: string
    labelEn: string
  }>
}

export interface PortakiStay {
  id: string
}

export interface ModuleRenderContext {
  lang: Lang
  property: PortakiProperty
  stay?: PortakiStay
}

export interface PortakiModuleDefinition {
  id: string
  label: Record<'fr' | 'en', string>
  icon: string
  navSlot: 'section'
  visibleOnStatus?: string[]
  mapOverlay?: boolean
  mapMarkers?: (ctx: ModuleRenderContext) => Promise<unknown[]>
  render: (ctx: ModuleRenderContext) => ReactNode
}

export function definePortakiModule(definition: PortakiModuleDefinition): PortakiModuleDefinition {
  return definition
}
