import type { ModuleContext } from '@portaki/module-sdk'
import { definePortakiModule } from '@portaki/module-sdk'

import { SectionsGuestView } from './components/SectionsGuestView'

export default definePortakiModule({
  id: 'sections',
  label: { fr: 'Sections', en: 'Sections' },
  description: {
    fr: 'Contenus éditoriaux du carnet d’accueil.',
    en: 'Welcome book editorial content.',
  },
  version: '1.0.0',
  icon: 'file-text',
  navSlot: 'section',
  render: ({ lang }: ModuleContext) => <SectionsGuestView lang={lang} />,
})
