'use client'

import type { HostModuleContext } from '@portaki/module-sdk'
import { hostModule } from '@portaki/module-sdk'

function TrmnlHostBody({ ctx }: { ctx: HostModuleContext }) {
  const mode = String(ctx.config.display_mode ?? 'host_dashboard')
  const fr = ctx.lang === 'fr'
  return (
    <section data-module="trmnl" className="space-y-3 text-sm">
      <h2 className="text-base font-semibold">TRMNL</h2>
      <p className="leading-relaxed opacity-90">
        {fr
          ? 'Collez l’URL webhook de votre Private Plugin TRMNL (Developer Edition) dans la configuration du module. Les templates Liquid sont dans ce dépôt : modules/trmnl/templates/.'
          : 'Paste your TRMNL Private Plugin webhook URL (Developer Edition) in the module settings. Liquid templates are in this repo under modules/trmnl/templates/.'}
      </p>
      <p className="text-[12px] opacity-70">
        {fr ? 'Mode d’affichage : ' : 'Display mode: '}
        <code className="rounded bg-black/5 px-1 py-0.5 dark:bg-white/10">{mode}</code>
      </p>
    </section>
  )
}

export default hostModule('trmnl')
  .label('TRMNL ePaper', 'TRMNL ePaper')
  .description(
    'Affichage séjour sur écran e-ink TRMNL (webhook).',
    'Stay information on a TRMNL e-ink screen (webhook).',
  )
  .icon('monitor')
  .version('1.0.0')
  .navSlot('section')
  .hostRender((ctx) => <TrmnlHostBody ctx={ctx} />)
  .build()
