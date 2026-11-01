import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { SignatureInfo } from '@/types/protocol.ts'
import { signedName, useDigitalSignatureStore } from '@/stores/digitalSignatureStore.ts'
import { useDocumentStore } from '@/stores/documentStore.ts'
import { FakeEnginePort } from '@tests/fakes/FakeEnginePort.ts'

const openedResult = { status: 'opened', version: '1.7', byteLength: 4, objectCount: 9, pageCount: 1, repaired: false, xfa: 'absent' }
const pages = [{ id: '3-0', width: 612, height: 792, rotate: 0 }]
const request = { fieldId: '70-0', certificate: new Uint8Array([48, 130]), password: 'secret', reason: 'Approved', location: '' }
const now = Date.UTC(2026, 9, 18, 12, 0, 0)
const current = { signer: 'Ada Lovelace', validFrom: Date.UTC(2026, 0, 1), validUntil: Date.UTC(2027, 0, 1), validity: 'current' }
const expired = { ...current, validFrom: Date.UTC(2019, 0, 1), validUntil: Date.UTC(2020, 0, 1), validity: 'expired' }
const report: SignatureInfo = {
  fieldId: '70-0',
  fieldName: 'Signature',
  signer: 'Ada Lovelace',
  signedAt: 'D:20261018120000Z',
  reason: 'Approved',
  status: 'intact',
  problem: null,
  identity: 'expired',
  identityProblem: 'the certificate was valid until 2020-01-01',
  changedAfter: false,
}

function settle() {
  return new Promise((resolve) => setTimeout(resolve, 0))
}

async function createStore() {
  const port = FakeEnginePort.attach()
  const documents = useDocumentStore.getState()
  const opening = documents.open('contract.pdf', new Uint8Array(4))
  await port.answerNext(openedResult)
  await port.answerNext(pages)
  await opening
  const store = useDigitalSignatureStore.getState()
  return { port, documents, store }
}

beforeEach(() => {
  vi.useFakeTimers({ toFake: ['Date'] })
  vi.setSystemTime(new Date(now))
})

describe('digitalSignatureStore', () => {
  it('checks the certificate, signs with the date of now, reopens the signed file under a signed name and hands it back', async () => {
    const { port, store } = await createStore()
    store.openDialog()
    const signing = store.sign(request)
    expect(useDigitalSignatureStore.getState().signing).toBe(true)
    expect(port.requests.at(-1)).toMatchObject({ kind: 'inspectCertificate', certificate: request.certificate, password: 'secret', now })
    await port.answerNext(current)
    await settle()
    expect(port.requests.at(-1)).toMatchObject({ kind: 'signField', ...request, date: 'D:20261018120000Z' })
    await port.answerNext(new Uint8Array([37, 80, 68, 70]))
    expect(port.requests.at(-1)).toMatchObject({ kind: 'open' })
    await port.answerNext(openedResult)
    await port.answerNext(pages)
    const signed = await signing
    expect(signed?.name).toBe('contract-signed.pdf')
    expect(useDocumentStore.getState().document).toMatchObject({ phase: 'ready', name: 'contract-signed.pdf', session: 2 })
    expect(useDigitalSignatureStore.getState()).toMatchObject({ dialogOpen: false, signing: false, warning: null })
    expect(signedName('contract-signed.pdf')).toBe('contract-signed.pdf')
  })

  it('keeps the dialog open with a readable reason when the certificate file is refused, and lists checked signatures', async () => {
    const { port, store } = await createStore()
    store.openDialog()
    for (const [error, notice] of [
      ['this file is not a valid certificate (.p12 / .pfx)', 'This file is not a valid certificate (.p12 / .pfx).'],
      ['the password does not open the certificate file', 'The password does not open the certificate file.'],
    ]) {
      const signing = store.sign(request)
      await Promise.resolve()
      port.answer({ id: port.lastId, ok: false, error })
      expect(await signing).toBeNull()
      expect(useDigitalSignatureStore.getState()).toMatchObject({ dialogOpen: true, signing: false, notice })
    }
    const loading = store.reload()
    expect(port.requests.at(-1)).toMatchObject({ kind: 'signatures', now })
    await port.answerNext([report])
    await loading
    expect(useDigitalSignatureStore.getState().reports).toEqual([report])
  })

  it('warns about an expired certificate and signs only once the warning is confirmed', async () => {
    const { port, store } = await createStore()
    store.openDialog()
    const warned = store.sign(request)
    await port.answerNext(expired)
    expect(await warned).toBeNull()
    expect(port.requests.at(-1)?.kind).toBe('inspectCertificate')
    expect(useDigitalSignatureStore.getState()).toMatchObject({
      dialogOpen: true,
      signing: false,
      warning: 'The certificate of Ada Lovelace expired on 2020-01-01. Signatures made with it show as expired.',
    })
    store.dismissWarning()
    expect(useDigitalSignatureStore.getState().warning).toBeNull()
    const signing = store.sign(request, true)
    expect(port.requests.at(-1)).toMatchObject({ kind: 'signField', ...request })
    await port.answerNext(new Uint8Array([37, 80, 68, 70]))
    await port.answerNext(openedResult)
    await port.answerNext(pages)
    expect((await signing)?.name).toBe('contract-signed.pdf')
  })
})
