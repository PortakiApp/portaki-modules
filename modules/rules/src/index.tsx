import type { ModuleContext } from '@portaki/module-sdk'
import { definePortakiModule } from '@portaki/module-sdk'

import { RulesGuestView } from './components/RulesGuestView'

export default definePortakiModule({
  id: 'rules',
  label: { fr: 'Règlement', en: 'House rules' },
  description: {
    fr: 'Consignes du logement.',
    en: 'House rules.',
  },
  version: '1.0.0',
  icon: 'scale',
  navSlot: 'section',
  render: ({ lang }: ModuleContext) => <RulesGuestView lang={lang} />,
})
