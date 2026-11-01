import { describe, expect, it } from 'vitest'
import type { Permissions, Request } from '@/types/protocol.ts'
import { useProtectionStore } from '@/stores/protectionStore.ts'
import { FakeEnginePort } from '@tests/fakes/FakeEnginePort.ts'

const everything: Permissions = { print: true, printFaithful: true, modify: true, copy: true, annotate: true, fillForms: true, assemble: true }
const locked = { encrypted: true, ownerRights: false, permissions: { ...everything, copy: false }, withheld: [] }
const restricted = { encrypted: true, ownerRights: false, permissions: { ...everything, fillForms: false, annotate: false }, withheld: ['fillForms', 'annotate', 'designForms'] }

function createStore() {
  const port = FakeEnginePort.attach()
  const store = useProtectionStore.getState()
  return { port, store }
}

const last = (port: FakeEnginePort): Request => port.requests[port.requests.length - 1]

describe('protectionStore', () => {
  it('asks for the permissions password before the protection can change', async () => {
    const { port, store } = createStore()
    const opening = store.openDialog()
    await port.answerNext(locked)
    await opening
    expect(useProtectionStore.getState()).toMatchObject({ dialogOpen: true, current: locked })
    const wrong = store.grantOwnerRights('guess')
    expect(last(port)).toMatchObject({ kind: 'grantOwnerRights', password: 'guess' })
    await port.answerNext(false)
    await port.answerNext(locked)
    await wrong
    expect(useProtectionStore.getState().notice).toBe('That is not the permissions password')
    const right = store.grantOwnerRights('boss')
    await port.answerNext(true)
    await port.answerNext({ ...locked, ownerRights: true })
    await right
    expect(useProtectionStore.getState()).toMatchObject({ notice: null, current: { ownerRights: true } })
  })

  it('reads the restrictions and lifts them once the permissions password is given', async () => {
    const { port, store } = createStore()
    expect(store.permits('fillForms')).toBe(true)
    const loading = store.reload()
    expect(last(port).kind).toBe('protection')
    await port.answerNext(restricted)
    await loading
    expect(useProtectionStore.getState().permits('fillForms')).toBe(false)
    expect(useProtectionStore.getState().permits('assemble')).toBe(true)
    const unlocking = store.grantOwnerRights('boss')
    await port.answerNext(true)
    await port.answerNext({ ...restricted, ownerRights: true, withheld: [] })
    await unlocking
    expect(useProtectionStore.getState().permits('fillForms')).toBe(true)
  })

  it('protects with fresh random bytes and says the protection comes with the next save', async () => {
    const { port, store } = createStore()
    const protecting = store.protect('open', 'boss', { ...everything, modify: false })
    const request = last(port) as Extract<Request, { kind: 'protect' }>
    expect(request).toMatchObject({ kind: 'protect', userPassword: 'open', ownerPassword: 'boss', permissions: { modify: false } })
    expect(request.entropy).toHaveLength(68)
    await port.answerNext(undefined)
    await protecting
    expect(useProtectionStore.getState()).toMatchObject({ dialogOpen: false, pending: 'The protection is written when you save the document' })
    const removing = store.unprotect()
    expect(last(port).kind).toBe('unprotect')
    await port.answerNext(undefined)
    await removing
    expect(useProtectionStore.getState().pending).toBe('The document is saved without protection from now on')
  })
})
