'use client'

import type { JSONContent } from '@tiptap/core'
import { generateHTML } from '@tiptap/html'
import StarterKit from '@tiptap/starter-kit'
import { usePortakiQuery } from '@portaki/sdk'
import type { LangCode } from '@portaki/module-sdk'
import { ModuleSection } from '@portaki/module-sdk'

type ContentResponse = {
  contentFr?: unknown
  contentEn?: unknown
}

const htmlExtensions = [StarterKit]

function contentToHtml(raw: unknown): string {
  if (raw == null) return ''
  try {
    const doc = (typeof raw === 'string' ? JSON.parse(raw) : raw) as JSONContent
    return generateHTML(doc, htmlExtensions)
  } catch {
    return ''
  }
}

export function RulesGuestView({ lang }: { lang: LangCode }) {
  const { data, isLoading, isError } = usePortakiQuery<ContentResponse>('rules.content', {})

  if (isLoading) {
    return (
      <ModuleSection title={lang === 'fr' ? 'Règlement' : 'House rules'}>
        <p className="text-sm opacity-70">{lang === 'fr' ? 'Chargement…' : 'Loading…'}</p>
      </ModuleSection>
    )
  }

  if (isError) {
    return null
  }

  const html = contentToHtml(lang === 'en' ? data?.contentEn : data?.contentFr)
  if (!html) {
    return null
  }

  return (
    <ModuleSection title={lang === 'fr' ? 'Règlement intérieur' : 'House rules'}>
      <div
        className="portaki-rules-prose text-[15px] leading-relaxed [&_h1]:text-2xl [&_h2]:text-xl [&_p]:my-2 [&_ul]:my-2"
        dangerouslySetInnerHTML={{ __html: html }}
      />
    </ModuleSection>
  )
}
