import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { DOCUMENT_CHANNEL, DocumentChannel } from '@/services/DocumentChannel.ts'
import { documentLibrary, type StoredFile } from '@/services/DocumentLibrary.ts'
import { engine } from '@/services/EngineClient.ts'
import { NEWER_ELSEWHERE, SAVE_DELAY, useAutoSaveStore } from '@/stores/autoSaveStore.ts'
import { useDocumentStore } from '@/stores/documentStore.ts'
import { FakeEnginePort } from '@tests/fakes/FakeEnginePort.ts'

let port: FakeEnginePort
let file: string
let stored: StoredFile
let otherTab: DocumentChannel | null = null

beforeEach(async () => {
  port = FakeEnginePort.attach()
  const opening = useDocumentStore.getState().open('report.pdf', new Uint8Array([1]))
  await port.answerNext({ status: 'opened', version: '1.7', byteLength: 1, objectCount: 3, pageCount: 1, repaired: false })
  await port.answerNext([{ id: '3-0', width: 612, height: 792, rotate: 0 }])
  await opening
  stored = await documentLibrary.add('report.pdf', new Uint8Array([1]))
  file = stored.id
  vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] })
  saver().track(file, stored.modified)
})

afterEach(() => {
  otherTab?.close()
  otherTab = null
})

function openInOtherTab() {
  otherTab = new DocumentChannel(new BroadcastChannel(DOCUMENT_CHANNEL))
  otherTab.follow(file)
  return otherTab
}

const saver = () => useAutoSaveStore.getState()

async function change(revision: string) {
  const rotating = engine.request({ kind: 'rotatePages', pageIds: ['3-0'], quarterTurns: 1 })
  await port.answerNext({ pages: [], dirtyPages: ['3-0'] }, revision)
  await rotating
}

async function answerSave(bytes: Uint8Array, revision: string) {
  await vi.waitFor(() => expect(port.requests.at(-1)?.kind).toBe('save'))
  await port.answerNext(bytes, revision)
  await saver().flush()
}

describe('autoSaveStore', () => {
  it('saves a changed document into the library once the edits pause', async () => {
    expect(saver().status).toBe('saved')
    await change('4.0')
    await change('5.0')
    expect(vi.getTimerCount()).toBe(1)
    expect(saver().status).toBe('pending')
    vi.advanceTimersByTime(SAVE_DELAY)
    await answerSave(new Uint8Array([9, 9]), '5.0')
    expect(saver().status).toBe('saved')
    expect((await documentLibrary.open(file))?.bytes).toEqual(new Uint8Array([9, 9]))
  })

  it('leaves untracked documents alone', async () => {
    saver().track(null)
    await change('4.0')
    expect(vi.getTimerCount()).toBe(0)
    expect(port.requests.at(-1)?.kind).toBe('rotatePages')
    expect(saver().status).toBe('idle')
  })

  it('reports a failed save', async () => {
    await documentLibrary.remove(file)
    await change('4.0')
    vi.advanceTimersByTime(SAVE_DELAY)
    await answerSave(new Uint8Array([9]), '4.0')
    expect(saver()).toMatchObject({ status: 'failed', error: 'the document is no longer stored' })
  })

  it('refuses to overwrite a version another tab stored in the meantime', async () => {
    await documentLibrary.replace(file, new Uint8Array([5]), stored.modified)
    await change('4.0')
    vi.advanceTimersByTime(SAVE_DELAY)
    await answerSave(new Uint8Array([9]), '4.0')
    expect(saver()).toMatchObject({ status: 'conflict', error: NEWER_ELSEWHERE })
    expect((await documentLibrary.open(file))?.bytes).toEqual(new Uint8Array([5]))
    await change('5.0')
    expect(vi.getTimerCount()).toBe(0)
  })

  it('keeps saving its own versions one after another', async () => {
    await change('4.0')
    vi.advanceTimersByTime(SAVE_DELAY)
    await answerSave(new Uint8Array([9]), '4.0')
    await change('5.0')
    vi.advanceTimersByTime(SAVE_DELAY)
    await answerSave(new Uint8Array([8]), '5.0')
    expect(saver().status).toBe('saved')
    expect((await documentLibrary.open(file))?.bytes).toEqual(new Uint8Array([8]))
  })

  it('notices the document open in another tab and stops once that tab saves', async () => {
    const other = openInOtherTab()
    await vi.waitFor(() => expect(saver().elsewhere).toBe(true))
    other.announceSave(stored.modified + 1)
    await vi.waitFor(() => expect(saver().status).toBe('conflict'))
    other.follow(null)
    await vi.waitFor(() => expect(saver().elsewhere).toBe(false))
  })

  it('tells other tabs about its saves', async () => {
    const heard = vi.fn()
    openInOtherTab().listen({ others: () => {}, saved: heard })
    await change('4.0')
    vi.advanceTimersByTime(SAVE_DELAY)
    await answerSave(new Uint8Array([9]), '4.0')
    const saved = await documentLibrary.open(file)
    await vi.waitFor(() => expect(heard).toHaveBeenCalledWith(saved?.file.modified))
  })
})
