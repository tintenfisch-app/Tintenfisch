import { createElement } from 'react'
import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it } from 'vitest'
import type { EditRight } from '@/types/protocol.ts'
import { RestrictionBanner } from '@/components/protection/RestrictionBanner.tsx'
import { useProtectionStore } from '@/stores/protectionStore.ts'

const permissions = { print: true, printFaithful: true, modify: false, copy: true, annotate: false, fillForms: false, assemble: true }

function shownFor(withheld: EditRight[]): string {
  useProtectionStore.setState({ current: { encrypted: true, ownerRights: false, permissions, withheld } })
  return renderToStaticMarkup(createElement(RestrictionBanner))
}

describe('RestrictionBanner', () => {
  it('lists what the document withholds and offers to unlock it with the permissions password', () => {
    const markup = shownFor(['fillForms', 'annotate'])
    expect(markup).toContain('Without its permissions password you cannot fill in forms, add or change comments and links.')
    expect(markup).toContain('aria-label="Permissions password"')
    expect(markup).toContain('Unlock')
  })

  it('stays away from documents without restrictions', () => {
    expect(shownFor([])).toBe('')
    useProtectionStore.setState({ current: null })
    expect(renderToStaticMarkup(createElement(RestrictionBanner))).toBe('')
  })
})
