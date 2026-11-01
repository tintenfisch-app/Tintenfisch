import { create } from 'zustand'
import { CancelledError, engine } from '@/services/EngineClient.ts'
import { onDocumentReplaced } from '@/stores/documentStore.ts'
import type { EditRight, Permissions, ProtectionState } from '@/types/protocol.ts'

const ENTROPY = 68

interface ProtectionStore {
  dialogOpen: boolean
  current: ProtectionState | null
  notice: string | null
  pending: string | null
  reload(): Promise<void>
  permits(right: EditRight): boolean
  openDialog(): Promise<void>
  closeDialog(): void
  grantOwnerRights(password: string): Promise<void>
  protect(userPassword: string, ownerPassword: string, permissions: Permissions): Promise<void>
  unprotect(): Promise<void>
  reset(): void
}

export function withholds(current: ProtectionState | null, right: EditRight): boolean {
  return current?.withheld.includes(right) ?? false
}

export const useProtectionStore = create<ProtectionStore>()((set, get) => {
  let sequence = 0

  const attempt = async (work: () => Promise<void>) => {
    try {
      await work()
    } catch (error) {
      if (!(error instanceof CancelledError)) set({ notice: (error as Error).message })
    }
  }

  const finish = (pending: string) => set({ dialogOpen: false, notice: null, pending })

  return {
    dialogOpen: false,
    current: null,
    notice: null,
    pending: null,

    async reload() {
      const loading = ++sequence
      await attempt(async () => {
        const current = await engine.request({ kind: 'protection' })
        if (loading === sequence) set({ current })
      })
    },

    permits(right) {
      return !withholds(get().current, right)
    },

    async openDialog() {
      await attempt(async () => set({ dialogOpen: true, current: await engine.request({ kind: 'protection' }), notice: null }))
    },

    closeDialog() {
      set({ dialogOpen: false, notice: null })
    },

    async grantOwnerRights(password) {
      await attempt(async () => {
        const granted = await engine.request({ kind: 'grantOwnerRights', password })
        const current = await engine.request({ kind: 'protection' })
        set({ current, notice: granted ? null : 'That is not the permissions password' })
      })
    },

    async protect(userPassword, ownerPassword, permissions) {
      await attempt(async () => {
        const entropy = crypto.getRandomValues(new Uint8Array(ENTROPY))
        await engine.request({ kind: 'protect', userPassword, ownerPassword, permissions, entropy })
        finish('The protection is written when you save the document')
      })
    },

    async unprotect() {
      await attempt(async () => {
        await engine.request({ kind: 'unprotect' })
        finish('The document is saved without protection from now on')
      })
    },

    reset() {
      sequence++
      set({ dialogOpen: false, current: null, notice: null, pending: null })
    },
  }
})

onDocumentReplaced((ready) => {
  useProtectionStore.getState().reset()
  if (ready) void useProtectionStore.getState().reload()
})
