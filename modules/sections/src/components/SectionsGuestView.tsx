'use client'

import type { JSONContent } from '@tiptap/core'
import { generateHTML } from '@tiptap/html'
import StarterKit from '@tiptap/starter-kit'
import { usePortakiQuery } from '@portaki/sdk'
import type { LangCode } from '@portaki/module-sdk'
import { ModuleSection } from '@portaki/module-sdk'

type SectionRow = {
  id: string
  sortOrder: number
  titleFr: string
  titleEn: string
  contentFr: unknown
  contentEn: unknown
}

type ListResponse = {
  sections: SectionRow[]
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

export function SectionsGuestView({ lang }: { lang: LangCode }) {
  const { data, isLoading, isError } = usePortakiQuery<ListResponse>('sections.list', {})

  if (isLoading) {
    return (
      <ModuleSection>
        <p className="text-sm opacity-70">{lang === 'fr' ? 'Chargement…' : 'Loading…'}</p>
      </ModuleSection>
    )
  }

  if (isError || !data?.sections?.length) {
    return null
  }

  return (
    <div className="space-y-10">
      {data.sections.map((section) => {
        const title = lang === 'en' ? section.titleEn || section.titleFr : section.titleFr
        const html = contentToHtml(lang === 'en' ? section.contentEn : section.contentFr)
        return (
          <ModuleSection key={section.id} title={title}>
            {html ? (
              <div
                className="portaki-sections-prose text-[15px] leading-relaxed [&_h1]:text-2xl [&_h2]:text-xl [&_h3]:text-lg [&_p]:my-2 [&_ul]:my-2 [&_ol]:my-2"
                dangerouslySetInnerHTML={{ __html: html }}
              />
            ) : null}
          </ModuleSection>
        )
      })}
    </div>
  )
}
