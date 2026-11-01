import { createElement } from 'react'
import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it } from 'vitest'
import type { XfaContent } from '@/types/protocol.ts'
import { XfaNotice } from '@/components/forms/XfaNotice.tsx'
import { useDocumentStore } from '@/stores/documentStore.ts'

function shownFor(xfa: XfaContent): string {
  useDocumentStore.setState({
    document: { phase: 'ready', name: 'form.pdf', session: 1, pages: [], version: '1.7', byteLength: 1, objectCount: 1, pageCount: 1, repaired: false, xfa },
  })
  return renderToStaticMarkup(createElement(XfaNotice))
}

describe('XfaNotice', () => {
  it('warns about XFA content only when the form carries some', () => {
    expect(shownFor('absent')).toBe('')
    expect(shownFor('hybrid')).toContain('This form contains XFA content, which Tintenfisch does not support. The XFA part may be lost when you save.')
    expect(shownFor('dynamic')).toContain('dynamic XFA form')
  })
})
