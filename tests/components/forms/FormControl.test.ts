import { createElement } from 'react'
import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it } from 'vitest'
import type { FormField } from '@/types/protocol.ts'
import { FormControl } from '@/components/forms/FormControl.tsx'
import { useProtectionStore } from '@/stores/protectionStore.ts'

const widget = { id: '11-0', pageId: '4-0', rect: { x: 0, y: 0, width: 50, height: 20 }, onState: null }

const field = (overrides: Partial<FormField>): FormField => ({
  id: '11-0',
  name: 'code',
  partialName: 'code',
  kind: 'text',
  multiline: false,
  password: false,
  maxLength: 5,
  editable: false,
  readOnly: false,
  required: false,
  text: '',
  on: null,
  chosen: [],
  options: [],
  widgets: [widget],
  ...overrides,
})

const markup = (overrides: Partial<FormField>) => renderToStaticMarkup(createElement(FormControl, { field: field(overrides), widget, cssScale: 1 }))

describe('FormControl', () => {
  it('limits single and multiline text to the length the field allows', () => {
    expect(markup({})).toContain('maxLength="5"')
    expect(markup({ multiline: true })).toMatch(/<textarea[^>]*maxLength="5"/)
    expect(markup({ maxLength: null })).not.toContain('maxLength')
  })

  it('locks the field while the document withholds form filling', () => {
    expect(markup({})).not.toContain('disabled=""')
    const permissions = { print: true, printFaithful: true, modify: false, copy: true, annotate: false, fillForms: false, assemble: false }
    useProtectionStore.setState({ current: { encrypted: true, ownerRights: false, permissions, withheld: ['fillForms'] } })
    expect(markup({})).toContain('disabled=""')
  })
})
