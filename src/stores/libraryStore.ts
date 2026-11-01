import { create } from 'zustand'
import { hasPdfHeader } from '@/lib/pdfHeader.ts'
import { documentLibrary, type StoredFile } from '@/services/DocumentLibrary.ts'
import { useAutoSaveStore } from '@/stores/autoSaveStore.ts'
import { onDocumentReplaced, useDocumentStore } from '@/stores/documentStore.ts'
import { useToastStore } from '@/stores/toastStore.ts'

interface LibraryStore {
  files: StoredFile[]
  current: string | null
  missing: boolean
  deleting: StoredFile | null
  refresh(): Promise<void>
  openNew(name: string, bytes: Uint8Array): Promise<string | null>
  show(id: string): Promise<boolean>
  goHome(): Promise<void>
  askToDelete(file: StoredFile | null): void
  confirmDelete(): Promise<void>
}

const report = (error: unknown) => useToastStore.getState().show('library', (error as Error).message)

export const useLibraryStore = create<LibraryStore>()((set, get) => {
  const leave = async () => {
    const autoSave = useAutoSaveStore.getState()
    await autoSave.flush()
    autoSave.track(null)
    set({ current: null })
  }

  return {
    files: [],
    current: null,
    missing: false,
    deleting: null,

    async refresh() {
      try {
        set({ files: await documentLibrary.list() })
      } catch (error) {
        report(error)
      }
    },

    async openNew(name, bytes) {
      if (!hasPdfHeader(bytes)) {
        report(new Error(`${name} is not a PDF file.`))
        return null
      }
      await leave()
      try {
        const documents = useDocumentStore.getState()
        await documents.open(name, bytes.slice())
        const opened = useDocumentStore.getState().document
        if (opened.phase === 'failed') {
          documents.close()
          throw new Error(`${name} could not be opened: ${opened.message}`)
        }
        const file = await documentLibrary.add(name, bytes)
        void globalThis.navigator?.storage?.persist?.()
        set({ current: file.id, missing: false })
        useToastStore.getState().clear('library')
        if (opened.phase === 'ready') useAutoSaveStore.getState().track(file.id)
        return file.id
      } catch (error) {
        report(error)
        return null
      }
    },

    async show(id) {
      if (id === get().current) return true
      await leave()
      try {
        const stored = await documentLibrary.open(id)
        if (!stored) {
          useDocumentStore.getState().close()
          set({ missing: true })
          return false
        }
        set({ current: id, missing: false })
        await useDocumentStore.getState().open(stored.file.name, stored.bytes)
      } catch (error) {
        report(error)
      }
      return true
    },

    async goHome() {
      await leave()
      useDocumentStore.getState().close()
      await get().refresh()
    },

    askToDelete(file) {
      set({ deleting: file })
    },

    async confirmDelete() {
      const file = get().deleting
      if (!file) return
      set({ deleting: null })
      try {
        if (file.id === get().current) {
          await leave()
          useDocumentStore.getState().close()
        }
        await documentLibrary.remove(file.id)
        await get().refresh()
      } catch (error) {
        report(error)
      }
    },
  }
})

onDocumentReplaced((ready) => useAutoSaveStore.getState().track(ready ? useLibraryStore.getState().current : null))
