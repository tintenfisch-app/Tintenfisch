import { create } from 'zustand'
import { documentChannel } from '@/services/DocumentChannel.ts'
import { documentLibrary } from '@/services/DocumentLibrary.ts'
import { engine } from '@/services/EngineClient.ts'
import { useDocumentStore } from '@/stores/documentStore.ts'

export const SAVE_DELAY = 1000

export const NEWER_ELSEWHERE = 'A newer version of this document was saved in another tab.'

export type SaveStatus = 'idle' | 'pending' | 'saving' | 'saved' | 'failed' | 'conflict'

interface AutoSaveStore {
  status: SaveStatus
  error: string | null
  elsewhere: boolean
  track(file: string | null, version?: number): void
  flush(): Promise<void>
}

export const useAutoSaveStore = create<AutoSaveStore>()((set, get) => {
  let file: string | null = null
  let basedOn = 0
  let savedRevision = ''
  let timer: ReturnType<typeof setTimeout> | null = null
  let queue: Promise<void> = Promise.resolve()

  const cancel = () => {
    if (timer !== null) clearTimeout(timer)
    timer = null
  }

  const stop = () => {
    cancel()
    set({ status: 'conflict', error: NEWER_ELSEWHERE })
  }

  const save = async () => {
    const saving = file
    const revision = engine.revision
    if (!saving || revision === savedRevision || get().status === 'conflict') return
    set({ status: 'saving', error: null })
    try {
      const stored = await documentLibrary.replace(saving, await useDocumentStore.getState().save(), basedOn)
      if (saving !== file) return
      if (!stored) return stop()
      basedOn = stored.modified
      savedRevision = revision
      documentChannel.announceSave(stored.modified)
      set({ status: timer !== null ? 'pending' : 'saved', error: null })
    } catch (error) {
      if (saving === file) set({ status: 'failed', error: (error as Error).message })
    }
  }

  const enqueue = () => {
    queue = queue.then(save)
  }

  engine.onChange(() => {
    if (!file || engine.revision === savedRevision || get().status === 'conflict') return
    cancel()
    timer = setTimeout(() => {
      timer = null
      enqueue()
    }, SAVE_DELAY)
    if (get().status !== 'saving') set({ status: 'pending', error: null })
  })

  documentChannel.listen({
    others: (count) => set({ elsewhere: count > 0 }),
    saved: (modified) => {
      if (file && modified > basedOn) stop()
    },
  })

  return {
    status: 'idle',
    error: null,
    elsewhere: false,

    track(tracked, version = 0) {
      cancel()
      basedOn = tracked !== null && tracked === file ? Math.max(basedOn, version) : version
      file = tracked
      savedRevision = engine.revision
      documentChannel.follow(tracked)
      set({ status: tracked ? 'saved' : 'idle', error: null })
    },

    flush() {
      if (timer !== null) {
        cancel()
        enqueue()
      }
      return queue
    },
  }
})
