import { useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { useAutoSaveStore } from '@/stores/autoSaveStore.ts'
import { useLibraryStore } from '@/stores/libraryStore.ts'
import { Button } from '@/components/ui/Button.tsx'
import { Dialog, DialogActions } from '@/components/ui/Dialog.tsx'

export function NewerVersionDialog() {
  const conflict = useAutoSaveStore((store) => store.status === 'conflict')
  const navigate = useNavigate()
  const [busy, setBusy] = useState(false)
  if (!conflict) return null

  async function settle(work: () => Promise<void>) {
    setBusy(true)
    try {
      await work()
    } finally {
      setBusy(false)
    }
  }

  const reload = () => settle(() => useLibraryStore.getState().reloadLatest())

  const saveCopy = () =>
    settle(async () => {
      const id = await useLibraryStore.getState().saveCopy()
      if (id) await navigate({ to: '/documents/$documentId', params: { documentId: id } })
    })

  return (
    <Dialog title="Newer version in another tab" titleId="newer-version-title" role="alertdialog">
      <p className="text-ink-body">
        This document was saved in another tab after you opened it here. Autosave has stopped so neither version is lost. Reload the latest version, or keep your changes here as a separate copy.
      </p>
      <DialogActions>
        <Button disabled={busy} onClick={() => void saveCopy()}>
          Save a copy
        </Button>
        <Button look="primary" autoFocus disabled={busy} onClick={() => void reload()}>
          Reload latest
        </Button>
      </DialogActions>
    </Dialog>
  )
}
