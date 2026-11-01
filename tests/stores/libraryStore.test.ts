import { beforeEach, describe, expect, it, vi } from 'vitest'
import { documentLibrary } from '@/services/DocumentLibrary.ts'
import { useAutoSaveStore } from '@/stores/autoSaveStore.ts'
import { useDocumentStore } from '@/stores/documentStore.ts'
import { useLibraryStore } from '@/stores/libraryStore.ts'
import { useToastStore } from '@/stores/toastStore.ts'
import { FakeEnginePort } from '@tests/fakes/FakeEnginePort.ts'

const opened = { status: 'opened', version: '1.7', byteLength: 1, objectCount: 3, pageCount: 1, repaired: false, xfa: 'absent' }
const pdf = new TextEncoder().encode('%PDF-1.7\n')
const pages = [{ id: '3-0', width: 612, height: 792, rotate: 0 }]

let port: FakeEnginePort

beforeEach(async () => {
  port = FakeEnginePort.attach()
  for (const file of await documentLibrary.list()) await documentLibrary.remove(file.id)
})

const library = () => useLibraryStore.getState()

async function answerOpening() {
  await vi.waitFor(() => expect(port.requests.at(-1)?.kind).toBe('open'))
  await port.answerNext(opened)
  await vi.waitFor(() => expect(port.requests.at(-1)?.kind).toBe('pages'))
  await port.answerNext(pages)
  await vi.waitFor(() => expect(useDocumentStore.getState().document.phase).toBe('ready'))
}

describe('libraryStore', () => {
  it('keeps a newly opened file and hands back its id', async () => {
    const opening = library().openNew('report.pdf', pdf)
    await answerOpening()
    const id = await opening
    const [stored] = await documentLibrary.list()
    expect(stored).toMatchObject({ id, name: 'report.pdf', size: pdf.byteLength })
    expect(library().current).toBe(stored.id)
    expect(useDocumentStore.getState().document).toMatchObject({ phase: 'ready', name: 'report.pdf' })
  })

  it('opens a stored file by its id', async () => {
    const file = await documentLibrary.add('stored.pdf', new Uint8Array([7]))
    const showing = library().show(file.id)
    await answerOpening()
    expect(await showing).toBe(true)
    expect(port.requests[0]).toMatchObject({ kind: 'open', bytes: new Uint8Array([7]) })
    expect(useDocumentStore.getState().document).toMatchObject({ phase: 'ready', name: 'stored.pdf' })
  })

  it('reports unknown ids as missing', async () => {
    expect(await library().show('unknown')).toBe(false)
    expect(library().missing).toBe(true)
    expect(port.requests).toEqual([])
  })

  it('closes the document on the way home and forgets a deleted current file', async () => {
    const opening = library().openNew('report.pdf', pdf)
    await answerOpening()
    await opening
    const [stored] = await documentLibrary.list()
    library().askToDelete(stored)
    await library().confirmDelete()
    expect(useDocumentStore.getState().document.phase).toBe('empty')
    expect(library()).toMatchObject({ files: [], current: null, deleting: null })
    await library().goHome()
    expect(useDocumentStore.getState().document.phase).toBe('empty')
  })

  it('keeps a reported problem until the next file opens', async () => {
    useToastStore.getState().show('library', 'Only PDF files can be opened.')
    const opening = library().openNew('report.pdf', pdf)
    await answerOpening()
    await opening
    expect(useToastStore.getState().toasts.library).toBeUndefined()
  })

  it('refuses a file without a PDF header before the engine or the library see it', async () => {
    const png = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])
    expect(await library().openNew('picture.pdf', png)).toBeNull()
    expect(port.requests).toEqual([])
    expect(await documentLibrary.list()).toEqual([])
    expect(useToastStore.getState().toasts.library).toBe('picture.pdf is not a PDF file.')
  })

  it('keeps nothing when the engine cannot parse the file', async () => {
    const opening = library().openNew('broken.pdf', pdf)
    await vi.waitFor(() => expect(port.requests.at(-1)?.kind).toBe('open'))
    port.answer({ id: port.lastId, ok: false, error: 'no cross-reference table' })
    expect(await opening).toBeNull()
    expect(await documentLibrary.list()).toEqual([])
    expect(library().current).toBeNull()
    expect(useDocumentStore.getState().document.phase).toBe('empty')
    expect(useToastStore.getState().toasts.library).toBe('broken.pdf could not be opened: no cross-reference table')
  })

  it('reloads the stored version of the current file', async () => {
    const file = await documentLibrary.add('shared.pdf', pdf)
    const showing = library().show(file.id)
    await answerOpening()
    await showing
    await documentLibrary.replace(file.id, new Uint8Array([4, 2]), file.modified)
    const reloading = library().reloadLatest()
    await vi.waitFor(() => expect(port.requests.filter((request) => request.kind === 'open')).toHaveLength(2))
    expect(port.requests.at(-1)).toMatchObject({ kind: 'open', bytes: new Uint8Array([4, 2]) })
    await port.answerNext(opened)
    await vi.waitFor(() => expect(port.requests.at(-1)?.kind).toBe('pages'))
    await port.answerNext(pages)
    await reloading
    expect(library().current).toBe(file.id)
    expect(useAutoSaveStore.getState().status).toBe('saved')
  })

  it('keeps the changes of this tab as a copy and follows it from then on', async () => {
    const opening = library().openNew('shared.pdf', pdf)
    await answerOpening()
    const original = await opening
    const copying = library().saveCopy()
    await vi.waitFor(() => expect(port.requests.at(-1)?.kind).toBe('save'))
    await port.answerNext(new Uint8Array([7, 7]))
    const copy = await copying
    expect(copy).not.toBe(original)
    expect(library().current).toBe(copy)
    const stored = await documentLibrary.open(copy ?? '')
    expect(stored).toMatchObject({ file: { name: 'shared (copy).pdf' }, bytes: new Uint8Array([7, 7]) })
  })
})
