import { useAutoSaveStore } from '@/stores/autoSaveStore.ts'

export function OtherTabNotice() {
  const { elsewhere, status } = useAutoSaveStore()
  if (!elsewhere || status === 'conflict') return null
  return (
    <p className="rounded bg-surface px-4 py-2.5 text-ink-strong shadow-[inset_4px_0_0_var(--color-accent),var(--shadow-dialog)]" role="status">
      This document is open in another tab. Changes here may conflict.
    </p>
  )
}
