import { describe, expect, it } from 'vitest'
import { certificateWarning } from '@/lib/certificateWarning.ts'

const certificate = { signer: 'Ada Lovelace', validFrom: Date.UTC(2026, 9, 18), validUntil: Date.UTC(2126, 8, 24), validity: 'current' } as const

describe('certificateWarning', () => {
  it('stays quiet for a current certificate and names the date otherwise', () => {
    expect(certificateWarning(certificate)).toBeNull()
    expect(certificateWarning({ ...certificate, validity: 'expired' })).toBe(
      'The certificate of Ada Lovelace expired on 2126-09-24. Signatures made with it show as expired.',
    )
    expect(certificateWarning({ ...certificate, validity: 'notYetValid' })).toBe(
      'The certificate of Ada Lovelace is only valid from 2026-10-18. Signatures made with it show as not yet valid.',
    )
  })
})
