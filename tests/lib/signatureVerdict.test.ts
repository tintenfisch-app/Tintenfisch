import { describe, expect, it } from 'vitest'
import type { SignatureInfo } from '@/types/protocol.ts'
import { signatureVerdict } from '@/lib/signatureVerdict.ts'

const intact: SignatureInfo = {
  fieldId: '70-0',
  fieldName: 'Signature',
  signer: 'Ada Lovelace',
  signedAt: 'D:20261018120000Z',
  reason: '',
  status: 'intact',
  problem: null,
  identity: 'unverified',
  identityProblem: 'the certificate is self-signed, so nobody vouches for the signer\'s name',
  changedAfter: false,
}

describe('signatureVerdict', () => {
  it('never shows an intact signature as plainly valid while the signer is not verified', () => {
    expect(signatureVerdict(intact)).toEqual({
      label: 'Signature intact, signer identity not verified',
      tone: 'caution',
      detail: 'the certificate is self-signed, so nobody vouches for the signer\'s name',
    })
  })

  it('names expired and not yet valid certificates and later changes', () => {
    expect(signatureVerdict({ ...intact, identity: 'expired' }).label).toBe('Signature intact, certificate expired')
    expect(signatureVerdict({ ...intact, identity: 'notYetValid' }).label).toBe('Signature intact, certificate not yet valid')
    expect(signatureVerdict({ ...intact, identity: 'expired', changedAfter: true }).label).toBe(
      'Signature intact, certificate expired, document changed since signing',
    )
  })

  it('reports broken and unchecked signatures with their problem', () => {
    const broken = { ...intact, status: 'broken', problem: 'the signed bytes have changed', identity: null, identityProblem: null } as const
    expect(signatureVerdict(broken)).toEqual({ label: 'Signature broken: the signed bytes have changed', tone: 'negative', detail: null })
    expect(signatureVerdict({ ...broken, status: 'unsupported', problem: null })).toMatchObject({
      label: 'Signature not checked: unknown reason',
      tone: 'muted',
    })
  })
})
